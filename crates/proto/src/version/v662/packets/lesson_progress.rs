use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 183)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct LessonProgressPacket<V: ProtoVersion> {
    pub lesson_action: V::LessonAction,
    #[endianness(var)]
    pub score: i32,
    pub activity_id: String,
}
