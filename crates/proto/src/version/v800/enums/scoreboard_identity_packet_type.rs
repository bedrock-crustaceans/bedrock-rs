use super::super::types::ScoreboardId;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum ScoreboardIdentityPacketType {
    Update {
        #[vec_repr(u32)]
        #[vec_endianness(var)]
        identity_info: Vec<IdentityInfoUpdateEntry>,
    } = 0,
    Remove {
        #[vec_repr(u32)]
        #[vec_endianness(var)]
        identity_info: Vec<ScoreboardId>,
    } = 1,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct IdentityInfoUpdateEntry {
    pub scoreboard_id: ScoreboardId,
    #[endianness(var)]
    pub player_unique_id: i64,
}
