use bedrock_macros::{ProtoCodec, packet};

#[packet(id = 358)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ServerBoundCursorItemDragPacket {
    pub state: CursorItemDragState,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u8)]
#[repr(u8)]
pub enum CursorItemDragState {
    Start = 0,
    Stop = 1,
}
