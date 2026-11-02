//! The JSON shape check of the camera and aim-assist documents, run on the document's JSON
//! before its typed parse.
//!
//! Vanilla reads these files with strict cereal loaders: a member the schema does not bind, a
//! value of the wrong JSON type and a missing required member each fail the file. facet reads
//! `null` as an absent `Option`, `[]` as an empty map and coerces scalars, so the token is
//! checked here, against a table of the bound members, and the typed parse runs only on a
//! document that passed. The one non-strict reader is the camera preset's legacy slice below
//! 1.26.50, whose scalar reads [`coerce_scalars`] applies before the check.

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity, VanillaText,
};
use bedrock_addon_core::json::Value;

/// A member bound as an array holds something else.
pub const EXPECTED_ARRAY: &str = "expected an array";
/// A member bound with the required trait is absent.
pub const MISSING_REQUIRED: &str =
    bedrock_addon_core::diagnostic::cereal::MISSING_REQUIRED_FIELD_TEXT;
/// A member the schema does not bind (cereal's `UnrecognizedField` entry).
pub const NOT_IN_SCHEMA: &str = bedrock_addon_core::diagnostic::cereal::MEMBER_NOT_IN_SCHEMA_TEXT;

/// What a member's value must be.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    /// `true` or `false`.
    Bool,
    /// A number. An integer member has this shape too: the typed parse (`StrictInt`) refuses a
    /// real token with the same code at the same path.
    Float,
    /// A string. An enum member has this shape too: the typed parse refuses a name outside the
    /// enum as `InvalidEnumValue` at the same path.
    Str,
    /// An array of any length.
    List(&'static Shape),
    /// An array of exactly this many elements.
    Array(&'static Shape, usize),
    /// An object of bound members.
    Object(&'static [Member]),
    /// An object of any keys.
    Map(&'static Shape),
    /// A component storage: an object whose every key must name a component.
    Components(&'static [Member]),
}

/// One bound member of an object.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Member {
    /// The JSON key.
    pub key: &'static str,
    /// What its value must be.
    pub shape: Shape,
    /// Bound with the required trait.
    pub required: bool,
}

/// A required member.
pub const fn req(key: &'static str, shape: Shape) -> Member {
    Member {
        key,
        shape,
        required: true,
    }
}

/// An optional member.
pub const fn opt(key: &'static str, shape: Shape) -> Member {
    Member {
        key,
        shape,
        required: false,
    }
}

/// A parse finding at `at`.
pub fn finding(code: DiagnosticCode, severity: Severity, at: JsonPath) -> Diagnostic {
    Diagnostic::new(
        code,
        severity,
        Phase::Parse,
        Location::new(None, None, at, None),
    )
}

/// An error-level parse finding at `at`.
pub fn error(code: DiagnosticCode, at: JsonPath) -> Diagnostic {
    finding(code, Severity::Error, at)
}

/// `base` followed by the member `key`.
pub fn child(base: &JsonPath, key: &str) -> JsonPath {
    let mut path = base.clone();
    path.push(key);
    path
}

/// `base` followed by the element `index`.
pub fn element(base: &JsonPath, index: usize) -> JsonPath {
    let mut path = base.clone();
    path.push_index(index);
    path
}

/// A fixed vanilla text.
#[allow(
    clippy::unnecessary_wraps,
    reason = "every caller fills a code's optional text"
)]
pub fn fixed(text: &'static str) -> Option<VanillaText> {
    Some(VanillaText::Fixed(text))
}

/// A wrong JSON type at `at`, with cereal's text when it has one.
fn wrong_type(text: Option<&'static str>, at: &JsonPath, d: &mut Diagnostics) {
    d.push(error(
        DiagnosticCode::InvalidType {
            text: text.map(VanillaText::Fixed),
        },
        at.clone(),
    ));
}

/// The note of the information at a scalar [`coerce_scalars`] rewrote.
pub const SCALAR_READ_AS_BOUND_TYPE: &str = "a number read as a bool (0 is false, any other number true) or a bool read as a number (1 or 0), as vanilla's non-strict reader of presets below 1.26.50 reads it; written back in the member's own type";

/// The scalar reads of the game's non-strict cereal reader (the legacy camera preset slice):
/// in `value`, read as `shape`, a number where a bool is bound becomes `false` for 0 and
/// `true` otherwise, and a bool where a number is bound becomes 1 or 0. Strings take nothing
/// else, so string and enum members are left to the shape check, and so is `null`, which
/// passes neither gate. The paths (below `at`) of the values
/// changed are appended to `changed`, in document order.
pub fn coerce_scalars(
    value: &mut Value,
    shape: &Shape,
    at: &JsonPath,
    changed: &mut Vec<JsonPath>,
) {
    match shape {
        Shape::Bool => {
            if let Some(number) = value.as_number() {
                *value = Value::from(number.to_f64_lossy() != 0.0);
                changed.push(at.clone());
            }
        }
        Shape::Float => {
            if let Some(flag) = value.as_bool() {
                *value = Value::from(if flag { 1.0_f64 } else { 0.0 });
                changed.push(at.clone());
            }
        }
        Shape::Str | Shape::Components(_) => {}
        Shape::List(inner) | Shape::Array(inner, _) => {
            if let Some(array) = value.as_array_mut() {
                for (i, item) in array.iter_mut().enumerate() {
                    coerce_scalars(item, inner, &element(at, i), changed);
                }
            }
        }
        Shape::Map(inner) => {
            if let Some(object) = value.as_object_mut() {
                for (key, item) in object.iter_mut() {
                    let at = child(at, key.as_str());
                    coerce_scalars(item, inner, &at, changed);
                }
            }
        }
        Shape::Object(members) => {
            if let Some(object) = value.as_object_mut() {
                for (key, item) in object.iter_mut() {
                    if let Some(member) = members.iter().find(|m| m.key == key.as_str()) {
                        let at = child(at, key.as_str());
                        coerce_scalars(item, &member.shape, &at, changed);
                    }
                }
            }
        }
    }
}

/// Checks `value` against `shape`, reporting every failure as an error below `at`. `unknown` is
/// the text of a member the schema does not bind. Returns whether the value passed.
pub fn check(
    value: &Value,
    shape: &Shape,
    at: &JsonPath,
    unknown: &'static str,
    d: &mut Diagnostics,
) -> bool {
    match shape {
        Shape::Bool => {
            if value.as_bool().is_some() {
                return true;
            }
            wrong_type(None, at, d);
            false
        }
        Shape::Float => {
            if value.as_number().is_some() {
                return true;
            }
            wrong_type(None, at, d);
            false
        }
        Shape::Str => {
            if value.as_string().is_some() {
                return true;
            }
            wrong_type(None, at, d);
            false
        }
        Shape::List(inner) => {
            let Some(array) = value.as_array() else {
                wrong_type(Some(EXPECTED_ARRAY), at, d);
                return false;
            };
            let mut passed = true;
            for (i, item) in array.iter().enumerate() {
                passed &= check(item, inner, &element(at, i), unknown, d);
            }
            passed
        }
        Shape::Array(inner, length) => {
            let Some(array) = value.as_array() else {
                wrong_type(Some(EXPECTED_ARRAY), at, d);
                return false;
            };
            if array.len() != *length {
                wrong_type(None, at, d);
                return false;
            }
            let mut passed = true;
            for (i, item) in array.iter().enumerate() {
                passed &= check(item, inner, &element(at, i), unknown, d);
            }
            passed
        }
        Shape::Map(inner) => {
            let Some(object) = value.as_object() else {
                d.push(error(
                    bedrock_addon_core::diagnostic::cereal::expected_object(),
                    at.clone(),
                ));
                return false;
            };
            let mut passed = true;
            for (key, item) in object {
                passed &= check(item, inner, &child(at, key.as_str()), unknown, d);
            }
            passed
        }
        Shape::Object(members) => object(value, members, at, unknown, false, d),
        Shape::Components(members) => object(value, members, at, unknown, true, d),
    }
}

/// Checks an object of bound members: each present member against its shape, each unbound one
/// an `UnrecognizedField` (an `UnrecognizedComponent` in a component storage), each absent
/// required one a `MissingRequiredField`.
fn object(
    value: &Value,
    members: &[Member],
    at: &JsonPath,
    unknown: &'static str,
    components: bool,
    d: &mut Diagnostics,
) -> bool {
    let Some(object) = value.as_object() else {
        d.push(error(
            bedrock_addon_core::diagnostic::cereal::expected_object(),
            at.clone(),
        ));
        return false;
    };
    let mut passed = true;
    for (key, item) in object {
        let at = child(at, key.as_str());
        if let Some(member) = members.iter().find(|member| member.key == key.as_str()) {
            passed &= check(item, &member.shape, &at, unknown, d);
        } else {
            let code = if components {
                DiagnosticCode::UnrecognizedComponent { text: None }
            } else {
                DiagnosticCode::UnrecognizedField {
                    text: fixed(unknown),
                }
            };
            d.push(error(code, at));
            passed = false;
        }
    }
    for member in members {
        if member.required && object.get(member.key).is_none() {
            d.push(error(
                DiagnosticCode::MissingRequiredField {
                    text: fixed(MISSING_REQUIRED),
                },
                child(at, member.key),
            ));
            passed = false;
        }
    }
    passed
}

/// `^\w+$`: at least one ASCII letter, digit or underscore and nothing else.
fn word(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

/// `^\w+:\w+$`: a namespaced name.
pub fn namespace_name(text: &str) -> bool {
    text.split_once(':')
        .is_some_and(|(namespace, name)| word(namespace) && word(name))
}

/// `BasicNumericConstraint` `[min, max]`, both ends included: an `InvalidValue` error with
/// cereal's range text when `value` is outside.
// The range text is the game's shared one: bounds in brackets, a whole bound without a
// fraction (`[0, 100]`), a fractional bound printed with six decimals (`%f`).
pub fn in_range(value: f64, min: f64, max: f64, at: JsonPath, d: &mut Diagnostics) -> bool {
    let range = bedrock_addon_core::diagnostic::cereal::NumericRange::inclusive(min, max);
    if range.contains(value) {
        return true;
    }
    d.push(error(range.code(), at));
    false
}

/// A constraint with its own text: an `InvalidValue` error at `at`.
pub fn violation(text: &'static str, at: JsonPath, d: &mut Diagnostics) {
    d.push(error(
        DiagnosticCode::InvalidValue { text: fixed(text) },
        at,
    ));
}
