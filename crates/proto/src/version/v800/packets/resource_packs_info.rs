use uuid::Uuid;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 6)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ResourcePacksInfoPacket {
    pub resource_pack_required: bool,
    pub has_addon_packs: bool,
    pub has_scripts: bool,
    pub world_template_uuid: Uuid,
    pub world_template_version: String,
    #[vec_repr(u16)]
    #[vec_endianness(le)]
    pub resource_packs: Vec<ResourcePackEntry>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct ResourcePackEntry {
    pub id: Uuid,
    pub version: String,
    #[endianness(le)]
    pub size: u64,
    pub content_key: String,
    pub sub_pack_name: String,
    pub content_identity: String,
    pub has_scripts: bool,
    pub is_addon: bool,
    pub is_ray_tracing_capable: bool,
    pub cdn_url: String,
}