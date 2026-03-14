use crate::version::versions::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerBlockActionData<V: ProtoVersion> {
    pub player_action_type: V::PlayerActionType,
}
