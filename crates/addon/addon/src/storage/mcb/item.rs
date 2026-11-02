//! `minecraft:item` bodies.
//!
//! Layout: the identifier (string), the menu category (string, `none` when the document has
//! no `menu_category`), two bytes `00 00`, a `u32` component count, then per component the
//! FNV-1a 32 hash of its name (little-endian; the components are in name order) and its
//! fields, which no tag or length separates: a component this reader does not know ends the
//! decoding.
//!
//! Vanilla bakes 23 items, so most components are seen with one set of values. A field is
//! read freely only where the documents show what it is; every other byte is read at the one
//! value it was seen with, and the text then has the members the JSON twin has:
//!
//! | component | fields |
//! | --- | --- |
//! | `block_placer` | `block` (string), `00 01 00` (`replace_block_item: true`) |
//! | `bundle_interaction` | `0c 00 00 00` (twin: `{}`) |
//! | `compostable` | `composting_chance` (one byte) |
//! | `cooldown` | `category` (string), `duration` (f32), the string `use` |
//! | `display_name` | `value` (string) |
//! | `hand_equipped` | `01` (`value: true`) |
//! | `icon` | `textures`: count × (key, texture) strings |
//! | `max_stack_size` | `value` (one byte), `00` (the 1.26.30 twins write `{}`; the game reads the value) |
//! | `projectile` | `00 00 00 00`, `projectile_entity` (string) |
//! | `storage_item` | `40 00 00 00 01`, `banned_items`: count × (`01`, the string `name`, the item name), `00` |
//! | `storage_weight_limit` | `40 00 00 00` (twin: `{}`) |
//! | `storage_weight_modifier` | `04 00 00 00` (twin: `{}`) |
//! | `tags` | `tags`: count × string |
//! | `throwable` | `01`, 0.0, 0.0, two equal floats, `00` (`do_swing_animation: true`, `launch_power_scale` and `max_launch_power` the float: 1.5 at 1.26.30, 1.0 at 1.26.50) |
//!
//! The game bakes five more items in the same layout: three at 1.26.50
//! (`red_shrub`, `shelf_mushroom` and the drop-4 `ice_ball`) and two at 1.26.40 (the drop-4
//! `ice_crystal` and `icicle`).
// Checked against the 23 items of the game's packs at 1.26.30, each equal to its JSON twin;
// `red_shrub` and `shelf_mushroom` at 1.26.50 equal their published JSON twins.

use super::McbValue;
use super::reader::{Decoded, Members, Reader, UNSUPPORTED, insert, named, object, string};

/// The components vanilla bakes.
const COMPONENTS: [&str; 14] = [
    "minecraft:block_placer",
    "minecraft:bundle_interaction",
    "minecraft:compostable",
    "minecraft:cooldown",
    "minecraft:display_name",
    "minecraft:hand_equipped",
    "minecraft:icon",
    "minecraft:max_stack_size",
    "minecraft:projectile",
    "minecraft:storage_item",
    "minecraft:storage_weight_limit",
    "minecraft:storage_weight_modifier",
    "minecraft:tags",
    "minecraft:throwable",
];

/// The payload of an item document.
pub(super) fn decode(bytes: &[u8]) -> Decoded<McbValue> {
    let reader = &mut Reader::new(bytes);
    let identifier = reader.string()?;
    let category = reader.string()?;
    // Two fields of the description no vanilla item sets (its group and whether it is hidden
    // in commands are the candidates; which is which is not shown).
    reader.pinned(&[0, 0])?;
    let mut description: Members = vec![("identifier".into(), string(identifier))];
    if &*category != "none" {
        description.push((
            "menu_category".into(),
            object(vec![("category".into(), string(category))]),
        ));
    }
    let count = reader.u32()?;
    let mut components = Members::new();
    for _ in 0..count {
        let (name, value) = component(reader)?;
        insert(&mut components, name.into(), value)?;
    }
    reader.end()?;
    Ok(object(vec![
        ("description".into(), object(description)),
        ("components".into(), object(components)),
    ]))
}

/// One component: its name by the hash, and its JSON.
fn component(reader: &mut Reader<'_>) -> Decoded<(&'static str, McbValue)> {
    let empty = || object(Vec::new());
    let name = named(reader.u32()?, &COMPONENTS).ok_or(UNSUPPORTED)?;
    let value = match name {
        "minecraft:block_placer" => {
            let block = reader.string()?;
            reader.pinned(&[0, 1, 0])?;
            object(vec![
                ("block".into(), string(block)),
                ("replace_block_item".into(), McbValue::Bool(true)),
            ])
        }
        "minecraft:bundle_interaction" => {
            reader.pinned(&[12, 0, 0, 0])?;
            empty()
        }
        "minecraft:compostable" => {
            let chance = reader.u8()?;
            // 30 and 65 are seen; a byte that could start a longer integer is not.
            if chance >= 0x80 {
                return Err(UNSUPPORTED);
            }
            object(vec![(
                "composting_chance".into(),
                McbValue::UInt(u64::from(chance)),
            )])
        }
        "minecraft:cooldown" => {
            let category = reader.string()?;
            let duration = reader.f32()?;
            reader.pinned(b"\x03use")?;
            object(vec![
                ("category".into(), string(category)),
                ("duration".into(), McbValue::F32(duration)),
            ])
        }
        "minecraft:display_name" => {
            let value = reader.string()?;
            object(vec![("value".into(), string(value))])
        }
        "minecraft:hand_equipped" => {
            reader.pinned(&[1])?;
            object(vec![("value".into(), McbValue::Bool(true))])
        }
        "minecraft:icon" => {
            let mut textures = Members::new();
            for _ in 0..reader.count()? {
                let key = reader.string()?;
                let texture = reader.string()?;
                insert(&mut textures, key, string(texture))?;
            }
            object(vec![("textures".into(), object(textures))])
        }
        "minecraft:max_stack_size" => max_stack_size(reader)?,
        "minecraft:projectile" => {
            reader.pinned(&[0, 0, 0, 0])?;
            let entity = reader.string()?;
            object(vec![("projectile_entity".into(), string(entity))])
        }
        "minecraft:storage_item" => {
            reader.pinned(&[64, 0, 0, 0, 1])?;
            let banned = reader.list(|reader| {
                reader.pinned(b"\x01\x04name")?;
                Ok(object(vec![("name".into(), string(reader.string()?))]))
            })?;
            // An empty list was never seen, so whether the twin would have the member is
            // not shown.
            if matches!(&banned, McbValue::Array(items) if items.is_empty()) {
                return Err(UNSUPPORTED);
            }
            reader.pinned(&[0])?;
            object(vec![("banned_items".into(), banned)])
        }
        "minecraft:storage_weight_limit" => {
            reader.pinned(&[64, 0, 0, 0])?;
            empty()
        }
        "minecraft:storage_weight_modifier" => {
            reader.pinned(&[4, 0, 0, 0])?;
            empty()
        }
        "minecraft:tags" => {
            let tags = reader.list(|reader| Ok(string(reader.string()?)))?;
            object(vec![("tags".into(), tags)])
        }
        "minecraft:throwable" => throwable(reader)?,
        _ => return Err(UNSUPPORTED),
    };
    Ok((name, value))
}

/// `max_stack_size`: its one member `value` (the game's default is 64), one byte, then `00`:
/// the bundles' `01` is 1, `ice_ball`'s `10` is 16.
fn max_stack_size(reader: &mut Reader<'_>) -> Decoded<McbValue> {
    let value = reader.u8()?;
    // 16 is seen; a byte that could start a longer integer is not
    if value >= 0x80 {
        return Err(UNSUPPORTED);
    }
    reader.pinned(&[0])?;
    Ok(object(vec![(
        "value".into(),
        McbValue::UInt(u64::from(value)),
    )]))
}

/// `throwable`: `01`, two zero floats, two equal floats, `00`.
fn throwable(reader: &mut Reader<'_>) -> Decoded<McbValue> {
    reader.pinned(&[1, 0, 0, 0, 0, 0, 0, 0, 0])?;
    // Two floats, equal in every bake (1.5 and 1.0): which is which is not shown.
    let power = reader.f32()?;
    #[allow(clippy::float_cmp, reason = "the same bytes, not a nearby number")]
    if !power.is_finite() || reader.f32()? != power {
        return Err(UNSUPPORTED);
    }
    reader.pinned(&[0])?;
    Ok(object(vec![
        ("do_swing_animation".into(), McbValue::Bool(true)),
        ("launch_power_scale".into(), McbValue::F32(power)),
        ("max_launch_power".into(), McbValue::F32(power)),
    ]))
}
