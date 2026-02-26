use super::super::enums::ScorePacketType;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 108)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetScorePacket {
    pub score_packet_type: ScorePacketType,
}

// TODO: this kinda sucks, might wanna refactor later