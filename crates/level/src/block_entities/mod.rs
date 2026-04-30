use std::io::{Cursor, Read};

macro_rules! to_name {
    ($variant:ident $(,)?) => {
        stringify!($variant)
    };

    ($variant:ident, $($name:literal)?) => {
        $($name)?
    };
}

macro_rules! pattern_match {
    ($name:ident $variant:ident $vtype:ty) => {
        $name::$variant(_)
    };

    ($name:ident $variant:ident) => {
        $name::$variant
    };
}

macro_rules! impl_block_data {
    (
        $(#[$meta:meta])*
        pub enum $name:ident {
            $(
                $variant:ident $( ($vtype:ty) )? $(renamed $id:literal)?
            ),* $(,)?
        }
    ) => {
        pub const VANILLA_IDENTIFIERS: &[&str] = &[
            $(to_name!($variant, $($id)?)),*
        ];

        $(#[$meta])*
        pub enum $name {
            $(
                $(#[serde(rename = $id)])?
                $variant $( ($vtype) )?,
            )*
        }

        impl<'a> From<$name> for &'a str {
            fn from(id: $name) -> &'a str {
                match id {
                    $(
                        pattern_match!($name $variant $($vtype)?) => to_name!($variant, $($id)?),
                    )*
                }
            }
        }
    };
}

impl_block_data! {
    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    #[serde(tag = "id")]
    pub enum BlockData {
        ItemFrame(ItemFrame),
        GlowItemFrame(ItemFrame),
        Beacon(Beacon),
        SoulCampfire(Campfire),
        CalibratedSculkSensor(SculkSensor),
        Beehive(Beehive),
        Campfire(Campfire),
        Bell(Bell),
        Lectern(Lectern),
        Banner(Banner),
        Cauldron(Cauldron),
        BrewingStand(BrewingStand),
        MobSpawner(MobSpawner),
        Bed(Bed),
        ChiseledShelf(ChiseledShelf) renamed "ChiseledBookshelf",
        Noteblock(Noteblock) renamed "Music",
        Comparator(Comparator),
        EndPortal,
        CommandBlock(CommandBlock),
        HangingSign(Sign),
        FlowerPot(FlowerPot),
        BlastFurnace(Furnace),
        CopperGolemStatue(CopperGolemStatue),
        Chest(Chest),
        Furnace(Furnace),
        SculkCatalyst(SculkCatalyst),
        SculkSensor(SculkSensor),
        Shelf(Shelf),
        ShulkerBox(ShulkerBox),
        Skull(Skull),
        Sign(Sign),
        NetherReactor(NetherReactor)
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
mod beehive;
mod bell;
mod brewing_stand;
mod campfire;
mod cauldron;
mod chest;
mod chiseled_shelf;
mod command_block;
mod comparator;
mod conduit;
mod copper_golem;
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
mod mob_spawner;
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
pub use beehive::*;
pub use bell::*;
pub use brewing_stand::*;
pub use campfire::*;
pub use cauldron::*;
pub use chest::*;
pub use chiseled_shelf::*;
pub use command_block::*;
pub use comparator::*;
pub use conduit::*;
pub use copper_golem::*;
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
pub use mob_spawner::*;
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
