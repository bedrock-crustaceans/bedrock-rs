use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 151)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct UpdatePlayerGameTypePacket<V: ProtoVersion> {
    pub player_game_type: V::GameType,
    pub target_player: V::ActorUniqueID,
}