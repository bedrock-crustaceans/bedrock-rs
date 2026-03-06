use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 323)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct UpdateClientOptionsPacket {
    pub graphics_mode: Option<GraphicsMode>,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum GraphicsMode {
    Simple = 0,
    Fancy = 1,
    Advanced = 2,
    RayTraced = 3,
}
