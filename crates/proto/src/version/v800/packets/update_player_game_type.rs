use super::super::enums::GameType;
use super::super::types::ActorUniqueID;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 151)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct UpdatePlayerGameTypePacket {
    pub player_game_type: GameType,
    pub target_player: ActorUniqueID,
    #[endianness(var)]
    pub player_replay_state_component_tick: u32,
}