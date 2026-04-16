use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 112)]
#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum SetScoreboardIdentityPacket<V: ProtoVersion> {
    Update(Vec<IdentityInfoUpdateEntry<V>>) = 0,
    Remove(Vec<V::ScoreboardId>) = 1,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct IdentityInfoUpdateEntry<V: ProtoVersion> {
    pub scoreboard_id: V::ScoreboardId,
    #[endianness(var)]
    pub player_unique_id: i64,
}
