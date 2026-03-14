use crate::version::versions::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerBlockActions<V: ProtoVersion> {
    pub player_block_actions: Vec<V::PlayerBlockActionData>,
}
