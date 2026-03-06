use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 156)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PacketViolationWarningPacket<V: ProtoVersion> {
    pub violation_type: V::PacketViolationType,
    pub violation_severity: V::PacketViolationSeverity,
    pub violating_packet_id: V::MinecraftPacketIds,
    pub violation_context: String,
}
