use bedrock_network_macros::{ProtoCodec, packet};

#[packet(id = 362, direction = "client_to_server")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ServerBoundRequestOnlineMusicPlaybackPacket {
    #[endianness(le)]
    pub request_id: u32,
    pub sound_event_name: String,
}
