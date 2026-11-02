//! A block built in code, added to a pack through `PackWriter::add`,
//! and the pack loaded back through a `PackStack`.

use bedrock_addon::document::Resolution;
use bedrock_addon::families::block::component::{DestructibleByMining, LightEmission};
use bedrock_addon::families::block::{Block, Permutation, StateValues};
use bedrock_addon::id::BlockId;
use bedrock_addon::json::StrictInt;
use bedrock_addon::load::{LoadContext, LoadOptions};
use bedrock_addon::manifest::Manifest;
use bedrock_addon::pack::{Behavior, PackOrigin, PackStack, PackType, PackWriter};
use bedrock_addon::shared::MenuCategory;
use bedrock_addon::version::MinEngineVersion;

use super::paths;

fn ruby_lamp() -> Block {
    let mut ruby = Block::new(BlockId::from_static("demo:ruby_lamp"));
    ruby.description.menu_category = Some(MenuCategory::construction());
    ruby.description
        .states
        .insert("demo:lit".to_owned(), StateValues::bools());
    ruby.components.insert(DestructibleByMining::seconds(1.5));
    // The example's `geometry` component is left out: alone it is an error ("Block needs both
    // a "geometry" and "material_instances" component."), which the example's own
    // `assert!(!findings.has_errors())` would trip over.
    ruby.permutations
        .push(Permutation::when("q.block_state('demo:lit')").with(LightEmission(StrictInt(15))));
    ruby
}

#[test]
fn a_block_added_to_a_new_pack_loads_back_through_a_stack() {
    let ruby = ruby_lamp();
    let findings = ruby.validate(&LoadContext::detached());
    assert!(!findings.has_errors(), "{findings:?}");

    let mut manifest = Manifest::new("Ruby lamps", PackType::Behavior);
    manifest.header.min_engine_version = Some(MinEngineVersion::new(1, 26, 20));
    let id = manifest.identity();
    let mut writer = PackWriter::new(manifest);
    let at = writer.add(&ruby).unwrap();
    assert_eq!(at.as_str(), "blocks/demo/ruby_lamp.json");
    assert_eq!(
        paths(&writer),
        ["manifest.json", "blocks/demo/ruby_lamp.json"]
    );

    let pack = writer.into_pack(PackOrigin::World).unwrap();
    assert_eq!(*pack.id(), id);
    assert!(!pack.report().has_errors(), "{:?}", pack.report());
    let stack = PackStack::<Behavior>::of([pack], LoadOptions::default()).unwrap();
    let blocks = stack.blocks();
    assert!(blocks.rejected().is_empty(), "{:#?}", blocks.rejected());
    let Resolution::Defined(block) = blocks.resolve(&BlockId::from_static("demo:ruby_lamp")) else {
        panic!("demo:ruby_lamp is not defined: {blocks:#?}");
    };
    assert_eq!(block.origin().path, at);
    assert_eq!(block.value(), &ruby, "the block that was added");
    assert_eq!(blocks.len(), 1);
}
