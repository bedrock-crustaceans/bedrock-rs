use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 60)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetDifficultyPacket<V: ProtoVersion> {
    pub difficulty: V::Difficulty,
}