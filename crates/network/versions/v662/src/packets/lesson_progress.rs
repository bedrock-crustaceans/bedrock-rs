use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::{packet, ProtoCodec};

#[packet(id = 183, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct LessonProgressPacket<V: ProtoVersion> {
    pub lesson_action: V::LessonAction,
    #[endianness(var)]
    pub score: i32,
    pub activity_id: String,
}
