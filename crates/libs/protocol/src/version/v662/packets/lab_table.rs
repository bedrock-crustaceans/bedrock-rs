use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 109)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct LabTablePacket<V: ProtoVersion> {
    pub lab_table_packet_type: LabTableType,
    pub position: V::BlockPos,
    pub reaction: V::LabTableReactionType,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum LabTableType {
    StartCombine = 0,
    StartReaction = 1,
    Reset = 2,
}
