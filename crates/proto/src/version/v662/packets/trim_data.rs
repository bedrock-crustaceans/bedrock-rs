use bedrockrs_macros::{ProtoCodec, gamepacket};

#[gamepacket(id = 302)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct TrimDataPacket {
    pub trim_pattern_list: Vec<TrimPattern>,

    pub trim_material_list: Vec<TrimMaterial>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct TrimPattern {
    pub item_name: String,
    pub pattern_id: String,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct TrimMaterial {
    pub material_id: String,
    pub color: String,
    pub item_name: String,
}
