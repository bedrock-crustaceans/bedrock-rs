use crate::version::versions::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraPreset<V: ProtoVersion> {
    pub name: String,
    pub inherit_from: String,
    #[endianness(le)]
    pub pos_x: Option<f32>,
    #[endianness(le)]
    pub pos_y: Option<f32>,
    #[endianness(le)]
    pub pos_z: Option<f32>,
    #[endianness(le)]
    pub rot_x: Option<f32>,
    #[endianness(le)]
    pub rot_y: Option<f32>,
    #[endianness(le)]
    pub rot_speed: Option<f32>,
    pub snap_to_target: Option<bool>,
    #[endianness(le)]
    pub horizontal_rot_limit: Option<(f32, f32)>,
    #[endianness(le)]
    pub vertical_rot_limit: Option<(f32, f32)>,
    pub continue_targeting: Option<bool>,
    #[endianness(le)]
    pub block_listening_radius: Option<f32>,
    #[endianness(le)]
    pub view_offset: Option<(f32, f32)>,
    #[endianness(le)]
    pub entity_offset: Option<(f32, f32, f32)>,
    #[endianness(le)]
    pub radius: Option<f32>,
    pub listener: Option<AudioListener>,
    pub player_effects: Option<bool>,
    pub align_target_and_camera_forwards: Option<bool>,
    pub aim_assist_preset: Option<V::CameraAimAssistPreset>,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum AudioListener {
    Camera = 0,
    Player = 1,
}
