use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::ProtoCodec;
use uuid::Uuid;

#[derive(ProtoCodec, Clone, Debug)]
pub struct CommandOriginData<V: ProtoVersion> {
    pub command_type: V::CommandOriginType,
    pub command_uuid: Uuid,
    pub request_id: String,
    #[endianness(le)]
    pub player_id: i64,
}
