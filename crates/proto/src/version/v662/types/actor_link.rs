use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct ActorLink<V: ProtoVersion> {
    pub actor_unique_id_a: V::ActorUniqueID,
    pub actor_unique_id_b: V::ActorUniqueID,
    pub link_type: V::ActorLinkType,
    pub immediate: bool,
    /// Whether the link was changed by the passenger
    pub passenger_initiated: bool,
}
