use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum PredictionType {
    Player = 0,
    Vehicle {
        #[endianness(le)]
        rotation: (f32, f32),
    } = 1,
}
