//! Reading an atomic client entity as the game's cereal parse does under its strict error mask:
//! typed, with every unknown member, wrong type and failed constraint an error.

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, JsonPathSegment, Location, Phase, Severity,
    VanillaText, code_for_deserialize_error_kind,
};
use bedrock_addon_core::json::{
    ClampedText, EncodeError, JsonSlice, NullRule, Value, Width, clamp_text, parse_at,
    reject_nulls, unknown_members,
};
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    BETA_VERSION, ParseCx, VersionStruct, Versioned, write_payload,
};
use bedrock_addon_core::visual::ControllerUse;
use facet::Facet;

use super::AtomicClientEntity;

/// The text of the namespace constraint.
const NAMESPACE_TEXT: &str = "string must be prefixed with a namespace (eg. namespace:value)";
/// The text of a numeric constraint with a minimum of 0.
const NOT_NEGATIVE_TEXT: &str = "the input must be a number >= 0";

const AABB: &str = "minecraft:aabb_dimensions";
const CLIPPING: &str = "minecraft:companion_block_clipping";

/// The text of `slice` (whose value is `json`) with every number at one of `paths` that is
/// beyond `f32` replaced by the largest finite float of its sign, each with an `InvalidValue`
/// warning at its path below `at`; `None` when no number is.
fn finite_floats(
    slice: JsonSlice<'_>,
    json: &Value,
    paths: &[&[&str]],
    at: &JsonPath,
    d: &mut Diagnostics,
) -> Option<ClampedText> {
    clamp_text(
        slice,
        json,
        &|path| is_float(paths, path).then_some(Width::Float),
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

/// The float members of the document, each constrained to a minimum of 0: the AABB's two, the
/// clipping component's four and the three of each of its boundaries. A number beyond `f32` at one
/// of them is read as the largest finite float. The offsets of `companion_block_avoidance` are
/// elements, not members, and have no constraint: a number beyond `f32` there is a type error.
const FLOATS: [&[&str]; 12] = [
    &["components", AABB, "width"],
    &["components", AABB, "height"],
    &["components", CLIPPING, "min_hidden_seconds"],
    &["components", CLIPPING, "min_visible_seconds"],
    &["components", CLIPPING, "hide_delay_seconds"],
    &["components", CLIPPING, "hide_transition_seconds"],
    &["components", CLIPPING, "reveal_boundary", "margin"],
    &[
        "components",
        CLIPPING,
        "reveal_boundary",
        "lookahead_per_speed",
    ],
    &["components", CLIPPING, "reveal_boundary", "lookahead_max"],
    &["components", CLIPPING, "hide_boundary", "margin"],
    &[
        "components",
        CLIPPING,
        "hide_boundary",
        "lookahead_per_speed",
    ],
    &["components", CLIPPING, "hide_boundary", "lookahead_max"],
];

impl Versioned for AtomicClientEntity {
    const VERSION: FormatVersion = BETA_VERSION;
}

/// The three script lists are plain strings; nothing compiles them at load.
impl MolangFields for AtomicClientEntity {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

impl VersionStruct for AtomicClientEntity {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        // The strict error mask applies: every cereal entry is fatal.
        // The payload is parsed to a `Value` once: the float clamp and the unknown-member
        // scan read this one.
        let json = bedrock_addon_core::json::from_str_jsonc::<Value>(payload.text).ok();
        let clamped = json
            .as_ref()
            .and_then(|json| finite_floats(payload, json, &FLOATS, &cx.at, d));
        let payload = clamped.as_ref().map_or(payload, |clamped| JsonSlice {
            text: &clamped.text,
            offset: payload.offset,
        });
        let entity = match parse_at::<Self>(payload) {
            Ok(entity) => entity,
            Err(mut error) => {
                // The parse read the clamped text; the finding is of the file as written.
                if let Some(clamped) = &clamped {
                    error.span = clamped.written_span(error.span, payload.offset);
                }
                let code = code_for_deserialize_error_kind(
                    &error.error.kind,
                    !error.path.is_empty(),
                    false,
                );
                // A wrong type is the declared value's, however deep the reader got into it.
                let path = match code {
                    DiagnosticCode::InvalidType { .. } => declared(&error.path),
                    _ => error.path.clone(),
                };
                let mut at = cx.at.clone();
                at.append(&path);
                let mut found = finding(code, at);
                found.location.span = Some(error.span);
                d.push(found);
                return None;
            }
        };
        // The loader's input is a document input: a member written `null`, an optional one
        // included, is present and fails its type.
        if cx.load.null_rule == Some(NullRule::Rejected)
            && reject_nulls(&entity, payload, &cx.at, d)
        {
            return None;
        }
        let before = d
            .iter()
            .filter(|found| found.severity == Severity::Error)
            .count();
        if let Some(json) = &json {
            for (path, _) in unknown_members(<Self as Facet>::SHAPE, json) {
                let mut at = cx.at.clone();
                at.append(&path);
                // A key of `components` is a component; anywhere else it is a member.
                let code = if is_component(&path) {
                    DiagnosticCode::UnrecognizedComponent { text: None }
                } else {
                    DiagnosticCode::UnrecognizedField { text: None }
                };
                d.push(finding(code, at));
            }
        }
        let description_at = below(&cx.at, &["description"]);
        if !has_namespace(entity.description.identifier.as_str()) {
            // A namespace is required: the identifier matches `^(?:.)+:(?:.)+$`.
            d.push(finding(
                DiagnosticCode::InvalidValue {
                    text: Some(VanillaText::Fixed(NAMESPACE_TEXT)),
                },
                below(&description_at, &["identifier"]),
            ));
        }
        let controllers = entity.description.render_controllers.iter().flatten();
        for (index, controller) in controllers.enumerate() {
            if matches!(controller, ControllerUse::Conditional(_)) {
                // The list holds plain strings; the `{ name: condition }` form is no string.
                let mut at = below(&description_at, &["render_controllers"]);
                at.push_index(index);
                d.push(finding(DiagnosticCode::InvalidType { text: None }, at));
            }
        }
        for (members, value) in not_negative(&entity) {
            // The minimum is 0.0, inclusive.
            if value < 0.0 {
                d.push(finding(
                    DiagnosticCode::InvalidValue {
                        text: Some(VanillaText::Fixed(NOT_NEGATIVE_TEXT)),
                    },
                    below(&cx.at, &members),
                ));
            }
        }
        let errors = d
            .iter()
            .filter(|found| found.severity == Severity::Error)
            .count();
        (errors == before).then_some(entity)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}

/// Every float the entity writes that has a minimum of 0, with its members below the payload.
fn not_negative(entity: &AtomicClientEntity) -> Vec<(Vec<&'static str>, f32)> {
    let mut found = Vec::new();
    let Some(components) = &entity.components else {
        return found;
    };
    if let Some(aabb) = &components.aabb_dimensions {
        found.push((vec!["components", AABB, "width"], aabb.width.0));
        found.push((vec!["components", AABB, "height"], aabb.height.0));
    }
    if let Some(clipping) = &components.companion_block_clipping {
        for (name, value) in [
            ("min_hidden_seconds", clipping.min_hidden_seconds),
            ("min_visible_seconds", clipping.min_visible_seconds),
            ("hide_delay_seconds", clipping.hide_delay_seconds),
            ("hide_transition_seconds", clipping.hide_transition_seconds),
        ] {
            if let Some(value) = value {
                found.push((vec!["components", CLIPPING, name], value.0));
            }
        }
        for (boundary, written) in [
            ("reveal_boundary", clipping.reveal_boundary),
            ("hide_boundary", clipping.hide_boundary),
        ] {
            let Some(written) = written else { continue };
            for (name, value) in [
                ("margin", written.margin),
                ("lookahead_per_speed", written.lookahead_per_speed),
                ("lookahead_max", written.lookahead_max),
            ] {
                if let Some(value) = value {
                    found.push((vec!["components", CLIPPING, boundary, name], value.0));
                }
            }
        }
    }
    found
}

/// Whether `identifier` matches `^(?:.)+:(?:.)+$`: a `:` with at least one character on each
/// side.
pub(super) fn has_namespace(identifier: &str) -> bool {
    let count = identifier.chars().count();
    identifier
        .chars()
        .enumerate()
        .any(|(index, c)| c == ':' && index >= 1 && index + 1 < count)
}

/// `path` cut back to the value the schema declares a type for. Every declared value of the
/// document is an element of a list, or lies at most four members deep: a value of a slot
/// map or of `scripts` (three), a member of a component or of a clipping boundary (four).
fn declared(path: &JsonPath) -> JsonPath {
    // A description value lies three members deep (a slot map's value is a name, whatever
    // the reader looks for inside it); a component's member as deep as four.
    let deepest = match path.segments().next() {
        Some(JsonPathSegment::Key("description")) => 3,
        _ => 4,
    };
    let mut cut = JsonPath::new();
    for (depth, segment) in path.segments().enumerate() {
        match segment {
            JsonPathSegment::Key(_) if depth == deepest => break,
            JsonPathSegment::Key(key) => cut.push(key),
            JsonPathSegment::Index(index) => {
                cut.push_index(index);
                break;
            }
        }
    }
    cut
}

/// Whether `path` is a key directly below `components`.
fn is_component(path: &JsonPath) -> bool {
    let mut segments = path.segments();
    segments.len() == 2 && segments.next() == Some(JsonPathSegment::Key("components"))
}

/// An error at `at`.
fn finding(code: DiagnosticCode, at: JsonPath) -> Diagnostic {
    Diagnostic::new(
        code,
        Severity::Error,
        Phase::Parse,
        Location::new(None, None, at, None),
    )
}

/// `base` followed by `keys`.
fn below(base: &JsonPath, keys: &[&str]) -> JsonPath {
    let mut path = base.clone();
    for key in keys {
        path.push(key);
    }
    path
}

#[cfg(test)]
mod tests {
    use super::has_namespace;

    #[test]
    fn a_namespace_needs_a_character_on_each_side_of_a_colon() {
        assert!(has_namespace("demo:wisp"));
        assert!(has_namespace("minecraft:wisp"));
        assert!(has_namespace("a:b:c"));
        assert!(has_namespace(":a:b"));
        assert!(!has_namespace("wisp"));
        assert!(!has_namespace(":wisp"));
        assert!(!has_namespace("demo:"));
        assert!(!has_namespace(":"));
        assert!(!has_namespace(""));
    }
}
