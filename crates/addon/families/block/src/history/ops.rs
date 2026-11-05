//! JSON operations the legacy steps share.

use indexmap::IndexMap;

use super::legacy::{component_path, number, string};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, Phase, VanillaText};
use bedrock_addon_core::json::Value;

/// A JSON integer.
pub(crate) fn int(n: i64) -> Value {
    Value::from(n)
}

/// A float already truncated, rounded or clamped by a step, as an integer.
#[allow(
    clippy::cast_possible_truncation,
    reason = "the steps clamp or round first; out-of-range floats saturate"
)]
pub(crate) fn whole(n: f64) -> i64 {
    n as i64
}

/// A JSON real number.
pub(crate) fn real(n: f64) -> Value {
    Value::from(n)
}

/// A one-member JSON object.
pub(crate) fn single(key: &str, value: Value) -> Value {
    let mut object = facet_value::VObject::new();
    object.insert(key, value);
    object.into()
}

/// A numeric value clamped into `min..=max`; `ClampedValue` (an information: the game's
/// fixers clamp with no log, so the finding is the crate's own) when it moved.
pub(crate) fn clamp(
    value: f64,
    min: f64,
    max: f64,
    section: Option<usize>,
    name: &str,
    d: &mut Diagnostics,
) -> f64 {
    let clamped = value.clamp(min, max);
    if (clamped - value).abs() > f64::EPSILON {
        d.push(
            Diagnostic::info(DiagnosticCode::ClampedValue { text: None })
                .at(component_path(section, name))
                .phase(Phase::Upgrade),
        );
    }
    clamped
}

/// "Component %s is missing required field %s, component will be discarded"
/// (`DroppedComponent`, a kept error: the game's fixer logs the line at the content log's error
/// level, removes the component and goes on, so the block still loads).
pub(crate) fn missing_field(section: Option<usize>, name: &str, field: &str, d: &mut Diagnostics) {
    d.push_kept_error(
        Diagnostic::error(DiagnosticCode::DroppedComponent {
            text: Some(VanillaText::Formatted {
                template: "Component {} is missing required field {}, component will be discarded",
                args: vec![name.to_owned(), field.to_owned()],
            }),
        })
        .at(component_path(section, name))
        .phase(Phase::Upgrade),
    );
}

/// A member a step drops where the game's fixer drops it with no log (`DroppedField`, an
/// information: the crate's own, since the game says nothing). A drop the game logs is told where it is, as the
/// kept error it is.
pub(crate) fn dropped_field(
    path: &bedrock_addon_core::diagnostic::JsonPath,
    text: Option<VanillaText>,
    d: &mut Diagnostics,
) {
    d.push(
        Diagnostic::info(DiagnosticCode::DroppedField { text })
            .at(path.clone())
            .phase(Phase::Upgrade),
    );
}

/// The value of member `field` of an object, or the value itself when it is not an object:
/// vanilla's 1.19.10 unwrap. `None` when it is an object without the member.
pub(crate) fn unwrap(value: &Value, field: &str) -> Option<Value> {
    match value.as_object() {
        Some(object) => object.get(field).cloned(),
        None => Some(value.clone()),
    }
}

/// The game's render-layer opacity check over a section's `material_instances` JSON: the `*` instance
/// (following string aliases) renders `opaque`, the default.
pub(crate) fn is_opaque(material_instances: Option<&Value>) -> bool {
    let Some(map) = material_instances.and_then(Value::as_object) else {
        return true;
    };
    let mut name = "*".to_owned();
    for _ in 0..64 {
        match map.get(name.as_str()) {
            None => return true,
            Some(entry) => match string(entry) {
                Some(next) => name = next.to_owned(),
                None => {
                    return entry
                        .as_object()
                        .and_then(|o| o.get("render_method"))
                        .and_then(string)
                        .is_none_or(|method| method == "opaque");
                }
            },
        }
    }
    true
}

/// The geometry identifier of a `minecraft:geometry` JSON value.
pub(crate) fn geometry_identifier(value: &Value) -> Option<&str> {
    string(value).or_else(|| value.as_object()?.get("identifier").and_then(string))
}

/// An object member as a mutable ordered map: reads `value` (an object) into a map, applies
/// `f`, and writes it back.
pub(crate) fn edit_object(value: &mut Value, f: impl FnOnce(&mut IndexMap<String, Value>)) {
    if let Some(mut map) = super::legacy::object(value) {
        f(&mut map);
        *value = super::to_object(&map);
    }
}

/// A number member of a component value, for the input checks.
pub(crate) fn number_member(value: &Value, field: &str) -> Option<f64> {
    number(value).or_else(|| value.as_object()?.get(field).and_then(number))
}

#[cfg(test)]
mod unwrap_tests {
    use super::unwrap;
    use bedrock_addon_core::json::Value;

    fn json(text: &str) -> Value {
        bedrock_addon_core::json::from_str::<Value>(text).expect("test JSON")
    }

    #[test]
    fn unwrap_takes_the_member_of_an_object_and_a_scalar_whole() {
        assert_eq!(unwrap(&json(r#"{"a": 1, "b": 2}"#), "a"), Some(json("1")));
        assert_eq!(unwrap(&json("7"), "a"), Some(json("7")));
    }

    #[test]
    fn unwrap_of_an_object_without_the_member_is_none() {
        assert_eq!(unwrap(&json(r#"{"b": 2}"#), "a"), None);
    }
}
