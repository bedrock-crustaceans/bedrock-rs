//! What a stack load of JSON UI asks the host for: the two fixed files of every pack, then
//! the files the united lists name, and no other file below `ui/`.
// "Files under `ui/` that no pack lists are never
// read: the only enumeration is the `ui_defs` list".

use bedrock_addon::families::json_ui::ROOT_NAMESPACE;
use bedrock_addon::load::{
    Collection, CollectionSet, LoadOptions, RepositoryLoader, Residency, RootId, Source,
    StackLoader, StackOptions,
};
use bedrock_addon::pack::{OpenOptions, PackOrigin, StackSet, WorldPacks};

use super::responder::{Order, Responder, manifest};

/// Two resource packs: `top` (UUID 1) over `base` (UUID 2), each with its files below `ui/`.
fn world(top: &[(&str, &str)], base: &[(&str, &str)]) -> Responder {
    let mut responder = Responder::new(Order::Issue);
    for (n, folder, files) in [(1, "top", top), (2, "base", base)] {
        responder.file(
            1,
            &format!("{folder}/manifest.json"),
            manifest(n, "resources"),
        );
        for (path, text) in files {
            responder.file(1, &format!("{folder}/{path}"), text.as_bytes().to_vec());
        }
    }
    responder
}

/// The stacks of `responder`'s two packs after a stack load of the UI in `order`, and the
/// requests of that load.
fn load(responder: &Responder, order: Order, residency: Residency) -> (StackSet, Vec<String>) {
    let mut responder = responder.reordered(Order::Issue);
    let mut repository = RepositoryLoader::new(
        vec![Source::Directory {
            root: RootId(1),
            origin: PackOrigin::World,
        }],
        OpenOptions::default(),
    );
    let repository = responder.drive(&mut repository);
    let entries = format!(
        r#"[{{"pack_id": "{}", "version": [1, 0, 0]}}, {{"pack_id": "{}", "version": [1, 0, 0]}}]"#,
        uuid::Uuid::from_u128(1),
        uuid::Uuid::from_u128(2)
    );
    let world = WorldPacks::parse(None, Some(entries.as_bytes())).unwrap();
    let set = repository.stacks(&world, LoadOptions::default());
    assert_eq!(
        set.resource.packs().count(),
        2,
        "both packs are on the stack"
    );
    let mut responder = responder.reordered(order);
    let options = StackOptions {
        collections: CollectionSet::only([Collection::Ui]),
        resource: residency,
        ..StackOptions::default()
    };
    let mut loader = StackLoader::new(set, options);
    let set = responder.drive(&mut loader);
    let asked = responder
        .log
        .requests
        .iter()
        .map(|request| format!("{request:?}"))
        .collect();
    (set, asked)
}

const A_TOP: &str = r#"{ "a": { "type": "panel", "size": [1, 1] } }"#;
const A_BASE: &str = r#"{ "a": { "type": "panel" }, "below": { "type": "label" } }"#;

/// The top pack lists `ui/a.json` and `ui/only_base.json`; the base pack lists `ui/b.json`,
/// which no pack has. Both packs hold a file no list names.
fn listed_and_unlisted() -> Responder {
    world(
        &[
            (
                "ui/_ui_defs.json",
                r#"{ "ui_defs": ["ui/a.json", "ui/only_base.json"] }"#,
            ),
            ("ui/a.json", A_TOP),
            ("ui/unlisted.json", r#"{ "never": { "type": "panel" } }"#),
            ("ui/_global_variables.json", r#"{ "$top": 1 }"#),
        ],
        &[
            ("ui/_ui_defs.json", r#"{ "ui_defs": ["ui/b.json"] }"#),
            ("ui/a.json", A_BASE),
            ("ui/only_base.json", r#"{ "ob": { "type": "panel" } }"#),
            (
                "ui/deep/unlisted.json",
                r#"{ "never2": { "type": "panel" } }"#,
            ),
        ],
    )
}

#[test]
fn a_stack_load_asks_only_for_the_fixed_files_and_the_files_the_lists_name() {
    let responder = listed_and_unlisted();
    for order in Order::ALL {
        let (set, asked) = load(&responder, order, Residency::Transient);
        let count = |name: &str| asked.iter().filter(|r| r.contains(name)).count();
        // Never a file no list names, in either pack.
        assert_eq!(count("unlisted.json"), 0, "{order:?}: {asked:#?}");
        // The lists and the global variables of each pack that has them; each listed file of
        // each pack that has it (a list counts for the whole stack), once.
        assert_eq!(count("top/ui/_ui_defs.json"), 1, "{order:?}");
        assert_eq!(count("base/ui/_ui_defs.json"), 1, "{order:?}");
        assert_eq!(count("top/ui/_global_variables.json"), 1, "{order:?}");
        assert_eq!(count("top/ui/a.json"), 1, "{order:?}");
        assert_eq!(count("base/ui/a.json"), 1, "{order:?}");
        assert_eq!(count("base/ui/only_base.json"), 1, "{order:?}");
        assert_eq!(asked.len(), 6, "{order:?}: {asked:#?}");

        // What is loaded is what the lists name, merged from the lowest pack up.
        let ui = set.resource.ui();
        assert!(ui.get(ROOT_NAMESPACE, "a").is_some(), "{order:?}");
        assert!(ui.get(ROOT_NAMESPACE, "below").is_some(), "{order:?}");
        assert!(ui.get(ROOT_NAMESPACE, "ob").is_some(), "{order:?}");
        assert!(ui.get(ROOT_NAMESPACE, "never").is_none(), "{order:?}");
        assert!(ui.get(ROOT_NAMESPACE, "never2").is_none(), "{order:?}");
        let origin = ui.origin(&"ui/a.json".to_owned()).expect("the merged file");
        assert_eq!(origin.pack.index(), Some(0), "{order:?}");
        let layers: Vec<(Option<usize>, &str)> = ui
            .layers()
            .iter()
            .map(|layer| (layer.origin.pack.index(), layer.origin.path.as_str()))
            .collect();
        // The layers as the collection lists them (the top pack first): every copy read and
        // no other.
        assert_eq!(
            layers,
            [
                (Some(0), "ui/a.json"),
                (Some(0), "ui/_global_variables.json"),
                (Some(0), "ui/_ui_defs.json"),
                (Some(1), "ui/only_base.json"),
                (Some(1), "ui/a.json"),
                (Some(1), "ui/_ui_defs.json"),
            ],
            "{order:?}"
        );
        // The one finding: the listed file no pack has; nothing is reported as not loaded.
        let found: Vec<&str> = set
            .resource
            .report()
            .iter()
            .map(|f| f.code.id())
            // (The packs have no icon: the stack's own infos.)
            .filter(|name| *name != "core/pack_icon_missing")
            .collect();
        assert_eq!(found, ["json_ui/not_loaded"], "{order:?}");
    }
}

// The lazy accessor over the same packs (their bytes resident) gives the same collection as
// the stack load, in every completion order.
#[test]
fn the_stack_load_and_the_lazy_load_agree() {
    let responder = listed_and_unlisted();
    let (all, asked) = load(&responder, Order::Issue, Residency::All);
    // Under `All` every file is read once, the unlisted ones too; they are still not parsed.
    assert_eq!(
        asked.iter().filter(|r| r.contains("unlisted.json")).count(),
        2
    );
    let expected = format!("{:?}", all.resource.ui());
    assert!(!expected.contains("never"), "an unlisted file was loaded");
    for order in Order::ALL {
        let (set, _) = load(&responder, order, Residency::Transient);
        assert_eq!(format!("{:?}", set.resource.ui()), expected, "{order:?}");
        let (set, _) = load(&responder, order, Residency::All);
        assert_eq!(format!("{:?}", set.resource.ui()), expected, "{order:?}");
    }
}

// A stack whose packs list nothing reads the fixed files and stops.
#[test]
fn a_stack_without_a_listed_file_asks_for_the_fixed_files_only() {
    let responder = world(
        &[("ui/a.json", A_TOP), ("ui/b.json", A_BASE)],
        &[
            ("ui/_ui_defs.json", r#"{ "ui_defs": [] }"#),
            ("ui/a.json", A_BASE),
        ],
    );
    let (set, asked) = load(&responder, Order::Issue, Residency::Transient);
    assert_eq!(asked.len(), 1, "{asked:#?}");
    assert!(asked[0].contains("base/ui/_ui_defs.json"), "{asked:#?}");
    // L2 still stands: the stack has files below `ui/` and no list names one.
    let found: Vec<&str> = set
        .resource
        .report()
        .iter()
        .map(|f| f.code.id())
        // (The packs have no icon: the stack's own infos.)
        .filter(|name| *name != "core/pack_icon_missing")
        .collect();
    assert_eq!(found, ["core/no_ui_definitions"]);
}
