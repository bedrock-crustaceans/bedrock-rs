use crate::v662::enums::PlayerActionType;
use crate::ProtoVersion;
use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerBlockActionData<V: ProtoVersion> {
    pub action_type: PlayerActionType,
    pub position: V::BlockPos,
    #[endianness(var)]
    pub facing: i32,
}
