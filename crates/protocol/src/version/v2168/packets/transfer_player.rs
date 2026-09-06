use crate::ProtoVersion;
use bedrock_macros::{ProtoCodec, packet};

#[packet(id = 85)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct TransferPlayerPacket<V: ProtoVersion> {
    pub server_address: String,
    #[endianness(le)]
    pub server_port: u16,
    pub reload_world: bool,
    pub gatherings_config: Option<V::GatheringsConfig>,
}
