use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 193)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct RequestNetworkSettingsPacket {
    #[endianness(be)]
    pub client_network_version: i32,
}