use super::super::enums::BuildPlatform;
use super::super::types::{ActorUniqueID, SerializedSkin};
use bedrockrs_macros::ProtoCodec;
use uuid::Uuid;
use crate::v800::types::Color;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum PlayerListPacketType {
    Add {
        #[vec_repr(u32)]
        #[vec_endianness(var)]
        add_player_list: Vec<AddPlayerListEntry>,
        is_trusted_skin: bool,
    } = 0,
    Remove {
        #[vec_repr(u32)]
        #[vec_endianness(var)]
        remove_player_list: Vec<Uuid>
    } = 1,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct AddPlayerListEntry {
    pub uuid: Uuid,
    pub target_actor_id: ActorUniqueID,
    pub player_name: String,
    pub xbl_xuid: String,
    pub platform_chat_id: String,
    pub build_platform: BuildPlatform,
    pub serialized_skin: SerializedSkin,
    pub is_teacher: bool,
    pub is_host: bool,
    pub is_sub_client: bool,
    pub color: Color,
}
