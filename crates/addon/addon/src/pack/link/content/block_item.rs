//! The items a block names: `minecraft:destructible_by_mining`'s `item_specific_speeds[].item`,
//! held by the game's linked-asset check.

use super::super::{Findings, LinkCx, item_descriptor};
use super::block::{placed, sections};
use super::item::descriptor_of;
use crate::families::block::component::DestructibleByMining;

/// Each `item_specific_speeds[].item` of each section is an item of the stack or of the game;
/// one that is none is an `UnresolvedReference` warning at the member with the validator's
/// line (the game logs it and keeps the block).
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    for (_, document) in cx.behavior().blocks() {
        for (section, components) in sections(document) {
            let Some(DestructibleByMining::Params(params)) =
                components.get::<DestructibleByMining>()
            else {
                continue;
            };
            for (index, speed) in params.item_specific_speeds.iter().enumerate() {
                let held = at![&section; "minecraft:destructible_by_mining", "item_specific_speeds", index, "item"];
                let path = placed(document, &held);
                item_descriptor(cx, out, &speed.item, descriptor_of(document.origin(), &path));
            }
        }
    }
}
