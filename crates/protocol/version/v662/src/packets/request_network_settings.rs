use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 193, direction = "client_to_server")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct RequestNetworkSettingsPacket {
    #[endianness(be)]
    pub client_network_version: i32,
}
