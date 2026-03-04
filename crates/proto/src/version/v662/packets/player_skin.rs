use bedrockrs_macros::{gamepacket, ProtoCodec};
use uuid::Uuid;
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 93)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerSkinPacket<V: ProtoVersion> {
    pub uuid: Uuid,
    pub serialized_skin: V::SerializedSkin,
    pub new_skin_name: String,
    pub old_skin_name: String,
    pub trusted_marketplace_skin: bool,
}