use crate::ProtoVersion;
use bedrock_macros::{ProtoCodec, packet};

#[packet(id = 352)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct RecordStartedPacket<V: ProtoVersion> {
    pub block_position: V::NetworkBlockPosition,
    #[endianness(le)]
    pub server_sound_handle: i64,
}
