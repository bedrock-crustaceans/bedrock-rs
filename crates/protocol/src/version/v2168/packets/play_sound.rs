use crate::ProtoVersion;
use bedrock_macros::{ProtoCodec, packet};

#[packet(id = 86, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlaySoundPacket<V: ProtoVersion> {
    pub name: String,
    pub position: V::NetworkBlockPosition,
    #[endianness(le)]
    pub volume: f32,
    #[endianness(le)]
    pub pitch: f32,
    #[endianness(var)]
    pub loop_count: i32,
    #[endianness(le)]
    pub server_sound_handle: Option<i64>,
}
