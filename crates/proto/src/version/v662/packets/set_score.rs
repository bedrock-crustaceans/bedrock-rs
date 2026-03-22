use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 108)]
#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum SetScorePacket<V: ProtoVersion> {
    Change {
        score_packet_info: Vec<ScorePacketInfoChangeEntry<V>>,
    } = 0,
    Remove {
        score_packet_info: Vec<ScorePacketInfoRemoveEntry<V>>,
    } = 1,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct ScorePacketInfoChangeEntry<V: ProtoVersion> {
    pub id: V::ScoreboardId,
    pub objective_name: String,
    #[endianness(le)]
    pub score_value: i32,
    pub identity_definition_type: V::IdentityDefinitionType,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct ScorePacketInfoRemoveEntry<V: ProtoVersion> {
    pub id: V::ScoreboardId,
    pub objective_name: String,
    #[endianness(le)]
    pub score_value: i32,
}
