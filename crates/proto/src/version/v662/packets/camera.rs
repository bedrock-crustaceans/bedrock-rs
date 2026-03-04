use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 73)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraPacket<V: ProtoVersion> {
    pub camera_id: V::ActorUniqueID,
    pub target_player_id: V::ActorUniqueID,
}
