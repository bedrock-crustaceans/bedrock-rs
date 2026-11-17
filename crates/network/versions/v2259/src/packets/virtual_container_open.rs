use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::{ProtoCodec, packet};

#[packet(id = 366, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct VirtualContainerOpenPacket<V: ProtoVersion> {
    pub window_id: u8,
    #[endianness(var)]
    pub session_id: u64,
    pub title: String,
    pub slots: Vec<V::NetworkItemStackDescriptor>,
}
