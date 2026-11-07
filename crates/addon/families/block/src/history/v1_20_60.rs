//! Format 1.20.60: `description.properties` is gone; `unit_cube` is deprecated.

use indexmap::IndexMap;

use super::legacy::{BlockPayload, component_path};
use super::ops::{dropped_field, edit_object, is_opaque};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};

/// A block document of `[1.20.60, 1.21.30)`.
pub type Block = Legacy<BlockPayload, 1, 20, 60>;

/// `destructible_by_mining.item_specific_speeds` is removed;
/// `redstone_conductivity` is synthesised, as the game's legacy-compatibility load does.
impl LegacyStep<1, 20, 60> for BlockPayload {
    type Next = super::v1_21_30::Block;

    fn step(this: Block, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut block = this.0;
        let mut removed = Vec::new();
        for (section, map) in block.sections_mut() {
            let Some(mining) = map.get_mut("minecraft:destructible_by_mining") else {
                continue;
            };
            edit_object(mining, |m| {
                if m.shift_remove("item_specific_speeds").is_some() {
                    removed.push(section);
                }
            });
        }
        for section in removed {
            let mut at = component_path(section, "minecraft:destructible_by_mining");
            at.push("item_specific_speeds");
            dropped_field(&at, cx.diagnostics);
        }
        synthesize_redstone_conductivity(&mut block);
        Legacy(block)
    }
}

/// `v = !mi || (unit_cube && is_opaque(mi))`: the base gets a conductor when `v`; a
/// permutation (falling back to the base's `material_instances` and `unit_cube`) gets
/// `{redstone_conductor: v}` when its `v` differs from the base's.
pub(crate) fn synthesize_redstone_conductivity(block: &mut BlockPayload) {
    const NAME: &str = "minecraft:redstone_conductivity";
    let conducts =
        |mi: Option<&Value>, unit_cube: bool| mi.is_none() || (unit_cube && is_opaque(mi));
    let base_mi = block
        .components
        .get("minecraft:material_instances")
        .cloned();
    let base_cube = block.components.contains_key("minecraft:unit_cube");
    let base = conducts(base_mi.as_ref(), base_cube);
    let component = |conductor: bool| {
        let mut object = IndexMap::new();
        object.insert("redstone_conductor".to_owned(), Value::from(conductor));
        object.insert("allows_wire_to_step_down".to_owned(), Value::TRUE);
        super::to_object(&object)
    };
    if !block.components.contains_key(NAME) && base {
        block.components.insert(NAME.to_owned(), component(true));
        block.synthesized.push((None, NAME.to_owned()));
    }
    for (index, permutation) in block.permutations.iter_mut().enumerate() {
        let map = &mut permutation.components;
        if map.contains_key(NAME) {
            continue;
        }
        let mi = map.get("minecraft:material_instances").or(base_mi.as_ref());
        let cube = map.contains_key("minecraft:unit_cube") || base_cube;
        let v = conducts(mi, cube);
        if v != base {
            map.insert(NAME.to_owned(), component(v));
            block.synthesized.push((Some(index), NAME.to_owned()));
        }
    }
}
