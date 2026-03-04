use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 143)]
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