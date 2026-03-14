use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 108)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetScorePacket<V: ProtoVersion> {
    pub score_packet_type: V::ScorePacketType,
}

// TODO: this kinda sucks, might wanna refactor later
