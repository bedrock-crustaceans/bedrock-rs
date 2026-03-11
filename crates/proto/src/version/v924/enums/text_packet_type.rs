use bedrockrs_macros::ProtoCodec;
use bedrockrs_proto_core::ProtoCodec;
use bedrockrs_proto_core::error::ProtoCodecError;
use std::io::Cursor;

// ########## PROTOCOL QUIRK / COMPATIBILITY HACK ##########
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
    Raw(String) = (0 << 8) | 0,
    Chat {
        player_name: String,
        message: String,
    } = (1 << 8) | 1,
    Translate {
        message: String,
        parameter_list: Vec<String>,
    } = (2 << 8) | 2,
    Popup {
        message: String,
        parameter_list: Vec<String>,
    } = (2 << 8) | 3,
    JukeboxPopup {
        message: String,
        parameter_list: Vec<String>,
    } = (2 << 8) | 4,
    Tip(String) = (0 << 8) | 5,
    SystemMessage(String) = (0 << 8) | 6,
    Whisper {
        player_name: String,
        message: String,
    } = (1 << 8) | 7,
    Announcement {
        player_name: String,
        message: String,
    } = (1 << 8) | 8,
    TextObjectWhisper(String) = (0 << 8) | 9,
    TextObject(String) = (0 << 8) | 10,
    TextObjectAnnouncement(String) = (0 << 8) | 11,
}
