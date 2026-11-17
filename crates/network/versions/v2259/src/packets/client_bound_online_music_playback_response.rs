use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::{ProtoCodec, packet};

#[packet(id = 363, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientBoundOnlineMusicPlaybackResponsePacket<V: ProtoVersion> {
    #[endianness(le)]
    pub request_id: u32,
    pub state: MusicPlaybackState,
    pub shared_metadata: V::SignedAudioContent,
    pub playback_content: V::SignedAudioContent,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u8)]
#[repr(u8)]
pub enum MusicPlaybackState {
    Local = 0,
    Online = 1,
    NotFound = 2,
    Pending = 3,
}
