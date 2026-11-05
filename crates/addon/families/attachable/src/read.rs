//! Reading an attachable document as vanilla's strict schemas do: the root and the payload
//! member by member, the `description` typed, and every undeclared member an error.

use facet::Facet;
use facet_json::RawJson;
use indexmap::{IndexMap, IndexSet};

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, JsonPathSegment, Span, VanillaText,
    code_for_deserialize_error_kind,
};
use bedrock_addon_core::json::{
    ClampedText, JsonSlice, Value, Width, clamp_text, parse_at, unknown_members,
};
use bedrock_addon_core::molang::Molang;
use bedrock_addon_core::versioning::ParseCx;
use bedrock_addon_core::visual::{AnimateEntry, ControllerUse, is_molang_place};

use super::ItemBinding;

// ---- vanilla's schema texts  -----------------

/// The text for an undeclared member, or a key that does not fully match its pattern.
pub(crate) const CHILD_NOT_VALID: &str = "child '{}' not valid here.";
/// The text for a declared member of the wrong JSON type.
pub(crate) const UNKNOWN_OPTION_TYPE: &str =
    bedrock_addon_core::diagnostic::cereal::UNKNOWN_CHILD_OPTION_TEXT;
/// The text for a required child without a member.
pub(crate) const REQUIRED_CHILD: &str = bedrock_addon_core::diagnostic::cereal::REQUIRED_CHILD_TEXT;
/// The string node's valid-value regexes.
pub(crate) const NOT_ALLOWED_REGEX: &str =
    "Value '{}' not allowed.  Must match at least one of the following regex patterns: {}";

/// The payload key.
pub(crate) const PAYLOAD: &str = "minecraft:attachable";
/// The payload key of the client entity family, which the same root schema declares.
pub(crate) const CLIENT_ENTITY: &str = "minecraft:client_entity";
/// The pattern of a slot name (`materials`, `textures`, `geometry`, …), as vanilla prints it.
pub(crate) const MAP_KEY: &str = "[a-zA-Z0-9_.-]+";
/// The pattern of an object-form `item` key, as vanilla prints it.
pub(crate) const ITEM_KEY: &str = "[a-zA-Z0-9_.:-]+";
/// The pattern of a `scripts.variables` key, as vanilla prints it.
pub(crate) const VARIABLE_KEY: &str = "variable.[a-zA-Z_][a-zA-Z0-9_]*";

/// A vanilla text with its arguments.
#[allow(
    clippy::unnecessary_wraps,
    reason = "every code field is an `Option<VanillaText>`"
)]
pub(crate) fn text(template: &'static str, args: &[&str]) -> Option<VanillaText> {
    Some(VanillaText::Formatted {
        template,
        args: args.iter().map(|arg| (*arg).to_owned()).collect(),
    })
}

/// "child '<key>' not valid here." at `at` (the member itself).
pub(crate) fn not_valid_here(key: &str, at: JsonPath) -> Diagnostic {
    Diagnostic::error(DiagnosticCode::UnrecognizedField {
        text: text(CHILD_NOT_VALID, &[key]),
    })
    .at(at)
}

/// "Required child <child> not found" at `at` (the object that lacks it).
pub(crate) fn required(child: &str, at: JsonPath) -> Diagnostic {
    Diagnostic::error(DiagnosticCode::MissingRequiredField {
        text: text(REQUIRED_CHILD, &[child]),
    })
    .at(at)
}

/// "unknown child schema option type.  Allowed types: <allowed>" at `at`.
pub(crate) fn wrong_type(allowed: &str, at: JsonPath, span: Option<Span>) -> Diagnostic {
    let mut found = Diagnostic::error(DiagnosticCode::InvalidType {
        text: text(UNKNOWN_OPTION_TYPE, &[allowed]),
    })
    .at(at);
    found.location.span = span;
    found
}

/// Whether `key` fully matches `[a-zA-Z0-9_.-]+`, the pattern of every slot name.
pub(crate) fn is_map_key(key: &str) -> bool {
    !key.is_empty()
        && key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'-'))
}

/// Whether `key` fully matches `[a-zA-Z0-9_.:-]+`, the pattern of a conditional object's key.
pub(crate) fn is_condition_key(key: &str) -> bool {
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
/// name, as "Allowed types: …" lists them. The words are the crate's own: the game's text is a
/// template that takes the list, and the list is not a fixed string.
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
pub(crate) struct Described<'a> {
    /// The `description` object's text.
    pub(crate) slice: JsonSlice<'a>,
    /// `minecraft:attachable/description`.
    pub(crate) at: JsonPath,
    /// The `description` object as JSON, as written: taken out of the one `Value` the
    /// document is parsed to.
    pub(crate) json: Value,
}

/// Walks the root and the payload as vanilla's root schema declares them: `format_version`,
/// `minecraft:attachable` (an object holding the required object `description` and nothing
/// else) and `minecraft:client_entity`; any other member is "child '%s' not valid here.".
/// `None` when the description cannot be reached; the reason is in `d`.
pub(crate) fn describe<'a>(
    document: JsonSlice<'a>,
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
        if !matches!(key.as_str(), "format_version" | PAYLOAD | CLIENT_ENTITY) {
            d.push(not_valid_here(key, cx.at.with(&[key])));
            ok = false;
        }
    }
    let payload_at = cx.at.with(&[PAYLOAD]);
    let Some(payload) = root.get(PAYLOAD).and_then(|raw| document.nested(raw)) else {
        // A missing payload is the crate's own finding: vanilla has only a debug assertion.
        d.push(
            Diagnostic::error(DiagnosticCode::MissingRequiredField {
                text: Some(VanillaText::Fixed("No entity definition to parse")),
            })
            .at(cx.at.clone()),
        );
        return None;
    };
    let payload_json = root_json
        .and_then(|root| root.get(PAYLOAD))
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
            .and_then(|root| member(root, PAYLOAD))
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
/// and `pre_animation` are Molang (before 1.26.0): with the Molang members, the numbers read
/// as [`finite_floats`] says.
pub(crate) fn typed<D: for<'x> Facet<'x>>(
    described: &Described<'_>,
    names_only: bool,
    floats: &[&[&str]],
    statements: bool,
    d: &mut Diagnostics,
) -> Option<D> {
    // A `null` is no value of any declared member; the reader below would take it for an
    // absent one.
    let unknown = unknown_members(<D as Facet<'static>>::SHAPE, &described.json);
    let nulls = declared_nulls(&described.json, &unknown);
    if !nulls.is_empty() {
        for null in nulls {
            let mut at = described.at.clone();
            at.append(&null);
            d.push(wrong_type(allowed_at(&null, names_only), at, None));
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
    };
    let undeclared = undeclared_set(&unknown);
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
    let value = match parse_at::<D>(slice) {
        Ok(value) => value,
        Err(mut error) => {
            // The parse read the clamped text; the finding is of the file as written.
            if let Some(clamped) = &clamped {
                error.span = clamped.written_span(error.span, slice.offset);
            }
            let has_path = !error.path.is_empty();
            let code = code_for_deserialize_error_kind(&error.error.kind, has_path, false);
            // A wrong type is the declared member's, however deep the reader got into it.
            let path = match code {
                DiagnosticCode::InvalidType { .. } => declared(&error.path, names_only),
                _ => error.path.clone(),
            };
            let mut at = described.at.clone();
            at.append(&path);
            // Whatever the reader says of a value inside `item` (a `null` condition is to it
            // a missing value), the schema's finding is a wrong type.
            if error.path.segments().next() == Some(JsonPathSegment::Key("item")) {
                d.push(item_type_error(described, error.span));
                return None;
            }
            let (member, _) = last_member(&path);
            let found = match code {
                DiagnosticCode::MissingRequiredField { .. } => {
                    // The missing member is the path's last key; it is reported at its parent.
                    at.pop();
                    required(member.unwrap_or_default(), at)
                }
                DiagnosticCode::InvalidType { .. } => {
                    wrong_type(allowed_at(&path, names_only), at, Some(error.span))
                }
                code => {
                    let mut found = Diagnostic::error(code).at(at);
                    found.location.span = Some(error.span);
                    found
                }
            };
            d.push(found);
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

/// The paths of the `null`s of `json` (a description) that are not in a member its struct
/// does not declare (`unknown`, from `unknown_members`): those are undeclared, `null` or not.
fn declared_nulls<T>(json: &Value, unknown: &[(JsonPath, T)]) -> Vec<JsonPath> {
    let mut nulls = Vec::new();
    find_nulls(json, &mut JsonPath::new(), &mut nulls);
    if nulls.is_empty() || unknown.is_empty() {
        return nulls;
    }
    // Each `null` is looked up by its own prefixes (as deep as the JSON nests), not against
    // every undeclared member.
    let undeclared = undeclared_set(unknown);
    nulls.retain(|null| !is_undeclared(null.segments(), &undeclared));
    nulls
}

/// The undeclared members (`unknown`, from `unknown_members`), to look a path up in.
fn undeclared_set<T>(unknown: &[(JsonPath, T)]) -> IndexSet<JsonPath> {
    unknown.iter().map(|(member, _)| member.clone()).collect()
}

/// Whether the path `segments` is, or lies below, one of the `undeclared` members: looked up
/// by its own prefixes (as deep as the JSON nests), not against every undeclared member.
fn is_undeclared<'s>(
    mut segments: impl Iterator<Item = JsonPathSegment<'s>>,
    undeclared: &IndexSet<JsonPath>,
) -> bool {
    if undeclared.is_empty() {
        return false;
    }
    let mut prefix = JsonPath::new();
    segments.any(|segment| {
        match segment {
            JsonPathSegment::Key(key) => prefix.push(key),
            JsonPathSegment::Index(index) => prefix.push_index(index),
        }
        undeclared.contains(&prefix)
    })
}

/// The paths of the `null`s in `value`, in written order. `item` itself is left to
/// [`check_item`] (which gives it another code), and `min_engine_version` is kept as
/// written, whatever its elements are.
fn find_nulls(value: &Value, at: &mut JsonPath, found: &mut Vec<JsonPath>) {
    if value.is_null() {
        found.push(at.clone());
    } else if let Some(members) = value.as_object() {
        for (key, member) in members {
            let top = at.is_empty();
            if top && key.as_str() == "item" && member.is_null() {
                continue;
            }
            if top && key.as_str() == "min_engine_version" && !member.is_null() {
                continue;
            }
            at.push(key.as_str());
            find_nulls(member, at, found);
            at.pop();
        }
    } else if let Some(elements) = value.as_array() {
        for (index, element) in elements.iter().enumerate() {
            at.push_index(index);
            find_nulls(element, at, found);
            at.pop();
        }
    }
}

/// The JSON types the schema takes at `path` (from the description), as "Allowed types: …"
/// lists them: the condition of an object-form list entry or of `item` is Molang, everything
/// else goes by its member's name ([`allowed_types`]).
fn allowed_at(path: &JsonPath, names_only: bool) -> &'static str {
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
    if first == Some(JsonPathSegment::Key("item")) && segments.len() > 1 {
        return "molang";
    }
    match last_member(path) {
        // 1.8.x: `render_controllers` is an array of strings.
        (Some("render_controllers"), true) if names_only => "string",
        (member, in_array) => allowed_types(member, in_array),
    }
}

/// The finding for an `item` the typed reader refuses, which it reports at `item` whatever
/// is wrong inside: a value that is neither a string nor an object is reported by
/// `wrong_item` (an `InvalidValue` error with the schema's text); in an object, the first
/// member whose value is no number, boolean or string is no Molang.
fn item_type_error(described: &Described<'_>, span: Span) -> Diagnostic {
    // `item` (object form): "null, array, object → unknown
    // child schema option type".
    let item_at = described.at.with(&["item"]);
    let item = described
        .json
        .as_object()
        .and_then(|description| description.get("item"));
    let not_molang = item.and_then(Value::as_object).and_then(|members| {
        members.iter().find(|(_, value)| {
            value.as_string().is_none() && value.as_number().is_none() && value.as_bool().is_none()
        })
    });
    match not_molang {
        Some((key, _)) => wrong_type("molang", item_at.with(&[key.as_str()]), Some(span)),
        None => wrong_item(item_at, Some(span)),
    }
}

/// An `item` that is neither a string nor an object.
fn wrong_item(at: JsonPath, span: Option<Span>) -> Diagnostic {
    let mut found = Diagnostic::error(DiagnosticCode::InvalidValue {
        text: text(UNKNOWN_OPTION_TYPE, &["string, object"]),
    })
    .at(at);
    found.location.span = span;
    found
}

/// The rules of `description.item`, at 1.10.0 and up;
/// `json` is the description as written.
/// - a `null` is no item (the typed reader takes it for an absent member);
/// - the string form takes any string; one outside the object form's key pattern is warned
///   about (an information of the crate's own) and kept;
/// - every key of the object form fully matches `[a-zA-Z0-9_.:-]+`, and the object holds at
///   least one member;
/// - the members after the one vanilla reads are dropped, with an information of the
///   crate's own;
/// - a number as the condition of the member that binds reads as the empty text, which is
///   never true: an information of the crate's own, the binding kept.
pub(crate) fn check_item(
    item: Option<&ItemBinding>,
    json: &Value,
    at: &JsonPath,
    d: &mut Diagnostics,
) {
    let item_at = at.with(&["item"]);
    let Some(item) = item else {
        if json.as_object().is_some_and(|o| o.get("item").is_some()) {
            d.push(wrong_item(item_at, None));
        }
        return;
    };
    let members = match item {
        ItemBinding::Item(written) => {
            if !is_condition_key(written) {
                d.push(Diagnostic::info(DiagnosticCode::InvalidValue { text: None }).at(item_at));
            }
            return;
        }
        ItemBinding::Conditional(members) => members,
    };
    // The pattern child is required and matched in full.
    for key in members.keys() {
        if !is_condition_key(key) {
            d.push(not_valid_here(key, item_at.with(&[key])));
        }
    }
    if members.is_empty() {
        d.push(required(ITEM_KEY, item_at.clone()));
    }
    for key in item.ignored() {
        d.push(
            Diagnostic::info(DiagnosticCode::DroppedField { text: None }).at(item_at.with(&[key])),
        );
    }
    // `asString("")` of a number is the empty text.
    let bound = members
        .iter()
        .min_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    if let Some((key, Molang::Const(_))) = bound {
        d.push(
            Diagnostic::info(DiagnosticCode::InvalidValue { text: None }).at(item_at.with(&[key])),
        );
    }
}

/// Which numbers of a description are read into a float or kept as a double. What is the
/// family's: the float members its structs declare, `item`'s conditions, and
/// `min_engine_version`, which this family keeps as JSON. The Molang members of the shared
/// description schema, the walk, the rewriting of the text and the finding are core's
/// (`visual::is_molang_place`, `json::clamp_text`).
pub(crate) struct Numbers<'n> {
    /// The members the struct declares as floats, as paths from the description.
    pub(crate) floats: &'n [&'n [&'n str]],
    /// The elements of `initialize` and `pre_animation` are Molang (before 1.26.0).
    pub(crate) statements: bool,
    /// 1.8.x: no `item`, no `scripts.animate`, and `render_controllers` holds names only.
    pub(crate) names_only: bool,
}

impl Numbers<'_> {
    /// What holds the number at `path` (from the description); `None` where the schema
    /// declares no number, and a number is a wrong type or an undeclared member's.
    fn width(&self, path: &[JsonPathSegment<'_>]) -> Option<Width> {
        use JsonPathSegment::Key;
        if let [Key("min_engine_version"), _, ..] = path {
            // Kept as written below its top level, as a `Value`, whose numbers are doubles.
            return Some(Width::Double);
        }
        let declared = self.floats.iter().any(|float| {
            float.len() == path.len()
                && float
                    .iter()
                    .zip(path)
                    .all(|(name, segment)| *segment == Key(name))
        });
        // The Molang members: the schema's expression type takes a number, which
        // core holds as an `f32` constant.
        let molang = is_molang_place(path, self.statements, self.names_only)
            || (matches!(path, [Key("item"), Key(_)]) && !self.names_only);
        (declared || molang).then_some(Width::Float)
    }
}

/// The text of `slice` (whose value is `json`) with every number beyond what holds it
/// ([`Numbers::width`]: beyond `f32` in a float or Molang member, beyond a double in
/// `min_engine_version`) replaced by the largest finite number of its sign and width, each
/// with an `InvalidValue` warning at its path below `at`, on the number as written; `None`
/// when no number is: core's `json::clamp_text`, asked for the places of this family.
/// Vanilla's schema takes any JSON number for a float child and for a Molang one, and the
/// loader reads it with `asFloat`; a value that would read as infinity could not be written
/// back, so it is read as the nearest number that can.
pub(crate) fn finite_floats(
    slice: JsonSlice<'_>,
    json: &Value,
    numbers: &Numbers<'_>,
    undeclared: &IndexSet<JsonPath>,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> Option<ClampedText> {
    // Core's walk, rewriting and finding.
    clamp_text(
        slice,
        json,
        &|path| {
            // A number of a member the struct does not declare is that member's finding
            // alone.
            numbers
                .width(path)
                .filter(|_| !is_undeclared(path.iter().copied(), undeclared))
        },
        at,
        d,
    )
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
pub(crate) fn check_map<V>(
    map: Option<&IndexMap<String, V>>,
    name: &str,
    required_child: bool,
    at: &JsonPath,
    d: &mut Diagnostics,
) {
    // Pattern keys match in full; an empty object whose only option is a required pattern
    // fails.
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
/// fully matches `[a-zA-Z0-9_.:-]+`; the members after the one vanilla reads are dropped, with
/// an information. An empty object passes the schema; the loader's rules warn of it
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
        // The members after the first are dropped; the game says nothing, the crate reports a
        // `DroppedField` information.
        d.push(Diagnostic::info(DiagnosticCode::DroppedField { text: None }).at(at.with(&[key])));
    }
}

/// The rules of `scripts.variables`: every key fully matches
/// `variable.[a-zA-Z_][a-zA-Z0-9_]*`, the object holds at least one, and every value is exactly
/// `public`.
pub(crate) fn check_variables(
    variables: Option<&IndexMap<String, bedrock_addon_core::json::StrictString>>,
    at: &JsonPath,
    d: &mut Diagnostics,
) {
    // Every value is exactly `public`.
    let Some(variables) = variables else { return };
    let map_at = at.with(&["variables"]);
    for (key, value) in variables {
        let member_at = map_at.with(&[key]);
        if !is_variable_key(key) {
            d.push(not_valid_here(key, member_at));
        } else if value.0 != "public" {
            d.push(
                Diagnostic::error(DiagnosticCode::InvalidValue {
                    text: text(NOT_ALLOWED_REGEX, &[&value.0, "public"]),
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
pub(crate) fn check_identifier(identifier: &str, at: &JsonPath, d: &mut Diagnostics) {
    if identifier.is_empty() {
        d.push(
            Diagnostic::error(DiagnosticCode::InvalidValue {
                text: Some(VanillaText::Fixed("Missing or invalid identifier")),
            })
            .at(at.with(&["identifier"])),
        );
    }
}

/// The conditional objects of `render_controllers` ([`check_condition`]).
pub(crate) fn check_controllers(
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

/// The conditional objects of `scripts.animate` ([`check_condition`]); `at` is `scripts`.
pub(crate) fn check_animate(animate: Option<&[AnimateEntry]>, at: &JsonPath, d: &mut Diagnostics) {
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
pub(crate) fn check_min_engine_version(value: Option<&Value>, at: &JsonPath, d: &mut Diagnostics) {
    // The schema takes a string or an array; any other JSON type fails it.
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

    #[test]
    fn finding_the_nulls_of_declared_members_is_linear_in_nulls_and_undeclared_members() {
        // `n` undeclared members (half of them `null`) beside a map of `n` `null` values.
        assert_linear_timed_at("attachable declared nulls", 10_000, |n| {
            let undeclared: Vec<String> = (0..n)
                .map(|i| format!(r#""u{i}": {}"#, if i % 2 == 0 { "null" } else { "1" }))
                .collect();
            let nulls: Vec<String> = (0..n).map(|i| format!(r#""k{i}": null"#)).collect();
            let text = format!(
                r#"{{"identifier": "demo:a", {}, "materials": {{{}}}}}"#,
                undeclared.join(", "),
                nulls.join(", ")
            );
            let json: Value = bedrock_addon_core::json::from_str(&text).expect("test JSON");
            let unknown = unknown_members(<crate::Description as Facet<'static>>::SHAPE, &json);
            assert_eq!(unknown.len(), n);
            let (found, took) = timed(|| declared_nulls(&json, &unknown));
            assert_eq!(found.len(), n);
            took
        });
    }

    #[test]
    fn clamping_the_numbers_beyond_f32_is_linear_in_the_numbers() {
        // `n` conditional render controllers, each with a condition beyond `f32`, and as
        // many numbers no double holds in `min_engine_version`.
        assert_linear_timed_at("attachable clamp", 2_500, |n| {
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
