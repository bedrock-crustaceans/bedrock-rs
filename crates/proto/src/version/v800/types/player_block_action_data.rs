use super::super::enums::PlayerActionType;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerBlockActionData {
    pub player_action_type: PlayerActionType,
}