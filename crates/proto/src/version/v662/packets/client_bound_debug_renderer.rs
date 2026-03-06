use bedrockrs_macros::{packet, ProtoCodec};
use vek::Vec3;

#[packet(id = 163)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientBoundDebugRendererPacket {
    pub debug_marker_type: Type,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u32)]
#[enum_endianness(var)]
#[repr(u32)]
pub enum Type {
    Invalid = 0,
    ClearDebugMarkers = 1,
    AddDebugMarkerCube {
        text: String,
        #[endianness(le)]
        position: Vec3<f32>,
        #[endianness(le)]
        r: f32,
        #[endianness(le)]
        g: f32,
        #[endianness(le)]
        b: f32,
        #[endianness(le)]
        a: f32,
        #[endianness(le)]
        millisecond_duration: u64,
    } = 2,
}
