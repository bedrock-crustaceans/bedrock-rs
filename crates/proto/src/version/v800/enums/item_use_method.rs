use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i32)]
#[enum_endianness(var)]
#[repr(i32)]
pub enum ItemUseMethod {
    Unknown = -1,
    EquipArmor = 0,
    Eat = 1,
    Attack = 2,
    Consume = 3,
    Throw = 4,
    Shoot = 5,
    Place = 6,
    FillBottle = 7,
    FillBucket = 8,
    PourBucket = 9,
    UseTool = 10,
    Interact = 11,
    Retrieved = 12,
    Dyed = 13,
    Traded = 14,
    BrushingCompleted = 15,
}