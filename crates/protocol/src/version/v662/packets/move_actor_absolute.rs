use crate::version::versions::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 18)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct MoveActorAbsolutePacket<V: ProtoVersion> {
    pub move_data: V::MoveActorAbsoluteData,
}
