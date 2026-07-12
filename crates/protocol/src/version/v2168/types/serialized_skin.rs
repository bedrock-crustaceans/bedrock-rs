use crate::ProtoVersion;
use bedrock_macros::ProtoCodec;
use uuid::Uuid;

#[derive(ProtoCodec, Clone, Debug)]
pub struct SerializedSkin<V: ProtoVersion> {
    pub skin_id: String,
    pub play_fab_id: String,
    pub skin_resource_patch: String,
    #[endianness(le)]
    pub skin_image_width: u32,
    #[endianness(le)]
    pub skin_image_height: u32,
    pub skin_image_bytes: Vec<u8>,
    pub animations: Vec<SerializedSkinAnimationFrame<V>>,
    #[endianness(le)]
    pub cape_image_width: u32,
    #[endianness(le)]
    pub cape_image_height: u32,
    pub cape_image_bytes: Vec<u8>,
    pub geometry_data: String,
    pub geometry_data_engine_version: String,
    pub animation_data: String,
    pub cape_id: String,
    pub full_id: String,
    pub arm_size: V::ArmSizeType,
    #[endianness(le)]
    pub skin_color: i32,
    pub persona_pieces: Vec<PersonaPiecesEntry<V>>,
    pub piece_tint_colors: Vec<PieceTintColorsEntry>,
    pub is_premium_skin: bool,
    pub is_persona_skin: bool,
    pub is_persona_cape_on_classic_skin: bool,
    pub is_primary_user: bool,
    pub overrides_player_appearance: bool,
    pub trusted_skin_flag: String,
    pub profile_hash: String,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct SerializedSkinAnimationFrame<V: ProtoVersion> {
    #[endianness(le)]
    pub image_width: u32,
    #[endianness(le)]
    pub image_height: u32,
    pub image_bytes: Vec<u8>,
    pub animation_type: V::AnimatedTextureType,
    #[endianness(le)]
    pub frame_count: f32,
    pub animation_expression: V::AnimationExpression,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct PersonaPiecesEntry<V: ProtoVersion> {
    pub piece_id: String,
    pub piece_type: V::PersonaPieceType,
    pub pack_id: Uuid,
    pub is_default_piece: bool,
    pub product_id: String,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct PieceTintColorsEntry {
    pub piece_type: String,
    #[endianness(le)]
    pub colors: [i32; 4],
}
