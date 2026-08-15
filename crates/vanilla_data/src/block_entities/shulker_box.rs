use crate::block_entities::Chest;
use bedrock_level::types::ItemStack;
use facet::Facet;

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
    /// as a `Byte` rather than an `Int`.
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
    pub items: Vec<ItemStack>,
    #[facet(rename = "facing")]
    pub facing: f32,
}

impl ShulkerBox {
    /// Splits this box into the container it is and the face it is on.
    pub fn into_parts(self) -> (Chest, f32) {
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
    pub fn from_parts(contents: Chest, facing: f32) -> Self {
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
