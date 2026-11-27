use bedrock_network_macros::{ProtoCodec, packet};

#[packet(id = 367)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct VirtualContainerClosePacket {
    #[endianness(var)]
    pub session_id: u64,
    pub kind: VirtualContainerCloseKind,
    pub cause: VirtualContainerCloseCause,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u8)]
#[repr(u8)]
pub enum VirtualContainerCloseKind {
    Request = 0,
    Closed = 1,
    Confirmed = 2,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u8)]
#[repr(u8)]
pub enum VirtualContainerCloseCause {
    Unknown = 0,
    UserClosed = 1,
    ServerRequested = 2,
    UserBusy = 3,
    ClientNotReady = 4,
    InvalidContents = 5,
    UiCreationFailed = 6,
    UiCancelled = 7,
}
