use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::v662::types::ActorRuntimeID;
use crate::v748::enums::MovementEffectType;

#[gamepacket(id = 318)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct MovementEffectPacket {
    pub entity_runtime_id: ActorRuntimeID,
    pub effect_type: MovementEffectType,
    #[endianness(var)]
    pub duration: i32,
    #[endianness(var)]
    pub tick: u64,
}