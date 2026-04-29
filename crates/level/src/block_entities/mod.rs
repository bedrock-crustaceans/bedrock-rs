use std::io::{Cursor, Read};

macro_rules! as_string_slice {
    (
        $(#[$meta:meta])*
        pub enum $name:ident {
            $(
                $variant:ident ($vtype:ty)
            ),*
        }
    ) => {
        const VANILLA_VARIANTS: &[&str] = &[
            $(stringify!($variant)),*
        ];

        $(#[$meta])*
        pub enum $name {
            $(
                $variant ($vtype),
            )*
        }

        // impl<'a> TryFrom<&'a str> for $name {
        //     type Error = &'a str;

        //     fn try_from(value: &'a str) -> Result<Self, &'a str> {
        //         Ok(match value {
        //             $(stringify!($variant) => Self::$variant,)*
        //             _ => return Err(value)
        //         })
        //     }
        // }

        impl<'a> From<$name> for &'a str {
            fn from(id: $name) -> &'a str {
                match id {
                    $($name::$variant(_) => stringify!($variant),)*
                }
            }
        }
    }
}

as_string_slice! {
    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    #[serde(tag = "id")]
    pub enum BlockData {
        BlastFurnace(Furnace),
        CopperGolemStatue(CopperGolemStatue),
        Chest(Chest),
        Furnace(Furnace),
        SculkCatalyst(SculkCatalyst),
        SculkSensor(SculkSensor),
        Shelf(Shelf),
        ShulkerBox(ShulkerBox)
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct BlockEntity {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    #[serde(rename = "isMovable")]
    pub is_movable: bool,

    #[serde(flatten)]
    pub data: BlockData,
}

impl BlockEntity {
    pub fn from_disk<R>(reader: &mut Cursor<R>) -> crate::error::Result<Self>
    where
        Cursor<R>: Read,
    {
        let entity = nbtx::from_le_bytes(reader)?;
        Ok(entity)
    }
}

mod banner;
mod beacon;
mod bed;
mod bell;
mod cauldron;
mod chest;
mod command_block;
mod comparator;
mod conduit;
mod copper_golem;
mod enchantment_table;
mod end_gateway;
mod flower_pot;
mod furnace;
mod hopper;
mod item;
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
mod sculk_sensor;
mod shelf;
mod shulker_box;
mod sign;
mod skull;
mod structure_block;
mod trial_spawner;

pub use banner::*;
pub use beacon::*;
pub use bed::*;
pub use bell::*;
pub use cauldron::*;
pub use chest::*;
pub use command_block::*;
pub use comparator::*;
pub use conduit::*;
pub use copper_golem::*;
pub use enchantment_table::*;
pub use end_gateway::*;
pub use flower_pot::*;
pub use furnace::*;
pub use hopper::*;
pub use item::*;
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
pub use sculk_sensor::*;
pub use shelf::*;
pub use shulker_box::*;
pub use sign::*;
pub use skull::*;
pub use structure_block::*;
pub use trial_spawner::*;
