use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 196)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct UpdateClientInputLocksPacket {
    #[endianness(var)]
    pub input_lock_component_data: u32,
    #[endianness(le)]
    pub server_pos: (f32, f32, f32),
}
