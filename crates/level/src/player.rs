use std::io::Read;

use facet::Facet;

use crate::error::Result;
use crate::settings::Abilities;

#[derive(Facet, Debug)]
// A player record carries keys this crate does not model yet, and an
// unrecognised key is an error unless the record opts out.
#[facet(nbtx::allow_unknown_fields)]
#[facet(rename_all = "PascalCase")]
pub struct InventoryItemWithSlot {
    pub damage: i16,
    pub slot: i8,
    #[facet(rename = "id")]
    pub id: i16,
    pub count: i8,
}

#[derive(Facet, Debug)]
// A player record carries keys this crate does not model yet, and an
// unrecognised key is an error unless the record opts out.
#[facet(nbtx::allow_unknown_fields)]
#[facet(rename_all = "PascalCase")]
pub struct InventoryItem {
    pub damage: i16,
    #[facet(rename = "id")]
    pub id: i16,
    pub count: i8,
}

#[derive(Facet, Debug)]
// A player record carries keys this crate does not model yet, and an
// unrecognised key is an error unless the record opts out.
#[facet(nbtx::allow_unknown_fields)]
#[facet(rename_all = "PascalCase")]
pub struct StatusEffect {
    pub id: i8,
    pub duration: i32,
    pub duration_easy: i32,
    pub duration_normal: i32,
    pub duration_hard: i32,
    pub ambient: bool,
    pub amplifier: i8,
    pub show_particles: bool,
}

#[derive(Facet, Debug)]
// A player record carries keys this crate does not model yet, and an
// unrecognised key is an error unless the record opts out.
#[facet(nbtx::allow_unknown_fields)]
#[facet(rename_all = "PascalCase")]
pub struct Attribute {
    pub current: f32,
    pub max: f32,
    pub name: String,
    pub base: f32,
    /// TODO: Figure out what this is supposed to be. It seems to be linked to `minecraft:attack_damage`
    ///
    /// Held as a raw [`nbtx::Value`] until that is known: `usize` has no NBT tag
    /// (its width varies by target), and guessing a concrete one would decide the
    /// question by accident. A `Value` accepts whatever is actually there.
    pub modifiers: Option<nbtx::Value>,
}

#[derive(Facet, Debug)]
// A player record carries keys this crate does not model yet, and an
// unrecognised key is an error unless the record opts out.
#[facet(nbtx::allow_unknown_fields)]
#[facet(rename_all = "PascalCase")]
pub struct PlayerData {
    pub inventory_version: String,
    pub is_swimming: bool,
    pub enchantment_seed: i32,
    pub fall_distance: f32,
    pub mark_variant: i32,
    #[facet(rename = "LeasherID")]
    pub leasher_id: i64,
    #[facet(rename = "DimensionID")]
    pub dimension_id: i32,
    pub sheared: bool,
    pub sleep_timer: i16,
    pub spawn_y: i32,
    pub chested: bool,
    pub spawn_forced: bool,
    pub is_global: bool,
    pub strength: i32,
    pub cursor_selected_items: InventoryItem,
    pub surface: bool,
    pub strength_max: i32,
    pub color2: i8,
    #[facet(rename = "boundY")]
    pub bound_y: i32,
    #[facet(rename = "boundZ")]
    pub bound_z: i32,
    #[facet(rename = "limitedLife")]
    pub limited_life: i32,
    pub armor: [InventoryItem; 4],
    pub hurt_time: i16,
    pub is_gliding: bool,
    pub player_game_mode: i32,
    pub color: i8,
    /// `[yaw, pitch]`. A plain array rather than [`bedrock_shared::vector::Rotation`]: that is a tuple
    /// struct, which facet reflects as a compound of `0`/`1` fields, while the
    /// tag on disk is a list of floats. Convert with `Rotation::from`.
    pub rotation: [f32; 2],
    pub invulnerable: bool,
    pub is_angry: bool,
    pub active_effects: Vec<StatusEffect>,
    // pub mainhand: i16,
    pub natural_spawn: bool,
    pub death_time: i16,
    pub is_baby: bool,
    pub variant: i32,
    pub spawn_z: i32,
    pub loot_dropped: bool,
    pub selected_inventory_slot: i32,
    pub on_ground: bool,
    pub sneaking: bool,
    pub sleeping: bool,
    pub attributes: Vec<Attribute>,
    pub bed_position_z: i32,
    pub abilities: Abilities,
    #[facet(rename = "UniqueID")]
    pub unique_id: i64,
    pub definitions: Vec<String>,
    pub saddled: bool,
    pub show_bottom: bool,
    #[facet(rename = "TargetID")]
    pub target_id: i64,
    pub bed_position_y: i32,
    pub bed_position_x: i32,
    #[facet(rename = "hasBoundOrigin")]
    pub has_bound_origin: bool,
    pub is_tamed: bool,
    /// `[x, y, z]`. See [`Self::rotation`] for why this is not a [`bedrock_shared::vector::Position`].
    pub pos: [f32; 3],
    pub map_index: i32,
    #[facet(rename = "boundX")]
    pub bound_x: i32,
    pub fire: i16,
    pub ender_chest_inventory: [InventoryItemWithSlot; 27],
    pub is_autonomous: bool,
    pub persistent: bool,
    pub offhand: Vec<InventoryItem>,
    #[facet(rename = "SelectedContainerID")]
    pub selected_contain_id: i32,
    pub owner_new: i64,
    pub player_level: i32,
    /// Thank you Mojang.
    pub is_pregnant: bool,
    #[facet(rename = "id")]
    pub id: i32,
    pub air: i16,
    pub attack_time: i16,
    pub portal_cooldown: i32,
    pub spawn_x: i32,
    pub sitting: bool,
    pub player_level_progress: f32,
}

impl PlayerData {
    /// Reads a player record from little-endian (on-disk) NBT.
    ///
    /// Goes through [`crate::nbt`] rather than nbtx directly: a player record is
    /// dense with `Byte` flags, and Bedrock counts every non-zero one as true.
    pub fn read<R: Read>(mut data: R) -> Result<Self> {
        Ok(crate::nbt::from_le_bytes(&mut data)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nbtx::{Compound, Value};

    /// A record the game wrote can carry a flag byte other than `0`/`1`; every
    /// non-zero one is true. Pins the read path, not just the fold itself: these
    /// records only get the rule if they are decoded through [`crate::nbt`].
    #[test]
    fn flag_bytes_other_than_one_are_true() {
        let effect = Value::Compound(Compound::from_iter([
            ("Id".into(), Value::Byte(1)),
            ("Duration".into(), Value::Int(100)),
            ("DurationEasy".into(), Value::Int(100)),
            ("DurationNormal".into(), Value::Int(100)),
            ("DurationHard".into(), Value::Int(100)),
            ("Ambient".into(), Value::Byte(2)),
            ("Amplifier".into(), Value::Byte(0)),
            ("ShowParticles".into(), Value::Byte(-1)),
        ]));
        let bytes = nbtx::to_le_bytes(&effect).unwrap();

        let decoded: StatusEffect = crate::nbt::from_le_bytes(&mut bytes.as_slice()).unwrap();
        assert!(decoded.ambient);
        assert!(decoded.show_particles);

        // The same bytes read straight through nbtx say the opposite, which is
        // what `PlayerData::read` exists to avoid.
        let strict: StatusEffect = nbtx::from_le_bytes(&mut bytes.as_slice()).unwrap();
        assert!(!strict.ambient);
        assert!(!strict.show_particles);
    }
}
