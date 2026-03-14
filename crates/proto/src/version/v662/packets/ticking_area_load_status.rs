use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 179)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct TickingAreaLoadStatusPacket {
    pub waiting_for_preload: bool,
}
