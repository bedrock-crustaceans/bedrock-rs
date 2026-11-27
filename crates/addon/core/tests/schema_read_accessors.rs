//! The jsoncpp accessors of the spawn files' hand parsers: `Scalars::Accessors`.

use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath, Severity};
use bedrock_addon_core::json::kit::{Node, Profile, Read};
use bedrock_addon_core::json::{Value, from_str_jsonc};
use bedrock_addon_core::schema::{FromJson, Schema};
use bedrock_addon_core::versioning::{NullHandling, Reading, Scalars};
use facet::Facet;

/// Reads `text` as `T` under the accessors told at `severity`, every failure told: the value and
/// each finding as `severity path code`.
fn read_accessors<T: FromJson>(text: &str, severity: Severity) -> (Option<T>, Vec<String>) {
    let json: Value = from_str_jsonc(text).expect("JSON");
    let node = Node::new(&json, JsonPath::of_keys(["root"]));
    let mut found = Diagnostics::default();
    let reading = Reading::OWN
        .with_scalars(Scalars::Accessors(severity))
        .with_null(NullHandling::AbsentSilently);
    let mut read = Read::new(reading, &mut found).profile(Profile::EVERY_FAILURE);
    let value = T::from_json(&node, &mut read);
    read.finish();
    let told = found
        .iter()
        .map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code.id()))
        .collect();
    (value, told)
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Coerced {
    #[facet(default)]
    count: i32,
    #[facet(default)]
    distance: u32,
    #[facet(default)]
    age: u64,
    #[facet(default)]
    size: f32,
    #[facet(default)]
    on: bool,
    #[facet(default)]
    name: String,
    #[facet(default)]
    frames: Vec<i32>,
}

/// A value of another JSON type than the member's is coerced as the accessor coerces it and told
/// `INVALID_TYPE`: a real truncated (an `asInt64` one through 32 bits), a bool 0 or 1, a number
/// for a bool `!= 0`, a bool for a text its name, anything else the type's zero; a `null` that
/// reaches the read too.
#[test]
fn a_value_of_another_type_is_coerced_and_told_invalid_type() {
    let (value, told) = read_accessors::<Coerced>(
        r#"{"count": 2.9, "distance": true, "age": -1.5, "size": "1", "on": 5, "name": false, "frames": [null, "3", -2.9]}"#,
        Severity::Info,
    );
    assert_eq!(
        value,
        Some(Coerced {
            count: 2,
            distance: 1,
            age: u64::MAX,
            size: 0.0,
            on: true,
            name: "false".to_owned(),
            frames: vec![0, 0, -2],
        })
    );
    assert_eq!(
        told,
        [
            "Info root/count core/invalid_type",
            "Info root/distance core/invalid_type",
            "Info root/age core/invalid_type",
            "Info root/size core/invalid_type",
            "Info root/on core/invalid_type",
            "Info root/name core/invalid_type",
            "Info root/frames/0 core/invalid_type",
            "Info root/frames/1 core/invalid_type",
            "Info root/frames/2 core/invalid_type",
        ]
    );
}

/// An integer the member's width does not hold keeps its low bits and is told `INVALID_VALUE`;
/// a number beyond a float is the largest finite one, told the same.
#[test]
fn an_integer_beyond_the_width_keeps_its_low_bits_and_is_told_invalid_value() {
    let (value, told) = read_accessors::<Coerced>(
        r#"{"count": 4294967296, "distance": -1, "age": -1, "size": 1e39, "frames": [2147483648]}"#,
        Severity::Info,
    );
    assert_eq!(
        value,
        Some(Coerced {
            count: 0,
            distance: u32::MAX,
            age: u64::MAX,
            size: f32::MAX,
            frames: vec![i32::MIN],
            ..Coerced::default()
        })
    );
    assert_eq!(
        told,
        [
            "Info root/count core/invalid_value",
            "Info root/distance core/invalid_value",
            "Info root/age core/invalid_value",
            "Info root/size core/invalid_value",
            "Info root/frames/0 core/invalid_value",
        ]
    );
}

/// A value of the member's own type in its width is no finding, and a `null` member is absent
/// under the silent null rule.
#[test]
fn a_value_of_the_members_own_type_is_quiet() {
    let (value, told) = read_accessors::<Coerced>(
        r#"{"count": -7, "distance": 4294967295, "age": 18446744073709551615, "size": 2, "on": false, "name": "n", "frames": null}"#,
        Severity::Info,
    );
    assert_eq!(
        value,
        Some(Coerced {
            count: -7,
            distance: u32::MAX,
            age: u64::MAX,
            size: 2.0,
            on: false,
            name: "n".to_owned(),
            frames: Vec::new(),
        })
    );
    assert!(told.is_empty(), "{told:?}");
}

/// The flavour tells at the severity it names.
#[test]
fn the_accessors_tell_at_their_severity() {
    let (_, told) = read_accessors::<Coerced>(
        r#"{"count": "7", "distance": 4294967296}"#,
        Severity::Warning,
    );
    assert_eq!(
        told,
        [
            "Warning root/count core/invalid_type",
            "Warning root/distance core/invalid_value",
        ]
    );
}
