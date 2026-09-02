use crate::ProtoVersion;
use bedrock_macros::{ProtoCodec, packet};

#[packet(id = 348)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientBoundUpdateSoundDataPacket<V: ProtoVersion> {
    #[endianness(le)]
    pub server_sound_handle: i64,
    pub stop: V::SoundData,
    pub set_volume: V::SoundData,
    pub set_pitch: V::SoundData,
    pub fade: V::SoundData,
    pub seek_to: V::SoundData,
    pub pause: V::SoundData,
    pub resume: V::SoundData,
}
