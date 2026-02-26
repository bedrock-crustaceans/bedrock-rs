use super::super::enums::{AnimationMode, Mirror, Rotation};
use super::super::types::{ActorUniqueID, NetworkBlockPosition};
use bedrockrs_macros::ProtoCodec;
use vek::Vec3;

#[derive(ProtoCodec, Clone, Debug)]
pub struct StructureSettings {
    pub structure_palette_name: String,
    pub ignore_entities: bool,
    pub ignore_locks: bool,
    pub allow_non_ticking_player_and_ticking_area_chunks: bool,
    pub structure_size: NetworkBlockPosition,
    pub structure_offset: NetworkBlockPosition,
    pub last_edit_player: ActorUniqueID,
    pub rotation: Rotation,
    pub mirror: Mirror,
    pub animation_mode: AnimationMode,
    #[endianness(le)]
    pub animation_seconds: f32,
    #[endianness(le)]
    pub integrity_value: f32,
    #[endianness(le)]
    pub integrity_seed: u32,
    #[endianness(le)]
    pub rotation_pivot: Vec3<f32>,
}