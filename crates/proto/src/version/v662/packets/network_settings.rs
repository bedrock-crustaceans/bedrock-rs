use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 143)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct NetworkSettingsPacket<V: ProtoVersion> {
    #[endianness(le)]
    pub compression_threshold: u16,
    pub compression_algorithm: V::PacketCompressionAlgorithm,
    pub client_throttle_enabled: bool,
    pub client_throttle_threshold: i8,
    #[endianness(le)]
    pub client_throttle_scalar: f32,
}
