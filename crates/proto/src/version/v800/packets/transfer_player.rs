use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 85)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct TransferPlayerPacket {
    pub server_address: String,
    #[endianness(le)]
    pub server_port: u16,
    pub reload_world: bool,
}