use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 315)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ServerBoundDiagnosticsPacket {
    #[endianness(le)]
    pub avg_fps: f32,
    #[endianness(le)]
    pub avg_server_tick_time_ms: f32,
    #[endianness(le)]
    pub avg_client_tick_time_ms: f32,
    #[endianness(le)]
    pub avg_begin_frame_time_ms: f32,
    #[endianness(le)]
    pub avg_input_time_ms: f32,
    #[endianness(le)]
    pub avg_render_time_ms: f32,
    #[endianness(le)]
    pub avg_end_frame_time_ms: f32,
    #[endianness(le)]
    pub avg_remainder_time_percent: f32,
    #[endianness(le)]
    pub avg_unnacounted_time_percent: f32,
}