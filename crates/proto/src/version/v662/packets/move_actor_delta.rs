use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 111)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct MoveActorDeltaPacket<V: ProtoVersion> {
    pub move_data: V::MoveActorDeltaData,
}
