//! The rows that need the stack (`rp.data_driven_ui()`): discovery (d01, d02), the
//! extension-point rules across files (e02–e10), and how the packs combine (k01–k04). In
//! every stack the first pack is the highest priority; the fixtures' `pack_0` is the last.
//! The stack reads with the engine the crate tracks, whose catalogue is 1.26.60's.

use std::io::ErrorKind;

use bedrock_addon::diagnostic::{Diagnostic, Kind, Severity};
use bedrock_addon::families::ddui::{Component, Control, DataDrivenUi};
use bedrock_addon::id::{UiCompositionId, UiRootId};
use bedrock_addon::load::{
    Collection, CollectionSet, LoadOptions, RepositoryLoader, Request, Residency, RootId, Source,
    StackLoader, StackOptions,
};
use bedrock_addon::pack::{OpenOptions, PackOrigin, PackStack, Resource, WorldPacks};

use super::common::*;
use super::responder::{Order, Responder, manifest};

const OVERRIDEN: &str = "extension_point cannot be overriden at external packs.";
const EXTERNAL: &str =
    "external packs must not use the 'minecraft' namespace for DataDrivenUI documents";
const POINT_AT: &str = "minecraft:ui-composition/attribs/extension_point";
const DIVIDER: &str = r#"{ "component": "Form.Divider" }"#;

fn not_defined(name: &str) -> Found {
    error(
        "ddui/not_defined",
        POINT_AT,
        &format!("extension_point '{name}' is not defined.: extension point '{name}'"),
    )
}

/// A root named `identifier` whose panel holds `children`.
fn root_with(identifier: &str, children: &str) -> String {
    root_json(
        identifier,
        &format!(r#"{{ "component": "Panel", "children": [ {children} ] }}"#),
    )
}

fn point(name: &str) -> String {
    format!(r#"{{ "component": "ExtensionPoint", "attribs": {{ "name": "{name}" }} }}"#)
}

/// A root named `identifier` that declares the extension point `name`.
fn declaring(identifier: &str, name: &str) -> String {
    root_with(identifier, &point(name))
}

/// A composition named `identifier` spliced in at `name`, with one button labelled `label`.
fn spliced(identifier: &str, name: &str, label: &str) -> String {
    composition_json(
        identifier,
        &format!(r#""extension_point": "{name}""#),
        &format!(r#"{{ "component": "Form.Button", "attribs": {{ "label": "{label}" }} }}"#),
    )
}

fn root_id(text: &str) -> UiRootId {
    UiRootId::parse(text)
}

fn composition_id(text: &str) -> UiCompositionId {
    UiCompositionId::parse(text)
}

/// The identifiers of a registry in store order, each with the pack its document is from.
fn stored<'a, K, T: 'a>(
    entries: impl Iterator<
        Item = (
            &'a bedrock_addon::id::Id<K>,
            &'a bedrock_addon::document::Document<T>,
        ),
    >,
) -> Vec<(String, String)>
where
    K: bedrock_addon::id::Kind + 'a,
{
    entries
        .map(|(id, document)| {
            (
                id.as_str().to_owned(),
                document.origin().pack.name().to_owned(),
            )
        })
        .collect()
}

/// `(pack, file)` of each finding named `code`.
fn at(ui: &DataDrivenUi, code: &str) -> Vec<(String, String)> {
    ui.diagnostics()
        .filter(|f| f.code.id() == code)
        .map(place)
        .collect()
}

/// `(pack, file)` of each finding of the kind `kind`.
fn at_kind(ui: &DataDrivenUi, kind: Kind) -> Vec<(String, String)> {
    ui.diagnostics()
        .filter(|f| f.code.kind() == kind)
        .map(place)
        .collect()
}

fn place(finding: &Diagnostic) -> (String, String) {
    (
        finding
            .location
            .pack
            .as_ref()
            .map(|pack| pack.name().to_owned())
            .unwrap_or_default(),
        finding
            .location
            .path
            .as_ref()
            .map(|path| path.as_str().to_owned())
            .unwrap_or_default(),
    )
}

fn pair(pack: &str, file: &str) -> (String, String) {
    (pack.to_owned(), file.to_owned())
}

fn labels(ui: &DataDrivenUi, name: &str) -> Vec<String> {
    ui.extension_point_contents(name)
        .map(|control| {
            let attribs = control.attribs.as_ref().unwrap().as_object().unwrap();
            attribs
                .get("label")
                .unwrap()
                .as_string()
                .unwrap()
                .as_str()
                .to_owned()
        })
        .collect()
}

// ---- discovery -----------------------------------------------------------------------------

#[test]
fn d01_only_an_extension_of_exactly_json_is_read_deviation() {
    // vanilla = "developer log only": the two other files are an info finding each.
    let stack = stack(vec![external(
        1,
        "addon",
        vec![
            ("ddui/root/a.JSON", root_json("demo:upper", R_MARKUP)),
            ("ddui/root/b.txt", root_json("demo:text", R_MARKUP)),
            ("ddui/root/c.json", root_json("demo:c", R_MARKUP)),
            (
                "ddui/composition/d.jsonc",
                composition_json("demo:d", "", DIVIDER),
            ),
            (
                "ddui/composition/e.json",
                composition_json("demo:e", "", DIVIDER),
            ),
            ("ddui/f.json", root_json("demo:above", R_MARKUP)),
            ("ddui/rootx/g.json", root_json("demo:beside", R_MARKUP)),
        ],
    )]);
    let ui = stack.data_driven_ui();
    assert_eq!(stored(ui.roots.iter()), [pair("demo:c", "addon")]);
    assert_eq!(stored(ui.compositions.iter()), [pair("demo:e", "addon")]);
    assert_eq!(
        at(ui, "core/ignored_file"),
        [
            pair("addon", "ddui/root/a.JSON"),
            pair("addon", "ddui/root/b.txt"),
            pair("addon", "ddui/composition/d.jsonc"),
        ]
    );
    assert!(ui.diagnostics().all(|f| f.severity == Severity::Info));
    assert_eq!(ui.diagnostics().count(), 3);
    assert!(std::ptr::eq(ui, stack.data_driven_ui()));
}

#[test]
fn d02_a_sub_folder_is_read_in_an_add_on_pack_and_not_in_a_shipped_one() {
    let stack = stack(vec![
        external(
            1,
            "addon",
            vec![
                ("ddui/root/sub/x.json", root_json("demo:nested", R_MARKUP)),
                (
                    "ddui/root/sub/deeper/y.json",
                    root_json("demo:deeper", R_MARKUP),
                ),
                (
                    "ddui/composition/sub/c.json",
                    composition_json("demo:c", "", DIVIDER),
                ),
                ("ddui/root/sub/z.txt", String::new()),
            ],
        ),
        builtin(
            2,
            "vanilla",
            vec![
                ("ddui/root/top.json", root_json("minecraft:top", R_MARKUP)),
                (
                    "ddui/root/sub/x.json",
                    root_json("minecraft:nested", R_MARKUP),
                ),
                (
                    "ddui/composition/sub/c.json",
                    composition_json("minecraft:c", "", DIVIDER),
                ),
                ("ddui/root/sub/z.txt", String::new()),
            ],
        ),
    ]);
    let ui = stack.data_driven_ui();
    assert_eq!(
        stored(ui.roots.iter()),
        [
            pair("demo:deeper", "addon"),
            pair("demo:nested", "addon"),
            pair("minecraft:top", "vanilla")
        ]
    );
    assert_eq!(stored(ui.compositions.iter()), [pair("demo:c", "addon")]);
    // The file the shipped pack's walk never reaches is no ignored file either.
    assert_eq!(
        at(ui, "core/ignored_file"),
        [pair("addon", "ddui/root/sub/z.txt")]
    );
}

/// The paths a load read, in request order.
fn read_paths(responder: &Responder) -> Vec<String> {
    responder
        .log
        .requests
        .iter()
        .filter_map(|request| match request {
            Request::ReadFile { location, .. } => Some(location.path.as_str().to_owned()),
            Request::ListDir { .. } => None,
        })
        .collect()
}

/// A load asks for the bytes of the files it parses and of no other: not the files with
/// another extension, not the nested files of a shipped pack.
#[test]
fn a_load_requests_the_bytes_of_the_files_it_parses_only() {
    for (origin, nested_read) in [(PackOrigin::World, true), (PackOrigin::Package, false)] {
        let mut responder = Responder::new(Order::Issue);
        let id = if origin == PackOrigin::Package {
            "minecraft"
        } else {
            "demo"
        };
        let root = |name: &str| root_json(&format!("{id}:{name}"), R_MARKUP).into_bytes();
        responder
            .file(1, "rp/manifest.json", manifest(2, "resources"))
            .file(1, "rp/ddui/root/a.json", root("a"))
            .file(1, "rp/ddui/root/sub/b.json", root("b"))
            .file(1, "rp/ddui/root/c.txt", root("c"))
            .file(1, "rp/ddui/root/d.JSON", root("d"))
            .file(1, "rp/ddui/composition/e.png", &b"not read"[..])
            .file(1, "rp/ddui/other/f.json", root("f"));
        let mut repository = RepositoryLoader::new(
            vec![Source::Directory {
                root: RootId(1),
                origin,
            }],
            OpenOptions::default(),
        );
        let repository = responder.drive(&mut repository);
        let world = format!(
            r#"[{{"pack_id": "{}", "version": [1, 0, 0]}}]"#,
            uuid::Uuid::from_u128(2)
        );
        let world = WorldPacks::parse(Some(b"[]"), Some(world.as_bytes())).expect("a world list");
        let before = responder.log.requests.len();
        let options = StackOptions {
            collections: CollectionSet::only([Collection::DataDrivenUi]),
            resource: Residency::Transient,
            behavior: Residency::Transient,
            ..StackOptions::default()
        };
        let mut stack_loader =
            StackLoader::new(repository.stacks(&world, LoadOptions::default()), options);
        let loaded = responder.drive(&mut stack_loader);
        assert!(
            responder.log.requests.len() > before,
            "the documents were read"
        );
        let mut read = read_paths(&responder);
        read.retain(|path| path.contains("ddui/"));
        read.sort();
        read.dedup();
        let mut expected = vec!["rp/ddui/root/a.json".to_owned()];
        if nested_read {
            expected.push("rp/ddui/root/sub/b.json".to_owned());
        }
        assert_eq!(read, expected, "{origin:?}");
        let ui = loaded.resource.data_driven_ui();
        assert_eq!(ui.roots.len(), expected.len(), "{origin:?}");
        assert_eq!(at(ui, "core/ignored_file").len(), 3, "{origin:?}");
    }
}

/// V2: a file that cannot be read is skipped with the stack driver's `ReadError` error on the
/// file, with the loader's own text naming the file; the other files of the folder load, in every completion order, for a root and
/// for a composition.
#[test]
fn v2_a_file_that_cannot_be_read_is_skipped_with_a_read_error() {
    for order in Order::ALL {
        let mut responder = Responder::new(Order::Issue);
        responder
            .file(1, "rp/manifest.json", manifest(2, "resources"))
            .file(1, "rp/ddui/root/a.json", root_json("demo:a", R_MARKUP))
            .file(1, "rp/ddui/root/b.json", root_json("demo:b", R_MARKUP))
            .file(
                1,
                "rp/ddui/composition/c.json",
                composition_json("demo:c", "", DIVIDER),
            )
            .file(
                1,
                "rp/ddui/composition/d.json",
                composition_json("demo:d", "", DIVIDER),
            );
        let mut repository = RepositoryLoader::new(
            vec![Source::Directory {
                root: RootId(1),
                origin: PackOrigin::World,
            }],
            OpenOptions::default(),
        );
        let repository = responder.drive(&mut repository);
        let world = format!(
            r#"[{{"pack_id": "{}", "version": [1, 0, 0]}}]"#,
            uuid::Uuid::from_u128(2)
        );
        let world = WorldPacks::parse(Some(b"[]"), Some(world.as_bytes())).expect("a world list");
        let mut responder = responder.reordered(order);
        responder
            .failed(1, "rp/ddui/root/a.json", ErrorKind::PermissionDenied)
            .failed(1, "rp/ddui/composition/d.json", ErrorKind::NotFound);
        let options = StackOptions {
            collections: CollectionSet::only([Collection::DataDrivenUi]),
            resource: Residency::Transient,
            behavior: Residency::Transient,
            ..StackOptions::default()
        };
        let mut stack_loader =
            StackLoader::new(repository.stacks(&world, LoadOptions::default()), options);
        let loaded = responder.drive(&mut stack_loader);
        let ui = loaded.resource.data_driven_ui();
        let roots: Vec<&str> = ui.roots.iter().map(|(id, _)| id.as_str()).collect();
        assert_eq!(roots, ["demo:b"], "{order:?}");
        let compositions: Vec<&str> = ui.compositions.iter().map(|(id, _)| id.as_str()).collect();
        assert_eq!(compositions, ["demo:c"], "{order:?}");
        let mut unread: Vec<(String, Severity, String)> = loaded
            .resource
            .report()
            .iter()
            .filter_map(|f| match f.detail.as_deref() {
                // The read-error definitions carry the host's kind as the detail.
                Some(bedrock_addon::diagnostic::DiagnosticDetail::Io(kind))
                    if f.code.id() == "core/read_error_worded" => {
                    let path = f.location.path.as_ref()?.as_str().to_owned();
                    assert_eq!(
                        f.code.to_string(),
                        format!(
                            "Failed to load the contents of DataDrivenUI file {path}; it will be skipped."
                        ),
                        "{order:?}"
                    );
                    Some((path, f.severity, format!("{kind:?}")))
                }
                _ => None,
            })
            .collect();
        unread.sort();
        assert_eq!(
            unread,
            [
                (
                    "ddui/composition/d.json".to_owned(),
                    Severity::Error,
                    "NotFound".to_owned()
                ),
                (
                    "ddui/root/a.json".to_owned(),
                    Severity::Error,
                    "PermissionDenied".to_owned()
                ),
            ],
            "{order:?}"
        );
        // Nothing else is found: the unread file is no rejected document of the family.
        // (The pack has no icon: that is the stack's own info.)
        let report = loaded.resource.report();
        let others = report
            .iter()
            .filter(|f| f.code.id() != "core/pack_icon_missing")
            .count();
        assert_eq!(others, 2, "{order:?}");
        assert_eq!(ui.diagnostics().count(), 0, "{order:?}");
    }
}

// ---- extension points ----------------------------------------------------------------------

#[test]
fn e02_a_shipped_composition_at_a_declared_point_is_stored() {
    let stack = stack(vec![builtin(
        1,
        "vanilla",
        vec![
            ("ddui/root/r.json", declaring("minecraft:r", "demo:slot")),
            (
                "ddui/composition/c.json",
                spliced("minecraft:c", "demo:slot", "one"),
            ),
        ],
    )]);
    let ui = stack.data_driven_ui();
    assert_eq!(ui.diagnostics().count(), 0);
    assert_eq!(
        stored(ui.compositions.iter()),
        [pair("minecraft:c", "vanilla")]
    );
    assert_eq!(labels(ui, "demo:slot"), ["one"]);
    assert!(labels(ui, "demo:other").is_empty());
    let root = ui.roots.get(&root_id("minecraft:r")).unwrap();
    assert_eq!(root.extension_points(), ["demo:slot"]);
}

/// From 1.26.60 a point below `Context.List` is declared, so a composition
/// spliced in there is stored.
#[test]
fn e02_a_point_below_a_context_list_takes_a_shipped_composition() {
    let listed = format!(
        r#"{{ "component": "Context.List", "children": [ {} ] }}"#,
        point("demo:listed")
    );
    let stack = stack(vec![builtin(
        1,
        "vanilla",
        vec![
            ("ddui/root/r.json", root_with("minecraft:r", &listed)),
            (
                "ddui/composition/c.json",
                spliced("minecraft:c", "demo:listed", "one"),
            ),
        ],
    )]);
    let ui = stack.data_driven_ui();
    assert_eq!(ui.diagnostics().count(), 0);
    assert_eq!(labels(ui, "demo:listed"), ["one"]);
    let root = ui.roots.get(&root_id("minecraft:r")).unwrap();
    assert_eq!(root.extension_points(), ["demo:listed"]);
}

#[test]
fn e03_a_shipped_composition_at_an_undeclared_point_is_refused() {
    let composition = spliced("minecraft:c", "demo:none", "one");
    let stack = stack(vec![builtin(
        1,
        "vanilla",
        vec![
            ("ddui/root/r.json", declaring("minecraft:r", "demo:slot")),
            ("ddui/composition/c.json", composition.clone()),
        ],
    )]);
    let ui = stack.data_driven_ui();
    assert!(ui.compositions.is_empty());
    assert_eq!(ui.compositions.rejected().len(), 1);
    let rejected = &ui.compositions.rejected()[0];
    assert_eq!(
        found(&rejected.diagnostics),
        [not_defined("demo:none"), failed_to_load()]
    );
    let finding = rejected.diagnostics.iter().next().unwrap();
    assert_eq!(spanned(&composition, finding), r#""demo:none""#);
    assert_eq!(place(finding), pair("vanilla", "ddui/composition/c.json"));
    assert_eq!(found(ui.diagnostics()), found(&rejected.diagnostics));
    assert!(labels(ui, "demo:none").is_empty());
}

/// The name is compared whole and with case.
#[test]
fn e03_a_point_is_declared_by_its_exact_name_only() {
    let files = |name: &str| {
        vec![
            ("ddui/root/r.json", declaring("minecraft:r", "demo:slot")),
            (
                "ddui/composition/c.json",
                spliced("minecraft:c", name, "one"),
            ),
        ]
    };
    for name in ["demo:Slot", "demo:slot ", "demo:slo", "slot"] {
        let stack = stack(vec![builtin(1, "vanilla", files(name))]);
        let ui = stack.data_driven_ui();
        assert!(ui.compositions.is_empty(), "{name}");
        assert_eq!(found(ui.diagnostics())[0], not_defined(name));
    }
}

#[test]
fn e04_an_add_on_composition_at_a_declared_point_is_refused_for_being_external_only() {
    let stack = stack(vec![external(
        1,
        "addon",
        vec![
            ("ddui/root/r.json", declaring("demo:r", "demo:slot")),
            (
                "ddui/composition/c.json",
                spliced("demo:c", "demo:slot", "one"),
            ),
        ],
    )]);
    let ui = stack.data_driven_ui();
    assert!(ui.compositions.is_empty());
    assert_eq!(
        found(ui.diagnostics()),
        [
            error("ddui/extension_point_overridden", POINT_AT, OVERRIDEN),
            failed_to_load()
        ]
    );
    assert!(labels(ui, "demo:slot").is_empty());
}

#[test]
fn e05_an_add_on_composition_at_an_undeclared_point_gets_both_messages_in_order() {
    let stack = stack(vec![external(
        1,
        "addon",
        vec![
            ("ddui/root/r.json", declaring("demo:r", "demo:slot")),
            (
                "ddui/composition/c.json",
                spliced("demo:c", "demo:none", "one"),
            ),
        ],
    )]);
    let ui = stack.data_driven_ui();
    assert!(ui.compositions.is_empty());
    assert_eq!(
        found(ui.diagnostics()),
        [
            error("ddui/extension_point_overridden", POINT_AT, OVERRIDEN),
            not_defined("demo:none"),
            failed_to_load()
        ]
    );
    assert_eq!(ui.compositions.rejected().len(), 1);
}

#[test]
fn e06_an_add_on_composition_without_an_extension_point_is_stored() {
    let stack = stack(vec![external(
        1,
        "addon",
        vec![(
            "ddui/composition/c.json",
            composition_json("demo:c", "", DIVIDER),
        )],
    )]);
    let ui = stack.data_driven_ui();
    assert_eq!(ui.diagnostics().count(), 0);
    let composition = ui.compositions.get(&composition_id("demo:c")).unwrap();
    assert_eq!(composition.attribs.extension_point, None);
    assert_eq!(
        composition.layout.markup[0].component,
        Component::FormDivider
    );
}

#[test]
fn e07_a_point_inside_a_composition_declares_nothing() {
    let stack = stack(vec![builtin(
        1,
        "vanilla",
        vec![
            ("ddui/root/r.json", root_json("minecraft:r", R_MARKUP)),
            (
                "ddui/composition/a.json",
                composition_json("minecraft:a", "", &point("demo:slot")),
            ),
            (
                "ddui/composition/b.json",
                spliced("minecraft:b", "demo:slot", "one"),
            ),
        ],
    )]);
    let ui = stack.data_driven_ui();
    assert_eq!(
        stored(ui.compositions.iter()),
        [pair("minecraft:a", "vanilla")]
    );
    assert_eq!(
        found(ui.diagnostics()),
        [not_defined("demo:slot"), failed_to_load()]
    );
}

/// A root with a nameless point is refused, and the load goes on: the
/// compositions are read and one spliced at the refused root's point is not defined
/// (vanilla: none is read, the collector throws out of the load).
#[test]
fn e08_a_nameless_point_refuses_its_root_and_the_compositions_are_still_read_deviation() {
    let stack = stack(vec![builtin(
        1,
        "vanilla",
        vec![
            (
                "ddui/root/r.json",
                root_with(
                    "minecraft:r",
                    &format!(
                        r#"{{ "component": "ExtensionPoint" }}, {}"#,
                        point("demo:slot")
                    ),
                ),
            ),
            (
                "ddui/composition/c.json",
                spliced("minecraft:c", "demo:slot", "one"),
            ),
        ],
    )]);
    let ui = stack.data_driven_ui();
    assert_eq!(ui.roots.len(), 0);
    assert!(labels(ui, "demo:slot").is_empty());
    let got = found(ui.diagnostics());
    assert_eq!(
        (got[0].0, got[0].1),
        (Severity::Error, "core/invalid_value"),
        "{got:#?}"
    );
    assert!(got.contains(&not_defined("demo:slot")), "{got:#?}");
}

#[test]
fn e09_the_contents_of_a_point_are_its_compositions_in_store_order() {
    let stack = stack(vec![builtin(
        1,
        "vanilla",
        vec![
            ("ddui/root/r.json", declaring("minecraft:r", "demo:slot")),
            (
                "ddui/composition/c1.json",
                spliced("minecraft:c1", "demo:slot", "one"),
            ),
            (
                "ddui/composition/c2.json",
                spliced("minecraft:c2", "demo:slot", "two"),
            ),
            (
                "ddui/composition/c3.json",
                composition_json("minecraft:c3", "", DIVIDER),
            ),
        ],
    )]);
    let ui = stack.data_driven_ui();
    assert_eq!(ui.diagnostics().count(), 0);
    assert_eq!(labels(ui, "demo:slot"), ["one", "two"]);
    let controls: Vec<&Control> = ui.extension_point_contents("demo:slot").collect();
    assert!(
        controls
            .iter()
            .all(|c| c.component == Component::FormButton)
    );
}

#[test]
fn e10_an_add_on_minecraft_composition_at_an_undeclared_point_gets_three_messages_in_order() {
    let stack = stack(vec![external(
        1,
        "addon",
        vec![(
            "ddui/composition/c.json",
            spliced("minecraft:c", "demo:none", "one"),
        )],
    )]);
    let ui = stack.data_driven_ui();
    assert!(ui.compositions.is_empty());
    assert_eq!(
        found(ui.diagnostics()),
        [
            error(
                "ddui/external_namespace",
                "minecraft:ui-composition/description/identifier",
                EXTERNAL
            ),
            error("ddui/extension_point_overridden", POINT_AT, OVERRIDEN),
            not_defined("demo:none"),
            failed_to_load()
        ]
    );
}

/// A composition whose parse failed never got as far as the validator: no lookup.
#[test]
fn a_composition_that_fails_the_parse_is_not_looked_up() {
    let text = composition_json(
        "minecraft:c",
        r#""extension_point": "demo:none""#,
        r#"{ "component": "Nope" }"#,
    );
    let stack = stack(vec![builtin(
        1,
        "vanilla",
        vec![("ddui/composition/c.json", text)],
    )]);
    let ui = stack.data_driven_ui();
    let got = found(ui.diagnostics());
    assert!(
        ui.diagnostics()
            .all(|f| f.code.kind() != Kind::UnresolvedReference),
        "{got:#?}"
    );
    assert_eq!(got.last(), Some(&failed_to_load()));
}

/// A root that was refused declares nothing.
#[test]
fn a_refused_root_s_points_are_not_declared() {
    let refused_root = root_with(
        "demo:r",
        &format!(r#"{}, {{ "component": "Nope" }}"#, point("demo:slot")),
    );
    let stack = stack(vec![builtin(
        1,
        "vanilla",
        vec![
            ("ddui/root/r.json", refused_root),
            (
                "ddui/composition/c.json",
                spliced("minecraft:c", "demo:slot", "one"),
            ),
        ],
    )]);
    let ui = stack.data_driven_ui();
    assert!(ui.roots.is_empty());
    assert!(ui.compositions.is_empty());
    assert!(found(ui.diagnostics()).contains(&not_defined("demo:slot")));
}

// ---- stack combination ---------------------------------------------------------------------

#[test]
fn k01_the_lowest_pack_s_definition_wins_deviation() {
    // vanilla = "silently replaced": the replaced definition is an info finding.
    let stack = stack(vec![
        external(
            1,
            "pack_1",
            vec![("ddui/root/s.json", root_with("demo:screen", DIVIDER))],
        ),
        external(2, "pack_0", vec![("ddui/root/other.json", r())]),
    ]);
    let ui = stack.data_driven_ui();
    assert_eq!(stored(ui.roots.iter()), [pair("demo:screen", "pack_0")]);
    let kept = ui.roots.get(&root_id("demo:screen")).unwrap();
    assert_eq!(
        kept.layout.markup[0].children()[0].component,
        Component::PanelText
    );
    assert_eq!(
        at(ui, "core/shadowed_definition"),
        [pair("pack_1", "ddui/root/s.json")]
    );
    assert!(ui.diagnostics().all(|f| f.severity == Severity::Info));
}

#[test]
fn k01_of_three_packs_the_lowest_wins_and_both_others_are_shadowed() {
    let screen = |child: &str| root_with("demo:screen", child);
    let stack = stack(vec![
        external(1, "top", vec![("ddui/root/s.json", screen(DIVIDER))]),
        external(
            2,
            "middle",
            vec![(
                "ddui/root/s.json",
                screen(r#"{ "component": "Panel.Spacing" }"#),
            )],
        ),
        external(3, "bottom", vec![("ddui/root/s.json", r())]),
    ]);
    let ui = stack.data_driven_ui();
    assert_eq!(stored(ui.roots.iter()), [pair("demo:screen", "bottom")]);
    assert_eq!(
        at(ui, "core/shadowed_definition"),
        [
            pair("top", "ddui/root/s.json"),
            pair("middle", "ddui/root/s.json")
        ]
    );
    // Compositions combine the same way.
    let composition = |label: &str| {
        composition_json(
            "demo:c",
            "",
            &format!(r#"{{ "component": "Form.Button", "attribs": {{ "label": "{label}" }} }}"#),
        )
    };
    let stack = self::stack(vec![
        external(
            1,
            "top",
            vec![("ddui/composition/c.json", composition("top"))],
        ),
        external(
            2,
            "middle",
            vec![("ddui/composition/c.json", composition("middle"))],
        ),
        external(
            3,
            "bottom",
            vec![("ddui/composition/c.json", composition("bottom"))],
        ),
    ]);
    let ui = stack.data_driven_ui();
    assert_eq!(stored(ui.compositions.iter()), [pair("demo:c", "bottom")]);
    assert_eq!(at(ui, "core/shadowed_definition").len(), 2);
}

#[test]
fn k01_within_one_pack_the_later_file_replaces_the_earlier_one() {
    let stack = stack(vec![external(
        1,
        "addon",
        vec![
            ("ddui/root/a.json", root_with("demo:screen", DIVIDER)),
            ("ddui/root/b.json", r()),
        ],
    )]);
    let ui = stack.data_driven_ui();
    let kept = ui.roots.get(&root_id("demo:screen")).unwrap();
    assert_eq!(kept.origin().path.as_str(), "ddui/root/b.json");
    assert_eq!(
        at(ui, "core/shadowed_definition"),
        [pair("addon", "ddui/root/a.json")]
    );
}

/// Identifiers are compared byte for byte.
#[test]
fn k01_identifiers_that_differ_in_case_are_two_definitions() {
    let stack = stack(vec![
        external(
            1,
            "top",
            vec![("ddui/root/s.json", root_json("demo:Screen", R_MARKUP))],
        ),
        external(2, "bottom", vec![("ddui/root/s.json", r())]),
    ]);
    let ui = stack.data_driven_ui();
    assert_eq!(
        stored(ui.roots.iter()),
        [pair("demo:Screen", "top"), pair("demo:screen", "bottom")]
    );
    assert_eq!(ui.diagnostics().count(), 0);
}

#[test]
fn k02_a_replacement_keeps_the_place_of_what_it_replaces() {
    let stack = stack(vec![
        external(
            1,
            "pack_1",
            vec![
                ("ddui/root/a.json", root_json("demo:a", R_MARKUP)),
                ("ddui/root/b.json", root_json("demo:b", R_MARKUP)),
            ],
        ),
        external(
            2,
            "pack_0",
            vec![("ddui/root/z.json", root_json("demo:a", R_MARKUP))],
        ),
    ]);
    let ui = stack.data_driven_ui();
    assert_eq!(
        stored(ui.roots.iter()),
        [pair("demo:a", "pack_0"), pair("demo:b", "pack_1")]
    );
}

#[test]
fn k03_a_lower_definition_that_fails_replaces_nothing() {
    let broken = root_with("demo:screen", r#"{ "component": "Nope" }"#);
    let stack = stack(vec![
        external(1, "pack_1", vec![("ddui/root/s.json", r())]),
        external(2, "pack_0", vec![("ddui/root/s.json", broken.clone())]),
    ]);
    let ui = stack.data_driven_ui();
    assert_eq!(stored(ui.roots.iter()), [pair("demo:screen", "pack_1")]);
    assert_eq!(ui.roots.rejected().len(), 1);
    assert!(at_kind(ui, Kind::Shadowed).is_empty());
    // Three packs: the lowest fails, the middle one is the lowest that loads.
    let stack = self::stack(vec![
        external(
            1,
            "top",
            vec![("ddui/root/s.json", root_with("demo:screen", DIVIDER))],
        ),
        external(2, "middle", vec![("ddui/root/s.json", r())]),
        external(3, "bottom", vec![("ddui/root/s.json", broken.clone())]),
    ]);
    let ui = stack.data_driven_ui();
    assert_eq!(stored(ui.roots.iter()), [pair("demo:screen", "middle")]);
    assert_eq!(
        at(ui, "core/shadowed_definition"),
        [pair("top", "ddui/root/s.json")]
    );
    // The failing one in the middle: the lowest still wins, over the top one.
    let stack = self::stack(vec![
        external(
            1,
            "top",
            vec![("ddui/root/s.json", root_with("demo:screen", DIVIDER))],
        ),
        external(2, "middle", vec![("ddui/root/s.json", broken)]),
        external(3, "bottom", vec![("ddui/root/s.json", r())]),
    ]);
    let ui = stack.data_driven_ui();
    assert_eq!(stored(ui.roots.iter()), [pair("demo:screen", "bottom")]);
}

/// A composition refused at the stack stage (its point is undeclared) is refused before it is
/// stored, as one refused by the parse is: the higher pack's definition stays, as the client
/// keeps it.
#[test]
fn k03_a_lower_composition_refused_at_the_stack_stage_replaces_nothing() {
    let root = ("ddui/root/r.json", declaring("minecraft:r", "demo:slot"));
    let good = |label: &str| spliced("minecraft:c", "demo:slot", label);
    let bad = spliced("minecraft:c", "demo:none", "bad");
    let stack = stack(vec![
        builtin(
            1,
            "pack_1",
            vec![root.clone(), ("ddui/composition/c.json", good("kept"))],
        ),
        builtin(2, "pack_0", vec![("ddui/composition/c.json", bad.clone())]),
    ]);
    let ui = stack.data_driven_ui();
    assert_eq!(
        stored(ui.compositions.iter()),
        [pair("minecraft:c", "pack_1")]
    );
    assert_eq!(labels(ui, "demo:slot"), ["kept"]);
    assert!(at_kind(ui, Kind::Shadowed).is_empty());
    let stack = self::stack(vec![
        builtin(
            1,
            "top",
            vec![root.clone(), ("ddui/composition/c.json", good("top"))],
        ),
        builtin(
            2,
            "middle",
            vec![("ddui/composition/c.json", good("middle"))],
        ),
        builtin(3, "bottom", vec![("ddui/composition/c.json", bad.clone())]),
    ]);
    let ui = stack.data_driven_ui();
    assert_eq!(
        stored(ui.compositions.iter()),
        [pair("minecraft:c", "middle")]
    );
    assert_eq!(labels(ui, "demo:slot"), ["middle"]);
    // Refused in the top pack: nothing of it is left, the lowest wins.
    let stack = self::stack(vec![
        builtin(1, "top", vec![root, ("ddui/composition/c.json", bad)]),
        builtin(
            2,
            "middle",
            vec![("ddui/composition/c.json", good("middle"))],
        ),
        builtin(
            3,
            "bottom",
            vec![("ddui/composition/c.json", good("bottom"))],
        ),
    ]);
    let ui = stack.data_driven_ui();
    assert_eq!(
        stored(ui.compositions.iter()),
        [pair("minecraft:c", "bottom")]
    );
    assert_eq!(labels(ui, "demo:slot"), ["bottom"]);
}

#[test]
fn k04_every_root_of_the_stack_is_read_before_the_first_composition() {
    let stack = stack(vec![
        builtin(
            1,
            "pack_1",
            vec![(
                "ddui/composition/c.json",
                spliced("minecraft:c", "demo:slot", "one"),
            )],
        ),
        builtin(
            2,
            "pack_0",
            vec![("ddui/root/r.json", declaring("minecraft:r", "demo:slot"))],
        ),
    ]);
    let ui = stack.data_driven_ui();
    assert_eq!(ui.diagnostics().count(), 0);
    assert_eq!(labels(ui, "demo:slot"), ["one"]);
}

/// The point a composition names is looked up in the roots that are stored: a root replaced
/// by a lower pack's stops declaring what only it declared.
#[test]
fn k04_a_replaced_root_s_points_are_gone() {
    let stack = stack(vec![
        builtin(
            1,
            "top",
            vec![
                ("ddui/root/r.json", declaring("minecraft:r", "demo:slot")),
                (
                    "ddui/composition/c.json",
                    spliced("minecraft:c", "demo:slot", "one"),
                ),
            ],
        ),
        builtin(
            2,
            "bottom",
            vec![("ddui/root/r.json", declaring("minecraft:r", "demo:other"))],
        ),
    ]);
    let ui = stack.data_driven_ui();
    assert!(ui.compositions.is_empty());
    assert!(found(ui.diagnostics()).contains(&not_defined("demo:slot")));
}

/// The findings of the group: the roots' first, then the compositions'.
#[test]
fn the_group_s_findings_are_the_roots_then_the_compositions() {
    let stack: PackStack<Resource> = stack(vec![external(
        1,
        "addon",
        vec![
            (
                "ddui/composition/c.json",
                composition_json("c", "", DIVIDER),
            ),
            ("ddui/root/r.json", root_json("r", R_MARKUP)),
        ],
    )]);
    let ui = stack.data_driven_ui();
    let files: Vec<String> = ui.diagnostics().map(|f| place(f).1).collect();
    assert_eq!(
        files,
        [
            "ddui/root/r.json",
            "ddui/root/r.json",
            "ddui/composition/c.json",
            "ddui/composition/c.json"
        ]
    );
}

/// The fragment `tests/determinism.rs` picks up: one document per root type, each loading
/// without a finding.
#[test]
fn the_determinism_fragment_loads_without_a_finding() {
    let stack = stack(vec![external(
        1,
        "addon",
        vec![
            (
                "ddui/root/determinism.json",
                fixture("determinism/resource_pack/ddui/root/determinism.json"),
            ),
            (
                "ddui/composition/determinism.json",
                fixture("determinism/resource_pack/ddui/composition/determinism.json"),
            ),
        ],
    )]);
    let ui = stack.data_driven_ui();
    assert_eq!(ui.diagnostics().count(), 0);
    let root = ui.roots.get(&root_id("determinism:screen")).unwrap();
    assert_eq!(root.extension_points(), ["determinism:slot"]);
    assert!(
        ui.compositions
            .contains(&composition_id("determinism:extra"))
    );
}

#[test]
fn the_default_path_of_a_minecraft_root_is_read_back_by_a_shipped_pack() {
    use bedrock_addon::document::{Content, Placed};
    use bedrock_addon::families::ddui::UiRoot;

    let built = UiRoot::new(
        root_id("minecraft:screen"),
        vec![Control::with_children(Component::Panel, Vec::new())],
    );
    let path: &'static str = Box::leak(built.default_path().as_str().to_owned().into_boxed_str());
    assert_eq!(path, "ddui/root/screen.json");
    let stack = stack(vec![builtin(
        1,
        "vanilla",
        vec![(path, built.to_json().unwrap())],
    )]);
    let ui = stack.data_driven_ui();
    assert_eq!(
        stored(ui.roots.iter()),
        [pair("minecraft:screen", "vanilla")]
    );
}
