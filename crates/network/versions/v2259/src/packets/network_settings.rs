use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::{packet, ProtoCodec};

#[packet(id = 143, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct NetworkSettingsPacket<V: ProtoVersion> {
    #[endianness(le)]
    pub compression_threshold: u16,
    pub compression_algorithm: V::PacketCompressionAlgorithm,
    pub client_throttle_enabled: bool,
    pub client_throttle_threshold: i8,
    #[endianness(le)]
    pub client_throttle_scalar: f32,
    pub sctp_recovery_profile: SctpRecoveryProfile,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u8)]
#[repr(u8)]
pub enum SctpRecoveryProfile {
    Default = 0,
    Fast = 1,
    Balanced = 2,
}
