//! Unit tests of the render controller internals the integration tests cannot name.

use super::read::{MAXIMUM, V1_8_0, controller_of, controller_path, read_pair};

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

/// What `read_pair` makes of `text` at the member `offset`: whether it read a pair, and its findings as
/// `name at path: text`, in the order they are told.
fn pair(text: &str) -> (bool, Vec<String>) {
    use bedrock_addon_core::json::node_tree::{Node, Walk};
    let mut walk = Walk::new(
        super::read::RULES,
        &bedrock_addon_core::diagnostic::JsonPath::of_keys(["offset"]),
    );
    let node = Node::of(bedrock_addon_core::json::JsonSlice::new(text)).unwrap();
    let read = read_pair(&mut walk, node).is_some();
    let findings = walk
        .diagnostics
        .iter()
        .map(|d| format!("{} at {}: {}", d.code.name(), d.location.json_path, d.code))
        .collect();
    (read, findings)
}

/// A pair of expressions is told of by its elements first, each at its index, then by its
/// length; what is no array is the type error alone.
#[test]
fn a_pair_of_expressions_is_told_of_by_its_elements_then_by_its_length() {
    const NO_ARRAY: &str =
        "InvalidType at offset: unknown child schema option type.  Allowed types: array";
    assert_eq!(pair(r#"[1, "q.x"]"#), (true, vec![]));
    let element = |index: usize| {
        format!(
            "InvalidType at offset/{index}: unknown child schema option type.  Allowed types: ExpressionNode"
        )
    };
    let length = |text: &str| format!("InvalidValue at offset: Array too {text}");
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
        let (read, told) = pair(text);
        assert!(!read, "{text}");
        assert_eq!(told, findings, "{text}");
    }
}

/// W2: under the family's rules a member that failed fails its object, and the required check
/// is then not made (the opposite of geometry, which judges by the errors raised).
#[test]
fn a_failed_member_fails_the_object_and_the_required_check_is_not_made() {
    use bedrock_addon_core::diagnostic::JsonPath;
    use bedrock_addon_core::json::node_tree::{Group, Hit, Node, Walk};
    let groups = [Group::name("a"), Group::required("b")];
    let mut walk = Walk::new(super::read::RULES, &JsonPath::new());
    let ok = walk.object(Node::root(r#"{"a": 1}"#).unwrap(), &groups, |_, _, _, _| {
        Hit::Failed
    });
    assert!(!ok, "a member that failed must fail the object");
    let findings: Vec<_> = walk.diagnostics.iter().map(|d| d.code.name()).collect();
    assert!(
        findings.is_empty(),
        "the required check must not run after a failed member: {findings:?}"
    );
}

/// W3: a number a float does not hold is told, as a `LossyConversion` information.
#[test]
fn a_float_that_narrows_is_told_as_lossy() {
    use bedrock_addon_core::diagnostic::{JsonPath, Severity};
    use bedrock_addon_core::json::node_tree::{Node, Walk};
    let mut walk = Walk::new(super::read::RULES, &JsonPath::new());
    walk.enter("x");
    assert_eq!(
        walk.float(Node::root("0.123456789012").unwrap()),
        Some(0.123_456_79)
    );
    let findings: Vec<_> = walk
        .diagnostics
        .iter()
        .map(|d| (d.severity, d.code.name(), d.location.json_path.to_string()))
        .collect();
    assert_eq!(
        findings,
        [(Severity::Info, "LossyConversion", "x".to_owned())]
    );
}
