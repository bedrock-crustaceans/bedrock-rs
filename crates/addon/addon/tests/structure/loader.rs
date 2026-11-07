//! The structure walk leaves the Java `.nbt` templates of a base-game pack out: a stack load never asks for their bytes. An add-on's `.nbt` file is
//! still read, for its `IgnoredFile` info.

use bedrock_addon::diagnostic::{Severity, codes};
use bedrock_addon::load::{
    Collection, CollectionSet, LoadOptions, RepositoryLoader, RootId, Source, StackLoader,
    StackOptions,
};
use bedrock_addon::pack::{OpenOptions, PackOrigin, StackSet, WorldPacks};

use super::responder::{Order, Responder, manifest};

/// A base-game pack (root 1) and an add-on (root 2), each with one `.nbt` template and one
/// structure file that is no structure.
fn world() -> Responder {
    let mut responder = Responder::new(Order::Issue);
    responder
        .file(1, "vanilla/manifest.json", manifest(1, "data"))
        .file(
            1,
            "vanilla/structures/village/house.nbt",
            &b"\x1f\x8b template"[..],
        )
        .file(1, "vanilla/structures/broken.mcstructure", &b"no nbt"[..])
        .file(2, "addon/manifest.json", manifest(2, "data"))
        .file(
            2,
            "addon/structures/ruins/tower.nbt",
            &b"\x1f\x8b template"[..],
        )
        .file(2, "addon/structures/other.mcstructure", &b"no nbt"[..]);
    responder
}

/// The stacks after a stack load of the structures, and what the responder was asked.
fn load(order: Order) -> (StackSet, Vec<String>) {
    let mut responder = world();
    let mut repository = RepositoryLoader::new(
        vec![
            Source::Directory {
                root: RootId(1),
                origin: PackOrigin::Package,
            },
            Source::Directory {
                root: RootId(2),
                origin: PackOrigin::World,
            },
        ],
        OpenOptions::default(),
    );
    let repository = responder.drive(&mut repository);
    let entries = format!(
        r#"[{{"pack_id": "{}", "version": [1, 0, 0]}}, {{"pack_id": "{}", "version": [1, 0, 0]}}]"#,
        uuid::Uuid::from_u128(2),
        uuid::Uuid::from_u128(1)
    );
    let world = WorldPacks::parse(Some(entries.as_bytes()), None).unwrap();
    let set = repository.stacks(&world, LoadOptions::default());
    assert_eq!(
        set.behavior.packs().count(),
        2,
        "both packs are on the stack"
    );
    let mut responder = responder.reordered(order);
    let before = responder.log.requests.len();
    let options = StackOptions {
        collections: CollectionSet::only([Collection::Structures]),
        ..StackOptions::default()
    };
    let mut loader = StackLoader::new(set, options);
    let set = responder.drive(&mut loader);
    let asked = responder.log.requests[before..]
        .iter()
        .map(|request| format!("{request:?}"))
        .collect();
    (set, asked)
}

#[test]
fn a_base_game_pack_s_nbt_templates_are_never_asked_for() {
    for order in Order::ALL {
        let (set, asked) = load(order);
        let asked_for = |name: &str| asked.iter().filter(|r| r.contains(name)).count();
        assert_eq!(asked_for("house.nbt"), 0, "{order:?}: {asked:#?}");
        // Everything else of the walk is read: the add-on's template (for its info) and both
        // structure files.
        assert_eq!(asked_for("tower.nbt"), 1, "{order:?}: {asked:#?}");
        assert_eq!(asked_for("broken.mcstructure"), 1, "{order:?}");
        assert_eq!(asked_for("other.mcstructure"), 1, "{order:?}");
        // What the collection reports is what it reported when it read the templates.
        let structures = set.behavior.structures();
        let ignored: Vec<&str> = structures
            .diagnostics()
            .filter(|f| f.code.id() == "core/ignored_file")
            .inspect(|f| assert_eq!(f.severity, Severity::Info))
            .filter_map(|f| {
                f.location
                    .path
                    .as_ref()
                    .map(bedrock_addon::storage::PackPath::as_str)
            })
            .collect();
        assert_eq!(ignored, ["structures/ruins/tower.nbt"], "{order:?}");
        // Nothing of the stack is reported as not loaded: the template is no file of the walk.
        assert!(
            !set.behavior
                .report()
                .iter()
                .any(|f| f.is(&codes::FILE_NOT_LOADED)),
            "{order:?}"
        );
    }
}
