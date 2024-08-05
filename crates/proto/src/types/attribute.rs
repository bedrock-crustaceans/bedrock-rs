use bedrockrs_core::int::{LE, VAR};
use bedrockrs_proto_derive::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone)]
pub struct Attribute {
    name: String,
    min: LE<f32>,
    current: LE<f32>,
    max: LE<f32>,
}
