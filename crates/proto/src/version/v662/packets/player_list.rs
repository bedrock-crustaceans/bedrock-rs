use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 63)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerListPacket<V: ProtoVersion> {
    pub action: V::PlayerListPacketType,
}
