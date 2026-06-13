use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 74)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct BossEventPacket<V: ProtoVersion> {
    pub target_actor_id: V::ActorUniqueID,
    pub player_actor_id: V::ActorUniqueID,
    pub event_type: BossEventUpdateType,
    pub name: String,
    pub filtered_name: String,
    #[endianness(le)]
    pub health_percent: f32,
    pub color: u8,
    pub overlay: u8,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u8)]
#[repr(u8)]
pub enum BossEventUpdateType {
    Add = 0,
    PlayerAdded = 1,
    Remove = 2,
    PlayerRemoved = 3,
    UpdatePercent = 4,
    UpdateName = 5,
    UpdateProperties = 6,
    UpdateStyle = 7,
    Query = 8,
}
