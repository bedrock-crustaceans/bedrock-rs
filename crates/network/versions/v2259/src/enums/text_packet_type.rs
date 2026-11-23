use bedrock_network_macros::ProtoCodec;

// ########## PROTOCOL HACK ##########
//
// Vanilla writes two separate bytes:
//
//     category:      u8
//     discriminant:  u8
//
// Each enum variant belongs to a fixed category, so instead of manually
// serializing both values, we encode them into a single big-endian u16:
//
//     combined = (category << 8) | discriminant
//
// Because we serialize the u16 in big-endian, the bytes appear on the wire as:
//
//     [ category ][ discriminant ]
//
// This exactly matches vanilla’s layout while letting the derive macro
// handle the discriminant automatically.

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u16)]
#[enum_endianness(be)]
#[repr(u16)]
pub enum TextPacketType {
    Raw(String) = 0,
    Chat {
        player_name: String,
        message: String,
    } = 257, // category 1
    Translate {
        message: String,
        parameter_list: Vec<String>,
        queue: bool,
    } = 514, // category 2
    Popup {
        message: String,
        parameter_list: Vec<String>,
        queue: bool,
    } = 515, // category 2
    JukeboxPopup {
        message: String,
        parameter_list: Vec<String>,
        queue: bool,
    } = 516, // category 2
    Tip(String) = 5,
    SystemMessage(String) = 6,
    Whisper {
        player_name: String,
        message: String,
    } = 263, // category 1
    Announcement {
        player_name: String,
        message: String,
    } = 264, // category 1
    TextObjectWhisper(String) = 9,
    TextObject(String) = 10,
    TextObjectAnnouncement(String) = 11,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_network_core::ProtoCodec;
    use std::io::Cursor;

    #[test]
    fn jukebox_popup_ends_with_the_queue_flag() {
        let bytes = [0x02, 0x04, 0x01, b'm', 0x01, 0x01, b'p', 0x01];
        let value = TextPacketType::deserialize(&mut Cursor::new(&bytes[..])).unwrap();
        let mut encoded = Vec::new();
        value.serialize(&mut encoded).unwrap();
        assert_eq!(encoded, bytes, "encoding differs from the input");
        assert!(matches!(value, TextPacketType::JukeboxPopup { queue: true, .. }));
    }
}
