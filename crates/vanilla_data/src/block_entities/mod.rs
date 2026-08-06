use std::io::{Cursor, Read};

use bedrock_level::error::{Error, Result};
use bstr::BStr;
use nbtx::{Compound, Value};

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

/// The same pattern, binding the payload instead of discarding it.
///
/// `$binding` is passed in rather than invented here so that the pattern and the
/// expression that uses it come from the same expansion and refer to the same
/// variable.
macro_rules! bind_match {
    ($name:ident $variant:ident $vtype:ty, $binding:ident) => {
        $name::$variant($binding)
    };

    ($name:ident $variant:ident, $binding:ident) => {
        $name::$variant
    };
}

/// Decodes one variant's payload from the record's remaining keys.
///
/// A payload-carrying variant decodes its own type from what is left of the
/// compound; a payload-less one ignores it.
macro_rules! decode_variant {
    ($name:ident $variant:ident $vtype:ty, $rest:ident) => {
        $name::$variant(bedrock_level::nbt::from_value::<$vtype>($rest)?)
    };

    ($name:ident $variant:ident, $rest:ident) => {
        $name::$variant
    };
}

/// Encodes one variant's payload back into the keys it occupies.
macro_rules! encode_variant {
    ($name:ident $variant:ident $vtype:ty, $binding:ident) => {
        nbtx::to_value($binding)?
    };

    ($name:ident $variant:ident, $binding:ident) => {
        Value::Compound(Compound::new())
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

        impl $name {
            /// Decodes the payload for block-entity id `id` out of `rest`: the
            /// record's compound with the keys every record shares already
            /// removed.
            ///
            /// `Ok(None)` means no variant claims that id.
            pub fn from_value(id: &str, rest: Value) -> Result<Option<Self>> {
                Ok(Some(match id {
                    $(
                        to_name!($variant, $($id)?) =>
                            decode_variant!($name $variant $($vtype)?, rest),
                    )*
                    _ => return Ok(None),
                }))
            }

            /// The payload as the compound of keys it occupies in a record.
            ///
            /// A payload-less variant contributes no keys.
            pub fn to_value(&self) -> Result<Value> {
                Ok(match self {
                    $(
                        bind_match!($name $variant $($vtype)?, payload) =>
                            encode_variant!($name $variant $($vtype)?, payload),
                    )*
                })
            }
        }
    };
}

impl_block_data! {
    /// The payload of a block entity, selected by the record's `id` key.
    ///
    /// Not a `#[derive(Facet)]` enum: nbtx carries an enum as a unit variant
    /// only — its name as a string, or its discriminant as an integer — and so
    /// cannot express "every variant has its own payload type, chosen by a
    /// sibling key". The dispatch is generated alongside the enum instead, as
    /// [`Self::from_value`]/[`Self::to_value`].
    #[derive(Debug, Clone, PartialEq)]
    pub enum BlockData {
        StructureBlock(StructureBlock),
        JigsawBlock(JigsawBlock),
        BrushableBlock(BrushableBlock),
        Dropper(Dispenser),
        DecoratedPot(DecoratedPot),
        Barrel(Chest),
        TrialSpawner(TrialSpawner),
        Dispenser(Dispenser),
        Vault(Vault),
        PistonArm(PistonArm),
        Jukebox(Jukebox),
        Hopper(Hopper),
        EndGateway(EndGateway),
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

/// The keys every block-entity record carries, whatever its `id`.
const SHARED_KEYS: [&str; 5] = ["x", "y", "z", "isMovable", "id"];

/// One block-entity record: the position and movability every record carries,
/// plus the payload its `id` selects.
///
/// The payload's keys sit in the same compound as the shared ones rather than
/// nested under a key of their own, so the split is done by hand in
/// [`Self::from_value`].
#[derive(Debug, Clone, PartialEq)]
pub struct BlockEntity {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub is_movable: bool,

    pub data: BlockData,
}

/// Removes `key` from the record, or reports it as malformed.
fn take(entries: &mut Compound, key: &str) -> Result<Value> {
    entries
        .shift_remove(BStr::new(key))
        .ok_or(Error::Invalid("block entity record: missing key"))
}

impl BlockEntity {
    pub fn from_disk<R>(reader: &mut Cursor<R>) -> Result<Self>
    where
        Cursor<R>: Read,
    {
        let value: Value = nbtx::from_le_bytes(reader)?;
        Self::from_value(value)
    }

    /// Splits a decoded record into its shared keys and its payload.
    pub fn from_value(value: Value) -> Result<Self> {
        let Value::Compound(mut entries) = value else {
            return Err(Error::Invalid("block entity record: not a compound"));
        };

        let mut coordinate = |key| {
            take(&mut entries, key)?
                .into_int()
                .map_err(|_| Error::Invalid("block entity record: coordinate is not an int"))
        };
        let x = coordinate("x")?;
        let y = coordinate("y")?;
        let z = coordinate("z")?;

        // A `Byte` flag: zero is false, any other value true.
        let is_movable = take(&mut entries, "isMovable")?
            .into_byte()
            .map_err(|_| Error::Invalid("block entity record: isMovable is not a byte"))?
            != 0;

        let id = take(&mut entries, "id")?
            .into_string()
            .map_err(|_| Error::Invalid("block entity record: id is not a string"))?;
        let id = String::from_utf8(id.into())
            .map_err(|_| Error::Invalid("block entity record: id is not utf-8"))?;

        let data = BlockData::from_value(&id, Value::Compound(entries))?
            .ok_or(Error::Invalid("block entity record: unknown id"))?;

        Ok(Self {
            x,
            y,
            z,
            is_movable,
            data,
        })
    }

    /// Rebuilds the record's compound: the payload's keys plus the shared ones.
    pub fn to_value(&self) -> Result<Value> {
        let Value::Compound(mut entries) = self.data.to_value()? else {
            return Err(Error::Invalid("block entity payload: not a compound"));
        };

        let id: &str = self.data.clone().into();
        entries.insert(SHARED_KEYS[0].into(), Value::Int(self.x));
        entries.insert(SHARED_KEYS[1].into(), Value::Int(self.y));
        entries.insert(SHARED_KEYS[2].into(), Value::Int(self.z));
        entries.insert(SHARED_KEYS[3].into(), Value::Byte(i8::from(self.is_movable)));
        entries.insert(SHARED_KEYS[4].into(), Value::String(id.into()));

        Ok(Value::Compound(entries))
    }
}

mod banner;
mod beacon;
mod bed;
mod beehive;
mod bell;
mod brewing_stand;
mod brushable_block;
mod campfire;
mod cauldron;
mod chest;
mod chiseled_shelf;
mod command_block;
mod comparator;
mod conduit;
mod copper_golem;
mod decorated_pot;
mod dispenser;
mod enchantment_table;
mod end_gateway;
mod flower_pot;
mod furnace;
mod hopper;
mod item_frame;
mod jigsaw_block;
mod jukebox;
mod lectern;
mod lodestone;
mod mob_spawner;
mod moving_block;
mod nether_reactor;
mod noteblock;
mod piston_arm;
mod sculk_catalyst;
mod sculk_sensor;
mod shelf;
mod shulker_box;
mod sign;
mod skull;
mod structure_block;
mod trial_spawner;
mod vault;

pub use banner::*;
pub use beacon::*;
pub use bed::*;
pub use beehive::*;
pub use bell::*;
pub use brewing_stand::*;
pub use brushable_block::*;
pub use campfire::*;
pub use cauldron::*;
pub use chest::*;
pub use chiseled_shelf::*;
pub use command_block::*;
pub use comparator::*;
pub use conduit::*;
pub use copper_golem::*;
pub use decorated_pot::*;
pub use dispenser::*;
pub use enchantment_table::*;
pub use end_gateway::*;
pub use flower_pot::*;
pub use furnace::*;
pub use hopper::*;
pub use item_frame::*;
pub use jigsaw_block::*;
pub use jukebox::*;
pub use lectern::*;
pub use lodestone::*;
pub use mob_spawner::*;
pub use moving_block::*;
pub use nether_reactor::*;
pub use noteblock::*;
pub use piston_arm::*;
pub use sculk_catalyst::*;
pub use sculk_sensor::*;
pub use shelf::*;
pub use shulker_box::*;
pub use sign::*;
pub use skull::*;
pub use structure_block::*;
pub use trial_spawner::*;
pub use vault::*;
