//! Reading the material files from JSON values: one pack's copy leniently, the wrapper gate,
//! and the typed read of a list and of a material map with the client's rules.

use std::collections::HashMap;

use facet::Facet;
use facet_value::VObject;
use indexmap::{IndexMap, IndexSet};

use super::codes;
use super::file::{
    KeyRules, Listed, MaterialDef, MaterialFile, MaterialFileForm, MaterialKey, SamplerState,
    StencilFace, StencilRefOverride, Variant, VertexField, base_name,
};
use super::list::{ListEntry, MaterialList, MaterialListKind};
use super::values::{
    BLEND_TARGETS, COMPARISON_FUNCS, FACE_KEYS, IGNORED_RENDER_STATE, IS_ANIMATED_TEXTURE,
    MSAA_SUPPORTS, PLAIN_KEYS, PRIMITIVE_MODES, SAMPLER_KEYS, SAMPLER_SLOTS, SHADER_STAGES,
    STENCIL_OPS, TEXTURE_FILTERS, TEXTURE_FORMATS, TEXTURE_WRAPS, VARIANT_MISSPELLINGS,
    VERTEX_FIELDS, render_state_bit,
};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Code, Diagnostic, DiagnosticDetail, Diagnostics, JsonPath, JsonPathSegment, Severity,
};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::json::kit::{Node, Profile, Read, Recovery};
use bedrock_addon_core::json::{
    MAX_JSON_DEPTH, Prepared, Value, blank_bom, check_depth, clamp_non_finite, last_wins,
};
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::schema::{self, FromJson, Schema, UnknownRule, read_document, read_each};
use bedrock_addon_core::version::{FormatVersion, SemVersion};
use bedrock_addon_core::versioning::{MemberFailure, NullHandling, Reading};

/// The wrapper's member, and the member of it that holds the version.
pub(super) const MATERIALS: &str = "materials";

/// The path `inner` of a material map as the file has it: below the wrapper's member when the
/// map is `wrapped` in one. A finding names its place in the file that was written.
pub(super) fn path_in_file(inner: &JsonPath, wrapped: bool) -> JsonPath {
    let at = if wrapped {
        JsonPath::new().child(MATERIALS)
    } else {
        JsonPath::new()
    };
    at.join(inner)
}

/// Moves the findings `found` of a material map into `out`, each at its path in the file.
pub(super) fn report_in_file(found: Diagnostics, wrapped: bool, out: &mut Diagnostics) {
    for mut finding in found {
        finding.location.json_path = path_in_file(&finding.location.json_path, wrapped);
        out.push(finding);
    }
}
/// The wrapper's version member.
const VERSION: &str = "version";

/// One pack's copy of a list or a material file, as read.
pub(super) struct Layer {
    /// The root value as written: a number past the range of a double is an infinity, which
    /// the typed read sees and no value kept as written holds ([`finite`](Self::finite)).
    pub(super) root: Value,
    /// The root with every such number the largest finite double of its sign, when it has one.
    finite: Option<Value>,
    /// Where a number past the range of a double was written, in the order of the file: it
    /// is read as the largest finite double of its sign.
    pub(super) clamped: IndexSet<JsonPath>,
    /// The finding for content after the root value, which is not read.
    pub(super) trailing: Option<Diagnostic>,
}

impl Layer {
    /// Reports each clamped number where the file has it: an information of the crate's own.
    pub(super) fn report_clamped(&self, out: &mut Diagnostics) {
        out.extend(self.trailing.clone());
        for at in &self.clamped {
            let code = Code::from(&core_codes::INVALID_VALUE);
            out.push(Diagnostic::info(code).at(at.clone()));
        }
    }

    /// The root with every number finite: what is kept as written.
    pub(super) fn finite(&self) -> &Value {
        self.finite.as_ref().unwrap_or(&self.root)
    }
}

/// Reads one pack's copy of a list or a material file: a BOM is skipped, comments are allowed,
/// of a member written twice the last stands, and a number past the doubles is noted. The
/// error is the reader's message.
pub(super) fn read_copy(bytes: &[u8]) -> Result<Layer, String> {
    // The nesting is checked before the encoding, so a text both too deep and not UTF-8 is
    // told as too deep. The mark is blanked first: the scan stops at it, and the offset in the
    // message counts it.
    let bytes = &*blank_bom(bytes);
    check_depth(bytes, MAX_JSON_DEPTH).map_err(|error| error.to_string())?;
    let prepared = Prepared::new(bytes).map_err(|error| error.to_string())?;
    let text = prepared.as_str();
    // jsoncpp keeps the last of two equal keys, so the text is cleaned before anything reads it.
    let root = bedrock_addon_core::json::from_str_jsonc(&last_wins(text))
        .map_err(|error| error.kind.to_string())?;
    let mut finite = root.clone();
    let clamped: IndexSet<JsonPath> = clamp_non_finite(&mut finite, None)
        .into_iter()
        .map(|number| number.path)
        .collect();
    let trailing = bedrock_addon_core::json::trailing_content_finding(
        bedrock_addon_core::json::JsonSlice::new(text),
    );
    Ok(Layer {
        finite: (!clamped.is_empty()).then_some(finite),
        root,
        clamped,
        trailing,
    })
}

/// One copy of a material file past the wrapper gate.
#[derive(Debug)]
pub(super) struct Unwrapped {
    /// The material map, an object: the wrapper's content without `version`, or the root
    /// itself.
    pub(super) map: Value,
    /// The wrapper's version, when the client unwraps the copy.
    pub(super) wrapped: Option<SemVersion>,
    /// The `materials` member of a wrapper the client does not unwrap, taken out of the map.
    pub(super) dropped: Option<Value>,
}

/// The version of a wrapper the client unwraps: `materials` is the root's only member, it is
/// not empty, and its `version` is `1.0.0` (build metadata aside).
///
/// The comparison looks at the numbers, the pre-release text and the any flags; a missing or
/// empty `version` is read as 0.0.0.
fn gate(root_members: usize, inner: &VObject) -> Option<SemVersion> {
    // An empty `materials` has no version and fails below like any other.
    if root_members != 1 {
        return None;
    }
    let text = inner.get(VERSION)?.as_string()?.as_str();
    let version = SemVersion::parse(text)?;
    let exact = (version.major, version.minor, version.patch) == (1, 0, 0)
        && version.pre_release.is_empty()
        && !version.is_any();
    exact.then_some(version)
}

/// `object` without its member `key`, the others in their order.
fn without(object: &VObject, key: &str) -> VObject {
    let mut rest = VObject::with_capacity(object.len());
    for (name, value) in object {
        if name.as_str() != key {
            rest.insert(name.as_str(), value.clone());
        }
    }
    rest
}

/// Applies the wrapper gate to the root of one copy. A `materials` object the client does not
/// unwrap is taken out of the map with an information: the client would silently read the root as
/// the material map and create one material named `materials`.
pub(super) fn unwrap_copy(root: VObject, out: &mut Diagnostics) -> Unwrapped {
    let Some(inner) = root.get(MATERIALS).and_then(Value::as_object) else {
        return Unwrapped {
            map: root.into_value(),
            wrapped: None,
            dropped: None,
        };
    };
    if let Some(version) = gate(root.len(), inner) {
        return Unwrapped {
            map: without(inner, VERSION).into_value(),
            wrapped: Some(version),
            dropped: None,
        };
    }
    let at = JsonPath::new().child(MATERIALS).child(VERSION);
    let code = Code::from(&core_codes::UNSUPPORTED_VERSION);
    out.push(Diagnostic::info(code).at(at));
    Unwrapped {
        dropped: root.get(MATERIALS).cloned().map(finite),
        map: without(&root, MATERIALS).into_value(),
        wrapped: None,
    }
}

/// Reports each key of `map` whose name before the first `:` an earlier key has; `file`
/// is the path the client's text names. The keys are taken bytewise, as the client's sorted
/// map holds them. Both keys stay in the document, so the finding is a kept error.
pub(super) fn check_duplicates(map: &Value, file: &str, out: &mut Diagnostics) {
    let mut keys: Vec<&str> = map.as_object().map_or_else(Vec::new, |map| {
        map.keys().map(facet_value::VString::as_str).collect()
    });
    keys.sort_unstable();
    let mut first: HashMap<&str, &str> = HashMap::with_capacity(keys.len());
    for key in keys {
        let name = base_name(key);
        let Some(earlier) = first.get(name) else {
            first.insert(name, key);
            continue;
        };
        let at = JsonPath::new().child(key);
        // The game logs this at the content log's error level and goes on with both keys: a kept
        // error.
        out.push_kept_error(
            Diagnostic::of(codes::DEFINED_TWICE.with([name, file, earlier, key])).at(at),
        );
    }
}

/// A material map read into its typed form.
pub(super) struct ReadFile {
    /// The file.
    pub(super) file: MaterialFile,
    /// Per material, in order: whether it carries a value the client cannot load, so that no
    /// material is created from it.
    pub(super) refused: Vec<bool>,
}

/// How a material file and a list are read: as the client's reader takes each member in turn,
/// a member it cannot take kept as written where the file has it (`or_kept`), a `null` absent
/// and kept, an element or a variant body of the wrong type left out alone, every finding an
/// information but the keyword and state lines.
const READING: Reading = Reading::OWN
    .strict()
    .with_null(NullHandling::AbsentKept)
    .with_on_member_failure(MemberFailure::DefaultAndTell);

/// The read of one material (`format` the wrapper's version: a versioned material) or one list.
fn reader(found: &mut Diagnostics, format: Option<FormatVersion>) -> Read<'_> {
    Read::new(READING, found)
        .profile(Profile::EVERY_FAILURE.with_recovery(Recovery::Value))
        .unknown_severity(Severity::Info)
        .keep_in_file_order()
        .format(format)
}

/// Whether the material being read follows the versioned key rules: the client stamps it with
/// its file's version, which the read carries as the document's format.
fn versioned(read: &Read<'_>) -> bool {
    read.known_format().is_some()
}

/// The format a material of `rules` is read at.
fn format_of(rules: KeyRules) -> Option<FormatVersion> {
    (rules == KeyRules::Versioned).then(|| FormatVersion::new(1, 0, 0))
}

/// One material, or one variant body, as read: the three spellings of a list member apart, as
/// the file writes them. [`MaterialDef`] is the value; each `Listed` gathers one member's three.
#[derive(Debug, Default, Facet, Schema)]
#[facet(rename_all = "camelCase", schema::unknown_with = def_unknown)]
struct Def {
    #[facet(schema::or_kept = "told", schema::read_beside = states)]
    states: Option<Vec<String>>,
    #[facet(rename = "+states", schema::or_kept = "told", schema::read_beside = states)]
    add_states: Option<Vec<String>>,
    #[facet(rename = "-states", schema::or_kept = "told", schema::read_beside = states)]
    sub_states: Option<Vec<String>>,
    #[facet(schema::or_kept = "told", schema::read_beside = defines)]
    defines: Option<Vec<String>>,
    #[facet(rename = "+defines", schema::or_kept = "told", schema::read_beside = defines)]
    add_defines: Option<Vec<String>>,
    #[facet(rename = "-defines", schema::or_kept = "told", schema::read_beside = defines)]
    sub_defines: Option<Vec<String>>,
    #[facet(schema::or_kept = "told", schema::read_beside = samplers)]
    sampler_states: Option<Vec<SamplerState>>,
    #[facet(
        rename = "+samplerStates",
        schema::or_kept = "told",
        schema::read_beside = samplers
    )]
    add_sampler_states: Option<Vec<SamplerState>>,
    #[facet(
        rename = "-samplerStates",
        schema::or_kept = "told",
        schema::read_with = unapplied_samplers
    )]
    sub_sampler_states: Option<Vec<SamplerState>>,
    #[facet(schema::or_kept = "told")]
    variants: Option<Vec<Bodies>>,
    #[facet(rename = "+variants", schema::or_kept = "told")]
    add_variants: Option<Vec<Bodies>>,
    #[facet(
        rename = "-variants",
        schema::or_kept = "told",
        schema::read_with = unread_variants
    )]
    sub_variants: Option<Vec<Bodies>>,
    #[facet(schema::or_kept = "told")]
    vertex_shader: Option<String>,
    #[facet(schema::or_kept = "told")]
    fragment_shader: Option<String>,
    #[facet(schema::or_kept = "told")]
    geometry_shader: Option<String>,
    #[facet(schema::or_kept = "told", schema::read_with = comparison)]
    depth_func: Option<String>,
    #[facet(schema::or_kept = "told")]
    front_face: Option<StencilFace>,
    #[facet(schema::or_kept = "told")]
    back_face: Option<StencilFace>,
    #[facet(schema::or_kept = "told", schema::read_with = stencil_override)]
    stencil_ref_override: Option<StencilRefOverride>,
    #[facet(schema::or_kept = "told", schema::read_with = unsigned)]
    stencil_ref: Option<u32>,
    #[facet(schema::or_kept = "told", schema::read_with = unsigned)]
    stencil_read_mask: Option<u32>,
    #[facet(schema::or_kept = "told", schema::read_with = unsigned)]
    stencil_write_mask: Option<u32>,
    #[facet(schema::or_kept = "told", schema::read_with = blend)]
    blend_src: Option<String>,
    #[facet(schema::or_kept = "told", schema::read_with = blend)]
    blend_dst: Option<String>,
    #[facet(schema::or_kept = "told", schema::read_with = blend)]
    alpha_src: Option<String>,
    #[facet(schema::or_kept = "told", schema::read_with = blend)]
    alpha_dst: Option<String>,
    #[facet(schema::or_kept = "told")]
    vertex_fields: Option<Vec<VertexField>>,
    #[facet(schema::or_kept = "told", schema::read_with = float)]
    depth_bias: Option<f32>,
    #[facet(schema::or_kept = "told", schema::read_with = float)]
    slope_scaled_depth_bias: Option<f32>,
    #[facet(schema::or_kept = "told", schema::read_with = msaa)]
    msaa_support: Option<String>,
    #[facet(schema::or_kept = "told", schema::read_with = primitive)]
    primitive_mode: Option<String>,
    #[facet(schema::or_kept = "told", schema::read_with = formats)]
    render_target_formats: Option<Vec<Vec<String>>>,
    /// Tested for presence only: any value.
    is_animated_texture: Option<Value>,
    extra: UnknownMembers,
}

// A leaf: the member is read by `stencil_override`, holds no number with a bound, no Molang.
bedrock_addon_core::impl_schema_leaf!(StencilRefOverride);
bedrock_addon_core::impl_no_molang!(StencilRefOverride);

/// One element of a `variants` array as read: variant name to body.
#[derive(Debug, Default, Facet, Schema)]
#[facet(transparent)]
struct Bodies(IndexMap<String, Def>);

impl From<Def> for MaterialDef {
    fn from(def: Def) -> Self {
        let samplers = |list: Option<Vec<SamplerState>>| {
            list.map(|list| {
                list.into_iter()
                    .map(|sampler| SamplerState {
                        extra: finite_members(&sampler.extra),
                        ..sampler
                    })
                    .collect()
            })
        };
        let variants = |list: Option<Vec<Bodies>>| {
            list.map(|list| {
                list.into_iter()
                    .map(|Bodies(bodies)| {
                        let bodies = bodies.into_iter().map(|(name, body)| (name, body.into()));
                        Variant(bodies.collect())
                    })
                    .collect()
            })
        };
        let face = |face: StencilFace| StencilFace {
            extra: finite_members(&face.extra),
            ..face
        };
        let vertex_fields = def.vertex_fields.map(|fields| {
            let field = |field: VertexField| VertexField {
                extra: finite_members(&field.extra),
                ..field
            };
            fields.into_iter().map(field).collect()
        });
        Self {
            states: Listed {
                replace: def.states,
                add: def.add_states,
                sub: def.sub_states,
            },
            defines: Listed {
                replace: def.defines,
                add: def.add_defines,
                sub: def.sub_defines,
            },
            sampler_states: Listed {
                replace: samplers(def.sampler_states),
                add: samplers(def.add_sampler_states),
                sub: samplers(def.sub_sampler_states),
            },
            vertex_shader: def.vertex_shader,
            fragment_shader: def.fragment_shader,
            geometry_shader: def.geometry_shader,
            depth_func: def.depth_func,
            front_face: def.front_face.map(face),
            back_face: def.back_face.map(face),
            stencil_ref_override: def.stencil_ref_override,
            stencil_ref: def.stencil_ref,
            stencil_read_mask: def.stencil_read_mask,
            stencil_write_mask: def.stencil_write_mask,
            blend_src: def.blend_src,
            blend_dst: def.blend_dst,
            alpha_src: def.alpha_src,
            alpha_dst: def.alpha_dst,
            vertex_fields,
            depth_bias: def.depth_bias,
            slope_scaled_depth_bias: def.slope_scaled_depth_bias,
            msaa_support: def.msaa_support,
            primitive_mode: def.primitive_mode,
            render_target_formats: def.render_target_formats,
            is_animated_texture: def.is_animated_texture.map(finite),
            variants: Listed {
                replace: variants(def.variants),
                add: variants(def.add_variants),
                sub: variants(def.sub_variants),
            },
            key_rules: None,
            extra: finite_members(&def.extra),
        }
    }
}

/// `value` with every number no double holds as the largest finite double of its sign: the
/// read sees the infinity (its finding is the copy's), the value kept as written is writable.
fn finite(mut value: Value) -> Value {
    clamp_non_finite(&mut value, None);
    value
}

/// [`finite`] for each member of `members`.
fn finite_members(members: &UnknownMembers) -> UnknownMembers {
    members
        .iter()
        .map(|(key, value)| (key.to_owned(), finite(value.clone())))
        .collect()
}

/// Which of the three plain list keys a material writes as an array.
#[derive(Clone, Copy)]
struct Plain {
    states: bool,
    defines: bool,
    samplers: bool,
}

/// Whether the client skips the prefixed list `key` of a material that has the `plain` keys.
fn hidden(rules: KeyRules, key: &str, plain: Plain) -> bool {
    match (rules, key) {
        (KeyRules::Versioned, "+states" | "-states") => plain.states,
        (KeyRules::Versioned, "+defines" | "-defines") | (KeyRules::Legacy, "+defines") => {
            plain.defines
        }
        (KeyRules::Versioned, "+samplerStates") => plain.samplers,
        (KeyRules::Legacy, "+states" | "-states" | "-defines" | "+samplerStates") => true,
        _ => false,
    }
}

/// The number as a whole number from 0 up, when it is one: an integer, or a float without a
/// fraction (`3.0`) in the range of a `u32`.
fn whole(number: &facet_value::VNumber) -> Option<u64> {
    number.to_u64().or_else(|| {
        let float = number.to_f64_lossy();
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "checked to be whole and in range"
        )]
        (float.fract() == 0.0 && (0.0..=f64::from(u32::MAX)).contains(&float))
            .then_some(float as u64)
    })
}

/// `key` without a `+` or `-` in front, when it has one.
fn unprefixed(key: &str) -> Option<&str> {
    key.strip_prefix('+').or_else(|| key.strip_prefix('-'))
}

/// A finding of the crate's own at `node`.
fn info(code: Code, node: &Node<'_>) -> Diagnostic {
    Diagnostic::error(code)
        .severity(Severity::Info)
        .at(node.path().clone())
}

/// Reads with `with` what the client never looks at (a `-variants` body, a prefixed list its
/// plain key hides, a sampler list it does not apply): the content is read for write-back and
/// told for its JSON types and numbers only; the lines the client tells of what it applies
/// (keywords, states, spellings, a sampler's slot) are not told.
fn unread<T>(
    node: &Node<'_>,
    read: &mut Read<'_>,
    with: impl FnOnce(&mut Read<'_>) -> Option<T>,
) -> Option<T> {
    let mut fork = read.fork();
    let value = with(&mut fork);
    fork.findings_mut()
        .retain(|finding| told_unread(finding, node));
    read.commit(fork);
    value
}

/// Whether `finding`, of content below `node` the client does not apply, is told: a wrong
/// type and a number out of its member's range are, the checks of what is applied are not.
fn told_unread(finding: &Diagnostic, node: &Node<'_>) -> bool {
    let id = finding.code.id();
    let applied_only = [
        codes::STATE_IGNORED.id(),
        codes::STATE_UNKNOWN.id(),
        core_codes::UNRECOGNIZED_FIELD.id(),
        core_codes::NO_EFFECT.id(),
        core_codes::MISSING_REQUIRED_FIELD.id(),
    ];
    if applied_only.contains(&id) {
        return false;
    }
    if id != core_codes::INVALID_VALUE.id() {
        return true;
    }
    // A keyword the tables lack, and a slot past the sampler slots, which a whole `u32` at
    // `samplerIndex` is: the number out of range of a `u32` is the member's own.
    let at = &finding.location.json_path;
    let slot = at.last_key() == Some("samplerIndex")
        && value_at(node, at)
            .and_then(Value::as_number)
            .and_then(whole)
            .is_some_and(|index| u32::try_from(index).is_ok());
    finding.severity != Severity::Error && !slot
}

/// The value at `at`, a path at or below `node`'s.
fn value_at<'v>(node: &Node<'v>, at: &JsonPath) -> Option<&'v Value> {
    let mut value = node.value();
    for step in at.segments().skip(node.path().len()) {
        value = match step {
            JsonPathSegment::Key(key) => value.as_object()?.get(key)?,
            JsonPathSegment::Index(index) => value.as_array()?.get(index)?,
        };
    }
    Some(value)
}

/// One of the spellings of a list member at `node` whose content `list` reads: a prefixed one
/// the client skips beside its plain key (`owner` the material) is read unread and told of no
/// effect.
fn listed<T>(
    node: &Node<'_>,
    owner: &Node<'_>,
    read: &mut Read<'_>,
    list: impl FnOnce(&mut Read<'_>) -> Option<Vec<T>>,
) -> Option<Vec<T>> {
    let is_array = |key: &str| {
        owner
            .value()
            .as_object()
            .and_then(|object| object.get(key))
            .is_some_and(Value::is_array)
    };
    let plain = Plain {
        states: is_array("states"),
        defines: is_array("defines"),
        samplers: is_array("samplerStates"),
    };
    let rules = if versioned(read) {
        KeyRules::Versioned
    } else {
        KeyRules::Legacy
    };
    let key = node.path().last_key().unwrap_or_default();
    if !hidden(rules, key, plain) {
        return list(read);
    }
    let read_list = unread(node, read, list)?;
    read.push(info(Code::from(&core_codes::NO_EFFECT), node));
    Some(read_list)
}

/// `states`, `+states`, `-states`: render states, an ignored one named and an unknown one told
/// with the client's text.
fn states(node: &Node<'_>, owner: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<String>> {
    listed(node, owner, read, |read| {
        read_each(node, read, |node, read| {
            let name = String::from_json(node, read)?;
            if name == IGNORED_RENDER_STATE {
                read.push(info(codes::STATE_IGNORED.with([name.as_str()]), node));
            } else if render_state_bit(&name).is_none() {
                // The game logs this at the content log's error level; the state is ignored
                // and the material loads: a kept error.
                let code = codes::STATE_UNKNOWN.with([name.as_str()]);
                read.push(Diagnostic::error(code).at(node.path().clone()).into_kept());
            }
            Some(name)
        })
    })
}

/// `defines`, `+defines`, `-defines`: free strings.
fn defines(node: &Node<'_>, owner: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<String>> {
    listed(node, owner, read, |read| {
        Vec::<String>::from_json(node, read)
    })
}

/// `samplerStates`, `+samplerStates`: the sampler entries the client applies.
fn samplers(node: &Node<'_>, owner: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<SamplerState>> {
    listed(node, owner, read, |read| {
        Vec::<SamplerState>::from_json(node, read)
    })
}

/// `-samplerStates`: the client asserts on the key and then ignores it, in both modes.
fn unapplied_samplers(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<SamplerState>> {
    read.push(info(Code::from(&core_codes::UNRECOGNIZED_FIELD), node));
    unread(node, read, |read| {
        Vec::<SamplerState>::from_json(node, read)
    })
}

/// `-variants`: the client never looks the key up.
fn unread_variants(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<Bodies>> {
    read.push(info(Code::from(&core_codes::UNRECOGNIZED_FIELD), node));
    unread(node, read, |read| Vec::<Bodies>::from_json(node, read))
}

/// A material's member no field takes: kept, and told only when it is a prefixed spelling of
/// a key that has none, or a misspelling of `variants`.
pub(super) fn def_unknown(key: &str, _: &Node<'_>, _: &mut Read<'_>) -> UnknownRule {
    let bare = unprefixed(key);
    let prefixed_plain =
        bare.is_some_and(|bare| PLAIN_KEYS.contains(&bare) || bare == IS_ANIMATED_TEXTURE);
    if prefixed_plain || VARIANT_MISSPELLINGS.contains(&bare.unwrap_or(key)) {
        UnknownRule::Keep
    } else {
        UnknownRule::Silent
    }
}

/// A face's member no field takes: told when it is a prefixed spelling of a face key.
pub(super) fn face_unknown(key: &str, _: &Node<'_>, _: &mut Read<'_>) -> UnknownRule {
    if unprefixed(key).is_some_and(|key| FACE_KEYS.contains(&key)) {
        UnknownRule::Keep
    } else {
        UnknownRule::Silent
    }
}

/// A sampler's member no field takes: told when it is a prefixed spelling of a sampler key.
pub(super) fn sampler_unknown(key: &str, _: &Node<'_>, _: &mut Read<'_>) -> UnknownRule {
    if unprefixed(key).is_some_and(|key| SAMPLER_KEYS.contains(&key)) {
        UnknownRule::Keep
    } else {
        UnknownRule::Silent
    }
}

/// A list entry's member no field takes: kept in silence, but `tag`, which the game logs at
/// the content log's error level as a removed keyword before the entry loads: a kept error.
pub(super) fn entry_unknown(key: &str, node: &Node<'_>, read: &mut Read<'_>) -> UnknownRule {
    if key == "tag" {
        let code = codes::KEY_REMOVED.with([key]);
        read.push(Diagnostic::error(code).at(node.path().clone()).into_kept());
    }
    UnknownRule::Silent
}

/// The keyword at `node`, kept as written. One that is not in `table` is an error that refuses
/// the material (the client reads through the end of its map for it), unless `checked` is
/// false: the client does not look it up there.
fn keyword(node: &Node<'_>, read: &mut Read<'_>, table: &[&str], checked: bool) -> Option<String> {
    let text = String::from_json(node, read)?;
    if checked && !table.contains(&text.as_str()) {
        let code = Code::from(&core_codes::INVALID_VALUE);
        read.push(Diagnostic::error(code).at(node.path().clone()));
    }
    Some(text)
}

/// `depthFunc`, a face's `stencilFunc`.
pub(super) fn comparison(node: &Node<'_>, read: &mut Read<'_>) -> Option<String> {
    keyword(node, read, &COMPARISON_FUNCS, true)
}

/// A face's stencil operations.
pub(super) fn stencil_op(node: &Node<'_>, read: &mut Read<'_>) -> Option<String> {
    keyword(node, read, &STENCIL_OPS, true)
}

/// `blendSrc`, `blendDst`, `alphaSrc`, `alphaDst`.
fn blend(node: &Node<'_>, read: &mut Read<'_>) -> Option<String> {
    keyword(node, read, &BLEND_TARGETS, true)
}

/// `msaaSupport`.
fn msaa(node: &Node<'_>, read: &mut Read<'_>) -> Option<String> {
    keyword(node, read, &MSAA_SUPPORTS, true)
}

/// `primitiveMode`.
fn primitive(node: &Node<'_>, read: &mut Read<'_>) -> Option<String> {
    keyword(node, read, &PRIMITIVE_MODES, true)
}

/// A vertex field's `field`.
pub(super) fn vertex_field(node: &Node<'_>, read: &mut Read<'_>) -> Option<String> {
    keyword(node, read, &VERTEX_FIELDS, true)
}

/// Whether the client skips the sampler entry `owner` without reading its keywords: it reads
/// the index first, and an entry without one (absent, `null`, out of range) is skipped.
fn skipped(owner: &Node<'_>) -> bool {
    let index = owner
        .value()
        .as_object()
        .and_then(|object| object.get("samplerIndex"))
        .and_then(Value::as_number)
        .and_then(whole);
    index.is_none_or(|index| index >= u64::from(SAMPLER_SLOTS))
}

/// A sampler's `textureFilter`.
pub(super) fn texture_filter(
    node: &Node<'_>,
    owner: &Node<'_>,
    read: &mut Read<'_>,
) -> Option<String> {
    keyword(node, read, &TEXTURE_FILTERS, !skipped(owner))
}

/// A sampler's `textureWrap`.
pub(super) fn texture_wrap(
    node: &Node<'_>,
    owner: &Node<'_>,
    read: &mut Read<'_>,
) -> Option<String> {
    keyword(node, read, &TEXTURE_WRAPS, !skipped(owner))
}

/// A sampler's `shaderStage`, which a legacy material does not read.
pub(super) fn shader_stage(
    node: &Node<'_>,
    owner: &Node<'_>,
    read: &mut Read<'_>,
) -> Option<String> {
    let checked = !skipped(owner) && versioned(read);
    keyword(node, read, &SHADER_STAGES, checked)
}

/// A sampler's `comparisonFunc`, which a legacy material does not read.
pub(super) fn sampler_comparison(
    node: &Node<'_>,
    owner: &Node<'_>,
    read: &mut Read<'_>,
) -> Option<String> {
    let checked = !skipped(owner) && versioned(read);
    keyword(node, read, &COMPARISON_FUNCS, checked)
}

/// What the client says of a sampler entry once read: one without an index is skipped (told
/// as missing), one past the slots too.
pub(super) fn sampler_rules(sampler: &mut SamplerState, node: &Node<'_>, read: &mut Read<'_>) {
    match sampler.sampler_index {
        None => read.push(info(Code::from(&core_codes::MISSING_REQUIRED_FIELD), node)),
        // An assertion in a versioned material, silence in a legacy one.
        Some(index) if index >= SAMPLER_SLOTS => {
            let at = Node::new(node.value(), node.path().child("samplerIndex"));
            read.push(info(Code::from(&core_codes::INVALID_VALUE), &at));
        }
        Some(_) => {}
    }
}

/// The whole number from 0 to the largest `u32` at `node` (`asUInt`). A number past the
/// doubles (an infinity here) is out of range like any other, and has its finding from the
/// copy's clamp.
pub(super) fn unsigned(node: &Node<'_>, read: &mut Read<'_>) -> Option<u32> {
    let Some(number) = node.value().as_number() else {
        read.wrong_type(node);
        return None;
    };
    let wide = number.to_f64_lossy();
    if number.is_float() && wide == 0.0 && wide.is_sign_negative() {
        // An unsigned member holds no sign: the file's `-0.0` is held, and written, as 0.
        // The crate notes it; the client reads an unsigned 0 either way.
        read.push(info(Code::from(&core_codes::VALUE_REWRITTEN), node));
    }
    let value = whole(number).and_then(|whole| u32::try_from(whole).ok());
    if value.is_none() && wide.is_finite() {
        read.push(info(Code::from(&core_codes::INVALID_VALUE), node));
    }
    value
}

/// The float at `node`. A number past the range of a float is the largest finite float of its
/// sign, with an information (one past the doubles has the clamp's): never an infinity the
/// writer would refuse.
fn float(node: &Node<'_>, read: &mut Read<'_>) -> Option<f32> {
    let Some(number) = node.value().as_number() else {
        read.wrong_type(node);
        return None;
    };
    let wide = number.to_f64_lossy();
    #[allow(
        clippy::cast_possible_truncation,
        reason = "rounded on purpose, then checked"
    )]
    let narrow = wide as f32;
    if narrow.is_finite() {
        return Some(narrow);
    }
    if wide.is_finite() {
        read.push(info(Code::from(&core_codes::INVALID_VALUE), node));
    }
    Some(if wide < 0.0 { f32::MIN } else { f32::MAX })
}

/// `stencilRefOverride`, which the client's `asBool` reads: a boolean, or a number (vanilla's
/// own `ui.material` writes `0`), kept as it is written.
fn stencil_override(node: &Node<'_>, read: &mut Read<'_>) -> Option<StencilRefOverride> {
    let value = node.value();
    if let Some(flag) = value.as_bool() {
        return Some(StencilRefOverride::Flag(flag));
    }
    let Some(number) = value.as_number() else {
        read.wrong_type(node);
        return None;
    };
    let number = number.to_f64_lossy();
    // A number past the doubles is held as the largest finite one of its sign.
    let number = if number.is_finite() {
        number
    } else if number < 0.0 {
        f64::MIN
    } else {
        f64::MAX
    };
    Some(StencilRefOverride::Number(number))
}

/// `renderTargetFormats`: an array of arrays of format names. An element that is no array is
/// left out; an unknown name is kept with an information (the client reads it as an unknown
/// texture format).
fn formats(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<Vec<String>>> {
    read_each(node, read, |node, read| {
        read_each(node, read, |node, read| {
            let name = String::from_json(node, read)?;
            if !TEXTURE_FORMATS.contains(&name.as_str()) {
                read.push(info(Code::from(&core_codes::INVALID_VALUE), node));
            }
            Some(name)
        })
    })
}

/// Reads the material map `map` (an object) of a file whose wrapper version is `wrapped` and
/// whose name is `stem`. `rules` gives the key rules of a key the stack merge worked out;
/// `None` leaves them to the form. A number past the doubles is an infinity here, its finding
/// the copy's.
///
/// A root with a string `vertexShader` is one material named `stem`, read with the legacy
/// rules (the client stamps no version on it). A member that is no object is no material: it
/// is kept in the file's `extra`, a `null` silently (the merge reads it as "no change") and
/// anything else with an information.
pub(super) fn read_file(
    map: &Value,
    wrapped: Option<SemVersion>,
    stem: &str,
    rules: &dyn Fn(&str) -> Option<KeyRules>,
    out: &mut Diagnostics,
) -> ReadFile {
    let mut materials = IndexMap::new();
    let mut extra = UnknownMembers::default();
    let mut refused = Vec::new();
    let empty = VObject::new();
    let object = map.as_object().unwrap_or(&empty);
    if object.get("vertexShader").is_some_and(Value::is_string) {
        let (mut def, refusing) = read_material(map, JsonPath::new(), KeyRules::Legacy, out);
        // Whatever the merge worked out for a key of that name: nothing is stamped on it.
        def.key_rules = rules(stem).map(|_| KeyRules::Legacy);
        materials.insert(MaterialKey::new(stem), def);
        refused.push(refusing);
        let form = MaterialFileForm::Single {
            stem: stem.to_owned(),
            wrapper: wrapped,
        };
        let file = MaterialFile {
            form,
            materials,
            extra,
        };
        return ReadFile { file, refused };
    }
    let form = wrapped.map_or(MaterialFileForm::Flat, |version| {
        MaterialFileForm::Wrapped { version }
    });
    for (key, value) in object {
        let key = key.as_str();
        if !value.is_object() {
            if !value.is_null() {
                let code = Code::from(&core_codes::INVALID_TYPE);
                out.push(info(code, &Node::new(value, JsonPath::new().child(key))));
            }
            extra.insert(key.to_owned(), finite(value.clone()));
            continue;
        }
        let own = rules(key);
        let at = JsonPath::new().child(key);
        let (mut def, refusing) = read_material(value, at, own.unwrap_or(form.key_rules()), out);
        def.key_rules = own;
        materials.insert(MaterialKey::parse(key), def);
        refused.push(refusing);
    }
    let file = MaterialFile {
        form,
        materials,
        extra,
    };
    ReadFile { file, refused }
}

/// The material `object` at `at`, read with `rules`, and whether it carries a keyword the
/// client cannot load, so that no material is created from it.
fn read_material(
    object: &Value,
    at: JsonPath,
    rules: KeyRules,
    out: &mut Diagnostics,
) -> (MaterialDef, bool) {
    let before = out.len();
    let mut read = reader(out, format_of(rules));
    let def: Option<Def> = read_document(&Node::new(object, at), &mut read);
    let refused = out.iter().skip(before).any(Diagnostic::rejects);
    (def.map_or_default(MaterialDef::from), refused)
}

/// Reads a list file from its root value; `None` when the root is no array.
///
/// An element that is no object, and an entry without a string `path`, are dropped with an
/// information. `tag` is reported with the client's text and the entry loads.
pub(super) fn read_list(
    kind: MaterialListKind,
    root: &Value,
    out: &mut Diagnostics,
) -> Option<MaterialList> {
    read_list_placed(kind, root, out).map(|(list, _)| list)
}

/// One entry of a list, and the index it has in the file.
struct Placed(usize, ListEntry);

impl FromJson for Placed {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let entry = ListEntry::from_json(node, read)?;
        let index = match node.path().segments().last() {
            Some(JsonPathSegment::Index(index)) => index,
            _ => 0,
        };
        Some(Self(index, entry))
    }
}

/// [`read_list`], with the index each entry the list holds has in the file: the reader drops
/// an element that is no object and one that names no file.
pub(super) fn read_list_placed(
    kind: MaterialListKind,
    root: &Value,
    out: &mut Diagnostics,
) -> Option<(MaterialList, Vec<usize>)> {
    root.as_array()?;
    let mut read = reader(out, None);
    let read_entries: Vec<Placed> =
        read_document(&Node::new(root, JsonPath::new()), &mut read).unwrap_or_default();
    let (places, entries) = read_entries
        .into_iter()
        .map(|Placed(at, entry)| (at, entry))
        .unzip();
    Some((MaterialList { kind, entries }, places))
}

/// The finding for the copy at `path` that is no JSON.
pub(super) fn parse_error(path: &PackPath, message: String) -> Diagnostic {
    Diagnostic::of(codes::PARSE_ERROR.with([path.as_str(), message.as_str()]))
        .detail(DiagnosticDetail::Syntax(message))
}

/// The root of a copy that is no object, or of a list that is no array: nothing is read.
pub(super) fn wrong_root() -> Diagnostic {
    let code = Code::from(&core_codes::INVALID_TYPE);
    Diagnostic::error(code)
}

/// The name of the file at `path` without its directory and its extension.
pub(super) fn stem_of(path: &str) -> &str {
    let name = path.rsplit_once('/').map_or(path, |(_, name)| name);
    name.rsplit_once('.').map_or(name, |(stem, _)| stem)
}

/// The file in the root object `root` of a file named `stem`, as a stack of one pack reads it:
/// the wrapper gate, then the material map. `rules` are the key rules a merge worked out.
pub(super) fn read_root(
    root: &VObject,
    stem: &str,
    rules: &dyn Fn(&str) -> Option<KeyRules>,
    out: &mut Diagnostics,
) -> MaterialFile {
    let unwrapped = unwrap_copy(root.clone(), out);
    let mut read = read_file(&unwrapped.map, unwrapped.wrapped, stem, rules, out);
    if let Some(dropped) = unwrapped.dropped {
        read.file.extra.insert(MATERIALS.to_owned(), dropped);
    }
    read.file
}

#[cfg(test)]
mod tests {
    use super::read_copy;
    use bedrock_addon_core::json::MAX_JSON_DEPTH;

    /// A copy nested too deep and not UTF-8 is told too deep, at the offset of the first bracket
    /// past the limit counted in the file as written: a byte-order mark counts.
    #[test]
    fn a_copy_too_deep_and_not_utf8_is_told_too_deep_with_and_without_a_bom() {
        for (bom, offset) in [
            (&b""[..], MAX_JSON_DEPTH),
            (&b"\xEF\xBB\xBF"[..], MAX_JSON_DEPTH + 3),
        ] {
            let mut bytes = bom.to_vec();
            bytes.extend("[".repeat(MAX_JSON_DEPTH + 1).bytes());
            bytes.push(0xff);
            assert_eq!(
                read_copy(&bytes).err(),
                Some(format!(
                    "JSON nesting deeper than the limit at byte {offset}"
                )),
                "bom: {bom:?}"
            );
        }
    }
}
