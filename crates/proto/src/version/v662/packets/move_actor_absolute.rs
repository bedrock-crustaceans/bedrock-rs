use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 18)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct MoveActorAbsolutePacket<V: ProtoVersion> {
    pub move_data: V::MoveActorAbsoluteData,
}
