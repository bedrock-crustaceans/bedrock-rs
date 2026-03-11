use crate::version::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, packet};
use vek::{Vec2, Vec3};

#[packet(id = 161)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CorrectPlayerMovePredictionPacket<V: ProtoVersion> {
    pub prediction_type: V::PredictionType,
    #[endianness(le)]
    pub position: Vec3<f32>,
    #[endianness(le)]
    pub velocity: Vec3<f32>,
    #[endianness(le)]
    rotation: Vec2<f32>,
    #[endianness(le)]
    vehicle_angular_velocity: Option<f32>,
    pub on_ground: bool,
    #[endianness(var)]
    pub tick: u64,
}
