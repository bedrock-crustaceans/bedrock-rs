use std::io::{Cursor, Read};

pub const VANILLA_NAMESPACE: &str = "minecraft";

macro_rules! as_string_slice {
    (
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        pub enum $name:ident {
            $($v:ident),*
        }
    ) => {
        const VANILLA_VARIANTS: &[&str] = &[
            $(stringify!($v)),*
        ];

        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        pub enum $name {
            $($v),*
        }

        impl<'a> TryFrom<&'a str> for VanillaBlockEntityId {
            type Error = &'a str;

            fn try_from(value: &'a str) -> Result<Self, &'a str> {
                Ok(match value {
                    $(stringify!($v) => Self::$v,)*
                    _ => return Err(value)
                })
            }
        }
    }
}

as_string_slice! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum VanillaBlockEntityId {
        SculkCatalyst
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CustomBlockEntityId {
    pub namespace: String,
    pub block: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BlockEntityId {
    Vanilla(VanillaBlockEntityId),
    Custom(CustomBlockEntityId),
}

mod block_entity_id {
    use super::{
        BlockEntityId, CustomBlockEntityId, VANILLA_NAMESPACE, VANILLA_VARIANTS,
        VanillaBlockEntityId,
    };
    use serde::{Deserialize, Deserializer, Serializer, de::Unexpected};

    /// Deserializes a block version.
    pub fn deserialize<'de, D>(de: D) -> Result<BlockEntityId, D::Error>
    where
        D: Deserializer<'de>,
    {
        let id = String::deserialize(de)?;
        let mut split = id.split(':');

        let Some(namespace) = split.next() else {
            return Err(serde::de::Error::invalid_value(
                Unexpected::Str(&id),
                &"a namespace ending in a colon",
            ));
        };

        let Some(block) = split.next() else {
            return Err(serde::de::Error::invalid_value(
                Unexpected::Unit,
                &"a block entity name",
            ));
        };

        // Verify that there are no more parts
        if split.next().is_some() {
            return Err(serde::de::Error::invalid_length(
                3,
                &"expected only two name components",
            ));
        }

        if namespace == VANILLA_NAMESPACE {
            let Ok(id) = VanillaBlockEntityId::try_from(block) else {
                return Err(serde::de::Error::unknown_variant(block, VANILLA_VARIANTS));
            };

            Ok(BlockEntityId::Vanilla(id))
        } else {
            Ok(BlockEntityId::Custom(CustomBlockEntityId {
                namespace: namespace.to_owned(),
                block: block.to_owned(),
            }))
        }
    }

    /// Serializes a block version.
    pub fn serialize<S>(v: &BlockEntityId, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct BlockEntity {
    #[serde(with = "block_entity_id")]
    pub id: BlockEntityId,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    #[serde(rename = "isMovable")]
    pub is_movable: bool,
}

impl BlockEntity {
    pub fn from_disk<R>(reader: &mut Cursor<R>) -> crate::error::Result<Self>
    where
        Cursor<R>: Read,
    {
        let entity = nbtx::from_le_bytes(reader)?;
        dbg!(&entity);
        Ok(entity)
    }
}

mod banner;
mod beacon;
mod bed;
mod bell;
mod cauldron;
mod command_block;
mod comparator;
mod conduit;
mod enchantment_table;
mod end_gateway;
mod flower_pot;
mod furnace;
mod hopper;
mod item_frame;
mod jigsaw;
mod jukebox;
mod lectern;
mod lodestone;
mod monster_spawner;
mod moving_block;
mod nether_reactor;
mod noteblock;
mod piston;
mod sculk_catalyst;
mod sign;
mod skull;
mod structure_block;
mod trial_spawner;

pub use banner::*;
pub use beacon::*;
pub use bed::*;
pub use bell::*;
pub use cauldron::*;
pub use command_block::*;
pub use comparator::*;
pub use conduit::*;
pub use enchantment_table::*;
pub use end_gateway::*;
pub use flower_pot::*;
pub use furnace::*;
pub use hopper::*;
pub use item_frame::*;
pub use jigsaw::*;
pub use jukebox::*;
pub use lectern::*;
pub use lodestone::*;
pub use monster_spawner::*;
pub use moving_block::*;
pub use nether_reactor::*;
pub use noteblock::*;
pub use piston::*;
pub use sculk_catalyst::*;
pub use sign::*;
pub use skull::*;
pub use structure_block::*;
pub use trial_spawner::*;
