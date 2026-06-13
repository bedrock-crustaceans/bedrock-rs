use bedrock_macros::{ProtoCodec, packet};

#[packet(id = 348)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientBoundUpdateSoundDataPacket {
    #[endianness(le)]
    pub server_sound_handle: i64,
    pub sound_type: String,
}
