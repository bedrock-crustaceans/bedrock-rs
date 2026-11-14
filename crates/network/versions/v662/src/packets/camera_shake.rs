use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::{packet, ProtoCodec};

#[packet(id = 159, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraShakePacket<V: ProtoVersion> {
    #[endianness(le)]
    pub intensity: f32,
    #[endianness(le)]
    pub seconds: f32,
    pub shake_type: V::CameraShakeType,
    pub shake_action: V::CameraShakeAction,
}
