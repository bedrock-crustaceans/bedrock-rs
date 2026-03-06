use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 105)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetDefaultGameTypePacket<V: ProtoVersion> {
    pub default_game_type: V::GameType,
}
