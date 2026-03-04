use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 112)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetScoreboardIdentityPacket<V: ProtoVersion> {
    pub scoreboard_identity_packet_type: V::ScoreboardIdentityPacketType,
}

// TODO: same thing here, scoreboard seem to be a bit janky. Might refactor