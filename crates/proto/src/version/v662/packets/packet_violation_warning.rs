use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 156)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PacketViolationWarningPacket<V: ProtoVersion> {
    pub violation_type: V::PacketViolationType,
    pub violation_severity: V::PacketViolationSeverity,
    pub violating_packet_id: V::MinecraftPacketIds,
    pub violation_context: String,
}