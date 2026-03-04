use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 111)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct MoveActorDeltaPacket<V: ProtoVersion> {
    pub move_data: V::MoveActorDeltaData,
}