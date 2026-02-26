use super::super::types::PlayerBlockActionData;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerBlockActions {
    #[vec_repr(i32)]
    #[vec_endianness(var)]
    pub player_block_actions: Vec<PlayerBlockActionData>,
}