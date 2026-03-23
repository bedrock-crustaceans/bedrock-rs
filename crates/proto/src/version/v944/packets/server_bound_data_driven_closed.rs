use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 343)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ServerBoundDataDrivenClosedPacket {
    #[endianness(le)]
    pub form_id: i32,
    pub close_reason: DataDrivenScreenClosedReason
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u8)]
#[repr(u8)]
pub enum DataDrivenScreenClosedReason {
    ProgrammaticClose = 0,
    ProgrammaticCloseAll = 1,
    ClientCancelled = 2,
    UserBusy = 3,
    InvalidForm = 4,
}