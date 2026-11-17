use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::{packet, ProtoCodec};

#[packet(id = 86, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlaySoundPacket<V: ProtoVersion> {
    pub name: String,
    pub position: V::NetworkBlockPosition,
    #[endianness(le)]
    pub volume: f32,
    #[endianness(le)]
    pub pitch: f32,
}
