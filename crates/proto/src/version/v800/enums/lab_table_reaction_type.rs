use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum LabTableReactionType {
    None = 0,
    IceBomb = 1,
    Bleach = 2,
    ElephantToothpaste = 3,
    Fertilizer = 4,
    HeatBlock = 5,
    MagnesiumSalts = 6,
    MiscFire = 7,
    MiscExplosion = 8,
    MiscLava = 9,
    MiscMystical = 10,
    MiscSmoke = 11,
    MiscLargeSmoke = 12,
}