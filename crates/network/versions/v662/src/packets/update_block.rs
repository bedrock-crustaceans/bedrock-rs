use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::{packet, ProtoCodec};

#[packet(id = 21, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct UpdateBlockPacket<V: ProtoVersion> {
    pub block_position: V::NetworkBlockPosition,
    #[endianness(var)]
    pub block_runtime_id: u32,
    #[endianness(var)]
    pub flags: u32,
    #[endianness(var)]
    pub layer: u32,
}
