use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum TextPacketType {
    Raw(String) = 0,
    Chat {
        player_name: String,
        message: String,
    } = 1,
    Translate {
        message: String,
        #[vec_repr(u32)]
        #[vec_endianness(var)]
        parameter_list: Vec<String>,
    } = 2,
    Popup {
        message: String,
        #[vec_repr(u32)]
        #[vec_endianness(var)]
        parameter_list: Vec<String>,
    } = 3,
    JukeboxPopup {
        message: String,
        #[vec_repr(u32)]
        #[vec_endianness(var)]
        parameter_list: Vec<String>,
    } = 4,
    Tip(String) = 5,
    SystemMessage(String) = 6,
    Whisper {
        player_name: String,
        message: String,
    } = 7,
    Announcement {
        player_name: String,
        message: String,
    } = 8,
    TextObjectWhisper(String) = 9,
    TextObject(String) = 10,
    TextObjectAnnouncement(String) = 11,
}