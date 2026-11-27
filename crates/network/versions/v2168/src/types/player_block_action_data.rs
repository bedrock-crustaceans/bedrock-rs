use bedrock_network_version_v662::enums::PlayerActionType;
use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerBlockActionData<V: ProtoVersion> {
    pub action_type: PlayerActionType,
    pub position: V::BlockPos,
    #[endianness(var)]
    pub facing: i32,
}
