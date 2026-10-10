use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 113, direction = "client_to_server")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetLocalPlayerAsInitializedPacket<V: ProtoVersion> {
    pub player_id: V::ActorRuntimeID,
}
