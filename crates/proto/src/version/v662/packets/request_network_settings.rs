use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 193)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct RequestNetworkSettingsPacket {
    #[endianness(be)]
    pub client_network_version: i32,
}
