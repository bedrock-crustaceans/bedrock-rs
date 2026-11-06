//! The description schema client entities and attachables share, read as vanilla's strict
//! schemas read it: the root and the payload member by member (`describe`), the
//! `description` typed with every undeclared member an error (`typed`), and the rules every
//! version's description shares (`check_*`). One reader for both families; a `Profile`
//! holds what is the family's and where the two readers differ today.

use facet::Facet;
use facet_json::RawJson;
use indexmap::{IndexMap, IndexSet};

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, JsonPathSegment, Span, VanillaText,
    code_for_deserialize_error_kind,
};
use bedrock_addon_core::json::{
    ClampedText, JsonSlice, ScalarFailure, ScalarRules, Value, Width, clamp_text, parse_at,
    parse_with_rules, unknown_members,
};
use bedrock_addon_core::molang::Molang;
use bedrock_addon_core::versioning::ParseCx;

use crate::{AnimateEntry, ControllerUse, is_molang_place};

// ---- the texts vanilla's JSON schema checks print -----------------

/// The text for an undeclared member, or a key that does not fully match its pattern.
pub const CHILD_NOT_VALID: &str = "child '{}' not valid here.";
/// The text for a required child without a member.
pub const REQUIRED_CHILD: &str = bedrock_addon_core::diagnostic::cereal::REQUIRED_CHILD_TEXT;
/// The string node's valid-value regexes.
pub const NOT_ALLOWED_REGEX: &str =
    "Value '{}' not allowed.  Must match at least one of the following regex patterns: {}";

/// The payload key of a client entity.
pub const CLIENT_ENTITY: &str = "minecraft:client_entity";
/// The payload key of an attachable.
pub const ATTACHABLE: &str = "minecraft:attachable";
/// The pattern of a slot name (`materials`, `textures`, `geometry`, …), as vanilla prints it.
pub(crate) const MAP_KEY: &str = "[a-zA-Z0-9_.-]+";
/// The pattern of a `scripts.variables` key, as vanilla prints it.
pub(crate) const VARIABLE_KEY: &str = "variable.[a-zA-Z_][a-zA-Z0-9_]*";

/// How a family reads the description schema the client entity and attachable share: its
/// payload, the scalar rules of its slices, the members of its own, and where the two readers
/// differ today. Each difference is pinned by the family's tests (`drifts.rs`, and D1 by both
/// `scalar_tests.expected`); which side is the game's is not decided here.
#[derive(Clone, Copy, Debug)]
pub struct Profile {
    /// The payload key this family reads.
    pub payload: &'static str,
    /// The payload key the same root schema declares for the twin.
    pub other_payload: &'static str,
    /// Whether a file with both payloads is told of, with an information at the twin's (the
    /// client entity loader reads its own first and loads it alone).
    pub notes_other_payload: bool,
    /// The scalar rules the description is read by.
    pub rules: ScalarRules,
    /// D1: whether a `null` of a declared member is told with its span.
    pub null_span: bool,
    /// D2: what holds a number below `min_engine_version`'s top level; `None` leaves it as
    /// written, unclamped.
    pub min_engine_version_width: Option<Width>,
    /// D3: the words of a list entry's condition of the wrong type.
    pub condition_types: ConditionTypes,
    /// The family's own member of the description, when it has one.
    pub own: Option<OwnMember>,
}

/// D3: how "Allowed types: …" words a list entry's condition (`render_controllers[0].c`)
/// the typed read refuses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConditionTypes {
    /// By the condition's key, as any member is (`allowed_types`): `string`.
    ByName,
    /// As the place it is (`allowed_at`): `molang`.
    ByPlace,
}

/// A member of the description only one family declares (attachable's `item`): a `null` of it
/// is not the shared walk's, its object form's values are Molang, and any failure the typed
/// read reports inside it is the family's to tell.
#[derive(Clone, Copy, Debug)]
pub struct OwnMember {
    /// The member's name.
    pub name: &'static str,
    /// The finding for a failure inside the member, whose span is `span`.
    pub refused: fn(&Described<'_>, Span) -> Diagnostic,
}

/// A vanilla text with its arguments.
#[allow(
    clippy::unnecessary_wraps,
    reason = "every code field is an `Option<VanillaText>`"
)]
pub fn text(template: &'static str, args: &[&str]) -> Option<VanillaText> {
    Some(VanillaText::Formatted {
        template,
        args: args.iter().map(|arg| (*arg).to_owned()).collect(),
    })
}

/// `"child '<key>' not valid here."` at `at` (the member itself).
pub fn not_valid_here(key: &str, at: JsonPath) -> Diagnostic {
    Diagnostic::error(DiagnosticCode::UnrecognizedField {
        text: text(CHILD_NOT_VALID, &[key]),
    })
    .at(at)
}

/// `"Required child <child> not found"` at `at` (the object that lacks it).
pub fn required(child: &str, at: JsonPath) -> Diagnostic {
    Diagnostic::error(DiagnosticCode::MissingRequiredField {
        text: text(REQUIRED_CHILD, &[child]),
    })
    .at(at)
}

/// `"unknown child schema option type.  Allowed types: <allowed>"` at `at`.
pub fn wrong_type(allowed: &str, at: JsonPath, span: Option<Span>) -> Diagnostic {
    let mut found =
        Diagnostic::error(bedrock_addon_core::diagnostic::cereal::unknown_child_option(allowed))
            .at(at);
    found.location.span = span;
    found
}

/// Whether `key` fully matches `[a-zA-Z0-9_.-]+`, the pattern of every slot name.
pub fn is_map_key(key: &str) -> bool {
    !key.is_empty()
        && key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'-'))
}

/// Whether `key` fully matches `[a-zA-Z0-9_.:-]+`, the pattern of a conditional object's key.
pub fn is_condition_key(key: &str) -> bool {
    !key.is_empty()
        && key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b':' | b'-'))
}

/// Whether `key` fully matches `variable.[a-zA-Z_][a-zA-Z0-9_]*`; the `.` is the regex's
/// any-character.
pub(crate) fn is_variable_key(key: &str) -> bool {
    let Some(rest) = key.strip_prefix("variable") else {
        return false;
    };
    let mut chars = rest.chars();
    // Any one character, then an identifier.
    if chars.next().is_none() {
        return false;
    }
    let name = chars.as_str().as_bytes();
    matches!(name.first(), Some(b) if b.is_ascii_alphabetic() || *b == b'_')
        && name.iter().all(|b| b.is_ascii_alphanumeric() || *b == b'_')
}

/// The JSON types vanilla's schema option of a `description` member takes, by the member's
/// name, as "Allowed types: …" lists them. The game's text is a template over the schema's
/// own type list; the words of each list here are the crate's.
fn allowed_types(member: Option<&str>, in_array: bool) -> &'static str {
    match member {
        Some("min_engine_version") => "string, array",
        Some("texture_index") => "int",
        Some("enable_attachables" | "held_item_ignores_lighting" | "hide_armor") => "bool",
        Some("held_item_scale") => "float",
        Some("render_controllers" | "animate") if in_array => "string, object",
        Some("animation_controllers") if in_array => "object",
        Some("initialize" | "pre_animation") if in_array => "string",
        Some(
            "render_controllers"
            | "animate"
            | "animation_controllers"
            | "initialize"
            | "pre_animation",
        ) => "array",
        Some(
            "scale"
            | "scaleX"
            | "scalex"
            | "scaleY"
            | "scaley"
            | "scaleZ"
            | "scalez"
            | "parent_setup"
            | "should_update_bones_and_effects_offscreen"
            | "should_update_effects_offscreen"
            | "hide_held_items",
        ) => "molang",
        Some(
            "description" | "materials" | "textures" | "geometry" | "spawn_egg" | "scripts"
            | "particle_effects" | "particle_emitters" | "animations" | "sound_effects"
            | "variables",
        ) => "object",
        // `identifier`, `queryable_geometry`, the members of `spawn_egg` that are text, and a
        // value inside one of the string maps.
        _ => "string",
    }
}

/// The description of a document: its slice, its path and its JSON.
pub struct Described<'a> {
    /// The `description` object's text.
    pub slice: JsonSlice<'a>,
    /// `<payload>/description`.
    pub at: JsonPath,
    /// The `description` object as JSON, as written: taken out of the one `Value` the
    /// document is parsed to.
    pub json: Value,
}

/// Walks the root and the payload as vanilla's root schema declares them: `format_version`,
/// the profile's payload (an object holding the required object `description` and nothing
/// else) and the twin's; any other member is "child '%s' not valid here.".
/// `None` when the description cannot be reached; the reason is in `d`.
pub fn describe<'a>(
    document: JsonSlice<'a>,
    profile: &Profile,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<Described<'a>> {
    // The document is parsed to a `Value` once, here: every "is it an object" question, the
    // float clamp and the undeclared-member scan read this one.
    let mut json = bedrock_addon_core::json::from_str_jsonc::<Value>(document.text).ok();
    let root_json = json.as_ref().and_then(Value::as_object);
    let root = object(document, root_json.is_some(), &cx.at, d)?;
    let mut ok = true;
    for key in root.keys() {
        let key_str = key.as_str();
        if key_str != "format_version"
            && key_str != profile.payload
            && key_str != profile.other_payload
        {
            d.push(not_valid_here(key, cx.at.with(&[key])));
            ok = false;
        }
    }
    let payload_at = cx.at.with(&[profile.payload]);
    let Some(payload) = root
        .get(profile.payload)
        .and_then(|raw| document.nested(raw))
    else {
        // A document without the payload: vanilla has only a debug assertion here, so the
        // message is the crate's own.
        d.push(
            Diagnostic::error(DiagnosticCode::MissingRequiredField {
                text: Some(VanillaText::Fixed("No entity definition to parse")),
            })
            .at(cx.at.clone()),
        );
        return None;
    };
    if profile.notes_other_payload && root.contains_key(profile.other_payload) {
        // A file with both payloads loads the client entity alone and the game says nothing.
        // The crate notes the payload that is not loaded, and not written, with an information
        // at the member, whatever it holds.
        d.push(
            Diagnostic::info(DiagnosticCode::DroppedField { text: None })
                .at(cx.at.with(&[profile.other_payload])),
        );
    }
    let payload_json = root_json
        .and_then(|root| root.get(profile.payload))
        .and_then(Value::as_object);
    let payload_members = object(payload, payload_json.is_some(), &payload_at, d)?;
    for key in payload_members.keys() {
        if key != "description" {
            d.push(not_valid_here(key, payload_at.with(&[key])));
            ok = false;
        }
    }
    let Some(description) = payload_members
        .get("description")
        .and_then(|raw| payload.nested(raw))
    else {
        d.push(required("description", payload_at));
        return None;
    };
    let at = payload_at.with(&["description"]);
    let is_object = payload_json
        .and_then(|payload| payload.get("description"))
        .is_some_and(|description| description.as_object().is_some());
    let taken = if is_object {
        json.as_mut()
            .and_then(|root| member(root, profile.payload))
            .and_then(|mut payload| member(&mut payload, "description"))
    } else {
        None
    };
    let Some(json) = taken else {
        d.push(wrong_type("object", at, None));
        return None;
    };
    ok.then_some(Described {
        slice: description,
        at,
        json,
    })
}

/// The member `key` of the object `value`, moved out of it.
fn member(value: &mut Value, key: &str) -> Option<Value> {
    value.as_object_mut()?.remove(key)
}

/// The typed description `D` of `described`, with every member `D` does not declare reported
/// as "child '%s' not valid here." (an error: the schemas are strict). `None` when the typed
/// parse fails or a member is undeclared. `floats` are the members `D` declares as floats
/// (paths from the description) and `statements` says whether the elements of `initialize`
/// and `pre_animation` are Molang (before 1.26.0); `names_only` is the 1.8.x shape. With the
/// Molang members ([`is_molang_place`], and the values of the profile's own member) and
/// `min_engine_version` as the profile reads it, the numbers read as `clamp_text` says
/// ([`Numbers`]). A number of a member `D` does not declare is that member's finding alone.
///
/// A number beyond `f32` is read as the largest finite float in a float member and in a
/// Molang member alike.
pub fn typed<D: for<'x> Facet<'x>>(
    described: &Described<'_>,
    profile: &Profile,
    names_only: bool,
    floats: &[&[&str]],
    statements: bool,
    d: &mut Diagnostics,
) -> Option<D> {
    let unknown = unknown_members(<D as Facet<'static>>::SHAPE, &described.json);
    let undeclared: IndexSet<JsonPath> = unknown.iter().map(|(member, _)| member.clone()).collect();
    // A `null` is no value of any declared member: the strict schema finds the child by its
    // name and none of its type options takes a null, so the game answers "unknown child
    // schema option type" and refuses the file. The typed read below would take the `null` for
    // an absent member and write the file without it.
    let own = profile.own.map(|own| own.name);
    let nulls = declared_nulls(&described.json, &undeclared, own);
    if !nulls.is_empty() {
        for null in nulls {
            let mut at = described.at.clone();
            at.append(&null);
            let span = if profile.null_span {
                bedrock_addon_core::json::span_at(described.slice, &null)
            } else {
                None
            };
            d.push(wrong_type(allowed_at(&null, names_only, own), at, span));
        }
        for (path, _) in &unknown {
            let (member, _) = last_member(path);
            let mut at = described.at.clone();
            at.append(path);
            d.push(not_valid_here(member.unwrap_or_default(), at));
        }
        return None;
    }
    let numbers = Numbers {
        floats,
        statements,
        names_only,
        min_engine_version: profile.min_engine_version_width,
        own,
    };
    let clamped = finite_floats(
        described.slice,
        &described.json,
        &numbers,
        &undeclared,
        &described.at,
        d,
    );
    let slice = clamped
        .as_ref()
        .map_or(described.slice, |clamped| JsonSlice {
            text: &clamped.text,
            offset: described.slice.offset,
        });
    let value = match parse_with_rules::<D>(slice, None, profile.rules) {
        // What the pre-pass tells of a conversion is not pushed: under these rules it tells
        // only a `null` read as a struct, and every `null` was refused above.
        Ok(read) => read.value,
        Err(failure) => {
            d.push(refused(
                failure,
                described,
                profile,
                slice,
                clamped.as_ref(),
                names_only,
            ));
            return None;
        }
    };
    for (path, _) in &unknown {
        let (member, _) = last_member(path);
        let mut at = described.at.clone();
        at.append(path);
        d.push(not_valid_here(member.unwrap_or_default(), at));
    }
    unknown.is_empty().then_some(value)
}

/// The finding for the typed read's `failure` of `slice` (the description's text, `clamped`
/// when numbers were clamped): at the declared member, worded as the profile words it.
fn refused(
    failure: ScalarFailure,
    described: &Described<'_>,
    profile: &Profile,
    slice: JsonSlice<'_>,
    clamped: Option<&ClampedText>,
    names_only: bool,
) -> Diagnostic {
    let own = profile.own.map(|own| own.name);
    // The failing value: its path below the description, its code, and its place in
    // the text read.
    let (code, failed, span) = match failure {
        ScalarFailure::Read(error) => (
            code_for_deserialize_error_kind(&error.error.kind, !error.path.is_empty(), false),
            error.path,
            error.span,
        ),
        // A scalar no arm of an untagged enum takes: a type error at the enum.
        ScalarFailure::Refused(refused) => (
            refused.finding(&JsonPath::new()).code,
            refused.path.clone(),
            refused.span.unwrap_or(Span {
                offset: u32::try_from(slice.offset).unwrap_or(u32::MAX),
                len: u32::try_from(slice.text.len()).unwrap_or(u32::MAX),
            }),
        ),
    };
    // The parse read the clamped text; the finding is of the file as written.
    let span = match clamped {
        Some(clamped) => clamped.written_span(span, slice.offset),
        None => span,
    };
    // A wrong type is the declared member's, however deep the reader got into it.
    // By legacy name: the variant and the definitions that were it.
    let path = match code.name() {
        "InvalidType" => declared(&failed, names_only),
        _ => failed.clone(),
    };
    let mut at = described.at.clone();
    at.append(&path);
    // Whatever the reader says of a value inside the family's own member, the finding
    // is the family's.
    if let Some(own) = profile.own
        && failed.segments().next() == Some(JsonPathSegment::Key(own.name))
    {
        return (own.refused)(described, span);
    }
    let (member, in_array) = last_member(&path);
    // By legacy name: the variant and the definitions that were it.
    match code.name() {
        "MissingRequiredField" => {
            // The missing member is the path's last key; it is reported at its parent.
            at.pop();
            required(member.unwrap_or_default(), at)
        }
        "InvalidType" => {
            let allowed = match profile.condition_types {
                ConditionTypes::ByPlace => allowed_at(&path, names_only, own),
                ConditionTypes::ByName => match (member, in_array) {
                    // 1.8.x: `render_controllers` is an array of strings.
                    (Some("render_controllers"), true) if names_only => "string",
                    _ => allowed_types(member, in_array),
                },
            };
            wrong_type(allowed, at, Some(span))
        }
        _ => {
            let mut found = Diagnostic::error(code).at(at);
            found.location.span = Some(span);
            found
        }
    }
}

/// The paths of the `null`s of `json` (a description), in written order, that are not in a
/// member its struct does not declare (`undeclared`): those are undeclared, `null` or not.
/// `min_engine_version` is kept as written below its top level, whatever its elements are,
/// and a `null` family member `own` is the family's to tell.
pub(crate) fn declared_nulls(
    json: &Value,
    undeclared: &IndexSet<JsonPath>,
    own: Option<&str>,
) -> Vec<JsonPath> {
    /// A step of the walk: into a value, by the segment that leads to it, or back out of one.
    enum Step<'v> {
        Enter(&'v Value, Option<JsonPathSegment<'v>>),
        Leave,
    }
    let mut nulls = Vec::new();
    // The steps still to take, the next one last. One path is kept for the whole walk, a
    // segment pushed on the way into a value and popped on the way out, so the walk is
    // linear in the value however deep it is; a path is copied only for a `null`.
    let mut at = JsonPath::new();
    let mut open: Vec<Step<'_>> = vec![Step::Enter(json, None)];
    while let Some(step) = open.pop() {
        let (value, segment) = match step {
            Step::Leave => {
                at.pop();
                continue;
            }
            Step::Enter(value, segment) => (value, segment),
        };
        match segment {
            None => {}
            Some(JsonPathSegment::Key(key)) => at.push(key),
            Some(JsonPathSegment::Index(index)) => at.push_index(index),
        }
        if segment.is_some() {
            open.push(Step::Leave);
        }
        if value.is_null() {
            nulls.push(at.clone());
        } else if let Some(members) = value.as_object() {
            let first = open.len();
            for (key, member) in members {
                let top = at.is_empty();
                if top && Some(key.as_str()) == own && member.is_null() {
                    continue;
                }
                if top && key.as_str() == "min_engine_version" && !member.is_null() {
                    continue;
                }
                let key = JsonPathSegment::Key(key.as_str());
                open.push(Step::Enter(member, Some(key)));
            }
            open[first..].reverse();
        } else if let Some(elements) = value.as_array() {
            let first = open.len();
            for (index, element) in elements.iter().enumerate() {
                let index = JsonPathSegment::Index(index);
                open.push(Step::Enter(element, Some(index)));
            }
            open[first..].reverse();
        }
    }
    if undeclared.is_empty() {
        return nulls;
    }
    nulls.retain(|null| {
        let segments: Vec<JsonPathSegment<'_>> = null.segments().collect();
        !below_undeclared(&segments, undeclared)
    });
    nulls
}

/// The types the schema option at `path` (from the description) takes, for a value that is
/// none of them: the member's own, or its elements'; a value of a map, of a condition object
/// or of the family's object-form `own` member is Molang, and below 1.10 a name of
/// `animation_controllers[]` is a string.
pub(crate) fn allowed_at(path: &JsonPath, names_only: bool, own: Option<&str>) -> &'static str {
    let segments: Vec<JsonPathSegment<'_>> = path.segments().collect();
    let first = segments.first().copied();
    let below_element = segments
        .windows(2)
        .any(|pair| matches!(pair, [JsonPathSegment::Index(_), JsonPathSegment::Key(_)]));
    if below_element {
        // 1.8.x `animation_controllers[]` maps a name to a controller's name.
        return if first == Some(JsonPathSegment::Key("animation_controllers")) {
            "string"
        } else {
            "molang"
        };
    }
    if let (Some(own), Some(JsonPathSegment::Key(key))) = (own, first)
        && key == own
        && segments.len() > 1
    {
        return "molang";
    }
    match last_member(path) {
        // 1.8.x: `render_controllers` is an array of strings.
        (Some("render_controllers"), true) if names_only => "string",
        (member, in_array) => allowed_types(member, in_array),
    }
}

/// Which numbers of a description are read into a float or kept as a double: the float
/// members the struct declares, the Molang members of the shared schema ([`is_molang_place`])
/// and of the family's own member, and `min_engine_version`, which the families keep as JSON.
/// The walk, the rewriting of the text and the finding are core's (`json::clamp_text`).
#[derive(Clone, Copy, Debug)]
pub struct Numbers<'n> {
    /// The members the struct declares as floats, as paths from the description.
    pub floats: &'n [&'n [&'n str]],
    /// The elements of `initialize` and `pre_animation` are Molang (before 1.26.0).
    pub statements: bool,
    /// 1.8.x: no own member, no `scripts.animate`, and `render_controllers` holds names only.
    pub names_only: bool,
    /// What holds a number below `min_engine_version`'s top level (D2); `None`: nothing.
    pub min_engine_version: Option<Width>,
    /// The family's own member, whose object form's values are Molang.
    pub own: Option<&'n str>,
}

impl Numbers<'_> {
    /// What holds the number at `path` (from the description); `None` where the schema
    /// declares no number, and a number is a wrong type or an undeclared member's.
    fn width(&self, path: &[JsonPathSegment<'_>]) -> Option<Width> {
        use JsonPathSegment::Key;
        if let [Key("min_engine_version"), _, ..] = path {
            // Kept as written below its top level, as a `Value`, whose numbers are doubles.
            return self.min_engine_version;
        }
        let declared = is_float(self.floats, path);
        // The Molang members: the schema's expression type takes a number, which core holds
        // as an `f32` constant.
        let own = matches!((self.own, path), (Some(own), [Key(member), Key(_)]) if own == *member);
        let molang =
            is_molang_place(path, self.statements, self.names_only) || (own && !self.names_only);
        (declared || molang).then_some(Width::Float)
    }
}

/// The text of `slice` (whose value is `json`) with every number beyond what holds it
/// ([`Numbers`]: beyond `f32` in a float or Molang member, beyond its width in
/// `min_engine_version`) replaced by the largest finite number of its sign and width, each
/// with an `InvalidValue` warning at its path below `at`, on the number as written; `None`
/// when no number is: core's `json::clamp_text`, asked for the places of the schema.
/// Vanilla's schema takes any JSON number for a float child and for a Molang one, and the
/// loader reads it with `asFloat`; a value that would read as infinity could not be written
/// back, so it is read as the nearest number that can.
pub fn finite_floats(
    slice: JsonSlice<'_>,
    json: &Value,
    numbers: &Numbers<'_>,
    undeclared: &IndexSet<JsonPath>,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> Option<ClampedText> {
    clamp_text(
        slice,
        json,
        &|path| {
            // A number of a member the struct does not declare is that member's finding
            // alone.
            numbers
                .width(path)
                .filter(|_| !below_undeclared(path, undeclared))
        },
        at,
        d,
    )
}

/// Whether `path` is one of `floats` (each a path of member names).
fn is_float(floats: &[&[&str]], path: &[JsonPathSegment<'_>]) -> bool {
    floats.iter().any(|float| {
        float.len() == path.len()
            && float
                .iter()
                .zip(path)
                .all(|(name, segment)| *segment == JsonPathSegment::Key(name))
    })
}

/// Whether `path` is, or lies below, one of the `undeclared` members:
/// looked up by its own prefixes, not against every undeclared member.
fn below_undeclared(path: &[JsonPathSegment<'_>], undeclared: &IndexSet<JsonPath>) -> bool {
    if undeclared.is_empty() {
        return false;
    }
    let mut prefix = JsonPath::new();
    path.iter().any(|segment| {
        match segment {
            JsonPathSegment::Key(key) => prefix.push(key),
            JsonPathSegment::Index(index) => prefix.push_index(*index),
        }
        undeclared.contains(&prefix)
    })
}

/// `path` cut back to the value the schema declares a type for: a member of `description`,
/// `spawn_egg` or `scripts`, the value of a slot map, or an element of a list (and, in a
/// conditional object, the member below it). A reader that fails inside such a value reports
/// a path into it.
fn declared(path: &JsonPath, names_only: bool) -> JsonPath {
    const MAPS: [&str; 8] = [
        "materials",
        "textures",
        "geometry",
        "particle_effects",
        "particle_emitters",
        "animations",
        "sound_effects",
        "variables",
    ];
    // Lists whose elements are scalars, and lists whose elements may be objects of scalars.
    const SCALAR_LISTS: [&str; 2] = ["initialize", "pre_animation"];
    const OBJECT_LISTS: [&str; 3] = ["render_controllers", "animate", "animation_controllers"];
    let mut cut = JsonPath::new();
    // How many more segments the value just entered may take; `None` while still descending
    // through `spawn_egg` and `scripts`.
    let mut left: Option<usize> = None;
    for segment in path.segments() {
        if left == Some(0) {
            break;
        }
        match segment {
            JsonPathSegment::Key(key) => cut.push(key),
            JsonPathSegment::Index(index) => cut.push_index(index),
        }
        left = match (left, segment) {
            (Some(n), _) => Some(n - 1),
            (None, JsonPathSegment::Key("spawn_egg" | "scripts")) => None,
            (None, JsonPathSegment::Key(key)) if MAPS.contains(&key) => Some(1),
            // 1.8.x: `render_controllers` holds names only.
            (None, JsonPathSegment::Key("render_controllers")) if names_only => Some(1),
            (None, JsonPathSegment::Key(key)) if SCALAR_LISTS.contains(&key) => Some(1),
            (None, JsonPathSegment::Key(key)) if OBJECT_LISTS.contains(&key) => Some(2),
            (None, _) => Some(0),
        };
    }
    cut
}

/// The last member name of `path`, and whether the path ends in an array element below it.
fn last_member(path: &JsonPath) -> (Option<&str>, bool) {
    let mut in_array = false;
    let mut segments: Vec<JsonPathSegment<'_>> = path.segments().collect();
    while let Some(segment) = segments.pop() {
        match segment {
            JsonPathSegment::Key(key) => return (Some(key), in_array),
            JsonPathSegment::Index(_) => in_array = true,
        }
    }
    (None, in_array)
}

/// The members of the object `slice`; "unknown child schema option type" when it is none.
/// `is_object` says whether the slice's value is a JSON object: comments may come before it,
/// so the text's first character does not say, and the document's `Value` does. (A root that
/// is no object never reaches a struct: the load rejects it first, "nothing to parse in".)
fn object<'a>(
    slice: JsonSlice<'a>,
    is_object: bool,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> Option<IndexMap<String, RawJson<'a>>> {
    if !is_object {
        d.push(wrong_type("object", at.clone(), None));
        return None;
    }
    match parse_at::<IndexMap<String, RawJson<'a>>>(slice) {
        Ok(members) => Some(members),
        Err(error) => {
            let mut found = Diagnostic::error(DiagnosticCode::MalformedInput {
                message: error.error.kind.to_string(),
                text: None,
            })
            .at(at.clone());
            found.location.span = Some(error.span);
            d.push(found);
            None
        }
    }
}

// ---- the checks every version's description shares ------------------------------------------

/// The pattern rules of one string map: every key fully matches `[a-zA-Z0-9_.-]+`, and a map
/// whose pattern child is required holds at least one member.
pub fn check_map<V>(
    map: Option<&IndexMap<String, V>>,
    name: &str,
    required_child: bool,
    at: &JsonPath,
    d: &mut Diagnostics,
) {
    // Pattern keys must match in full, not just contain a match; an empty object whose only
    // option is a required pattern fails.
    let Some(map) = map else { return };
    let map_at = at.with(&[name]);
    for key in map.keys() {
        if !is_map_key(key) {
            d.push(not_valid_here(key, map_at.with(&[key])));
        }
    }
    if required_child && map.is_empty() {
        d.push(required(MAP_KEY, map_at));
    }
}

/// The rules of one conditional object of `render_controllers` or `scripts.animate`: every key
/// fully matches `[a-zA-Z0-9_.:-]+`; the members after the one vanilla reads are dropped with an
/// information. An empty object passes the schema; the loader's rules warn of it
/// (`validate::empty_conditions`).
pub(crate) fn check_condition<'m>(
    members: &'m IndexMap<String, Molang>,
    ignored: impl Iterator<Item = &'m str>,
    at: &JsonPath,
    d: &mut Diagnostics,
) {
    for key in members.keys() {
        if !is_condition_key(key) {
            d.push(not_valid_here(key, at.with(&[key])));
        }
    }
    for key in ignored {
        // Members after the one the game reads get a `DroppedField` information of the crate's
        // own; the game drops them without a message.
        d.push(Diagnostic::info(DiagnosticCode::DroppedField { text: None }).at(at.with(&[key])));
    }
}

/// The rules of `scripts.variables`: every key fully matches
/// `variable.[a-zA-Z_][a-zA-Z0-9_]*`, the object holds at least one, and every value is exactly
/// `public`.
pub fn check_variables(
    variables: Option<&IndexMap<String, String>>,
    at: &JsonPath,
    d: &mut Diagnostics,
) {
    // The schema accepts exactly one valid value for a variable, `public`.
    let Some(variables) = variables else { return };
    let map_at = at.with(&["variables"]);
    for (key, value) in variables {
        let member_at = map_at.with(&[key]);
        if !is_variable_key(key) {
            d.push(not_valid_here(key, member_at));
        } else if value != "public" {
            d.push(
                Diagnostic::error(DiagnosticCode::InvalidValue {
                    text: text(NOT_ALLOWED_REGEX, &[value, "public"]),
                })
                .at(member_at),
            );
        }
    }
    if variables.is_empty() {
        d.push(required(VARIABLE_KEY, map_at));
    }
}

/// An identifier that is empty names nothing.
pub fn check_identifier(identifier: &str, at: &JsonPath, d: &mut Diagnostics) {
    if identifier.is_empty() {
        d.push(
            Diagnostic::error(DiagnosticCode::InvalidValue {
                text: Some(VanillaText::Fixed("Missing or invalid identifier")),
            })
            .at(at.with(&["identifier"])),
        );
    }
}

/// The conditional objects of `render_controllers` (`check_condition`).
pub fn check_controllers(
    controllers: Option<&[ControllerUse]>,
    at: &JsonPath,
    d: &mut Diagnostics,
) {
    for (index, controller) in controllers.into_iter().flatten().enumerate() {
        if let ControllerUse::Conditional(members) = controller {
            let mut element_at = at.with(&["render_controllers"]);
            element_at.push_index(index);
            check_condition(members, controller.ignored(), &element_at, d);
        }
    }
}

/// The conditional objects of `scripts.animate` (`check_condition`); `at` is `scripts`.
pub fn check_animate(animate: Option<&[AnimateEntry]>, at: &JsonPath, d: &mut Diagnostics) {
    for (index, entry) in animate.into_iter().flatten().enumerate() {
        if let AnimateEntry::Conditional(members) = entry {
            let mut element_at = at.with(&["animate"]);
            element_at.push_index(index);
            check_condition(members, entry.ignored(), &element_at, d);
        }
    }
}

/// `min_engine_version` is a string or an array; any other JSON type fails the schema. A
/// string that is no version passes (the loader reads 0.0.0).
pub fn check_min_engine_version(value: Option<&Value>, at: &JsonPath, d: &mut Diagnostics) {
    // The 1.8 schema declares the member as a string or an array; any other type fails it.
    if let Some(value) = value
        && value.as_string().is_none()
        && !value.is_array()
    {
        d.push(wrong_type(
            "string, array",
            at.with(&["min_engine_version"]),
            None,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slot_names_match_their_pattern_in_full() {
        assert!(is_map_key("default"));
        assert!(is_map_key("a.b-c_D9"));
        assert!(!is_map_key("my:tex"));
        assert!(!is_map_key(""));
        assert!(!is_map_key("a b"));
    }

    #[test]
    fn condition_keys_may_hold_a_colon() {
        assert!(is_condition_key("controller.render.a"));
        assert!(is_condition_key("demo:walk"));
        assert!(!is_condition_key("a b"));
        assert!(!is_condition_key(""));
    }

    #[test]
    fn variable_keys_take_any_character_after_variable() {
        assert!(is_variable_key("variable.foo"));
        assert!(is_variable_key("variable_foo"));
        assert!(is_variable_key("variable._a1"));
        assert!(!is_variable_key("variable."));
        assert!(!is_variable_key("variable"));
        assert!(!is_variable_key("variable.1a"));
        assert!(!is_variable_key("variable.a-b"));
        assert!(!is_variable_key("v.foo"));
    }
}

#[cfg(test)]
mod cost {
    use bedrock_addon_core::test_cost::{assert_linear_timed_at, timed};

    use super::*;

    /// The undeclared members of `names`, as `typed` hands them over.
    fn undeclared(names: impl Iterator<Item = String>) -> IndexSet<JsonPath> {
        names.map(|name| JsonPath::of_keys([name])).collect()
    }

    #[test]
    fn finding_the_nulls_of_declared_members_is_linear_in_nulls_and_undeclared_members() {
        // `n` undeclared members (half of them `null`) beside a map of `n` `null` values.
        assert_linear_timed_at("visual declared nulls", 10_000, |n| {
            let members: Vec<String> = (0..n)
                .map(|i| format!(r#""u{i}": {}"#, if i % 2 == 0 { "null" } else { "1" }))
                .collect();
            let nulls: Vec<String> = (0..n).map(|i| format!(r#""k{i}": null"#)).collect();
            let text = format!(
                r#"{{"identifier": "demo:a", {}, "materials": {{{}}}}}"#,
                members.join(", "),
                nulls.join(", ")
            );
            let json: Value = bedrock_addon_core::json::from_str(&text).expect("test JSON");
            let undeclared = undeclared((0..n).map(|i| format!("u{i}")));
            let (found, took) = timed(|| declared_nulls(&json, &undeclared, None));
            assert_eq!(found.len(), n);
            assert_eq!(found[0].to_string(), "materials/k0");
            took
        });
    }

    /// A `null` 120 levels down has its whole path, and the walk keeps one path for all of
    /// them: `n` such `null`s, each at the end of its own chain of lists. No member is
    /// undeclared, so the walk is what is timed.
    #[test]
    fn finding_nulls_120_levels_down_is_linear_in_the_nulls() {
        const LEVELS: usize = 120;
        // `materials`, the slot, then lists down to the `null`.
        let lists = LEVELS - 2;
        assert_linear_timed_at("visual declared nulls, 120 levels", 1_000, |n| {
            let chains: Vec<String> = (0..n)
                .map(|i| format!(r#""k{i}": {}null{}"#, "[".repeat(lists), "]".repeat(lists)))
                .collect();
            let text = format!(
                r#"{{"identifier": "demo:a", "min_engine_version": [null], "materials": {{{}}}}}"#,
                chains.join(", ")
            );
            let json: Value = bedrock_addon_core::json::from_str(&text).expect("test JSON");
            let (found, took) = timed(|| declared_nulls(&json, &IndexSet::new(), None));
            // The one in `min_engine_version` is not among them.
            assert_eq!(found.len(), n);
            for (i, at) in [(0, &found[0]), (n - 1, &found[n - 1])] {
                assert_eq!(at.len(), LEVELS);
                assert_eq!(
                    at.to_string(),
                    format!("materials/k{i}{}", "/0".repeat(lists))
                );
            }
            took
        });
    }

    /// A `null` 120 levels below an undeclared member is that member's, one as deep below a
    /// declared member is found.
    #[test]
    fn a_null_120_levels_below_an_undeclared_member_is_left_to_it() {
        let lists = 118;
        let chain = format!("{}null{}", "[".repeat(lists), "]".repeat(lists));
        let text = format!(r#"{{"u": {{"x": {chain}}}, "materials": {{"k": {chain}}}}}"#);
        let json: Value = bedrock_addon_core::json::from_str(&text).expect("test JSON");
        let found: Vec<String> =
            declared_nulls(&json, &undeclared(std::iter::once("u".to_owned())), None)
                .iter()
                .map(ToString::to_string)
                .collect();
        assert_eq!(found, [format!("materials/k{}", "/0".repeat(lists))]);
    }

    /// The order is the written one, a `null` `min_engine_version` is one of them, and what
    /// lies below its top level is not.
    #[test]
    fn the_nulls_are_listed_in_written_order() {
        let text = r#"{"b": null, "min_engine_version": null, "a": [1, null, {"x": null, "min_engine_version": [null]}], "u": {"x": null}, "c": null}"#;
        let json: Value = bedrock_addon_core::json::from_str(text).expect("test JSON");
        let found: Vec<String> =
            declared_nulls(&json, &undeclared(std::iter::once("u".to_owned())), None)
                .iter()
                .map(ToString::to_string)
                .collect();
        assert_eq!(
            found,
            [
                "b",
                "min_engine_version",
                "a/1",
                "a/2/x",
                "a/2/min_engine_version/0",
                "c"
            ]
        );
    }

    /// A `null` family member is the family's, one below it is the walk's.
    #[test]
    fn a_null_own_member_is_left_to_the_family() {
        let json: Value =
            bedrock_addon_core::json::from_str(r#"{"item": null, "a": {"item": null}, "b": null}"#)
                .expect("test JSON");
        let found: Vec<String> = declared_nulls(&json, &IndexSet::new(), Some("item"))
            .iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(found, ["a/item", "b"]);
    }

    #[test]
    fn clamping_the_numbers_beyond_f32_is_linear_in_the_numbers() {
        // `n` conditional render controllers, each with a condition beyond `f32`, and as
        // many numbers no double holds in `min_engine_version`.
        assert_linear_timed_at("visual clamp", 2_500, |n| {
            let controllers: Vec<String> = (0..n)
                .map(|i| format!(r#"{{"controller.render.c{i}": 1e39}}"#))
                .collect();
            let versions = vec!["-1e999"; n];
            let text = format!(
                r#"{{"identifier": "demo:a", "min_engine_version": [{}], "render_controllers": [{}]}}"#,
                versions.join(", "),
                controllers.join(", ")
            );
            let json: Value = bedrock_addon_core::json::from_str(&text).expect("test JSON");
            let numbers = Numbers {
                floats: &[],
                statements: false,
                names_only: false,
                min_engine_version: Some(Width::Double),
                own: Some("item"),
            };
            let mut found = Diagnostics::default();
            let (clamped, took) = timed(|| {
                finite_floats(
                    JsonSlice::new(&text),
                    &json,
                    &numbers,
                    &IndexSet::new(),
                    &JsonPath::new(),
                    &mut found,
                )
                .expect("numbers to clamp")
            });
            assert_eq!(found.len(), 2 * n);
            assert_eq!(clamped.text.matches("3.4028235e38").count(), n);
            assert_eq!(clamped.text.matches("-1.7976931348623157e308").count(), n);
            took
        });
    }
}
