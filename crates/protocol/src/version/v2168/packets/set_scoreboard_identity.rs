use crate::ProtoVersion;
use bedrock_macros::{ProtoCodec, packet};

#[packet(id = 112)]
#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum SetScoreboardIdentityPacket<V: ProtoVersion> {
    Update(Vec<IdentityInfoEntry<V>>) = 0,
    Remove(Vec<IdentityInfoEntry<V>>) = 1,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct IdentityInfoEntry<V: ProtoVersion> {
    pub scoreboard_id: V::ScoreboardId,
    #[endianness(var)]
    pub player_unique_id: i64,
}
