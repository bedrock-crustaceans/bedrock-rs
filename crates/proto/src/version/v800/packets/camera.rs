use super::super::types::ActorUniqueID;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 73)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraPacket {
    pub camera_id: ActorUniqueID,
    pub target_player_id: ActorUniqueID,
}