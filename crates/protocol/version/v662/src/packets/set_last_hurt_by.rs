use bedrock_protocol_version::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 96, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetLastHurtByPacket<V: ProtoVersion> {
    pub last_hurt_by: V::ActorType,
}
