use crate::ProtoVersion;
use bedrock_macros::{ProtoCodec, packet};

#[packet(id = 360)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ServerBoundRegisterAudioContentPacket<V: ProtoVersion> {
    pub registrations: Vec<AudioContentRegistrationEntry<V>>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct AudioContentRegistrationEntry<V: ProtoVersion> {
    pub audio_content_id: String,
    pub shared_metadata: V::SignedAudioContent,
    pub server_content: V::SignedAudioContent,
    pub playback_content: V::SignedAudioContent,
}
