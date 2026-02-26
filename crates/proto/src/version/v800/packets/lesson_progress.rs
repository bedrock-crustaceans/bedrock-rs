use super::super::enums::LessonAction;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 183)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct LessonProgressPacket {
    pub lesson_action: LessonAction,
    #[endianness(var)]
    pub score: i32,
    pub activity_id: String,
}