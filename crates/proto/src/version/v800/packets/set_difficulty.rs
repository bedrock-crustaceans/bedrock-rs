use super::super::enums::Difficulty;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 60)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetDifficultyPacket {
    pub difficulty: Difficulty,
}