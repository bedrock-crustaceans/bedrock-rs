use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 156, direction = "client_to_server")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PacketViolationWarningPacket<V: ProtoVersion> {
    pub violation_type: V::PacketViolationType,
    pub violation_severity: V::PacketViolationSeverity,
    pub violating_packet_id: V::MinecraftPacketIds,
    pub violation_context: String,
}
