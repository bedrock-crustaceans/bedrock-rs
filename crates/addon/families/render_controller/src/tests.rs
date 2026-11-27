//! Unit tests of the render controller internals the integration tests cannot name.

use super::read::{MAXIMUM, PROFILE, V1_8_0, controller_of, controller_path, pair};
use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
use bedrock_addon_core::json::kit::{Node, Read, Texts};
use bedrock_addon_core::json::parse::{Options, parse};
use bedrock_addon_core::schema::{FromJson, Schema, read_document};
use bedrock_addon_core::versioning::Reading;
use facet::Facet;

/// `text` read as `T` at the member `at` under the schema's read: the value and the findings
/// as `name at path: text`, in the order they are told.
fn read_as<T: FromJson>(text: &str, at: &[&str], root: bool) -> (Option<T>, Vec<String>) {
    let parsed = parse(text, Options::default()).unwrap();
    let mut found = Diagnostics::default();
    let mut read = Read::new(Reading::TREE, &mut found)
        .profile(PROFILE)
        .texts(Texts::TREE);
    let node = Node::parsed(&parsed, JsonPath::of_keys(at.iter().copied()));
    let value = if root {
        read_document::<T>(&node, &mut read)
    } else {
        T::from_json(&node, &mut read)
    };
    drop(read);
    let told = found
        .iter()
        .map(|d| format!("{} at {}: {}", d.code.id(), d.location.json_path, d.code))
        .collect();
    (value, told)
}

#[test]
fn the_schema_bounds_are_1_8_0_and_65535_0_0() {
    assert_eq!(V1_8_0.to_string(), "1.8.0");
    assert_eq!(MAXIMUM.to_string(), "65535.0.0");
}

#[test]
fn a_finding_below_a_controller_names_it() {
    let at = controller_path("controller.render.a", Some("textures"));
    assert_eq!(
        at.to_string(),
        "render_controllers/controller.render.a/textures"
    );
    assert_eq!(controller_of(&at), Some("controller.render.a"));
    assert_eq!(controller_of(&controller_path("x", None)), Some("x"));
    assert_eq!(
        controller_of(&bedrock_addon_core::diagnostic::JsonPath::new()),
        None
    );
}

/// What `pair` makes of `text` at the member `offset`: whether it read a pair, and its findings
/// as `name at path: text`, in the order they are told.
fn pair_of(text: &str) -> (bool, Vec<String>) {
    let parsed = parse(text, Options::default()).unwrap();
    let mut found = Diagnostics::default();
    let mut read = Read::new(Reading::TREE, &mut found)
        .profile(PROFILE)
        .texts(Texts::TREE);
    let read_it = pair(
        &Node::parsed(&parsed, JsonPath::of_keys(["offset"])),
        &mut read,
    )
    .is_some();
    drop(read);
    let findings = found
        .iter()
        .map(|d| format!("{} at {}: {}", d.code.id(), d.location.json_path, d.code))
        .collect();
    (read_it, findings)
}

/// A pair of expressions is told of by its elements first, each at its index, then by its
/// length; what is no array is the type error alone.
#[test]
fn a_pair_of_expressions_is_told_of_by_its_elements_then_by_its_length() {
    const NO_ARRAY: &str = "core/unknown_child_option at offset: unknown child schema option type.  Allowed types: array";
    assert_eq!(pair_of(r#"[1, "q.x"]"#), (true, vec![]));
    let element = |index: usize| {
        format!(
            "core/unknown_child_option at offset/{index}: unknown child schema option type.  Allowed types: ExpressionNode"
        )
    };
    let length = |text: &str| {
        let id = if text.starts_with("small") {
            "core/array_too_small"
        } else {
            "core/array_too_large"
        };
        format!("{id} at offset: Array too {text}")
    };
    for (text, findings) in [
        ("7", vec![NO_ARRAY.to_owned()]),
        ("{}", vec![NO_ARRAY.to_owned()]),
        ("[]", vec![length("small (0 < 2)")]),
        ("[1]", vec![length("small (1 < 2)")]),
        ("[{}]", vec![element(0), length("small (1 < 2)")]),
        ("[1, {}]", vec![element(1)]),
        ("[[], {}]", vec![element(0), element(1)]),
        ("[1, 2, 3]", vec![length("large (3 > 2)")]),
        (
            "[{}, 2, []]",
            vec![element(0), element(2), length("large (3 > 2)")],
        ),
        ("[1, 2, 3, {}]", vec![element(3), length("large (4 > 2)")]),
    ] {
        let (read, told) = pair_of(text);
        assert!(!read, "{text}");
        assert_eq!(told, findings, "{text}");
    }
}

/// Two members, the second required.
#[derive(Debug, PartialEq, Facet, Schema)]
struct Two {
    a: Option<bool>,
    b: bool,
}

/// One float.
#[derive(Debug, PartialEq, Facet, Schema)]
struct Float {
    x: f32,
}

/// W2: under the family's rules a member that failed fails its object, and the required check
/// is then not made (as geometry, the object is judged by the errors raised below it).
#[test]
fn a_failed_member_fails_the_object_and_the_required_check_is_not_made() {
    let (value, told) = read_as::<Two>(r#"{"a": 1}"#, &[], true);
    assert_eq!(value, None, "a member that failed must fail the object");
    assert_eq!(
        told,
        ["core/unknown_child_option at a: unknown child schema option type.  Allowed types: bool"],
        "the required check must not run after a failed member"
    );
}

/// W3: a number a float does not hold is told, as a `LossyConversion` information.
#[test]
fn a_float_that_narrows_is_told_as_lossy() {
    let (value, told) = read_as::<Float>(r#"{"x": 0.123456789012}"#, &[], true);
    assert_eq!(value, Some(Float { x: 0.123_456_79 }));
    assert_eq!(
        told,
        ["core/lossy_conversion at x: value loses precision when written"]
    );
}
