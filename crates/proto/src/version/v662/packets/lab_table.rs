use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 109)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct LabTablePacket<V: ProtoVersion> {
    pub lab_table_packet_type: Type,
    pub position: V::BlockPos,
    pub reaction: V::LabTableReactionType,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum Type {
    StartCombine = 0,
    StartReaction = 1,
    Reset = 2,
}