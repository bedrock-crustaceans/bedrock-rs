use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 105)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetDefaultGameTypePacket<V: ProtoVersion> {
    pub default_game_type: V::GameType,
}