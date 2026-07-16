use crate::ProtoVersion;
use bedrock_macros::{ProtoCodec, packet};

#[packet(id = 348)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientBoundUpdateSoundDataPacket<V: ProtoVersion> {
    #[endianness(le)]
    pub server_sound_handle: i64,
    pub stop: Option<V::SoundData>,
    pub set_volume: Option<V::SoundData>,
    pub set_pitch: Option<V::SoundData>,
    pub fade: Option<V::SoundData>,
    pub seek_to: Option<V::SoundData>,
    pub pause: Option<V::SoundData>,
    pub resume: Option<V::SoundData>,
}
