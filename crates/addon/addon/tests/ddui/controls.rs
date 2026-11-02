//! V5, the parse under the strict error mask. Every finding is an error and the file
//! is not loaded.

use bedrock_addon::diagnostic::{Diagnostic, DiagnosticCode, Severity};
use bedrock_addon::families::ddui::Component;
use bedrock_addon::version::FormatVersion;

use super::common::*;

const NAMESPACE_TEXT: &str = "string must be prefixed with a namespace (eg. namespace:value)";
const ROOT: &str = "minecraft:ui-root";
const COMPOSITION: &str = "minecraft:ui-composition";

/// Severity, code and path of each finding, the closing line left out.
pub fn codes<'a>(
    d: impl IntoIterator<Item = &'a Diagnostic>,
) -> Vec<(Severity, &'static str, String)> {
    d.into_iter()
        .filter(|f| !matches!(f.code, DiagnosticCode::DocumentRejected { .. }))
        .map(|f| (f.severity, f.code.name(), f.location.json_path.to_string()))
        .collect()
}

fn e(code: &'static str, path: &str) -> (Severity, &'static str, String) {
    (Severity::Error, code, path.to_owned())
}

/// The findings of a root file that must be refused, the closing line checked and left out.
#[track_caller]
fn refused(text: &str) -> Vec<(Severity, &'static str, String)> {
    refused_on(text, E20)
}

#[track_caller]
fn refused_on(text: &str, engine: FormatVersion) -> Vec<(Severity, &'static str, String)> {
    let d = rejects(root_in(text, &cx(engine, false)));
    assert_eq!(found(&d).last(), Some(&failed_to_load()), "{d:#?}");
    assert_eq!(
        d.iter()
            .filter(|f| matches!(f.code, DiagnosticCode::DocumentRejected { .. }))
            .count(),
        1
    );
    codes(&d)
}

/// The path of the `index`th child of `R`'s panel.
fn child(index: usize) -> String {
    format!("{ROOT}/layout/markup/0/children/{index}")
}

// ---- skeleton ------------------------------------------------------------------------------

#[test]
fn s01_a_root_without_description_is_refused() {
    let text = r().replace(r#""description": { "identifier": "demo:screen" },"#, "");
    assert_eq!(
        refused(&text),
        [e("MissingRequiredField", "minecraft:ui-root/description")]
    );
}

#[test]
fn every_required_member_of_the_skeleton_is_required() {
    let text = r#"{ "format_version": "1.21.130", "minecraft:ui-root": {} }"#;
    assert_eq!(
        refused(text),
        [
            e("MissingRequiredField", "minecraft:ui-root/description"),
            e("MissingRequiredField", "minecraft:ui-root/attribs"),
            e("MissingRequiredField", "minecraft:ui-root/layout"),
        ]
    );
    let text = r#"{ "format_version": "1.21.130", "minecraft:ui-root": { "description": {}, "attribs": {}, "layout": {} } }"#;
    assert_eq!(
        refused(text),
        [
            e(
                "MissingRequiredField",
                "minecraft:ui-root/description/identifier"
            ),
            e(
                "MissingRequiredField",
                "minecraft:ui-root/attribs/root_point"
            ),
            e("MissingRequiredField", "minecraft:ui-root/layout/markup"),
        ]
    );
    let text = r#"{ "format_version": "1.21.130", "minecraft:ui-composition": { "description": {}, "layout": {} } }"#;
    let d = rejects(composition(text));
    assert_eq!(
        codes(&d),
        [
            e(
                "MissingRequiredField",
                "minecraft:ui-composition/description/identifier"
            ),
            e(
                "MissingRequiredField",
                "minecraft:ui-composition/layout/markup"
            ),
            e("MissingRequiredField", "minecraft:ui-composition/attribs"),
        ]
    );
}

#[test]
fn s02_an_identifier_without_a_namespace_is_refused_with_the_constraint_s_text() {
    for identifier in ["screen", ":screen", "demo:", ":", ""] {
        let d = rejects(root(&root_json(identifier, R_MARKUP)));
        assert_eq!(
            found(&d),
            [
                error(
                    "InvalidValue",
                    "minecraft:ui-root/description/identifier",
                    NAMESPACE_TEXT
                ),
                failed_to_load()
            ],
            "{identifier:?}"
        );
    }
    let text = root_json("screen", R_MARKUP);
    let d = rejects(root(&text));
    assert_eq!(spanned(&text, d.iter().next().unwrap()), r#""screen""#);
    let d = rejects(composition(&composition_json(
        "extra",
        "",
        r#"{ "component": "Form.Divider" }"#,
    )));
    assert_eq!(
        found(&d)[0],
        error(
            "InvalidValue",
            "minecraft:ui-composition/description/identifier",
            NAMESPACE_TEXT
        )
    );
}

/// The shortest names the constraint takes, and a name with a second `:`.
#[test]
fn s02_one_character_each_side_of_the_colon_is_an_identifier() {
    for identifier in ["a:b", "a:b:c", "a::b", "a: "] {
        let document = loads(root(&root_json(identifier, R_MARKUP)));
        assert_eq!(document.description.identifier.as_str(), identifier);
    }
}

#[test]
fn s03_an_empty_root_point_is_refused() {
    let text = r().replace("minecraft:screen", "");
    assert_eq!(
        refused(&text),
        [e("InvalidValue", "minecraft:ui-root/attribs/root_point")]
    );
    // One character is not empty.
    let document = loads(root(&r().replace("minecraft:screen", " ")));
    assert_eq!(document.attribs.root_point, " ");
}

#[test]
fn s04_any_other_root_point_loads() {
    let document = loads(root(&r().replace("minecraft:screen", "demo:hud")));
    assert_eq!(document.attribs.root_point, "demo:hud");
    assert!(document.diagnostics().is_empty());
}

#[test]
fn s05_an_empty_markup_is_refused() {
    assert_eq!(
        refused(&root_json("demo:screen", "")),
        [e("InvalidValue", "minecraft:ui-root/layout/markup")]
    );
    let d = rejects(composition(&composition_json("demo:c", "", "")));
    assert_eq!(
        codes(&d),
        [e("InvalidValue", "minecraft:ui-composition/layout/markup")]
    );
    // One control is enough.
    let one = composition_json("demo:c", "", r#"{ "component": "Form.Divider" }"#);
    assert_eq!(loads(composition(&one)).layout.markup.len(), 1);
}

#[test]
fn s06_an_unknown_member_of_the_payload_is_an_error() {
    let text = r().replace(
        r#""attribs": { "root_point""#,
        r#""extra": 1, "attribs": { "root_point""#,
    );
    let d = rejects(root(&text));
    assert_eq!(
        codes(&d),
        [e("UnrecognizedField", "minecraft:ui-root/extra")]
    );
    assert_eq!(spanned(&text, d.iter().next().unwrap()), "1");
}

/// Unknown members at every level of the skeleton, several at once: each is reported.
#[test]
fn s06_unknown_members_at_every_level_are_each_an_error() {
    let text = r#"{ "format_version": "1.21.130", "minecraft:ui-root": {
  "description": { "identifier": "demo:screen", "a": 1 },
  "attribs": { "root_point": "minecraft:screen", "b": {}, "c": [] },
  "layout": { "markup": [ { "component": "Panel", "children": [], "d": null } ], "e": true },
  "f": "x" } }"#;
    assert_eq!(
        refused(text),
        [
            e("UnrecognizedField", "minecraft:ui-root/description/a"),
            e("UnrecognizedField", "minecraft:ui-root/attribs/b"),
            e("UnrecognizedField", "minecraft:ui-root/attribs/c"),
            e("UnrecognizedField", "minecraft:ui-root/layout/markup/0/d"),
            e("UnrecognizedField", "minecraft:ui-root/layout/e"),
            e("UnrecognizedField", "minecraft:ui-root/f"),
        ]
    );
}

/// A member named `extra` is as unknown as any other.
#[test]
fn a_member_named_extra_is_an_unknown_member() {
    let text = panel(r#"{ "component": "Form.Button", "extra": {} }"#);
    assert_eq!(
        refused(&text),
        [e("UnrecognizedField", &format!("{}/extra", child(0)))]
    );
}

#[test]
fn s07_a_composition_must_write_attribs_even_empty() {
    let markup = r#"{ "component": "Form.Divider" }"#;
    let without = composition_json("demo:c", "", markup).replace(r#""attribs": {  },"#, "");
    assert!(!without.contains("attribs"), "{without}");
    let d = rejects(composition(&without));
    assert_eq!(
        codes(&d),
        [e(
            "MissingRequiredField",
            "minecraft:ui-composition/attribs"
        )]
    );
    let document = loads(composition(&composition_json("demo:c", "", markup)));
    assert_eq!(document.attribs.extension_point, None);
    assert_eq!(document.description.identifier.as_str(), "demo:c");
}

#[test]
fn s08_an_empty_extension_point_is_refused() {
    let markup = r#"{ "component": "Form.Divider" }"#;
    let text = composition_json("minecraft:c", r#""extension_point": """#, markup);
    let d = rejects(composition_in(&text, &cx(E20, true)));
    assert_eq!(
        codes(&d),
        [e(
            "InvalidValue",
            "minecraft:ui-composition/attribs/extension_point"
        )]
    );
    // One character is a name (whether a root declares it is the stack's to say).
    let text = composition_json("minecraft:c", r#""extension_point": " ""#, markup);
    let document = loads(composition_in(&text, &cx(E20, true)));
    assert_eq!(document.attribs.extension_point.as_deref(), Some(" "));
}

/// A wrong JSON type for a bound member, at every member of the skeleton.
#[test]
fn a_wrong_type_for_a_bound_member_is_refused() {
    let cases = [
        (
            r#""description": { "identifier": "demo:screen" }"#,
            r#""description": []"#,
            "description",
        ),
        (
            r#""identifier": "demo:screen""#,
            r#""identifier": 5"#,
            "description/identifier",
        ),
        (
            r#""attribs": { "root_point": "minecraft:screen" }"#,
            r#""attribs": "x""#,
            "attribs",
        ),
        (
            r#""root_point": "minecraft:screen""#,
            r#""root_point": null"#,
            "attribs/root_point",
        ),
        (
            r#""root_point": "minecraft:screen""#,
            r#""root_point": ["minecraft:screen"]"#,
            "attribs/root_point",
        ),
    ];
    for (from, to, path) in cases {
        let text = r().replace(from, to);
        assert_ne!(text, r());
        assert_eq!(
            refused(&text),
            [e("InvalidType", &format!("{ROOT}/{path}"))],
            "{to}"
        );
    }
    let text = r#"{ "format_version": "1.21.130", "minecraft:ui-root": { "description": { "identifier": "demo:screen" }, "attribs": { "root_point": "minecraft:screen" }, "layout": { "markup": {} } } }"#;
    assert_eq!(
        refused(text),
        [e("InvalidType", "minecraft:ui-root/layout/markup")]
    );
    let text = text.replace(r#"{ "markup": {} }"#, "null");
    assert_eq!(
        refused(&text),
        [e("InvalidType", "minecraft:ui-root/layout")]
    );
    let text = r#"{ "format_version": "1.21.130", "minecraft:ui-root": [] }"#;
    assert_eq!(refused(text), [e("InvalidType", "minecraft:ui-root")]);
    // A control, its `component` and its `children`.
    assert_eq!(refused(&panel("5")), [e("InvalidType", &child(0))]);
    assert_eq!(
        refused(&panel(
            r#"{ "component": 5 }, { "component": ["Panel.Text"] }"#
        )),
        [
            e("InvalidType", &format!("{}/component", child(0))),
            e("InvalidType", &format!("{}/component", child(1))),
        ]
    );
    assert_eq!(
        refused(&panel(r#"{ "component": "Visibility", "children": {} }"#)),
        [e("InvalidType", &format!("{}/children", child(0)))]
    );
    assert_eq!(
        refused(&panel(r#"{ "component": "Visibility", "children": null }"#)),
        [e("InvalidType", &format!("{}/children", child(0)))]
    );
}

/// A written `extension_point` is a string: `null` is not "absent" (the text would otherwise
/// be rewritten on the way back).
#[test]
fn a_null_extension_point_is_a_wrong_type() {
    let markup = r#"{ "component": "Form.Divider" }"#;
    let text = composition_json("demo:c", r#""extension_point": null"#, markup);
    let d = rejects(composition(&text));
    assert_eq!(
        codes(&d),
        [e(
            "InvalidType",
            "minecraft:ui-composition/attribs/extension_point"
        )]
    );
    let text = composition_json("demo:c", r#""other": 1"#, markup);
    let d = rejects(composition(&text));
    assert_eq!(
        codes(&d),
        [e(
            "UnrecognizedField",
            "minecraft:ui-composition/attribs/other"
        )]
    );
}

// ---- controls ------------------------------------------------------------------------------

#[test]
fn c01_attribs_are_opaque() {
    let text = panel(
        r#"{ "component": "Form.Button", "attribs": { "label": 5, "anything": { "$path": "$.x" }, "nested": [1, {}] } }"#,
    );
    let document = loads(root(&text));
    assert!(document.diagnostics().is_empty());
    let button = &document.layout.markup[0].children()[0];
    assert_eq!(button.component, Component::FormButton);
    let attribs = button.attribs.as_ref().unwrap().as_object().unwrap();
    assert_eq!(attribs.len(), 3);
    assert!(attribs.get("nested").unwrap().as_array().is_some());
    assert_eq!(button.children, None);
}

/// Any JSON value is an `attribs`, not only an object.
#[test]
fn c01_attribs_of_any_json_type_load() {
    for attribs in ["null", "5", r#""text""#, "[]", "true", "{}"] {
        let text = panel(&format!(
            r#"{{ "component": "Form.Button", "attribs": {attribs} }}"#
        ));
        let document = loads(root(&text));
        assert!(
            document.layout.markup[0].children()[0].attribs.is_some(),
            "{attribs}"
        );
    }
}

#[test]
fn c02_attribs_on_a_type_that_binds_none_is_an_unknown_member() {
    for name in ["Panel.Spacing", "Panel.Decoration", "Form.Divider"] {
        let text = panel(&format!(r#"{{ "component": "{name}", "attribs": {{}} }}"#));
        let d = rejects(root(&text));
        assert_eq!(
            codes(&d),
            [e("UnrecognizedField", &format!("{}/attribs", child(0)))],
            "{name}"
        );
        assert_eq!(spanned(&text, d.iter().next().unwrap()), "{}");
        loads(root(&panel(&format!(r#"{{ "component": "{name}" }}"#))));
    }
    // `Container.Layout` binds `children` and no `attribs`.
    let text = root_json(
        "demo:screen",
        r#"{ "component": "Container.Layout", "attribs": {}, "children": [] }"#,
    );
    assert_eq!(
        refused(&text),
        [e(
            "UnrecognizedField",
            "minecraft:ui-root/layout/markup/0/attribs"
        )]
    );
}

#[test]
fn c03_children_on_a_leaf_is_an_unknown_member() {
    for name in [
        "Form.Button",
        "ExtensionPoint",
        "Panel.Text",
        "Panel.CloseButton",
        "Form.Dropdown",
        "Form.Slider",
        "Form.Switch",
        "Form.TextField",
        "Form.Divider",
    ] {
        let text = panel(&format!(
            r#"{{ "component": "{name}", "attribs": {{ "name": "x" }}, "children": [] }}"#
        ));
        let got = refused(&text);
        assert!(
            got.contains(&e("UnrecognizedField", &format!("{}/children", child(0)))),
            "{name}: {got:?}"
        );
    }
    assert_eq!(
        refused(&panel(r#"{ "component": "Form.Button", "children": [] }"#)),
        [e("UnrecognizedField", &format!("{}/children", child(0)))]
    );
}

#[test]
fn c04_a_container_must_write_children() {
    assert_eq!(
        refused(&root_json("demo:screen", r#"{ "component": "Panel" }"#)),
        [e(
            "MissingRequiredField",
            "minecraft:ui-root/layout/markup/0/children"
        )]
    );
    for name in [
        "Context.List",
        "Container.FixedGridLayout",
        "Form.ScrollView",
        "Visibility",
    ] {
        let text = panel(&format!(r#"{{ "component": "{name}" }}"#));
        assert_eq!(
            refused(&text),
            [e("MissingRequiredField", &format!("{}/children", child(0)))],
            "{name}"
        );
    }
    for name in ["Context", "Container.Layout"] {
        let text = root_json("demo:screen", &format!(r#"{{ "component": "{name}" }}"#));
        assert_eq!(
            refused(&text),
            [e(
                "MissingRequiredField",
                "minecraft:ui-root/layout/markup/0/children"
            )],
            "{name}"
        );
    }
}

#[test]
fn c05_empty_children_load() {
    let document = loads(root(&root_json(
        "demo:screen",
        r#"{ "component": "Panel", "children": [] }"#,
    )));
    assert_eq!(document.layout.markup[0].children, Some(Vec::new()));
}

#[test]
fn c06_panel_text_must_write_attribs() {
    let d = rejects(root(&panel(r#"{ "component": "Panel.Text" }"#)));
    assert_eq!(
        codes(&d),
        [e("MissingRequiredField", &format!("{}/attribs", child(0)))]
    );
}

#[test]
fn a_control_must_write_its_component() {
    let text = panel(r#"{ "attribs": {} }, {}"#);
    assert_eq!(
        refused(&text),
        [
            e("MissingRequiredField", &format!("{}/component", child(0))),
            e("MissingRequiredField", &format!("{}/component", child(1))),
        ]
    );
}

#[test]
fn c07_an_unknown_component_is_refused() {
    let text = panel(r#"{ "component": "Form.Label" }"#);
    let d = rejects(root(&text));
    assert_eq!(
        codes(&d),
        [e("InvalidValue", &format!("{}/component", child(0)))]
    );
    assert_eq!(spanned(&text, d.iter().next().unwrap()), r#""Form.Label""#);
}

#[test]
fn c08_a_component_outside_its_parent_s_set_is_refused() {
    // `Container.Item` directly under `Panel`.
    assert_eq!(
        refused(&panel(r#"{ "component": "Container.Item" }"#)),
        [e("InvalidValue", &format!("{}/component", child(0)))]
    );
    // `Panel.CloseButton` under `Context.List`.
    assert_eq!(
        refused(&panel(
            r#"{ "component": "Context.List", "children": [ { "component": "Panel.CloseButton" } ] }"#
        )),
        [e(
            "InvalidValue",
            &format!("{}/children/0/component", child(0))
        )]
    );
    // `ExtensionPoint` under `Context.List`.
    assert_eq!(
        refused(&panel(
            r#"{ "component": "Context.List", "children": [ { "component": "ExtensionPoint", "attribs": { "name": "x" } } ] }"#
        )),
        [e(
            "InvalidValue",
            &format!("{}/children/0/component", child(0))
        )]
    );
    // `Panel` under `Panel`.
    assert_eq!(
        refused(&panel(r#"{ "component": "Panel", "children": [] }"#)),
        [e("InvalidValue", &format!("{}/component", child(0)))]
    );
    // `Form.Button` at the top of a root's markup.
    assert_eq!(
        refused(&root_json(
            "demo:screen",
            r#"{ "component": "Form.Button" }"#
        )),
        [e(
            "InvalidValue",
            "minecraft:ui-root/layout/markup/0/component"
        )]
    );
    // A composition's markup is panel content: no `Panel`, no `Container.Item`.
    for misplaced in [
        r#"{ "component": "Panel", "children": [] }"#,
        r#"{ "component": "Container.Item" }"#,
        r#"{ "component": "Context", "children": [] }"#,
    ] {
        let d = rejects(composition(&composition_json("demo:c", "", misplaced)));
        assert_eq!(
            codes(&d),
            [e(
                "InvalidValue",
                "minecraft:ui-composition/layout/markup/0/component"
            )],
            "{misplaced}"
        );
    }
}

/// Several misplaced controls in one file, and a good one between them: each is reported and
/// what stands below a misplaced control is still read.
#[test]
fn c08_every_misplaced_control_of_a_file_is_reported() {
    let text = panel(
        r#"{ "component": "Container.Item" }, { "component": "Form.Divider" },
           { "component": "Panel", "children": [ { "component": "Panel.Text" } ] },
           { "component": "Form.Label", "children": [ { "component": "Nope" } ] }"#,
    );
    assert_eq!(
        refused(&text),
        [
            e("InvalidValue", &format!("{}/component", child(0))),
            e("InvalidValue", &format!("{}/component", child(2))),
            e(
                "MissingRequiredField",
                &format!("{}/children/0/attribs", child(2))
            ),
            e("InvalidValue", &format!("{}/component", child(3))),
        ]
    );
}

#[test]
fn c09_a_grid_takes_items_and_lists() {
    let text = panel(
        r#"{ "component": "Container.FixedGridLayout", "children": [
              { "component": "Context.List", "children": [ { "component": "Container.Item" } ] },
              { "component": "Container.Item", "attribs": { "item": 1 } } ] }"#,
    );
    let document = loads(root(&text));
    let grid = &document.layout.markup[0].children()[0];
    assert_eq!(grid.component, Component::ContainerFixedGridLayout);
    assert_eq!(grid.children().len(), 2);
    assert_eq!(
        grid.children()[0].children()[0].component,
        Component::ContainerItem
    );
    // Nothing else stands directly in a grid.
    let text = panel(
        r#"{ "component": "Container.FixedGridLayout", "children": [ { "component": "Form.Button" } ] }"#,
    );
    assert_eq!(
        refused(&text),
        [e(
            "InvalidValue",
            &format!("{}/children/0/component", child(0))
        )]
    );
}

#[test]
fn c10_contexts_and_layouts_nest() {
    let text = root_json(
        "demo:screen",
        r#"{ "component": "Container.Layout", "children": [ { "component": "Context", "children": [
             { "component": "Context", "attribs": { "data": { "$path": "$.x" } }, "children": [ { "component": "Panel", "children": [] } ] } ] } ] }"#,
    );
    let document = loads(root(&text));
    let mut control = &document.layout.markup[0];
    let mut seen = vec![control.component];
    while let Some(next) = control.children().first() {
        seen.push(next.component);
        control = next;
    }
    assert_eq!(
        seen,
        [
            Component::ContainerLayout,
            Component::Context,
            Component::Context,
            Component::Panel
        ]
    );
}

#[test]
fn c11_component_names_are_compared_with_case() {
    assert_eq!(
        refused(&root_json(
            "demo:screen",
            r#"{ "component": "panel", "children": [] }"#
        )),
        [e(
            "InvalidValue",
            "minecraft:ui-root/layout/markup/0/component"
        )]
    );
}

const C12: &str = r#"{ "component": "Panel", "children": [
    { "component": "ScrollableGridLayout", "attribs": { "data": 1 }, "children": [ { "component": "Container.Slot" } ] },
    { "component": "Visibility", "children": [ { "component": "Form.Image", "attribs": { "src": "x" } } ] } ] }"#;

#[test]
fn c12_the_1_26_45_catalogue_reads_the_new_types() {
    let text = root_json("demo:screen", C12);
    let document = loads(root_in(&text, &cx(E45, false)));
    let panel = &document.layout.markup[0];
    assert_eq!(
        panel.children()[0].component,
        Component::ScrollableGridLayout
    );
    assert_eq!(
        panel.children()[0].children()[0].component,
        Component::ContainerSlot
    );
    assert_eq!(
        panel.children()[1].children()[0].component,
        Component::FormImage
    );
    // The engine the crate tracks is past 1.26.45.
    loads(root_in(
        &text,
        &bedrock_addon::load::LoadContext::detached(),
    ));
}

#[test]
fn c12_container_item_is_gone_from_the_1_26_45_catalogue() {
    for markup in [
        r#"{ "component": "Panel", "children": [ { "component": "Context.List", "children": [ { "component": "Container.Item" } ] } ] }"#,
        r#"{ "component": "Panel", "children": [ { "component": "Container.FixedGridLayout", "children": [ { "component": "Container.Item" } ] } ] }"#,
    ] {
        let text = root_json("demo:screen", markup);
        let at = format!("{}/children/0/component", child(0));
        // The type is refused, with the last engine known to have it as a hint.
        assert_eq!(
            refused_on(&text, E45),
            [
                e("InvalidValue", &at),
                (Severity::Info, "FieldUntilEngine", at.clone())
            ]
        );
        let last: Vec<FormatVersion> = rejects(root_in(&text, &cx(E45, false)))
            .iter()
            .filter_map(|f| match f.code {
                DiagnosticCode::FieldUntilEngine { last } => Some(last),
                _ => None,
            })
            .collect();
        assert_eq!(
            last,
            [E20],
            "the 1.26.20 catalogue is the last with the type"
        );
        loads(root(&text));
    }
}

/// A name no catalogue has is refused without a hint, by either engine.
#[test]
fn c12_a_type_of_no_catalogue_gets_no_hint() {
    let text = panel(r#"{ "component": "Container.Gone" }"#);
    for engine in [E20, E45] {
        assert_eq!(
            refused_on(&text, engine),
            [e("InvalidValue", &format!("{}/component", child(0)))],
            "{engine:?}"
        );
    }
}

/// The same file on a 1.26.20 client: each later type is refused, with the engine that has it
/// as a hint.
#[test]
fn c12_the_1_26_20_catalogue_refuses_the_later_types_and_names_their_engine() {
    let text = root_json("demo:screen", C12);
    let d = rejects(root(&text));
    let hint = |path: &str| (Severity::Info, "FieldSinceEngine", path.to_owned());
    assert_eq!(
        codes(&d),
        [
            e("InvalidValue", &format!("{}/component", child(0))),
            hint(&format!("{}/component", child(0))),
            e(
                "InvalidValue",
                &format!("{}/children/0/component", child(1))
            ),
            hint(&format!("{}/children/0/component", child(1))),
        ]
    );
    let since: Vec<FormatVersion> = d
        .iter()
        .filter_map(|f| match f.code {
            DiagnosticCode::FieldSinceEngine { since } => Some(since),
            _ => None,
        })
        .collect();
    assert_eq!(since, [E45, E45]);
    // The engine just below 1.26.45 still reads with the 1.26.20 catalogue.
    assert_eq!(refused_on(&text, FormatVersion::new(1, 26, 44)).len(), 4);
    // A type that no later catalogue has gets no hint of a later engine: its hint names the
    // earlier one.
    let gone = panel(
        r#"{ "component": "Context.List", "children": [ { "component": "Container.Item" } ] }"#,
    );
    let found = refused_on(&gone, E45);
    assert_eq!(found.len(), 2);
    assert!(found.iter().all(|(_, code, _)| *code != "FieldSinceEngine"));
    // And a type the 1.26.20 catalogue lacks has no earlier engine to name.
    assert!(
        codes(&d)
            .iter()
            .all(|(_, code, _)| *code != "FieldUntilEngine")
    );
}

#[test]
fn c13_the_new_types_bind_what_the_retail_client_binds() {
    let cases = [
        (
            r#"{ "component": "Form.Image", "children": [] }"#,
            "UnrecognizedField",
            "children",
        ),
        (
            r#"{ "component": "Container.TextField", "children": [] }"#,
            "UnrecognizedField",
            "children",
        ),
        (
            r#"{ "component": "ScrollableGridLayout", "attribs": {} }"#,
            "MissingRequiredField",
            "children",
        ),
    ];
    for (control, code, member) in cases {
        assert_eq!(
            refused_on(&panel(control), E45),
            [e(code, &format!("{}/{member}", child(0)))],
            "{control}"
        );
    }
    // `Container.Slot` is a leaf too.
    let slot = panel(
        r#"{ "component": "Context.List", "children": [ { "component": "Container.Slot", "children": [] } ] }"#,
    );
    assert_eq!(
        refused_on(&slot, E45),
        [e(
            "UnrecognizedField",
            &format!("{}/children/0/children", child(0))
        )]
    );
    // Their `attribs` is optional.
    let text = panel(
        r#"{ "component": "Form.Image" }, { "component": "Container.TextField" },
           { "component": "ScrollableGridLayout", "children": [ { "component": "Container.Slot" } ] }"#,
    );
    loads(root_in(&text, &cx(E45, false)));
}

#[test]
fn c14_a_scrollable_grid_takes_list_content() {
    let text = panel(
        r#"{ "component": "ScrollableGridLayout", "children": [
              { "component": "Container.Slot" }, { "component": "Form.Button" },
              { "component": "Panel.Text", "attribs": {} },
              { "component": "Context.List", "children": [] },
              { "component": "ScrollableGridLayout", "children": [] } ] }"#,
    );
    let document = loads(root_in(&text, &cx(E45, false)));
    assert_eq!(document.layout.markup[0].children()[0].children().len(), 5);
    for outside in [
        r#"{ "component": "ExtensionPoint", "attribs": { "name": "x" } }"#,
        r#"{ "component": "Panel.CloseButton" }"#,
    ] {
        let text = panel(&format!(
            r#"{{ "component": "ScrollableGridLayout", "children": [ {outside} ] }}"#
        ));
        assert_eq!(
            refused_on(&text, E45),
            [e(
                "InvalidValue",
                &format!("{}/children/0/component", child(0))
            )],
            "{outside}"
        );
    }
}

/// A member written twice in a control: the last stands, and a span after the blanked copy is
/// where the file has it.
#[test]
fn a_finding_after_a_member_written_twice_keeps_its_place_in_the_file() {
    let text = panel(
        r#"{ "component": "Form.Button", "component": "Form.Divider" }, { "component": "Nope" }"#,
    );
    let d = rejects(root(&text));
    assert_eq!(
        codes(&d),
        [e("InvalidValue", &format!("{}/component", child(1)))]
    );
    assert_eq!(spanned(&text, d.iter().next().unwrap()), r#""Nope""#);
    let text = panel(r#"{ "component": "Nope", "component": "Form.Divider" }"#);
    assert_eq!(
        loads(root(&text)).layout.markup[0].children()[0].component,
        Component::FormDivider
    );
}

/// A syntax error below the payload is the file's, at its place.
#[test]
fn a_syntax_error_inside_the_payload_is_a_missing_object() {
    let text = panel(r#"{ "component": "Form.Button" "attribs": {} }"#);
    let got = found(&rejects(root(&text)));
    assert_eq!(got.last(), Some(&missing_object(ROOT)), "{got:#?}");
    let text = composition_json("demo:c", "", r#"{ "component": }"#);
    let got = found(&rejects(composition(&text)));
    assert_eq!(got.last(), Some(&missing_object(COMPOSITION)), "{got:#?}");
}
