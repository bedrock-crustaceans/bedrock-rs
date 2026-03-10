use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum ScoreboardIdentityPacketType<V: ProtoVersion> {
    Update {
        identity_info: Vec<IdentityInfoUpdateEntry<V>>,
    } = 0,
    Remove {
        identity_info: Vec<V::ScoreboardId>,
    } = 1,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct IdentityInfoUpdateEntry<V: ProtoVersion> {
    pub scoreboard_id: V::ScoreboardId,
    #[endianness(var)]
    pub player_unique_id: i64,
}
