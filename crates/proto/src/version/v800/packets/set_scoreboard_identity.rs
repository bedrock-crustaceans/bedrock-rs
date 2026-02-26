use super::super::enums::ScoreboardIdentityPacketType;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 112)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetScoreboardIdentityPacket {
    pub scoreboard_identity_packet_type: ScoreboardIdentityPacketType,
}

// TODO: same thing here, scoreboard seem to be a bit janky. Might refactor