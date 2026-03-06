use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 112)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetScoreboardIdentityPacket<V: ProtoVersion> {
    pub scoreboard_identity_packet_type: V::ScoreboardIdentityPacketType,
}

// TODO: same thing here, scoreboard seem to be a bit janky. Might refactor
