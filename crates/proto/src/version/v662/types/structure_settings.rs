use crate::version::versions::ProtoVersion;
use bedrockrs_macros::ProtoCodec;
use vek::Vec3;

#[derive(ProtoCodec, Clone, Debug)]
pub struct StructureSettings<V: ProtoVersion> {
    pub structure_palette_name: String,
    pub ignore_entities: bool,
    pub ignore_locks: bool,
    pub allow_non_ticking_player_and_ticking_area_chunks: bool,
    pub structure_size: V::NetworkBlockPosition,
    pub structure_offset: V::NetworkBlockPosition,
    pub last_edit_player: V::ActorUniqueID,
    pub rotation: V::Rotation,
    pub mirror: V::Mirror,
    pub animation_mode: V::AnimationMode,
    #[endianness(le)]
    pub animation_seconds: f32,
    #[endianness(le)]
    pub integrity_value: f32,
    #[endianness(le)]
    pub integrity_seed: u32,
    #[endianness(le)]
    pub rotation_pivot: Vec3<f32>,
}
