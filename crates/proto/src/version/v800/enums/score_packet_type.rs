use super::super::enums::IdentityDefinitionType;
use super::super::types::ScoreboardId;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum ScorePacketType {
    Change {
        #[vec_repr(u32)]
        #[vec_endianness(var)]
        score_packet_info: Vec<ScorePacketInfoChangeEntry>,
    } = 0,
    Remove {
        #[vec_repr(u32)]
        #[vec_endianness(var)]
        score_packet_info: Vec<ScorePacketInfoRemoveEntry>,
    } = 1,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct ScorePacketInfoChangeEntry {
    pub id: ScoreboardId,
    pub objective_name: String,
    #[endianness(le)]
    pub score_value: i32,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct ScorePacketInfoRemoveEntry {
    pub id: ScoreboardId,
    pub objective_name: String,
    #[endianness(le)]
    pub score_value: i32,
    pub identity_definition_type: IdentityDefinitionType,
}
