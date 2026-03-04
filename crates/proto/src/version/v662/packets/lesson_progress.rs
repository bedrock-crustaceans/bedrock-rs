use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 183)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct LessonProgressPacket<V: ProtoVersion> {
    pub lesson_action: V::LessonAction,
    #[endianness(var)]
    pub score: i32,
    pub activity_id: String,
}