use super::super::enums::PacketCompressionAlgorithm;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 143)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct NetworkSettingsPacket {
    #[endianness(le)]
    pub compression_threshold: u16,
    pub compression_algorithm: PacketCompressionAlgorithm,
    pub client_throttle_enabled: bool,
    pub client_throttle_threshold: i8,
    #[endianness(le)]
    pub client_throttle_scalar: f32,
}