use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 60)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetDifficultyPacket<V: ProtoVersion> {
    pub difficulty: V::Difficulty,
}
