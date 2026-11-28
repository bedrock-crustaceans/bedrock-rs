use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::{packet, ProtoCodec};

#[packet(id = 22, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct AddPaintingPacket<V: ProtoVersion> {
    pub target_actor_id: V::ActorUniqueID,
    pub target_runtime_id: V::ActorRuntimeID,
    #[endianness(le)]
    pub position: (f32, f32, f32),
    #[endianness(var)]
    pub direction: i32,
    pub motif: String,
}
