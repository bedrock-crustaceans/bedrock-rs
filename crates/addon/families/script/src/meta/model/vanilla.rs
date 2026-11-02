//! `metadata/vanilladata_modules/mojang-*.json`: the engine's registries at generation time
//! (blocks, items, entities, biomes, …), one generic envelope with one entry type per file
//!. The identifier lists generated from them are
//! [`bedrock_addon_core::vanilla::ids`].
//!
//! | File | Entry |
//! |---|---|
//! | `mojang-blocks.json` | [`BlockEntry`] (and `block_properties`) |
//! | `mojang-items.json` | [`ItemEntry`] |
//! | `mojang-biomes.json` | [`BiomeEntry`] |
//! | `mojang-enchantments.json` | [`EnchantmentEntry`] |
//! | the eight others | [`NameEntry`] |

use facet::Facet;

use super::ShapeError;
use crate::meta::MetaError;
use bedrock_addon_core::json::ScalarShape;

/// A vanilla data file: the common header and the entries.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct VanillaDataFile<T> {
    /// `module_type`: `vanilla_data`.
    pub module_type: String,
    /// `name`: `mojang-<type>`.
    pub name: String,
    /// `vanilla_data_type`: what the entries are (`block`, `item`, `entity`, `biome`, `effect`,
    /// `enchantment`, `dimension`, `cameraPresets`, `cooldownCategory`, `feature`,
    /// `potionEffect`, `potionDelivery`).
    pub vanilla_data_type: String,
    /// `data_items`: the entries, in the file's order.
    pub data_items: Vec<T>,
    /// `block_properties`: the block-state definitions; `mojang-blocks.json` only.
    pub block_properties: Option<Vec<BlockPropertyDef>>,
}

impl<T: for<'f> Facet<'f>> VanillaDataFile<T> {
    /// Reads the file strictly: a key no field takes is an error.
    ///
    /// # Errors
    ///
    /// Returns a [`MetaError`] when the bytes are not UTF-8, are not JSON of the expected
    /// shape, nest deeper than the JSON depth limit, or hold a member no field takes.
    pub fn parse(bytes: &[u8]) -> Result<Self, MetaError> {
        crate::meta::parse::strict(bytes).map(|(file, _)| file)
    }
}

/// An entry that is only a name: entities, effects, dimensions, camera presets, cooldown
/// categories, structure features, potion effects and potion delivery types.
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct NameEntry {
    /// `name`: `minecraft:…`; potion delivery types are bare (`Consume`).
    pub name: String,
}

/// One block (`mojang-blocks.json`).
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct BlockEntry {
    /// `name`: the identifier.
    pub name: String,
    /// `raw_id`: the numeric id.
    pub raw_id: i32,
    /// `serialization_id`: the legacy name (`tile.*`); not unique.
    pub serialization_id: String,
    /// `properties`: the block states it has, by name.
    pub properties: Vec<NameEntry>,
}

/// One block-state definition (`block_properties[]`).
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct BlockPropertyDef {
    /// `name`.
    pub name: String,
    /// `type`.
    #[facet(rename = "type")]
    pub ty: BlockPropertyType,
    /// `values`: the values it takes.
    pub values: Vec<BlockPropertyValue>,
}

/// `block_properties[].type`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet)]
#[repr(u8)]
pub enum BlockPropertyType {
    /// `bool`.
    #[facet(rename = "bool")]
    Bool,
    /// `int`.
    #[facet(rename = "int")]
    Int,
    /// `string`.
    #[facet(rename = "string")]
    String,
}

/// One entry of `block_properties[].values`.
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct BlockPropertyValue {
    /// `value`.
    pub value: BlockStateValue,
}

/// A block-state value: its JSON type follows the property's `type`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Facet)]
#[facet(proxy = ScalarShape)]
#[repr(u8)]
pub enum BlockStateValue {
    /// A JSON boolean.
    Bool(bool),
    /// A JSON integer.
    Int(i64),
    /// A JSON string.
    Str(String),
}

impl BlockStateValue {
    /// The property type this value's JSON type belongs to.
    #[must_use]
    pub fn property_type(&self) -> BlockPropertyType {
        match self {
            Self::Bool(_) => BlockPropertyType::Bool,
            Self::Int(_) => BlockPropertyType::Int,
            Self::Str(_) => BlockPropertyType::String,
        }
    }
}

impl TryFrom<ScalarShape> for BlockStateValue {
    type Error = ShapeError;

    fn try_from(json: ScalarShape) -> Result<Self, ShapeError> {
        match json {
            ScalarShape::Bool(flag) => Ok(Self::Bool(flag)),
            ScalarShape::Int(int) => Ok(Self::Int(int)),
            ScalarShape::UInt(uint) => i64::try_from(uint)
                .map(Self::Int)
                .map_err(|_| ShapeError::OutOfRange),
            ScalarShape::String(text) => Ok(Self::Str(text)),
            _ => Err(ShapeError::WrongType {
                expected: "a boolean, an integer or a string",
            }),
        }
    }
}

impl From<&BlockStateValue> for ScalarShape {
    fn from(value: &BlockStateValue) -> Self {
        match value {
            BlockStateValue::Bool(flag) => ScalarShape::Bool(*flag),
            BlockStateValue::Int(int) => match u64::try_from(*int) {
                Ok(uint) => ScalarShape::UInt(uint),
                Err(_) => ScalarShape::Int(*int),
            },
            BlockStateValue::Str(text) => ScalarShape::String(text.clone()),
        }
    }
}

/// One item (`mojang-items.json`).
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct ItemEntry {
    /// `name`: the identifier.
    pub name: String,
    /// `command_name`: the name commands take.
    pub command_name: String,
    /// `serialization_name`: the name saved in worlds.
    pub serialization_name: String,
    /// `serialization_id`: the legacy name (`item.*` or `tile.*`).
    pub serialization_id: String,
    /// `raw_id`: the numeric id; negative for block items.
    pub raw_id: i32,
}

/// One biome (`mojang-biomes.json`).
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct BiomeEntry {
    /// `name`: the identifier.
    pub name: String,
    /// `id`: the numeric id.
    pub id: i32,
}

/// One enchantment (`mojang-enchantments.json`).
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct EnchantmentEntry {
    /// `name`: the identifier.
    pub name: String,
    /// `value`: for an alias, the enchantment it names (`minecraft:bow_infinity` →
    /// `minecraft:infinity`).
    pub value: Option<String>,
}
