//! e01 (collection), e08 and e08b
//! (an `ExtensionPoint` without a string `attribs.name`). The rows that need the stack are in
//! `stack.rs`.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::families::ddui::{Component, Control, UiRoot};

use super::common::*;
use super::controls::codes;

fn point(name: &str) -> String {
    format!(r#"{{ "component": "ExtensionPoint", "attribs": {{ "name": "{name}" }} }}"#)
}

#[test]
fn e01_extension_points_are_collected_from_the_whole_tree() {
    let text = panel(&format!(
        r#"{{ "component": "Visibility", "children": [ {} ] }},
           {{ "component": "Form.ScrollView", "children": [ {{ "component": "Form.Divider" }}, {} ] }}"#,
        point("demo:slot"),
        point("demo:other")
    ));
    let document = loads(root(&text));
    assert_eq!(document.extension_points(), ["demo:slot", "demo:other"]);
    assert!(document.diagnostics().is_empty());
}

#[test]
fn e01_a_name_declared_twice_is_one_extension_point() {
    let text = panel(&format!(
        "{}, {}, {}",
        point("demo:slot"),
        point("demo:other"),
        point("demo:slot")
    ));
    let document = loads(root(&text));
    assert_eq!(document.extension_points(), ["demo:slot", "demo:other"]);
    // A root without one declares none.
    assert!(loads(root(&r())).extension_points().is_empty());
}

/// From 1.26.60 `ExtensionPoint` is an alternative of `ListContent`, so a point
/// below `Context.List` (or `ScrollableGridLayout`) is read and declared; a 1.26.45 engine
/// refuses the same control there.
#[test]
fn e01_a_point_below_a_context_list_is_declared_from_1_26_60() {
    let text = panel(&format!(
        r#"{{ "component": "Context.List", "children": [ {} ] }},
           {{ "component": "ScrollableGridLayout", "children": [ {} ] }}"#,
        point("demo:listed"),
        point("demo:grid")
    ));
    let document = loads(root_in(&text, &cx(E60, false)));
    assert!(document.diagnostics().is_empty());
    assert_eq!(document.extension_points(), ["demo:listed", "demo:grid"]);
    let d = rejects(root_in(&text, &cx(E45, false)));
    let refused: Vec<_> = found(&d)
        .into_iter()
        .filter(|f| f.0 == Severity::Error && f.1 == "InvalidValue")
        .map(|f| f.2)
        .collect();
    assert_eq!(
        refused,
        [
            "minecraft:ui-root/layout/markup/0/children/0/children/0/component",
            "minecraft:ui-root/layout/markup/0/children/1/children/0/component"
        ]
    );
}

/// The client's collector throws out of its whole load here and loads no composition at all,
/// with no line: a player-facing refusal, so an error that refuses the root. Ours still refuses only the root.
#[test]
fn e08_an_extension_point_without_a_string_name_refuses_the_root_deviation() {
    let text = panel(
        r#"{ "component": "ExtensionPoint" },
           { "component": "ExtensionPoint", "attribs": {} },
           { "component": "ExtensionPoint", "attribs": { "name": 5 } }"#,
    );
    let d = rejects(root(&text));
    let path = |index: usize| format!("minecraft:ui-root/layout/markup/0/children/{index}");
    assert_eq!(
        codes(&d),
        [
            (Severity::Error, "InvalidValue", path(0)),
            (Severity::Error, "InvalidValue", path(1)),
            (Severity::Error, "InvalidValue", path(2)),
        ]
    );
    let spans: Vec<&str> = d
        .iter()
        .filter(|f| f.code.name() == "InvalidValue")
        .map(|f| spanned(&text, f))
        .collect();
    assert_eq!(spans[0], r#"{ "component": "ExtensionPoint" }"#);
    assert_eq!(
        spans[2],
        r#"{ "component": "ExtensionPoint", "attribs": { "name": 5 } }"#
    );
}

/// Every `attribs` that is no object with a string `name`, beside one that is.
#[test]
fn e08_only_a_string_name_in_an_object_declares_a_point() {
    let bad = [
        "null",
        "5",
        r#""demo:slot""#,
        r#"["name"]"#,
        r#"{ "name": null }"#,
        r#"{ "name": ["x"] }"#,
        r#"{ "Name": "x" }"#,
    ];
    let mut controls: Vec<String> = bad
        .iter()
        .map(|attribs| format!(r#"{{ "component": "ExtensionPoint", "attribs": {attribs} }}"#))
        .collect();
    controls.push(point("demo:slot"));
    // An empty string is a string: the client registers the name `""`.
    controls.push(point(""));
    // Each bad one is an error that refuses the root.
    let d = rejects(root(&panel(&controls.join(", "))));
    let refusing = codes(&d);
    assert_eq!(refusing.len(), bad.len(), "{refusing:#?}");
    assert!(
        refusing
            .iter()
            .all(|f| f.0 == Severity::Error && f.1 == "InvalidValue")
    );
    // The good ones alone declare their names.
    let good = [point("demo:slot"), point("")];
    let document = loads(root(&panel(&good.join(", "))));
    assert_eq!(document.extension_points(), ["demo:slot", ""]);
}

#[test]
fn e08b_a_nameless_extension_point_in_a_composition_is_nothing() {
    let text = composition_json(
        "demo:c",
        "",
        r#"{ "component": "ExtensionPoint" }, { "component": "ExtensionPoint", "attribs": {} },
           { "component": "ExtensionPoint", "attribs": { "name": 5 } }"#,
    );
    let document = loads(composition(&text));
    assert!(document.diagnostics().is_empty());
    assert_eq!(document.layout.markup.len(), 3);
}

/// Only an `ExtensionPoint` declares a name: `attribs.name` of any other control is that
/// control's own business, in a loaded tree and in one built by hand.
#[test]
fn a_control_that_is_no_extension_point_declares_nothing_with_its_name() {
    let text = panel(&format!(
        r#"{{ "component": "Form.Button", "attribs": {{ "name": "demo:button" }} }},
           {{ "component": "Visibility", "attribs": {{ "name": "demo:shown" }}, "children": [ {} ] }}"#,
        point("demo:slot")
    ));
    let document = loads(root(&text));
    assert!(document.diagnostics().is_empty());
    assert_eq!(document.extension_points(), ["demo:slot"]);
    let named: Vec<(Component, Option<&str>)> = document.layout.markup[0]
        .children()
        .iter()
        .map(|control| (control.component, control.extension_point_name()))
        .collect();
    assert_eq!(
        named,
        [(Component::FormButton, None), (Component::Visibility, None)]
    );
    let attribs =
        bedrock_addon::json::from_str::<bedrock_addon::json::Value>(r#"{"name": "demo:built"}"#)
            .ok();
    for component in Component::ALL {
        let mut control = Control::new(component);
        control.attribs.clone_from(&attribs);
        assert_eq!(
            control.extension_point_name(),
            (component == Component::ExtensionPoint).then_some("demo:built"),
            "{component:?}"
        );
    }
}

/// The names are read from the value as it stands, so an edited root answers for its edit.
#[test]
fn extension_points_follow_the_tree_as_built() {
    let mut named = Control::new(Component::ExtensionPoint);
    named.attribs =
        facet_json::from_str::<bedrock_addon::json::Value>(r#"{"name": "demo:built"}"#).ok();
    assert!(named.attribs.is_some());
    let root = UiRoot::new(
        bedrock_addon::id::UiRootId::parse("demo:built"),
        vec![Control::with_children(
            Component::Panel,
            vec![Control::new(Component::ExtensionPoint), named],
        )],
    );
    assert_eq!(root.extension_points(), ["demo:built"]);
    assert_eq!(root.attribs.root_point, "minecraft:screen");
}
