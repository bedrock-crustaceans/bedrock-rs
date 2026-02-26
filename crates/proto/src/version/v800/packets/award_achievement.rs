use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 309)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct AwardAchievementPacket {
    #[endianness(le)]
    achievement_id: i32,
}