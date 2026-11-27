use bedrock_network_macros::{ProtoCodec, packet};
use bedrock_network_version_v924::packets::ServerBoundDataStoreValue;

#[packet(id = 332, direction = "client_to_server")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ServerBoundDataStorePacket {
    pub variable_name: String,
    pub data: ServerBoundDataStoreValue,
    #[endianness(le)]
    pub update_count: u32,
}
