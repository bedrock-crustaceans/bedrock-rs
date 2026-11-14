use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::{packet, ProtoCodec};

#[packet(id = 40, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetActorMotionPacket<V: ProtoVersion> {
    pub target_runtime_id: V::ActorRuntimeID,
    #[endianness(le)]
    pub motion: (f32, f32, f32),
    #[endianness(var)]
    pub server_tick: u64,
}
