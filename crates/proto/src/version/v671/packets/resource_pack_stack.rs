use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, gamepacket};

#[gamepacket(id = 7)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ResourcePackStackPacket<V: ProtoVersion> {
    pub texture_pack_required: bool,

    pub addon_list: Vec<PackEntry>,

    pub texture_pack_list: Vec<PackEntry>,
    pub base_game_version: V::BaseGameVersion,
    pub experiments: V::Experiments,
    pub include_editor_packs: bool,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct PackEntry {
    pub id: String,
    pub version: String,
    pub sub_pack_name: String,
}
