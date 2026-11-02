//! The blocks an item names.

use bedrock_addon_core::diagnostic::{JsonPath, Severity, VanillaText};
use bedrock_addon_core::id::BlockId;
use bedrock_addon_core::shared::BlockDescriptor;

use super::super::{
    At, Findings, LinkCx, Target, block_descriptor, block_specifier, kinds, unresolved,
};
use super::item::{Entry, component, descriptor_of, items};
use crate::families::item::component::{BlockPlacer, Digger, EntityPlacer};

pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    for item in items(cx) {
        placed(cx, out, &item);
        descriptors(cx, out, &item);
    }
}

/// The block an item places or stands for is a block of the stack or of the game:
/// `minecraft:block_placer.block`, the legacy `minecraft:block`, and the `crop_result` of a
/// legacy `minecraft:seed`, whose miss carries the game's developer-log text.
fn placed(cx: &LinkCx<'_>, out: &mut Findings, item: &Entry<'_>) {
    if let Some(placer) = item.components.get::<BlockPlacer>() {
        let path = at![&component("minecraft:block_placer"); "block"];
        block_specifier(cx, out, &placer.block, descriptor_of(item.origin, &path));
    }
    // `minecraft:block` of a component-based document, or of one in the legacy format.
    let format_item = item.legacy.format_item.as_ref();
    let stands_for = item
        .legacy
        .block
        .as_ref()
        .or_else(|| format_item.and_then(|legacy| legacy.block.as_ref()));
    if let Some(block) = stands_for {
        named(cx, out, item, component("minecraft:block"), block, None);
    }
    let seed = format_item.and_then(|legacy| legacy.seed.as_ref());
    if let Some(crop) = seed.and_then(|seed| seed.crop_result.as_ref()) {
        // `SeedItemComponentLegacy`.
        let text = VanillaText::Formatted {
            template: "Item[{}] Seed Component : Failed to load crop_result[{}]. Are you using valid block names?",
            args: vec![item.id.as_str().to_owned(), crop.clone()],
        };
        let path = at![&component("minecraft:seed"); "crop_result"];
        named(cx, out, item, path, crop, Some(text));
    }
}

/// A block named by a plain string of `item` at `path`; a miss is an information, with `text`
/// when vanilla words it: the game's words for the legacy `minecraft:block` and for
/// `crop_result` go to the developer log alone, not the content log.
fn named(
    cx: &LinkCx<'_>,
    out: &mut Findings,
    item: &Entry<'_>,
    path: JsonPath,
    name: &str,
    text: Option<VanillaText>,
) {
    if name.is_empty() {
        return;
    }
    if let Target::Missing(hint) = cx.block(&BlockId::parse(name)) {
        let mut place = At::new(item.origin, path, Severity::Info);
        place.text = text;
        out.push(unresolved(place, kinds::BLOCK, name, hint));
    }
}

/// The block descriptors of `minecraft:block_placer.use_on`,
/// `minecraft:digger.destroy_speeds[].block` and `minecraft:entity_placer.use_on` /
/// `dispense_on`.
fn descriptors(cx: &LinkCx<'_>, out: &mut Findings, item: &Entry<'_>) {
    let mut each = |base: JsonPath, list: &[BlockDescriptor]| {
        for (index, descriptor) in list.iter().enumerate() {
            let path = at![&base; index];
            block_descriptor(cx, out, descriptor, descriptor_of(item.origin, &path));
        }
    };
    if let Some(placer) = item.components.get::<BlockPlacer>() {
        each(at![&component("minecraft:block_placer"); "use_on"], &placer.use_on);
    }
    if let Some(digger) = item.components.get::<Digger>() {
        for (index, speed) in digger.destroy_speeds.iter().enumerate() {
            let path = at![&component("minecraft:digger"); "destroy_speeds", index, "block"];
            block_descriptor(cx, out, &speed.block, descriptor_of(item.origin, &path));
        }
    }
    if let Some(placer) = item.components.get::<EntityPlacer>() {
        let base = component("minecraft:entity_placer");
        let mut each = |key: &str, list: &[BlockDescriptor]| {
            for (index, descriptor) in list.iter().enumerate() {
                let path = at![&base; key, index];
                block_descriptor(cx, out, descriptor, descriptor_of(item.origin, &path));
            }
        };
        each("use_on", &placer.use_on);
        each("dispense_on", &placer.dispense_on);
    }
}
