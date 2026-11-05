//! Reading an atomic client entity as the game's cereal parse does under its strict error mask:
//! typed, with every unknown member, wrong type and failed constraint an error.

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, JsonPathSegment, Severity, VanillaText,
};
use bedrock_addon_core::json::{
    ClampedText, EncodeError, JsonSlice, NullRule, ScalarFailure, ScalarRead, Value, Width,
    clamp_text, parse_with_rules, reject_nulls, unknown_members,
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
        // The scalars are read by the slice's rules (`Family::reading`: strict, a number beyond
        // its float a type error).
        let read = parse_with_rules::<Self>(payload, cx.load.null_rule, cx.load.scalar_rules);
        let entity = match read {
            Ok(ScalarRead { value, .. }) => {
                // What the pre-pass tells of a conversion is not pushed: under these rules it tells only
                // a `null` read as a struct, which the typed read never reported (the null rule does).
                value
            }
            Err(ScalarFailure::Refused(misread)) => {
                d.push(misread.finding(&cx.at));
                return None;
            }
            Err(ScalarFailure::Read(mut error)) => {
                // The parse read the clamped text; the finding is of the file as written.
                if let Some(clamped) = &clamped {
                    error.span = clamped.written_span(error.span, payload.offset);
                }
                // A wrong type is the declared value's, however deep the reader got into it.
                // The code reads the slice's own blank flag where the old code forced `false`:
                // a member slice cannot be blank, so the result is equal.
                if matches!(
                    error.code_with_path(error.has_path()),
                    DiagnosticCode::InvalidType { .. }
                ) {
                    error.path = declared(&error.path);
                }
                d.push(error.into_diagnostic(&cx.at, |code, _| code));
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
                d.push(Diagnostic::error(code).at(at));
            }
        }
        let description_at = cx.at.child("description");
        if !has_namespace(entity.description.identifier.as_str()) {
            // A namespace is required: the identifier matches `^(?:.)+:(?:.)+$`.
            d.push(
                Diagnostic::error(DiagnosticCode::InvalidValue {
                    text: Some(VanillaText::Fixed(NAMESPACE_TEXT)),
                })
                .at(description_at.child("identifier")),
            );
        }
        let controllers = entity.description.render_controllers.iter().flatten();
        for (index, controller) in controllers.enumerate() {
            if matches!(controller, ControllerUse::Conditional(_)) {
                // The list holds plain strings; the `{ name: condition }` form is no string.
                let at = description_at.child("render_controllers").index(index);
                d.push(Diagnostic::error(DiagnosticCode::InvalidType { text: None }).at(at));
            }
        }
        for (members, value) in not_negative(&entity) {
            // The minimum is 0.0, inclusive.
            if value < 0.0 {
                d.push(
                    Diagnostic::error(DiagnosticCode::InvalidValue {
                        text: Some(VanillaText::Fixed(NOT_NEGATIVE_TEXT)),
                    })
                    .at(cx.at.with(&members)),
                );
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
        found.push((vec!["components", AABB, "width"], aabb.width));
        found.push((vec!["components", AABB, "height"], aabb.height));
    }
    if let Some(clipping) = &components.companion_block_clipping {
        for (name, value) in [
            ("min_hidden_seconds", clipping.min_hidden_seconds),
            ("min_visible_seconds", clipping.min_visible_seconds),
            ("hide_delay_seconds", clipping.hide_delay_seconds),
            ("hide_transition_seconds", clipping.hide_transition_seconds),
        ] {
            if let Some(value) = value {
                found.push((vec!["components", CLIPPING, name], value));
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
                    found.push((vec!["components", CLIPPING, boundary, name], value));
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
