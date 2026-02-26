use super::super::enums::{AnimatedTextureType, AnimationExpression};
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct SerializedSkin {
    pub skin_id: String,
    pub play_fab_id: String,
    pub skin_resource_patch: String,
    #[endianness(le)]
    pub skin_image_width: u32,
    #[endianness(le)]
    pub skin_image_height: u32,
    pub skin_image_bytes: String,
    #[vec_repr(u32)]
    #[vec_endianness(le)]
    pub animations: Vec<SerializedSkinAnimationFrame>,
    #[endianness(le)]
    pub cape_image_width: u32,
    #[endianness(le)]
    pub cape_image_height: u32,
    pub cape_image_bytes: String,
    pub geometry_data: String,
    pub geometry_data_engine_version: String,
    pub animation_data: String,
    pub cape_id: String,
    pub full_id: String,
    pub arm_size: String,
    pub skin_color: String,
    #[vec_repr(u32)]
    #[vec_endianness(le)]
    pub persona_pieces: Vec<PersonaPiecesEntry>,
    #[vec_repr(u32)]
    #[vec_endianness(le)]
    pub piece_tint_colors: Vec<PieceTintColorsEntry>,
    pub is_premium_skin: bool,
    pub is_persona_skin: bool,
    pub is_persona_cape_on_classic_skin: bool,
    pub is_primary_user: bool,
    pub overrides_player_appearance: bool,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct SerializedSkinAnimationFrame {
    #[endianness(le)]
    pub image_width: u32,
    #[endianness(le)]
    pub image_height: u32,
    pub image_bytes: String,
    pub animation_type: AnimatedTextureType,
    #[endianness(le)]
    pub frame_count: f32,
    pub animation_expression: AnimationExpression,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct PersonaPiecesEntry {
    pub piece_id: String,
    pub piece_type: String,
    pub pack_id: String,
    pub is_default_piece: bool,
    pub product_id: String,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct PieceTintColorsEntry {
    pub piece_type: String,
    pub piece_tint_color: String,
}
