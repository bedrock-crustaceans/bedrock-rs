use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 124)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct LevelEventGenericPacket<V: ProtoVersion> {
    pub event_id: V::LevelEvent,
    #[nbt]
    pub event_data: nbtx::Value, // TODO: NBT Structure
}