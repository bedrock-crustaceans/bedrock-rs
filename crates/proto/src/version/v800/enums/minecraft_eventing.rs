use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i32)]
#[enum_endianness(var)]
#[repr(i32)]
pub enum InteractionType {
    Breeding = 1,
    Taming = 2,
    Curing = 3,
    Crafted = 4,
    Shearing = 5,
    Milking = 6,
    Trading = 7,
    Feeding = 8,
    Igniting = 9,
    Coloring = 10,
    Naming = 11,
    Leashing = 12,
    Unleashing = 13,
    PetSleep = 14,
    Trusting = 15,
    Commanding = 16,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i32)]
#[enum_endianness(var)]
#[repr(i32)]
pub enum POIBlockInteractionType {
    None = 0,
    Extend = 1,
    Clone = 2,
    Lock = 3,
    Create = 4,
    CreateLocator = 5,
    Rename = 6,
    ItemPlaced = 7,
    ItemRemoved = 8,
    Cooking = 9,
    Dousing = 10,
    Lighting = 11,
    Haystack = 12,
    Filled = 13,
    Emptied = 14,
    AddDye = 15,
    DyeItem = 16,
    ClearItem = 17,
    EnchantArrow = 18,
    CompostItemPlaced = 19,
    RecoveredBonemeal = 20,
    BookPlaced = 21,
    BookOpened = 22,
    Disenchant = 23,
    Repair = 24,
    DisenchantAndRepair = 25,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i32)]
#[enum_endianness(le)]
#[repr(i32)]
pub enum TeleportationCause {
    Unknown = 0,
    Projectile = 1,
    ChorusFruit = 2,
    Command = 3,
    Behavior = 4,
    TeleportationCauseCount = 5,
}