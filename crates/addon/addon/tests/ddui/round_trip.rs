//! Write-back: a document that loads is written as its author wrote it (the written text
//! equals the input as JSON values, member for member) and reads back equal; a value the
//! client would refuse is refused by the writer with its path. `Placed::default_path`.

use bedrock_addon::document::{Content, Placed};
use bedrock_addon::families::ddui::{Component, Control, UiComposition, UiRoot};
use bedrock_addon::id::{UiCompositionId, UiRootId};
use bedrock_addon::json::{EncodeError, Value};
use bedrock_addon::version::FormatVersion;

use super::common::*;

fn json(text: &str) -> Value {
    bedrock_addon::json::from_str_jsonc::<Value>(text)
        .unwrap_or_else(|error| panic!("{text}: {error}"))
}

/// The path of a refusal.
#[track_caller]
fn refused_at<T: Content>(value: &T) -> String {
    match value.to_json() {
        Ok(text) => panic!("written: {text}"),
        Err(EncodeError::NotRepresentable { at, .. }) => at.to_string(),
        Err(other) => panic!("{other}"),
    }
}

/// `text` loads as a root on `engine`, writes the JSON it was read from and reads back equal.
#[track_caller]
fn root_round_trips(text: &str, engine: FormatVersion, builtin: bool) {
    let cx = cx(engine, builtin);
    let document = loads(root_in(text, &cx));
    let written = document.to_json().expect("writes");
    assert_eq!(json(&written), json(text), "as authored");
    let again = loads(root_in(&written, &cx));
    assert_eq!(again.value(), document.value());
    assert_eq!(again.to_json().expect("writes again"), written);
}

/// v01: the eight roots of the 1.26.20 base game, on a 1.26.20 engine.
// Source: GDK Dev-1.26.2004 `data/resource_packs/vanilla_1.21.130/ddui/root/`.
#[test]
fn v01_the_1_26_20_vanilla_roots_load_and_write_back_as_authored() {
    let files = fixture_files("v01_vanilla_1_26_20/ddui/root");
    assert_eq!(files.len(), 8);
    for (name, text) in &files {
        let document = loads(root_in(text, &cx(E20, true)));
        assert!(document.diagnostics().is_empty(), "{name}");
        assert!(document.extension_points().is_empty(), "{name}");
        assert_eq!(document.attribs.root_point, "minecraft:screen", "{name}");
        root_round_trips(text, E20, true);
    }
    let names: Vec<&str> = files.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(
        names,
        [
            "barrel_screen.json",
            "chest_screen.json",
            "copper_chest_screen.json",
            "custom_form.json",
            "default_chest_screen.json",
            "ender_chest_screen.json",
            "message_box.json",
            "shulker_box_screen.json"
        ]
    );
}

/// v02: the 1.26.45 copies load on their own engine, and on a 1.26.20 engine only
/// `message_box` does.
// Source: GDK Retail-1.26.4501 `data/resource_packs/vanilla_1.21.130/ddui/root/`.
#[test]
fn v02_the_1_26_45_vanilla_roots_fail_on_a_1_26_20_engine_where_they_use_later_types() {
    let files = fixture_files("v02_vanilla_1_26_45/ddui/root");
    assert_eq!(files.len(), 8);
    let mut loaded = Vec::new();
    for (name, text) in &files {
        root_round_trips(text, E45, true);
        match root_in(text, &cx(E20, true)) {
            Ok(_) => loaded.push(name.as_str()),
            Err(rejected) => {
                let errors: Vec<_> = rejected
                    .diagnostics
                    .iter()
                    .filter(|f| f.severity == bedrock_addon::diagnostic::Severity::Error)
                    .filter(|f| f.code.name() != "DocumentRejected")
                    .collect();
                assert!(!errors.is_empty(), "{name}");
                for error in errors {
                    assert_eq!(error.code.name(), "InvalidValue", "{name}");
                    let written = spanned(text, error);
                    assert!(
                        [
                            r#""Container.Slot""#,
                            r#""ScrollableGridLayout""#,
                            r#""Form.Image""#
                        ]
                        .contains(&written),
                        "{name}: {written}"
                    );
                }
            }
        }
    }
    assert_eq!(loaded, ["message_box.json"]);
    let custom_form = &files
        .iter()
        .find(|(name, _)| name == "custom_form.json")
        .unwrap()
        .1;
    let d = rejects(root_in(custom_form, &cx(E20, true)));
    assert_eq!(
        spanned(custom_form, d.iter().next().unwrap()),
        r#""Form.Image""#
    );
}

/// v04: the two script-UI roots of 1.26.60 (`custom_form` has all five types of 1.26.60,
/// `message_box` all but `Form.CloseButton`): they load on a 1.26.60 engine with no finding
/// and write back as authored; on a 1.26.45 engine each later type is an `InvalidValue` with a
/// `FieldSinceEngine` hint naming 1.26.60.
// Source: 1.26.60 `resource_packs/script_ui/ddui/root/`.
#[test]
fn v04_the_1_26_60_script_ui_roots_load_on_their_engine_and_write_back_as_authored() {
    let files = fixture_files("v04_vanilla/ddui/root");
    let names: Vec<&str> = files.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(names, ["custom_form.json", "message_box.json"]);
    for (name, text) in &files {
        let document = loads(root_in(text, &cx(E60, true)));
        assert!(document.diagnostics().is_empty(), "{name}");
        root_round_trips(text, E60, true);
        let later = [
            "Form.HorizontalStack",
            "Form.Panel",
            "Form.Text",
            "Form.Spacing",
            "Form.CloseButton",
        ];
        let used: Vec<&str> = later
            .into_iter()
            .filter(|component| text.contains(&format!(r#""component":"{component}""#)))
            .collect();
        let expected: &[&str] = if name == "custom_form.json" {
            &later
        } else {
            &[
                "Form.HorizontalStack",
                "Form.Panel",
                "Form.Text",
                "Form.Spacing",
            ]
        };
        assert_eq!(used, expected, "{name}");
        let d = rejects(root_in(text, &cx(E45, true)));
        let first = d.iter().next().unwrap();
        assert_eq!(first.severity, bedrock_addon::diagnostic::Severity::Error);
        assert_eq!(first.code.name(), "InvalidValue", "{name}");
        assert_eq!(spanned(text, first), r#""Form.Panel""#, "{name}");
        assert_eq!(found(&d)[1].1, "FieldSinceEngine", "{name}");
        assert!(found(&d)[1].3.contains("1.26.60"), "{:?}", found(&d)[1]);
    }
}

/// v03: the functional-test root is `minecraft:test_form`.
// Source: GDK Dev-1.26.2004 `data/test/assets/OreUIDataDrivenScreenTests/TestRP/ddui/root/`.
#[test]
fn v03_the_test_pack_s_root_loads_only_from_a_shipped_pack() {
    let text = fixture("v03_test_pack/ddui/root/testpack_form.json");
    let d = rejects(root(&text));
    assert_eq!(
        found(&d)[0].3,
        "external packs must not use the 'minecraft' namespace for DataDrivenUI documents"
    );
    root_round_trips(&text, E20, true);
    assert_eq!(
        loads(root_in(&text, &cx(E20, true)))
            .description
            .identifier
            .as_str(),
        "minecraft:test_form"
    );
}

#[test]
fn a_composition_writes_back_as_authored() {
    let markup = r#"{ "component": "Form.Button", "attribs": { "label": "x" } }, { "component": "Visibility", "attribs": null, "children": [] }"#;
    for (attribs, builtin, id) in [
        ("", false, "demo:c"),
        (r#""extension_point": "demo:slot""#, true, "minecraft:c"),
    ] {
        let text = composition_json(id, attribs, markup);
        let cx = cx(E20, builtin);
        let document = loads(composition_in(&text, &cx));
        let written = document.to_json().expect("writes");
        assert_eq!(json(&written), json(&text));
        assert_eq!(
            loads(composition_in(&written, &cx)).value(),
            document.value()
        );
    }
}

/// Nothing is invented: an absent `attribs` and an absent `children` stay absent, an empty or
/// `null` one stays written.
#[test]
fn absent_members_stay_absent_and_empty_ones_stay_written() {
    let text = panel(
        r#"{ "component": "Form.Button" }, { "component": "Form.Button", "attribs": {} },
           { "component": "Form.Button", "attribs": null },
           { "component": "Visibility", "children": [] }, { "component": "Visibility", "attribs": [], "children": [ { "component": "Panel.Spacing" } ] }"#,
    );
    let written = loads(root(&text)).to_json().unwrap();
    assert_eq!(json(&written), json(&text));
    assert_eq!(written.matches("attribs").count(), 4, "{written}");
    assert_eq!(written.matches("children").count(), 3, "{written}");
}

#[test]
fn the_default_paths_are_below_the_two_folders() {
    let root = UiRoot::new(
        UiRootId::parse("demo:shop/main"),
        vec![Control::with_children(Component::Panel, Vec::new())],
    );
    assert_eq!(
        root.default_path().as_str(),
        "ddui/root/demo/shop/main.json"
    );
    let root = UiRoot::new(UiRootId::parse("demo:screen"), Vec::new());
    assert_eq!(root.default_path().as_str(), "ddui/root/demo/screen.json");
    let composition = UiComposition::new(
        UiCompositionId::parse("demo:extra"),
        vec![Control::new(Component::FormDivider)],
    );
    assert_eq!(
        composition.default_path().as_str(),
        "ddui/composition/demo/extra.json"
    );
}

/// A built document writes what the reader takes, at the path the walk reads.
#[test]
fn a_built_document_writes_text_that_loads_equal() {
    let mut text_control = Control::new(Component::PanelText);
    text_control.attribs = Some(json(r#"{ "text": "hi" }"#));
    let built = UiRoot::new(
        UiRootId::parse("demo:built"),
        vec![Control::with_children(Component::Panel, vec![text_control])],
    );
    let written = built.to_json().expect("writes");
    assert_eq!(
        json(&written),
        json(
            r#"{ "format_version": "1.21.130", "minecraft:ui-root": {
                 "description": { "identifier": "demo:built" },
                 "attribs": { "root_point": "minecraft:screen" },
                 "layout": { "markup": [ { "component": "Panel", "children": [ { "component": "Panel.Text", "attribs": { "text": "hi" } } ] } ] } } }"#
        )
    );
    assert_eq!(*loads(root(&written)).value(), built);
    let composition = UiComposition::new(
        UiCompositionId::parse("demo:extra"),
        vec![Control::new(Component::FormDivider)],
    );
    let written = composition.to_json().expect("writes");
    // `attribs` is required even when empty.
    assert!(
        json(&written)
            .as_object()
            .unwrap()
            .get("minecraft:ui-composition")
            .unwrap()
            .as_object()
            .unwrap()
            .get("attribs")
            .is_some()
    );
    assert_eq!(
        *loads(super::common::composition(&written)).value(),
        composition
    );
}

// ---- what the writer refuses ---------------------------------------------------------------

fn valid_root() -> UiRoot {
    let mut text = Control::new(Component::PanelText);
    text.attribs = Some(json("{}"));
    UiRoot::new(
        UiRootId::parse("demo:screen"),
        vec![Control::with_children(Component::Panel, vec![text])],
    )
}

/// `to_json` never returns text the loader refuses: each rule of the strict parse, refused
/// with the path of the member.
#[test]
fn the_writer_refuses_what_the_reader_refuses_with_its_path() {
    valid_root().to_json().expect("the base value writes");
    type Edit = Box<dyn Fn(&mut UiRoot)>;
    let cases: Vec<(&str, Edit, &str)> = vec![
        (
            "an identifier without a namespace",
            Box::new(|r| r.description.identifier = UiRootId::parse("screen")),
            "minecraft:ui-root/description/identifier",
        ),
        (
            "an identifier with an empty name",
            Box::new(|r| r.description.identifier = UiRootId::parse("demo:")),
            "minecraft:ui-root/description/identifier",
        ),
        (
            "an empty root point",
            Box::new(|r| r.attribs.root_point.clear()),
            "minecraft:ui-root/attribs/root_point",
        ),
        (
            "an empty markup",
            Box::new(|r| r.layout.markup.clear()),
            "minecraft:ui-root/layout/markup",
        ),
        (
            "a control outside the root set",
            Box::new(|r| r.layout.markup.push(Control::new(Component::FormButton))),
            "minecraft:ui-root/layout/markup/1/component",
        ),
        (
            "a container without children",
            Box::new(|r| r.layout.markup[0].children = None),
            "minecraft:ui-root/layout/markup/0/children",
        ),
        (
            "Panel.Text without attribs",
            Box::new(|r| r.layout.markup[0].children.as_mut().unwrap()[0].attribs = None),
            "minecraft:ui-root/layout/markup/0/children/0/attribs",
        ),
        (
            "attribs on a type that binds none",
            Box::new(|r| {
                let mut spacing = Control::new(Component::PanelSpacing);
                spacing.attribs = Some(json("null"));
                r.layout.markup[0].children.as_mut().unwrap().push(spacing);
            }),
            "minecraft:ui-root/layout/markup/0/children/1/attribs",
        ),
        (
            "children on a leaf",
            Box::new(|r| {
                let button = Control::with_children(Component::FormButton, Vec::new());
                r.layout.markup[0].children.as_mut().unwrap().push(button);
            }),
            "minecraft:ui-root/layout/markup/0/children/1/children",
        ),
        (
            "a control outside its parent's set",
            Box::new(|r| {
                let nested = Control::with_children(Component::Panel, Vec::new());
                r.layout.markup[0].children.as_mut().unwrap().push(nested);
            }),
            "minecraft:ui-root/layout/markup/0/children/1/component",
        ),
    ];
    for (what, edit, path) in cases {
        let mut root = valid_root();
        edit(&mut root);
        assert_eq!(refused_at(&root), path, "{what}");
    }
    let mut composition = UiComposition::new(
        UiCompositionId::parse("demo:c"),
        vec![Control::new(Component::FormDivider)],
    );
    composition.to_json().expect("writes");
    composition.attribs.extension_point = Some(String::new());
    assert_eq!(
        refused_at(&composition),
        "minecraft:ui-composition/attribs/extension_point"
    );
    composition.attribs.extension_point = Some("demo:slot".to_owned());
    composition
        .to_json()
        .expect("a named extension point writes");
    composition.layout.markup[0] = Control::with_children(Component::Panel, Vec::new());
    assert_eq!(
        refused_at(&composition),
        "minecraft:ui-composition/layout/markup/0/component"
    );
}

/// The catalogue is the engine's and `to_json` has no engine: a tree some catalogue accepts
/// whole is written, a tree no catalogue accepts is refused where the newest one refuses it.
#[test]
fn the_writer_takes_a_tree_that_one_catalogue_accepts_whole() {
    let list = |below: Vec<Control>| {
        UiRoot::new(
            UiRootId::parse("demo:screen"),
            vec![Control::with_children(
                Component::Panel,
                vec![Control::with_children(Component::ContextList, below)],
            )],
        )
    };
    let old = list(vec![Control::new(Component::ContainerItem)]);
    let written = old.to_json().expect("the 1.26.20 catalogue takes it");
    assert_eq!(*loads(root(&written)).value(), old);
    let new = list(vec![Control::new(Component::ContainerSlot)]);
    let written = new.to_json().expect("the 1.26.45 catalogue takes it");
    assert_eq!(*loads(root_in(&written, &cx(E45, false))).value(), new);
    let mixed = list(vec![
        Control::new(Component::ContainerSlot),
        Control::new(Component::ContainerItem),
    ]);
    assert_eq!(
        refused_at(&mixed),
        "minecraft:ui-root/layout/markup/0/children/0/children/1/component"
    );
}

/// `depth` panels' worth of nesting: `Visibility` in `Visibility`, built without recursion.
fn nested(depth: usize) -> UiRoot {
    let mut control = Control::with_children(Component::Visibility, Vec::new());
    for _ in 0..depth {
        control = Control::with_children(Component::Visibility, vec![control]);
    }
    UiRoot::new(
        UiRootId::parse("demo:deep"),
        vec![Control::with_children(Component::Panel, vec![control])],
    )
}

/// A tree built in memory may nest without bound: it drops without a stack frame per level,
/// reads its extension points the same way, and the writer refuses it rather than write text
/// the reader's depth limit (128 arrays and objects) turns away.
#[test]
fn a_tree_of_any_depth_drops_and_is_refused_past_the_reader_s_limit() {
    let deep = nested(300_000);
    assert!(deep.extension_points().is_empty());
    assert!(matches!(
        deep.to_json(),
        Err(EncodeError::NotRepresentable { .. })
    ));
    drop(deep);
    // Whatever is written reads back, at every depth around the limit.
    let mut written_depths = 0;
    for depth in 0..=70 {
        let tree = nested(depth);
        match tree.to_json() {
            Ok(text) => {
                written_depths += 1;
                assert_eq!(*loads(root(&text)).value(), tree, "depth {depth}");
            }
            Err(EncodeError::NotRepresentable { .. }) => {}
            Err(other) => panic!("{other}"),
        }
    }
    // 128 levels: the envelope, the payload, `layout`, `markup`, and two per control.
    assert_eq!(written_depths, 61);
    // The deepest text the reader takes, read from text as a file would be.
    let text = nested(60).to_json().unwrap();
    assert!(nested(61).to_json().is_err());
    loads(root(&text));
}

/// `attribs` nested past the limit is refused by the writer too.
#[test]
fn attribs_nested_past_the_reader_s_limit_are_refused() {
    let mut value = json("0");
    for _ in 0..200 {
        let mut array = facet_value::VArray::new();
        array.push(value);
        value = Value::from(array);
    }
    let mut root = valid_root();
    root.layout.markup[0].children.as_mut().unwrap()[0].attribs = Some(value);
    assert_eq!(
        refused_at(&root),
        "minecraft:ui-root/layout/markup/0/children/0/attribs"
    );
}

/// A leaf one level past the deepest container the reader takes: its object would be the
/// 129th level, so the writer refuses it at the control.
#[test]
fn a_leaf_one_level_past_the_reader_s_limit_is_refused() {
    let chain = |leaf: bool| {
        let innermost = if leaf {
            vec![Control::new(Component::FormDivider)]
        } else {
            Vec::new()
        };
        let mut control = Control::with_children(Component::Visibility, innermost);
        for _ in 0..60 {
            control = Control::with_children(Component::Visibility, vec![control]);
        }
        UiRoot::new(
            UiRootId::parse("demo:deep"),
            vec![Control::with_children(Component::Panel, vec![control])],
        )
    };
    let text = chain(false).to_json().expect("128 levels are written");
    loads(root(&text));
    let at = refused_at(&chain(true));
    assert!(at.ends_with("/children/0/component"), "{at}");
    assert_eq!(at.matches("children").count(), 62);
}

/// A slider whose `attribs` hold `value` once at the top, once inside an array below an
/// object, with a number a double holds and a member after it.
fn slider(value: &str) -> String {
    panel(&format!(
        r#"{{ "component": "Form.Slider", "attribs": {{ "value": {value}, "deep": {{ "list": [1, {value}] }}, "half": 0.5, "after": true }} }}"#
    ))
}

/// A number in `attribs` that no double holds refuses the document: the game's JSON parser stops at it
/// (a number-too-big parse error) and the game does not load the file (a JSON reader that clamped
/// would not match it).
#[test]
fn a_number_beyond_a_double_in_attribs_refuses_the_file() {
    for token in ["1e999", "-1e999", "1.8e308", "-1.8e308", "0.5e400"] {
        let text = slider(token);
        let refused = rejects(root(&text));
        assert_eq!(
            found(&refused),
            [
                error(
                    "MalformedInput",
                    "",
                    "Failed to parse the Minecraft Document JSON: Number too big to be stored in double."
                ),
                failed_to_load()
            ],
            "{token}"
        );
        assert_eq!(spanned(&text, refused.iter().next().unwrap()), token);
    }
}

/// The largest double itself is a number a double holds: no finding, written as authored.
#[test]
fn the_largest_double_in_attribs_is_no_finding() {
    for token in ["1.7976931348623157e308", "-1.7976931348623157e308", "1e39"] {
        let text = slider(token);
        let document = loads(root(&text));
        assert_eq!(found(document.diagnostics()), [], "{token}");
        let written = document.to_json().expect("writes");
        assert_eq!(json(&written), json(&text), "{token}");
    }
}

/// An integer in `attribs` beyond 64 bits is the double it spells (core's rule): held as the nearest double without a finding, never as a
/// string of its digits, and written as the shortest decimal of that double. The widest
/// integers 64 bits hold are held and written as authored.
#[test]
fn an_integer_beyond_64_bits_in_attribs_is_held_as_the_nearest_double_without_a_finding() {
    for token in ["18446744073709551615", "-9223372036854775808"] {
        let text = slider(token);
        let document = loads(root(&text));
        assert_eq!(found(document.diagnostics()), [], "{token}");
        let written = document.to_json().expect("writes");
        assert_eq!(json(&written), json(&text), "{token}");
        assert_eq!(written.matches(token).count(), 2, "{token}\n{written}");
    }
    for (token, held, spelt) in [
        (
            "18446744073709551616",
            18_446_744_073_709_551_616.0,
            "1.8446744073709552e19",
        ),
        (
            "-9223372036854775809",
            -9_223_372_036_854_775_809.0,
            "-9.223372036854776e18",
        ),
        (
            "123456789012345678901234567890",
            1.234_567_890_123_456_8e29,
            "1.2345678901234568e29",
        ),
        (
            "340282366920938463463374607431768211455",
            3.402_823_669_209_385e38,
            "3.402823669209385e38",
        ),
        // Beyond 128 bits, which the JSON reader once refused the file for.
        (
            "340282366920938463463374607431768211456",
            3.402_823_669_209_385e38,
            "3.402823669209385e38",
        ),
        (
            "-170141183460469231731687303715884105729",
            -1.701_411_834_604_692_3e38,
            "-1.7014118346046923e38",
        ),
    ] {
        let text = slider(token);
        let document = loads(root(&text));
        assert_eq!(found(document.diagnostics()), [], "{token}");
        let attribs = document.value().layout.markup[0].children()[0]
            .attribs
            .as_ref()
            .expect("attribs");
        let object = attribs.as_object().expect("an object");
        let deep = object
            .get("deep")
            .and_then(Value::as_object)
            .and_then(|deep| deep.get("list"))
            .and_then(Value::as_array)
            .and_then(|list| list.get(1));
        for value in [object.get("value"), deep] {
            let number = value.and_then(Value::as_number).expect("a number");
            assert!(number.is_float(), "{token}");
            assert_eq!(
                number.to_f64_lossy().to_bits(),
                f64::to_bits(held),
                "{token}"
            );
        }
        let written = document.to_json().expect("a loaded document is written");
        // The author's digits are respelt: the shortest decimal of the double, twice.
        assert!(!written.contains(token), "{token}\n{written}");
        assert_eq!(written.matches(spelt).count(), 2, "{token}\n{written}");
        assert_eq!(json(&written), json(&text), "{token}");
        let again = loads(root(&written));
        assert_eq!(found(again.diagnostics()), [], "{token}\n{written}");
        assert_eq!(again.value(), document.value(), "{token}");
        assert_eq!(again.to_json().expect("writes again"), written);
    }
}

/// A zero whose exponent alone is past the double range (`0e999`) is not seen by the document
/// text scan of core (it takes `0.5e400`); `attribs` tells it at the member with the game's text, each
/// occurrence an error, and the file is refused (the scan of core takes the
/// spellings that evaluate to an infinity first, with the same text at the document).
#[test]
fn an_exponent_beyond_a_double_in_attribs_refuses_the_file_at_the_member() {
    let token = "0e999";
    let text = slider(token);
    let refused = rejects(root(&text));
    let at = "minecraft:ui-root/layout/markup/0/children/0/attribs";
    assert_eq!(
        found(&refused),
        [
            error(
                "InvalidValue",
                at,
                "Failed to parse the Minecraft Document JSON: Number too big to be stored in double."
            ),
            error(
                "InvalidValue",
                at,
                "Failed to parse the Minecraft Document JSON: Number too big to be stored in double."
            ),
            failed_to_load()
        ],
        "{token}"
    );
}

/// An integer no double holds is what `1e999` is: it refuses the document.
#[test]
fn an_integer_beyond_a_double_in_attribs_refuses_the_file() {
    for sign in ["", "-"] {
        let token = format!("{sign}{}", "9".repeat(400));
        let text = slider(&token);
        let refused = rejects(root(&text));
        assert_eq!(
            found(&refused),
            [
                error(
                    "MalformedInput",
                    "",
                    "Failed to parse the Minecraft Document JSON: Number too big to be stored in double."
                ),
                failed_to_load()
            ],
            "{sign}"
        );
        assert_eq!(spanned(&text, refused.iter().next().unwrap()), token);
    }
}

/// A string of digits is a string: held and written as one, whatever its length, and a
/// number beside it is still read.
#[test]
fn a_long_string_of_digits_in_attribs_stays_a_string() {
    let text = panel(
        r#"{ "component": "Form.Slider", "attribs": { "id": "123456789012345678901234567890", "n": 7 } }"#,
    );
    let document = loads(root(&text));
    assert_eq!(found(document.diagnostics()), []);
    let written = document.to_json().expect("writes");
    assert_eq!(json(&written), json(&text));
}

/// A number no double holds as deep inside `attribs` as the reader's limit lets a file nest is
/// refused like any other; one level more is past the limit and refused as well.
#[test]
fn a_number_beyond_a_double_at_the_deepest_level_the_reader_takes_refuses_the_file() {
    // Seven levels stand above the slider's `attribs` value; 128 are read.
    let levels = 121;
    let text = panel(&format!(
        r#"{{ "component": "Form.Slider", "attribs": {}1e999{} }}"#,
        "[".repeat(levels),
        "]".repeat(levels)
    ));
    let refused = rejects(root(&text));
    assert_eq!(
        found(&refused),
        [
            error(
                "MalformedInput",
                "",
                "Failed to parse the Minecraft Document JSON: Number too big to be stored in double."
            ),
            failed_to_load()
        ]
    );
    // One level more is past the limit: the file is refused whole, with an error.
    let text = text.replace("[1e999]", "[[1e999]]");
    assert!(rejects(root(&text)).has_errors());
}

/// A `minecraft:` document is written flat: only a shipped pack may hold the namespace (V6),
/// and its walk does not read sub-folders.
#[test]
fn the_default_path_of_a_minecraft_document_has_no_namespace_folder() {
    let root = UiRoot::new(UiRootId::parse("minecraft:shop/main"), Vec::new());
    assert_eq!(root.default_path().as_str(), "ddui/root/shop_main.json");
    let composition = UiComposition::new(UiCompositionId::parse("minecraft:extra"), Vec::new());
    assert_eq!(
        composition.default_path().as_str(),
        "ddui/composition/extra.json"
    );
}
