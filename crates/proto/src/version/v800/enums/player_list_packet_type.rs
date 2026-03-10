use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::ProtoCodec;
use uuid::Uuid;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum PlayerListPacketType<V: ProtoVersion> {
    Add {
        add_player_list: Vec<AddPlayerListEntry<V>>,
        is_trusted_skin: bool,
    } = 0,
    Remove {
        remove_player_list: Vec<Uuid>,
    } = 1,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct AddPlayerListEntry<V: ProtoVersion> {
    pub uuid: Uuid,
    pub target_actor_id: V::ActorUniqueID,
    pub player_name: String,
    pub xbl_xuid: String,
    pub platform_chat_id: String,
    pub build_platform: V::BuildPlatform,
    pub serialized_skin: V::SerializedSkin,
    pub is_teacher: bool,
    pub is_host: bool,
    pub is_sub_client: bool,
    pub color: V::Color,
}
