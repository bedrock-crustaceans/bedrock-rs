//! Cost: every function that walks a file or a stack is called here on inputs built
//! beforehand, at `n` and at four times `n` entries, and held to the crate's one protocol
//! (`bedrock_addon_core::test_cost`): a function that looked at what it had seen for every entry
//! would take sixteen times as long for four times the input; the bound is eight.

use std::time::Duration;

use bedrock_addon_core::load::{LoadContext, LoadOptions, StackCx};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};

use super::load::{FileOutcome, UiFile, combine};
use super::{Component, Control, UiComposition, UiRoot, read, write};
use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
use bedrock_addon_core::document::{Content, Document, Origin};
use bedrock_addon_core::id::Id;
use bedrock_addon_core::json::{JsonSlice, Value};
use bedrock_addon_core::test_cost::{assert_linear_timed_at, timed};
use bedrock_addon_core::version::FormatVersion;

/// The smaller size; the larger is four times it.
const SINGLE: usize = 25_000;
/// The smaller size of the tests that read a text: a control takes some 50 microseconds to read.
const READ: usize = 1_000;
/// The smaller size of the tests of the combine, whose input is two documents an entry.
const COMBINE: usize = 5_000;
const ENGINE: FormatVersion = FormatVersion::new(1, 26, 45);

/// The payload of a root whose panel, `levels` `Visibility` controls down, holds `controls`.
fn payload(levels: usize, controls: &str) -> String {
    let mut markup = String::from(r#"{ "component": "Panel", "children": ["#);
    for _ in 0..levels {
        markup.push_str(r#"{ "component": "Visibility", "children": ["#);
    }
    markup.push_str(controls);
    for _ in 0..levels {
        markup.push_str("] }");
    }
    markup.push_str("] }");
    format!(
        r#"{{ "description": {{ "identifier": "demo:screen" }}, "attribs": {{ "root_point": "minecraft:screen" }}, "layout": {{ "markup": [ {markup} ] }} }}"#
    )
}

/// `count` controls, each `control(i)`, as the elements of an array.
fn list(count: usize, control: impl Fn(usize) -> String) -> String {
    let controls: Vec<String> = (0..count).map(control).collect();
    controls.join(", ")
}

/// Reads `text` as a root payload: the value, the findings and the time the read took.
fn read_root(text: &str) -> (Option<UiRoot>, Diagnostics, Duration) {
    let at = JsonPath::of_keys(["minecraft:ui-root"]);
    let mut diagnostics = Diagnostics::default();
    let (root, took) = timed(|| read::root(JsonSlice::new(text), ENGINE, &at, &mut diagnostics));
    (root, diagnostics, took)
}

fn button(i: usize) -> String {
    format!(r#"{{ "component": "Form.Button", "attribs": {{ "label": "b{i}", "n": [{i}] }} }}"#)
}

#[test]
fn reading_a_long_children_array_is_linear() {
    assert_linear_timed_at("read, flat", READ, |count| {
        let text = payload(0, &list(count, button));
        let (root, diagnostics, took) = read_root(&text);
        assert!(diagnostics.is_empty());
        assert_eq!(root.unwrap().layout.markup[0].children().len(), count);
        took
    });
}

/// The long array six levels down. A level is read once however many controls stand below
/// it, so the time grows with the controls and not with their product with the depth (which
/// the reader's limit on nesting bounds besides).
#[test]
fn reading_a_long_children_array_below_six_levels_is_linear() {
    assert_linear_timed_at("read, six levels down", READ, |count| {
        let text = payload(
            6,
            &list(count, |_| r#"{ "component": "Form.Divider" }"#.to_owned()),
        );
        let (root, diagnostics, took) = read_root(&text);
        assert!(diagnostics.is_empty());
        let root = root.unwrap();
        assert_eq!(super::controls(&root.layout.markup).count(), count + 7);
        took
    });
}

/// Every control refused: one finding each, each with its own path and span.
#[test]
fn reading_a_file_whose_every_control_is_refused_is_linear() {
    assert_linear_timed_at("read, every control refused", READ, |count| {
        let text = payload(
            3,
            &list(count, |i| match i % 4 {
                0 => r#"{ "component": "Nope" }"#.to_owned(),
                1 => r#"{ "component": "Form.Button", "children": [] }"#.to_owned(),
                2 => r#"{ "component": "Panel.Text" }"#.to_owned(),
                _ => r#"{ "component": "Panel", "children": [] }"#.to_owned(),
            }),
        );
        let (root, diagnostics, took) = read_root(&text);
        assert!(root.is_none());
        assert_eq!(diagnostics.len(), count);
        took
    });
}

/// A root with `count` extension points, each of its own name.
fn declaring(identifier: &str, first: usize, count: usize) -> UiRoot {
    let points = (first..first + count)
        .map(|i| {
            let mut point = Control::new(Component::ExtensionPoint);
            point.attribs =
                bedrock_addon_core::json::from_str::<Value>(&format!(r#"{{"name": "demo:p{i}"}}"#))
                    .ok();
            point
        })
        .collect();
    UiRoot::new(
        Id::parse(identifier.to_owned()),
        vec![Control::with_children(Component::Panel, points)],
    )
}

#[test]
fn collecting_the_extension_points_of_a_root_is_linear() {
    assert_linear_timed_at("extension points of one root", SINGLE, |count| {
        let root = declaring("demo:r", 0, count);
        let (points, took) = timed(|| root.extension_points());
        assert_eq!(points.len(), count);
        took
    });
}

#[test]
fn writing_a_long_children_array_is_linear() {
    assert_linear_timed_at("write, flat", SINGLE, |count| {
        let root = declaring("demo:r", 0, count);
        let (written, took) = timed(|| write::root_document(&root).expect("writes"));
        assert!(written.as_object().is_some());
        took
    });
}

fn origin(directory: &str, i: usize) -> Origin {
    let path = PackPath::new(&format!("ddui/{directory}/f{i}.json")).expect("a pack path");
    Origin::file(PackRef::app_data(), PackSide::Resource, path)
}

/// The context of a pack the game ships: a composition may name an extension point there.
fn shipped() -> LoadContext {
    let mut cx = LoadContext::detached();
    cx.is_base_game_pack = true;
    cx
}

/// What the walk hands the combine for `count` roots, each declaring one extension point of
/// its own, and `count` compositions, each spliced in at the point `point(i)` names.
fn files(count: usize, point: impl Fn(usize) -> usize, same_identifier: bool) -> Vec<UiFile> {
    let cx = shipped();
    let template = UiComposition::parse(
        &PackPath::from_static("ddui/composition/t.json"),
        br#"{ "format_version": "1.21.130", "minecraft:ui-composition": { "description": { "identifier": "minecraft:t" }, "attribs": { "extension_point": "demo:p0" }, "layout": { "markup": [ { "component": "Form.Divider" } ] } } }"#,
        &cx,
    )
    .expect("the template loads");
    let version = template.version().clone();
    let mut parsed = Vec::with_capacity(count * 2);
    for i in 0..count {
        let name = if same_identifier { 0 } else { i };
        let root = declaring(&format!("minecraft:r{name}"), i, 1);
        parsed.push(UiFile::Root(Box::new(FileOutcome {
            identifier: Some(root.description.identifier.as_str().to_owned()),
            result: Ok(Document::new(
                root,
                origin("root", i),
                version.clone(),
                Diagnostics::default(),
            )),
            extension_point: None,
        })));
    }
    for i in 0..count {
        let name = if same_identifier { 0 } else { i };
        let mut composition = template.value().clone();
        composition.description.identifier = Id::parse(format!("minecraft:c{name}"));
        let asked = format!("demo:p{}", point(i));
        composition.attribs.extension_point = Some(asked.clone());
        parsed.push(UiFile::Composition(Box::new(FileOutcome {
            identifier: Some(composition.description.identifier.as_str().to_owned()),
            result: Ok(Document::new(
                composition,
                origin("composition", i),
                version.clone(),
                Diagnostics::default(),
            )),
            extension_point: Some((asked, None)),
        })));
    }
    parsed
}

/// Every composition names a point of another root: one lookup each, whatever the number of
/// roots (the client walks every stored root for every composition).
#[test]
fn combining_compositions_against_many_roots_is_linear() {
    assert_linear_timed_at("combine, declared points", COMBINE, |count| {
        let parsed = files(count, |i| count - 1 - i, false);
        let options = LoadOptions::default();
        let (ui, took) = timed(|| combine(parsed, &StackCx::new(&options, &[])));
        assert_eq!(ui.roots.len(), count);
        assert_eq!(ui.compositions.len(), count);
        assert_eq!(ui.diagnostics().count(), 0);
        took
    });
}

/// Every composition names a point no root declares: each is refused with its finding.
#[test]
fn combining_compositions_at_undeclared_points_is_linear() {
    assert_linear_timed_at("combine, undeclared points", COMBINE, |count| {
        let parsed = files(count, |i| count + i, false);
        let options = LoadOptions::default();
        let (ui, took) = timed(|| combine(parsed, &StackCx::new(&options, &[])));
        assert_eq!(ui.compositions.len(), 0);
        assert_eq!(ui.compositions.rejected().len(), count);
        assert_eq!(ui.diagnostics().count(), count * 2);
        took
    });
}

/// Every file defines the one identifier: each replaces the one before it, in its place.
#[test]
fn combining_many_definitions_of_one_identifier_is_linear() {
    assert_linear_timed_at("combine, one identifier", COMBINE, |count| {
        let parsed = files(count, |_| count - 1, true);
        let options = LoadOptions::default();
        let (ui, took) = timed(|| combine(parsed, &StackCx::new(&options, &[])));
        assert_eq!(ui.roots.len(), 1);
        assert_eq!(ui.compositions.len(), 1);
        // Only the last root stands, and with it only the point it declares.
        assert_eq!(ui.diagnostics().count(), 2 * (count - 1));
        took
    });
}

/// Every number of an `attribs` is one no double holds: the first refuses the document
/// (one finding for the first number, not one replaced finding for each) and the read stops
/// there, in linear time whatever follows it.
#[test]
fn reading_attribs_whose_every_number_is_too_big_is_linear() {
    assert_linear_timed_at(
        "read, every number of attribs replaced",
        2 * READ,
        |count| {
            let numbers = list(count, |i| {
                if i % 2 == 0 { "1e999" } else { "-1e999" }.to_owned()
            });
            let text = payload(
                0,
                &format!(
                    r#"{{ "component": "Form.Slider", "attribs": {{ "list": [{numbers}] }} }}"#
                ),
            );
            let (root, diagnostics, took) = read_root(&text);
            assert!(root.is_none(), "the first number refuses the document");
            assert!(!diagnostics.is_empty());
            took
        },
    );
}

#[test]
fn loaded_file_types_implement_debug() {
    fn assert_debug<T: std::fmt::Debug>() {}
    assert_debug::<UiFile>();
    assert_debug::<FileOutcome<UiRoot>>();
}

#[test]
fn a_control_reads_its_children_by_the_field_name() {
    let control = Control {
        component: Component::FormPanel,
        attribs: None,
        children: None,
    };
    assert!(control.children().is_empty());
}
