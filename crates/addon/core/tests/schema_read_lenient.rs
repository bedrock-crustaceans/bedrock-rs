//! The jsoncpp variant of the texture files: `Scalars::Lenient`.

use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
use bedrock_addon_core::json::kit::{Node, Profile, Read};
use bedrock_addon_core::json::{Value, from_str_jsonc};
use bedrock_addon_core::schema::{FromJson, Schema};
use bedrock_addon_core::versioning::{Reading, Scalars};
use facet::Facet;

/// Reads `text` as `T` under the lenient profile, every failure told: the value and each
/// finding as `severity path code`.
fn read_lenient<T: FromJson>(text: &str) -> (Option<T>, Vec<String>) {
    let json: Value = from_str_jsonc(text).expect("JSON");
    let node = Node::new(&json, JsonPath::of_keys(["root"]));
    let mut found = Diagnostics::default();
    let mut read = Read::new(Reading::OWN.with_scalars(Scalars::Lenient), &mut found)
        .profile(Profile::EVERY_FAILURE);
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
    size: f32,
    #[facet(default)]
    on: bool,
    #[facet(default)]
    count: i32,
    #[facet(default)]
    name: String,
    #[facet(default)]
    frames: Vec<i32>,
}

/// The texture files' reader coerces every member and refuses none, and tells what is not of the
/// member's own type: a real or a bool for an integer, a bool or a string for a float, a string
/// for a bool, a number or a bool for a text; an integer past 32 bits keeps its low word.
#[test]
fn the_lenient_profile_coerces_every_member_and_tells_what_is_of_another_type() {
    let (value, told) = read_lenient::<Coerced>(
        r#"{"size": true, "on": "false", "count": 4294967297, "name": 5, "frames": [2.9, null, 3]}"#,
    );
    assert_eq!(
        value,
        Some(Coerced {
            size: 1.0,
            on: true,
            count: 1,
            name: String::new(),
            frames: vec![2, 0, 3],
        })
    );
    assert_eq!(
        told,
        [
            "Info root/size core/invalid_value",
            "Info root/on core/invalid_value",
            "Info root/count core/invalid_value",
            "Info root/name core/invalid_value",
            "Info root/frames/0 core/invalid_value",
            "Info root/frames/1 core/invalid_value",
        ]
    );
}

/// A value of the member's own kind is no finding: an integer in 32 bits, any number for a float
/// or a bool, a string for a text; a bool for a text is its name, and a number beyond a float is
/// the largest finite one, told.
#[test]
fn the_lenient_profile_is_quiet_for_the_members_own_kind() {
    let (value, told) = read_lenient::<Coerced>(
        r#"{"size": 2, "on": 1.5, "count": -7, "name": "n", "frames": [1]}"#,
    );
    assert_eq!(
        value,
        Some(Coerced {
            size: 2.0,
            on: true,
            count: -7,
            name: "n".to_owned(),
            frames: vec![1],
        })
    );
    assert!(told.is_empty(), "{told:?}");
    let (value, told) = read_lenient::<Coerced>(r#"{"size": 1e39, "name": true}"#);
    let value = value.expect("read");
    assert_eq!((value.size, value.name.as_str()), (f32::MAX, "true"));
    assert_eq!(
        told,
        [
            "Info root/size core/invalid_value",
            "Info root/name core/invalid_value"
        ]
    );
}
