use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 124)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct LevelEventGenericPacket<V: ProtoVersion> {
    pub event_id: V::LevelEvent,
    #[nbt]
    pub event_data: nbtx::Value, // TODO: NBT Structure
}
