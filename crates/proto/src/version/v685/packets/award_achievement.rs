use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 309)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct AwardAchievementPacket {
    #[endianness(le)]
    achievement_id: i32,
}
