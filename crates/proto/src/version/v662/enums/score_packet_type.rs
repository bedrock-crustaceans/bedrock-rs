use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum ScorePacketType<V: ProtoVersion> {
    Change {
        #[vec_repr(u32)]
        #[vec_endianness(var)]
        score_packet_info: Vec<ScorePacketInfoChangeEntry<V>>,
    } = 0,
    Remove {
        #[vec_repr(u32)]
        #[vec_endianness(var)]
        score_packet_info: Vec<ScorePacketInfoRemoveEntry<V>>,
    } = 1,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct ScorePacketInfoChangeEntry<V: ProtoVersion> {
    pub id: V::ScoreboardId,
    pub objective_name: String,
    #[endianness(le)]
    pub score_value: i32,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct ScorePacketInfoRemoveEntry<V: ProtoVersion> {
    pub id: V::ScoreboardId,
    pub objective_name: String,
    #[endianness(le)]
    pub score_value: i32,
    pub identity_definition_type: V::IdentityDefinitionType,
}
