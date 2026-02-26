use super::super::enums::ActorLinkType;
use super::super::types::ActorUniqueID;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct ActorLink {
    pub actor_unique_id_a: ActorUniqueID,
    pub actor_unique_id_b: ActorUniqueID,
    pub link_type: ActorLinkType,
    pub immediate: bool,
    /// Whether the link was changed by the passenger
    pub passenger_initiated: bool,
    #[endianness(le)]
    pub vehicle_angular_velocity: f32,
}
