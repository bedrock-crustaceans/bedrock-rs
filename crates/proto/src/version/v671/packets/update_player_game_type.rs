use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 151)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct UpdatePlayerGameTypePacket<V: ProtoVersion> {
    pub player_game_type: V::GameType,
    pub target_player: V::ActorUniqueID,
    #[endianness(var)]
    pub tick: u32,
}
