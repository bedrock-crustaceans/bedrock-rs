use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 18)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct MoveActorAbsolutePacket<V: ProtoVersion> {
    pub move_data: V::MoveActorAbsoluteData,
}