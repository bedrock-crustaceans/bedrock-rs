use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 111, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct MoveActorDeltaPacket<V: ProtoVersion> {
    pub move_data: V::MoveActorDeltaData,
}
