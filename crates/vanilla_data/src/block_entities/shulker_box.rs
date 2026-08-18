use crate::block_entities::Chest;
use bedrock_level::types::ItemStack;
use facet::Facet;

/// The face a shulker box is attached to, stored as a `Byte` tag holding a
/// numeric face index — confirmed by 102 real records spanning every value
/// `0..=5`. This is the standard Bedrock block-face index: `Down = 0,
/// Up = 1, North = 2, South = 3, West = 4, East = 5`. The facing lives only
/// here, never in block state — the generated block-palette data shows
/// `minecraft:*_shulker_box` carries no state properties at all.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Facet)]
#[facet(nbtx::variant_as(i8))]
#[repr(i8)]
pub enum Facing {
    Down = 0,
    Up = 1,
    North = 2,
    South = 3,
    West = 4,
    East = 5,
}

/// A shulker box: a [`Chest`]'s keys plus the face it is attached to, in one
/// flat compound.
///
/// The container's keys are inlined rather than nested under a `Chest` field,
/// because nbtx has no way to merge a field's keys into its parent's compound.
/// [`Self::contents`] and [`Self::from_parts`] convert between the two views.
#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(rename_all = "PascalCase")]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct ShulkerBox {
    /// See [`Chest::findable`]: a real record has been observed writing this
    /// as a `Byte` rather than an `Int`, and, since `lenient_width` is
    /// decode-only, a record read and written back re-encodes as the
    /// declared `Int` regardless of which tag it arrived under.
    #[facet(nbtx::lenient_width(i8))]
    pub findable: i32,
    #[facet(rename = "forceunpair")]
    pub force_unpair: Option<bool>,
    pub loot_table: Option<String>,
    pub loot_table_seed: Option<i32>,
    #[facet(rename = "pairlead")]
    pub pair_lead: Option<i8>,
    #[facet(rename = "pairx")]
    pub pair_x: Option<i32>,
    #[facet(rename = "pairz")]
    pub pair_z: Option<i32>,
    /// Absent entirely on 8 of the 102 real records in the fixture — an
    /// empty box, rather than an empty `Items` list written out. Defaulted
    /// rather than modeled as `Option` because the two are indistinguishable
    /// in meaning here: either way, the box holds nothing.
    #[facet(default)]
    pub items: Vec<ItemStack>,
    #[facet(rename = "facing")]
    pub facing: Facing,
}

impl ShulkerBox {
    /// Splits this box into the container it is and the face it is on.
    pub fn into_parts(self) -> (Chest, Facing) {
        (
            Chest {
                findable: self.findable,
                force_unpair: self.force_unpair,
                loot_table: self.loot_table,
                loot_table_seed: self.loot_table_seed,
                pair_lead: self.pair_lead,
                pair_x: self.pair_x,
                pair_z: self.pair_z,
                items: self.items,
            },
            self.facing,
        )
    }

    /// The container half of this box, cloned out of it.
    pub fn contents(&self) -> Chest {
        self.clone().into_parts().0
    }

    /// Builds a box from a container and the face it is on.
    pub fn from_parts(contents: Chest, facing: Facing) -> Self {
        Self {
            findable: contents.findable,
            force_unpair: contents.force_unpair,
            loot_table: contents.loot_table,
            loot_table_seed: contents.loot_table_seed,
            pair_lead: contents.pair_lead,
            pair_x: contents.pair_x,
            pair_z: contents.pair_z,
            items: contents.items,
            facing,
        }
    }
}
