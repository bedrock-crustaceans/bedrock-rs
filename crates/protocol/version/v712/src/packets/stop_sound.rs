use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 87, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct StopSoundPacket {
    pub sound_name: String,
    pub stop_all_sounds: bool,
    pub stop_music_legacy: bool,
}
