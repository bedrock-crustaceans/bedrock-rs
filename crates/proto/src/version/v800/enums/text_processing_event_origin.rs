use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i32)]
#[enum_endianness(le)]
#[repr(i32)]
pub enum TextProcessingEventOrigin {
    Unknown = -1,
    ServerChatPublic = 0,
    ServerChatWhisper = 1,
    SignText = 2,
    AnvilText = 3,
    BookAndQuillText = 4,
    CommandBlockText = 5,
    BlockActorDataText = 6,
    JoinEventText = 7,
    LeaveEventText = 8,
    SlashCommandChat = 9,
    CartographyText = 10,
    KickCommand = 11,
    TitleCommand = 12,
    SummonCommand = 13,
    ServerForm = 14,
    Count = 15,
}