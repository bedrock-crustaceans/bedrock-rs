use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 324)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerVideoCapturePacket {
    pub action: PlayerVideoCapturePacketAction
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum PlayerVideoCapturePacketAction {
    StopVideoCapture = 0,
    StartVideoCapture {
        #[endianness(le)]
        frame_rate: i32,
        file_prefix: String,
    } = 1,
    Unknown = 2,
}