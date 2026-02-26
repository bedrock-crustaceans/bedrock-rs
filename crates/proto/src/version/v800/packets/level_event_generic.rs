use super::super::enums::LevelEvent;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 124)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct LevelEventGenericPacket {
    pub event_id: LevelEvent,
    #[nbt]
    pub event_data: nbtx::Value, // TODO: NBT Structure
}