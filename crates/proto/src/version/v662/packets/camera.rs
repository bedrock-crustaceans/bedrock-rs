use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 73)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraPacket<V: ProtoVersion> {
    pub camera_id: V::ActorUniqueID,
    pub target_player_id: V::ActorUniqueID,
}
