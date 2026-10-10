use crate::ProtoVersion;
use bedrock_macros::{ProtoCodec, packet};
use strum_macros::{Display, EnumString};

#[packet(id = 359, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientBoundPlayAudioContentPacket<V: ProtoVersion> {
    pub shared_metadata: V::SignedAudioContent,
    pub playback_content: V::SignedAudioContent,
    #[str]
    pub playback_type: AudioContentPlaybackType,
    pub play_sound: V::PlaySoundPacket,
}

#[derive(Clone, Debug, EnumString, Display)]
pub enum AudioContentPlaybackType {
    Music,
    Sound,
}
