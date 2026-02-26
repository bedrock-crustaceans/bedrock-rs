use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 179)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct TickingAreaLoadStatusPacket {
    pub waiting_for_preload: bool,
}