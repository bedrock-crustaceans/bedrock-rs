use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 63)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerListPacket<V: ProtoVersion> {
    pub action: V::PlayerListPacketType,
}