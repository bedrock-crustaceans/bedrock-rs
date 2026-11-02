//! `bp.crafting_catalog()` through a `StackLoader`: the catalogue's
//! `combine` asks the stack which items and blocks its packs registered, so the loader combines
//! and installs those two collections first, in every completion order, also when only the
//! catalogue was selected.

use bedrock_addon::diagnostic::{DiagnosticCode, DiagnosticDetail};
use bedrock_addon::load::{
    Collection, CollectionSet, LoadOptions, RepositoryLoader, RootId, Source, StackLoader,
    StackOptions,
};
use bedrock_addon::pack::{OpenOptions, PackOrigin, StackSet, WorldPacks};

use super::common::item;
use super::responder::{Order, Responder, manifest};

const CATALOG: &str = r#"{"format_version": "1.21.60", "minecraft:crafting_items_catalog": {"categories": [{"category_name": "construction", "groups": [{"group_identifier": {"name": "custom:g"}, "items": ["custom:thing", "custom:ore", "custom:nothing", "custom:broken"]}]}]}}"#;
const ITEM: &str = r#"{"format_version": "1.21.60", "minecraft:item": {"description": {"identifier": "custom:thing"}, "components": {"minecraft:max_stack_size": 64}}}"#;
const BLOCK: &str = r#"{"format_version": "1.21.60", "minecraft:block": {"description": {"identifier": "custom:ore"}, "components": {}}}"#;
/// A block file the block family rejects.
const BROKEN: &str = r#"{"format_version": "1.21.60", "minecraft:block": {"description": {"identifier": "custom:broken"}, "components": 5}}"#;

/// One behaviour pack on disk (nothing resident): a catalogue, an item and two block files.
fn world() -> Responder {
    let mut responder = Responder::new(Order::Issue);
    responder
        .file(1, "bp/manifest.json", manifest(1, "data"))
        .file(1, "bp/item_catalog/crafting_item_catalog.json", CATALOG)
        .file(1, "bp/items/thing.json", ITEM)
        .file(1, "bp/blocks/ore.json", BLOCK)
        .file(1, "bp/blocks/broken.json", BROKEN);
    responder
}

/// The stacks after a stack load of `collections`, completed in `order`.
fn load(order: Order, collections: CollectionSet) -> StackSet {
    let mut responder = world();
    let mut repository = RepositoryLoader::new(
        vec![Source::Directory {
            root: RootId(1),
            origin: PackOrigin::World,
        }],
        OpenOptions::default(),
    );
    let repository = responder.drive(&mut repository);
    let entry = format!(
        r#"[{{"pack_id": "{}", "version": [1, 0, 0]}}]"#,
        uuid::Uuid::from_u128(1)
    );
    let world = WorldPacks::parse(Some(entry.as_bytes()), None).unwrap();
    let set = repository.stacks(&world, LoadOptions::default());
    let mut responder = responder.reordered(order);
    let options = StackOptions {
        collections,
        ..StackOptions::default()
    };
    let mut loader = StackLoader::new(set, options);
    responder.drive(&mut loader)
}

/// The names the merge could not resolve.
fn unresolved(set: &StackSet) -> Vec<String> {
    set.behavior
        .crafting_catalog()
        .diagnostics()
        .filter_map(|f| match f.detail.as_deref() {
            Some(DiagnosticDetail::Reference {
                kind: "item", id, ..
            }) => Some(id.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn a_stack_load_of_the_catalogue_alone_resolves_against_the_stack_s_items_and_blocks() {
    for order in Order::ALL {
        let set = load(order, CollectionSet::only([Collection::CraftingCatalog]));
        let catalog = set.behavior.crafting_catalog();
        let mut placed = vec![item("custom:thing")];
        let mut dropped = vec!["custom:nothing".to_owned(), "custom:broken".to_owned()];
        if cfg!(feature = "block") {
            placed.push(item("custom:ore"));
        } else {
            // With `item` alone every name that may be a pack's block is kept.
            placed = [
                "custom:thing",
                "custom:ore",
                "custom:nothing",
                "custom:broken",
            ]
            .map(item)
            .to_vec();
            dropped.clear();
        }
        assert_eq!(catalog.categories[0].groups[0].items, placed, "{order:?}");
        assert_eq!(unresolved(&set), dropped, "{order:?}");
        #[cfg(feature = "item")]
        assert_eq!(set.behavior.items().len(), 1, "{order:?}");
        #[cfg(feature = "block")]
        assert_eq!(set.behavior.blocks().len(), 1, "{order:?}");
        // The files were on disk only: had the loader not read and installed the collections
        // the catalogue needs, the lazy accessors would report each file as not loaded.
        assert!(
            !set.behavior
                .report()
                .iter()
                .any(|f| matches!(f.code, DiagnosticCode::FileNotLoaded)),
            "{order:?}"
        );
    }
}

#[test]
fn every_completion_order_of_a_full_load_gives_the_same_catalogue() {
    let reference = format!(
        "{:?}",
        load(Order::Issue, CollectionSet::ALL)
            .behavior
            .crafting_catalog()
    );
    assert!(reference.contains("custom:thing"));
    for order in Order::ALL {
        let set = load(order, CollectionSet::ALL);
        assert_eq!(
            format!("{:?}", set.behavior.crafting_catalog()),
            reference,
            "{order:?}"
        );
    }
}
