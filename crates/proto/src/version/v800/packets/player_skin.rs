use super::super::types::SerializedSkin;
use bedrockrs_macros::{gamepacket, ProtoCodec};
use uuid::Uuid;

#[gamepacket(id = 93)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerSkinPacket {
    pub uuid: Uuid,
    pub serialized_skin: SerializedSkin,
    pub new_skin_name: String,
    pub old_skin_name: String,
    pub trusted_marketplace_skin: bool,
}