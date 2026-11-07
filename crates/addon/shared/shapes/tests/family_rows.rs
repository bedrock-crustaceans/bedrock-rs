//! The family fixture rows whose members are shared types, copied under
//! `tests/fixtures/<family>/<row>.json` with only the members that hold the shared type.
//! Each test pins what the shared type does for its row: the value read, the form written back
//! and the type's own findings. What the row asserts beyond that (Molang compile versions,
//! dropping a bad entry, family defaults and constraints) is the family's and is not tested
//! here.

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "tests read fixtures and call facet-json directly"
)]
mod common;

use bedrock_addon_core::diagnostic::{JsonPath, Kind, Severity};
use bedrock_addon_core::molang::SourceForm;
use bedrock_addon_shared_shapes::{
    BlockDescriptor, BlockSpecifier, CreativeCategory, ItemDescriptor, MenuCategory,
};
use common::{compact, normalised};
use facet::Facet;

/// Reads a fixture the way a family reads its file, and requires the written value to equal the
/// fixture.
fn read_fixture<T: for<'a> Facet<'a>>(text: &str) -> T {
    let value: T = facet_json::from_str_borrowed_jsonc(text).expect("the fixture reads");
    assert_eq!(compact(&value), normalised(text), "written form differs");
    value
}

/// The vanilla text of the single `InvalidValue` finding of `descriptor`, as it renders.
fn block_text(descriptor: &BlockDescriptor) -> String {
    let found = descriptor.check(&JsonPath::new(), Severity::Warning);
    assert_eq!(found.len(), 1);
    let code = &found.iter().next().unwrap().code;
    assert_eq!(code.kind(), Kind::InvalidValue, "{code:?}");
    code.vanilla().expect("a vanilla text")
}

/// The `tags` text of a `Tags` descriptor, which holds a plain string.
fn block_tags(descriptor: &BlockDescriptor) -> &str {
    match descriptor {
        BlockDescriptor::Tags { tags, .. } => {
            assert!(matches!(
                tags.form(),
                SourceForm::String {
                    context_version: None
                }
            ));
            tags.as_str()
        }
        other => panic!("not tags: {other:?}"),
    }
}

#[derive(Facet)]
struct SpawnsOnBlockFilter {
    #[facet(rename = "minecraft:spawns_on_block_filter")]
    on: Vec<BlockDescriptor>,
}

#[derive(Facet)]
struct SpawnsOnBlockTags {
    #[facet(rename = "minecraft:spawns_on_block_filter")]
    on: BlockDescriptor,
}

#[derive(Facet)]
struct DescriptorKept {
    with_a_bad_entry: SpawnsOnBlockFilter,
    empty: SpawnsOnBlockFilter,
}

/// sg28e / s09o: `[5, "minecraft:grass_block"]` keeps both entries, the first `Invalid` with
/// the C15 text; `[]` holds none and reports nothing.
fn descriptor_kept(text: &str) {
    let fixture: DescriptorKept = read_fixture(text);
    let on = &fixture.with_a_bad_entry.on;
    assert_eq!(on.len(), 2);
    assert_eq!(
        block_text(&on[0]),
        "BlockDescriptor requires a string, block object, or any_tag object. Cannot parse 5"
            .to_owned()
    );
    assert!(matches!(on[1], BlockDescriptor::Block { .. }));
    assert!(on[1].check(&JsonPath::new(), Severity::Warning).is_empty());
    assert!(fixture.empty.on.is_empty());
}

#[test]
fn sg14_block_filters() {
    #[derive(Facet)]
    struct Above {
        blocks: BlockDescriptor,
        distance: u32,
    }
    #[derive(Facet)]
    struct Row {
        #[facet(rename = "minecraft:spawns_on_block_filter")]
        on: Vec<BlockDescriptor>,
        #[facet(rename = "minecraft:spawns_on_block_prevented_filter")]
        prevented: Vec<BlockDescriptor>,
        #[facet(rename = "minecraft:spawns_above_block_filter")]
        above: Above,
    }
    let row: Row = read_fixture(include_str!(
        "fixtures/spawn_groups/sg14_block_filters.json"
    ));
    for list in [&row.on, &row.prevented] {
        assert!(
            matches!(list.as_slice(), [BlockDescriptor::Block { name, .. }] if name.as_str() == "test:block")
        );
    }
    assert!(matches!(
        &row.above.blocks,
        BlockDescriptor::Block { object: false, .. }
    ));
    assert_eq!(row.above.distance, 10);
}

#[test]
fn sg28e_block_descriptor_kept() {
    descriptor_kept(include_str!(
        "fixtures/spawn_groups/sg28e_block_descriptor_kept.json"
    ));
}

#[test]
fn sg50_tags_compile() {
    let row: SpawnsOnBlockTags =
        read_fixture(include_str!("fixtures/spawn_groups/sg50_tags_compile.json"));
    assert_eq!(block_tags(&row.on), "q.any_tag('grass')");
    assert!(row.on.check(&JsonPath::new(), Severity::Warning).is_empty());
}

#[test]
fn sg52_tags_shape_then_molang() {
    #[derive(Facet)]
    struct Above {
        blocks: Vec<BlockDescriptor>,
        distance: u32,
    }
    #[derive(Facet)]
    struct Row {
        #[facet(rename = "minecraft:spawns_above_block_filter")]
        above: Above,
    }
    let row: Row = read_fixture(include_str!(
        "fixtures/spawn_groups/sg52_tags_shape_then_molang.json"
    ));
    let blocks = &row.above.blocks;
    assert_eq!(
        block_text(&blocks[0]),
        "BlockDescriptor requires a string, block object, or any_tag object. Cannot parse 5"
            .to_owned()
    );
    // The shape is fine; the Molang error is the compile pass's.
    assert_eq!(block_tags(&blocks[1]), "1 +");
    assert!(
        blocks[1]
            .check(&JsonPath::new(), Severity::Warning)
            .is_empty()
    );
    assert_eq!(row.above.distance, 2);
}

#[test]
fn s04_molang_version_from_format() {
    // The descriptor holds the text without a version; the family applies the version of its
    // reader path.
    let row: SpawnsOnBlockTags = read_fixture(include_str!(
        "fixtures/spawn/s04_molang_version_from_format.json"
    ));
    assert_eq!(block_tags(&row.on), "q.any_tag('dirt')");
}

#[test]
fn s09_descriptor_errors() {
    #[derive(Facet)]
    struct Row {
        shape: BlockDescriptor,
        name: BlockDescriptor,
        state: BlockDescriptor,
    }
    let row: Row = read_fixture(include_str!("fixtures/spawn/s09_descriptor_errors.json"));
    assert_eq!(
        block_text(&row.shape),
        "BlockDescriptor requires a string, block object, or any_tag object. Cannot parse 5"
            .to_owned()
    );
    assert_eq!(
        block_text(&row.name),
        "Only strings are valid for name. The invalid name value: 7\n".to_owned()
    );
    assert_eq!(
        block_text(&row.state),
        "Invalid state type during block parsing: block name minecraft:wool, state name color, state type 3. State type can only be bool, int, or string.".to_owned()
    );
}

#[test]
fn s09o_block_descriptor_kept() {
    descriptor_kept(include_str!(
        "fixtures/spawn/s09o_block_descriptor_kept.json"
    ));
}

#[test]
fn m05_descriptor_tags() {
    #[derive(Facet)]
    struct Repair {
        items: Vec<ItemDescriptor>,
    }
    #[derive(Facet)]
    struct Speed {
        block: BlockDescriptor,
        speed: u32,
    }
    #[derive(Facet)]
    struct Row {
        repair_items: Vec<Repair>,
        destroy_speeds: Vec<Speed>,
    }
    let row: Row = read_fixture(include_str!("fixtures/item/m05_descriptor_tags.json"));
    let tags = |d: &ItemDescriptor| match d {
        ItemDescriptor::Tags { tags, .. } => tags.as_str().to_owned(),
        other => panic!("not tags: {other:?}"),
    };
    assert_eq!(
        tags(&row.repair_items[0].items[0]),
        "query.any_tag('planks')"
    );
    // Kept although the compile pass rejects it (outside the `{tags}` query set).
    assert_eq!(tags(&row.repair_items[1].items[0]), "query.is_baby");
    assert_eq!(
        block_tags(&row.destroy_speeds[0].block),
        "query.any_tag('stone')"
    );
    assert_eq!(row.destroy_speeds[0].speed, 5);
}

#[test]
fn v08_menu_category() {
    #[derive(Facet)]
    struct Row {
        empty: MenuCategory,
        group_without_namespace: MenuCategory,
    }
    let row: Row = read_fixture(include_str!("fixtures/item/v08_menu_category.json"));
    // `{}` reads with every member absent; the version's default category is item's (D7).
    assert_eq!(row.empty, MenuCategory::default());
    // The namespace rule that fails the item at 1.21.60 is item's; the type reads the group.
    let m = &row.group_without_namespace;
    assert_eq!(m.category, Some(CreativeCategory::Items));
    assert_eq!(m.group.as_deref(), Some("bundles"));
}

#[test]
fn m08_descriptor_tags() {
    #[derive(Facet)]
    struct Row {
        compiles: BlockDescriptor,
        outside_the_query_set: BlockDescriptor,
    }
    let row: Row = read_fixture(include_str!("fixtures/entity/m08_descriptor_tags.json"));
    assert_eq!(block_tags(&row.compiles), "query.any_tag('stone')");
    assert_eq!(block_tags(&row.outside_the_query_set), "query.is_baby");
}

#[test]
fn m06_bone_visibility_and_descriptors() {
    #[derive(Facet)]
    struct PlacementCondition {
        block_filter: Vec<BlockDescriptor>,
    }
    #[derive(Facet)]
    struct Row {
        conditions: Vec<PlacementCondition>,
    }
    let row: Row = read_fixture(include_str!(
        "fixtures/block/m06_bone_visibility_and_descriptors.json"
    ));
    assert_eq!(
        block_tags(&row.conditions[0].block_filter[0]),
        "query.any_tag('stone')"
    );
    assert_eq!(
        block_tags(&row.conditions[1].block_filter[0]),
        "q.block_state('x')"
    );
}

#[test]
fn jig12_predicates() {
    #[derive(Facet)]
    struct Predicate {
        predicate_type: String,
        block_state: BlockSpecifier,
    }
    #[derive(Facet)]
    struct Row {
        string: Predicate,
        object: Predicate,
    }
    let row: Row = read_fixture(include_str!("fixtures/jigsaw/jig12_predicates.json"));
    assert_eq!(row.string.predicate_type, "minecraft:blockstate_match");
    assert_eq!(row.string.block_state.name.as_str(), "minecraft:stone");
    assert!(!row.string.block_state.object);
    let object = &row.object.block_state;
    assert!(object.object);
    assert_eq!(object.name.as_str(), "minecraft:oak_log");
    assert_eq!(object.states.len(), 1);
}
