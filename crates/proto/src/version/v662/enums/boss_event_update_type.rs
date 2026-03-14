use crate::version::versions::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i32)]
#[enum_endianness(le)]
#[repr(i32)]
pub enum BossEventUpdateType<V: ProtoVersion> {
    Add {
        name: String,
        #[endianness(le)]
        health_percent: f32,
        #[endianness(le)]
        darken_screen: u16,
        #[endianness(var)]
        color: u32,
        #[endianness(var)]
        overlay: u32,
    } = 0,
    PlayerAdded {
        player_id: V::ActorUniqueID,
    } = 1,
    Remove = 2,
    PlayerRemoved {
        player_id: V::ActorUniqueID,
    } = 3,
    UpdatePercent {
        #[endianness(le)]
        health_percent: f32,
    } = 4,
    UpdateName {
        name: String,
    } = 5,
    UpdateProperties {
        #[endianness(le)]
        darken_screen: u16,
        #[endianness(var)]
        color: u32,
        #[endianness(var)]
        overlay: u32,
    } = 6,
    UpdateStyle {
        #[endianness(var)]
        color: u32,
        #[endianness(var)]
        overlay: u32,
    } = 7,
    Query {
        player_id: V::ActorUniqueID,
    } = 8,
}
