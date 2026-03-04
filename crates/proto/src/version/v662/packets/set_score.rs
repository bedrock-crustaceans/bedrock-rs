use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 108)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetScorePacket<V: ProtoVersion> {
    pub score_packet_type: V::ScorePacketType,
}

// TODO: this kinda sucks, might wanna refactor later