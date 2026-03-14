use crate::version::ProtoVersion;
use bedrockrs_macros::ProtoCodec;
use vek::{Vec2, Vec3};

#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraInstruction<V: ProtoVersion> {
    pub set: Option<SetInstruction<V>>,
    pub clear: Option<bool>,
    pub fade: Option<FadeInstruction>,
    pub target: Option<TargetInstruction<V>>,
}

// VERIFY: SetInstruction & FadeInstruction

#[derive(ProtoCodec, Clone, Debug)]
pub struct EaseData<V: ProtoVersion> {
    pub ease_type: V::EasingType,
    #[endianness(le)]
    pub ease_time: f32,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct SetInstruction<V: ProtoVersion> {
    #[endianness(le)]
    pub runtime_id: i32,
    pub ease_data: Option<EaseData<V>>,
    #[endianness(le)]
    pub position: Option<Vec3<f32>>,
    #[endianness(le)]
    pub rotation: Option<Vec2<f32>>,
    #[endianness(le)]
    pub facing: Option<Vec3<f32>>,
    #[endianness(le)]
    pub view_offset: Option<Vec2<f32>>,
    #[endianness(le)]
    pub entity_offset: Option<Vec3<f32>>,
    pub default_preset: Option<bool>,
    pub remove_ignore_starting_values_component: bool,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct TimeData {
    #[endianness(le)]
    pub fade_in_time: f32,
    #[endianness(le)]
    pub wait_time: f32,
    #[endianness(le)]
    pub fade_out_time: f32,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct Color {
    #[endianness(le)]
    pub r: f32,
    #[endianness(le)]
    pub g: f32,
    #[endianness(le)]
    pub b: f32,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct FadeInstruction {
    pub time_data: Option<TimeData>,
    pub color: Option<Color>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct TargetInstruction<V: ProtoVersion> {
    #[endianness(le)]
    pub target_center_offset: Option<Vec3<f32>>,
    pub actor_unique_id: V::ActorUniqueID,
}
