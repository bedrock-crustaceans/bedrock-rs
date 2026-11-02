//! `bp.items()`: the accessor walks `items/` of every pack bottom-up and the top pack's
//! definition wins.

use bedrock_addon::families::item::component::MaxStackSize;
use bedrock_addon::id::ItemId;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{Behavior, PackStack};

use super::pack_common::bp;

const LOW: &[u8] = br#"{"format_version": "1.26.30", "minecraft:item": {"description": {"identifier": "demo:ruby"}, "components": {"minecraft:max_stack_size": 8}}}"#;
const HIGH: &[u8] = br#"{"format_version": "1.26.30", "minecraft:item": {"description": {"identifier": "demo:ruby"}, "components": {"minecraft:max_stack_size": 16}}}"#;
const OTHER: &[u8] = br#"{"format_version": "1.10", "minecraft:item": {"description": {"identifier": "demo:old"}, "components": {"minecraft:max_stack_size": 4}}}"#;
const BROKEN: &[u8] = br#"{"format_version": "1.26.30", "minecraft:item": {"description": {}}}"#;

#[test]
fn the_top_pack_wins_and_every_items_file_is_read() {
    let top = bp(1).file("items/ruby.json", HIGH).open();
    let bottom = bp(2)
        .file("items/ruby.json", LOW)
        .file("items/sub/old.json", OTHER)
        .file("items/broken.json", BROKEN)
        .file("items/readme.txt", b"not an item")
        .open();
    let stack = PackStack::<Behavior>::of([top, bottom], LoadOptions::default()).unwrap();
    let items = stack.items();
    assert_eq!(items.len(), 2);
    let ruby = items.get(&ItemId::new("demo:ruby").unwrap()).unwrap();
    assert_eq!(ruby.components.get::<MaxStackSize>().unwrap().value, 16);
    assert!(items.contains(&ItemId::new("demo:old").unwrap()));
    assert_eq!(items.rejected().len(), 1);
    // the same reference on a later call
    assert!(std::ptr::eq(items, stack.items()));
}
