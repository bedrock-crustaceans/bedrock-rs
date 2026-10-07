use crate::ProtoVersion;
use crate::version::v662::packets::AttributeEntry;
use bedrock_macros::{ProtoCodec, packet};

#[packet(id = 13)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct AddActorPacket<V: ProtoVersion> {
    pub target_actor_id: V::ActorUniqueID,
    pub target_runtime_id: V::ActorRuntimeID,
    pub actor_type: String,
    #[endianness(le)]
    pub position: (f32, f32, f32),
    #[endianness(le)]
    pub velocity: (f32, f32, f32),
    #[endianness(le)]
    pub rotation: (f32, f32),
    #[endianness(le)]
    pub y_head_rotation: f32,
    #[endianness(le)]
    pub y_body_rotation: f32,
    pub attributes: Vec<AttributeEntry>,
    pub actor_data: Vec<V::DataItem>,
    pub synced_properties: V::PropertySyncData,
    pub actor_links: Vec<V::ActorLink>,
    pub passenger_of_block_data: Option<V::PassengerOfBlockArguments>,
}
