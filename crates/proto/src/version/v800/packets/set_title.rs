use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 88)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetTitlePacket {
    pub title_type: TitleType,
    pub title_text: String,
    #[endianness(var)]
    pub fade_in_time: i32,
    #[endianness(var)]
    pub stay_time: i32,
    #[endianness(var)]
    pub fade_out_time: i32,
    pub xuid: String,
    pub platform_online_id: String,
    pub filtered_title_text: String,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i32)]
#[enum_endianness(var)]
#[repr(i32)]
pub enum TitleType {
    Clear = 0,
    Reset = 1,
    Title = 2,
    Subtitle = 3,
    Actionbar = 4,
    Times = 5,
    TitleTextObject = 6,
    SubtitleTextObject = 7,
    ActionbarTextObject = 8,
}
