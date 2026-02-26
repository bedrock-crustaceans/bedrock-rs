use super::super::enums::{MinecraftPacketIds, PacketViolationSeverity, PacketViolationType};
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 156)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PacketViolationWarningPacket {
    pub violation_type: PacketViolationType,
    pub violation_severity: PacketViolationSeverity,
    pub violating_packet_id: MinecraftPacketIds,
    pub violation_context: String,
}