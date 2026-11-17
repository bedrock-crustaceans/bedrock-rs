use bedrock_network_macros::{ProtoCodec, packet};
use crate::packets::MusicPlaybackState;

#[packet(id = 365, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientBoundMusicPlaybackFailurePacket {
    pub failures: Vec<MusicPlaybackFailureEntry>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct MusicPlaybackFailureEntry {
    pub state: MusicPlaybackState,
    pub description: String,
}
