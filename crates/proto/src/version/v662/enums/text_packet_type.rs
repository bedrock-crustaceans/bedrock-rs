use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u8)]
#[repr(u8)]
pub enum TextPacketType {
    Raw(String) = 0,
    Chat {
        player_name: String,
        message: String,
    } = 1,
    Translate {
        message: String,
        parameter_list: Vec<String>,
    } = 2,
    Popup {
        message: String,
        parameter_list: Vec<String>,
    } = 3,
    JukeboxPopup {
        message: String,
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
