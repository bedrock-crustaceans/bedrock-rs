use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 73, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraPacket<V: ProtoVersion> {
    pub camera_id: V::ActorUniqueID,
    pub target_player_id: V::ActorUniqueID,
}
