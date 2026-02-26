use super::super::types::BlockPos;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i32)]
#[enum_endianness(var)]
#[repr(i32)]
pub enum PlayerActionType {
    Unknown = -1,
    StartDestroyBlock {
        position: BlockPos,
        #[endianness(var)]
        facing: i32,
    } = 0,
    AbortDestroyBlock {
        position: BlockPos,
        #[endianness(var)]
        facing: i32,
    } = 1,
    StopDestroyBlock {
        position: BlockPos,
        #[endianness(var)]
        facing: i32,
    } = 2,
    GetUpdatedBlock = 3,
    DropItem = 4,
    StartSleeping = 5,
    StopSleeping = 6,
    Respawn = 7,
    StartJump = 8,
    StartSprinting = 9,
    StopSprinting = 10,
    StartSneaking = 11,
    StopSneaking = 12,
    CreativeDestroyBlock = 13,
    ChangeDimensionAck = 14,
    StartGliding = 15,
    StopGliding = 16,
    DenyDestroyBlock = 17,
    CrackBlock {
        position: BlockPos,
        #[endianness(var)]
        facing: i32,
    } = 18,
    ChangeSkin = 19,
    DeprecatedUpdatedEnchantingSeed = 20,
    StartSwimming = 21,
    StopSwimming = 22,
    StartSpinAttack = 23,
    StopSpinAttack = 24,
    InteractWithBlock = 25,
    PredictDestroyBlock {
        position: BlockPos,
        #[endianness(var)]
        facing: i32,
    } = 26,
    ContinueDestroyBlock {
        position: BlockPos,
        #[endianness(var)]
        facing: i32,
    } = 27,
    StartItemUseOn = 28,
    StopItemUseOn = 29,
    HandledTeleport = 30,
    MissedSwing = 31,
    StartCrawling = 32,
    StopCrawling = 33,
    StartFlying = 34,
    StopFlying = 35,
    ClientAckServerData = 36,
    Count = 37,
}
