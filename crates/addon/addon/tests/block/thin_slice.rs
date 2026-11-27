//! The thin slice: a folder behaviour pack with one block without
//! permutations, and its resource pack (manifest only), load through
//! `Pack::from_files` and `PackStack::of`; the block resolves and its network NBT has the expected shape.

use bedrock_addon::document::Resolution;
use bedrock_addon::id::BlockId;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{Behavior, PackStack, Resource};

use super::common::manifest;
use super::stacks::pack;

/// The block: base components only.
const RUBY: &str = r#"{
  "format_version": "1.26.20",
  "minecraft:block": {
    "description": {
      "identifier": "demo:ruby_block",
      "menu_category": { "category": "construction", "group": "minecraft:itemGroup.name.ore" }
    },
    "components": {
      "minecraft:destructible_by_mining": { "seconds_to_destroy": 3 },
      "minecraft:friction": 0.4,
      "minecraft:light_emission": 4
    }
  }
}"#;

/// The behaviour pack and its resource pack.
fn stacks() -> (PackStack<Behavior>, PackStack<Resource>) {
    let bp = pack(vec![
        ("manifest.json", manifest(1, "data", "ruby bp").into_bytes()),
        ("blocks/ruby.json", RUBY.as_bytes().to_vec()),
    ]);
    let rp = pack(vec![(
        "manifest.json",
        manifest(2, "resources", "ruby rp").into_bytes(),
    )]);
    let behavior =
        PackStack::<Behavior>::of([bp], LoadOptions::default()).expect("a behaviour pack");
    let resource =
        PackStack::<Resource>::of([rp], LoadOptions::default()).expect("a resource pack");
    (behavior, resource)
}

#[test]
fn the_folder_packs_load_and_the_block_is_defined() {
    let (bp, rp) = stacks();
    assert_eq!(rp.packs().len(), 1, "the resource pack is on its stack");
    let id = BlockId::from_static("demo:ruby_block");
    let blocks = bp.blocks();
    assert!(blocks.rejected().is_empty(), "{:#?}", blocks.rejected());
    let Resolution::Defined(block) = blocks.resolve(&id) else {
        panic!("demo:ruby_block is not defined: {blocks:#?}");
    };
    assert_eq!(block.origin().path.as_str(), "blocks/ruby.json");
    assert!(block.permutations.is_empty());
    assert!(
        blocks
            .diagnostics()
            .all(|d| d.severity < bedrock_addon::diagnostic::Severity::Warning),
        "{:#?}",
        blocks.diagnostics().collect::<Vec<_>>()
    );
}

#[cfg(feature = "nbt")]
#[test]
fn the_network_nbt_has_the_shape_of_11() {
    use nbtx::{Compound, Value, ValueList};

    let (bp, _) = stacks();
    let id = BlockId::from_static("demo:ruby_block");
    let block = bp.blocks().get(&id).expect("defined");
    let compound = |entries: Vec<(&str, Value)>| {
        let mut c = Compound::default();
        for (key, value) in entries {
            c.insert(key.into(), value);
        }
        Value::Compound(c)
    };
    let expected = compound(vec![
        (
            "components",
            compound(vec![
                (
                    "minecraft:destructible_by_mining",
                    compound(vec![("seconds_to_destroy", Value::Float(3.0))]),
                ),
                (
                    "minecraft:friction",
                    compound(vec![("value", Value::Float(0.4))]),
                ),
                (
                    "minecraft:light_emission",
                    compound(vec![("value", Value::Int(4))]),
                ),
            ]),
        ),
        (
            "menu_category",
            compound(vec![
                ("category", Value::String("construction".into())),
                (
                    "group",
                    Value::String("minecraft:itemGroup.name.ore".into()),
                ),
                ("is_hidden_in_commands", Value::Byte(0)),
            ]),
        ),
        ("permutations", Value::List(ValueList::Compound(Vec::new()))),
        ("properties", Value::List(ValueList::Compound(Vec::new()))),
        ("molangVersion", Value::Int(13)),
        (
            "vanillaBlockData",
            compound(vec![("block_id", Value::Int(0))]),
        ),
    ]);
    assert_eq!(block.to_network_nbt(), expected);
}
