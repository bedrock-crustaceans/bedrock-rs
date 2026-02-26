use vek::Vec2;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum PredictionType {
    Player = 0,
    Vehicle {
        #[endianness(le)]
        rotation: Vec2<f32>,
        #[endianness(le)]
        vehicle_angular_velocity: Option<f32>,
    } = 1,
}