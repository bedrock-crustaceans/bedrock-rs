use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::{ProtoCodec, packet};
use bedrock_network_version_v2193::packets::PlaySoundPosition;

#[packet(id = 86, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlaySoundPacket<V: ProtoVersion> {
    pub name: String,
    pub position: PlaySoundPosition<V>,
    #[endianness(le)]
    pub volume: f32,
    #[endianness(le)]
    pub pitch: f32,
    #[endianness(var)]
    pub loop_count: i32,
    pub bypass_listener_range_check: bool,
    #[endianness(le)]
    pub server_sound_handle: Option<i64>,
    #[endianness(le)]
    pub playback_position_seconds: Option<f32>,
    #[endianness(le)]
    pub start_delay_seconds: Option<f32>,
    pub attach_to_actor_runtime_id: Option<V::ActorRuntimeID>,
}
