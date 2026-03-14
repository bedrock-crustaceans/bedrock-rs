use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 151)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct UpdatePlayerGameTypePacket<V: ProtoVersion> {
    pub player_game_type: V::GameType,
    pub target_player: V::ActorUniqueID,
    #[endianness(var)]
    pub tick: u32,
}
