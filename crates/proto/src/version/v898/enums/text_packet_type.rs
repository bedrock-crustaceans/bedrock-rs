use bedrockrs_proto_core::ProtoCodec;
use bedrockrs_proto_core::error::ProtoCodecError;
use std::io::Cursor;

#[derive(Clone, Debug)]
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

impl ProtoCodec for TextPacketType {
    fn proto_serialize(&self, stream: &mut Vec<u8>) -> Result<(), ProtoCodecError> {
        let category: u8 = match self {
            TextPacketType::Raw(_)
            | TextPacketType::Tip(_)
            | TextPacketType::SystemMessage(_)
            | TextPacketType::TextObjectWhisper(_)
            | TextPacketType::TextObject(_)
            | TextPacketType::TextObjectAnnouncement(_) => 0,
            TextPacketType::Chat { .. }
            | TextPacketType::Whisper { .. }
            | TextPacketType::Announcement { .. } => 1,
            _ => 2,
        };

        u8::proto_serialize(&category, stream)?;

        match category {
            0 => {
                String::proto_serialize(&"raw".to_string(), stream)?;
                String::proto_serialize(&"tip".to_string(), stream)?;
                String::proto_serialize(&"systemMessage".to_string(), stream)?;
                String::proto_serialize(&"textObjectWhisper".to_string(), stream)?;
                String::proto_serialize(&"textObjectAnnouncement".to_string(), stream)?;
                String::proto_serialize(&"textObject".to_string(), stream)?;
            }
            1 => {
                String::proto_serialize(&"chat".to_string(), stream)?;
                String::proto_serialize(&"whisper".to_string(), stream)?;
                String::proto_serialize(&"announcement".to_string(), stream)?;
            }
            _ => {
                String::proto_serialize(&"translate".to_string(), stream)?;
                String::proto_serialize(&"popup".to_string(), stream)?;
                String::proto_serialize(&"jukeboxPopup".to_string(), stream)?;
            }
        }

        match self {
            TextPacketType::Raw(message) => {
                u8::proto_serialize(&0, stream)?;
                message.proto_serialize(stream)?;
            }
            TextPacketType::Chat {
                player_name,
                message,
            } => {
                u8::proto_serialize(&1, stream)?;
                player_name.proto_serialize(stream)?;
                message.proto_serialize(stream)?;
            }
            TextPacketType::Translate {
                message,
                parameter_list,
            } => {
                u8::proto_serialize(&2, stream)?;
                message.proto_serialize(stream)?;
                parameter_list.proto_serialize(stream)?;
            }
            TextPacketType::Popup {
                message,
                parameter_list,
            } => {
                u8::proto_serialize(&3, stream)?;
                message.proto_serialize(stream)?;
                parameter_list.proto_serialize(stream)?;
            }
            TextPacketType::JukeboxPopup {
                message,
                parameter_list,
            } => {
                u8::proto_serialize(&4, stream)?;
                message.proto_serialize(stream)?;
                parameter_list.proto_serialize(stream)?;
            }
            TextPacketType::Tip(message) => {
                u8::proto_serialize(&5, stream)?;
                message.proto_serialize(stream)?;
            }
            TextPacketType::SystemMessage(message) => {
                u8::proto_serialize(&6, stream)?;
                message.proto_serialize(stream)?;
            }
            TextPacketType::Whisper {
                player_name,
                message,
            } => {
                u8::proto_serialize(&7, stream)?;
                player_name.proto_serialize(stream)?;
                message.proto_serialize(stream)?;
            }
            TextPacketType::Announcement {
                player_name,
                message,
            } => {
                u8::proto_serialize(&8, stream)?;
                player_name.proto_serialize(stream)?;
                message.proto_serialize(stream)?;
            }
            TextPacketType::TextObjectWhisper(message) => {
                u8::proto_serialize(&9, stream)?;
                message.proto_serialize(stream)?;
            }
            TextPacketType::TextObject(message) => {
                u8::proto_serialize(&10, stream)?;
                message.proto_serialize(stream)?;
            }
            TextPacketType::TextObjectAnnouncement(message) => {
                u8::proto_serialize(&11, stream)?;
                message.proto_serialize(stream)?;
            }
        }

        Ok(())
    }

    fn proto_deserialize(stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError> {
        let category = u8::proto_deserialize(stream)?;

        match category {
            0 => {
                String::proto_deserialize(stream)?;
                String::proto_deserialize(stream)?;
                String::proto_deserialize(stream)?;
                String::proto_deserialize(stream)?;
                String::proto_deserialize(stream)?;
                String::proto_deserialize(stream)?;
            }
            _ => {
                String::proto_deserialize(stream)?;
                String::proto_deserialize(stream)?;
                String::proto_deserialize(stream)?;
            }
        }

        let discriminant = u8::proto_deserialize(stream)?;

        match discriminant {
            0 => Ok(Self::Raw(String::proto_deserialize(stream)?)),
            1 => Ok(Self::Chat {
                player_name: String::proto_deserialize(stream)?,
                message: String::proto_deserialize(stream)?,
            }),
            2 => Ok(Self::Translate {
                message: String::proto_deserialize(stream)?,
                parameter_list: Vec::proto_deserialize(stream)?,
            }),
            3 => Ok(Self::Popup {
                message: String::proto_deserialize(stream)?,
                parameter_list: Vec::proto_deserialize(stream)?,
            }),
            4 => Ok(Self::JukeboxPopup {
                message: String::proto_deserialize(stream)?,
                parameter_list: Vec::proto_deserialize(stream)?,
            }),
            5 => Ok(Self::Tip(String::proto_deserialize(stream)?)),
            6 => Ok(Self::SystemMessage(String::proto_deserialize(stream)?)),
            7 => Ok(Self::Whisper {
                player_name: String::proto_deserialize(stream)?,
                message: String::proto_deserialize(stream)?,
            }),
            8 => Ok(Self::Announcement {
                player_name: String::proto_deserialize(stream)?,
                message: String::proto_deserialize(stream)?,
            }),
            9 => Ok(Self::TextObjectWhisper(String::proto_deserialize(stream)?)),
            10 => Ok(Self::TextObject(String::proto_deserialize(stream)?)),
            11 => Ok(Self::TextObjectAnnouncement(String::proto_deserialize(
                stream,
            )?)),
            invalid => Err(ProtoCodecError::InvalidEnumID(
                format!("{invalid}"),
                "TextPacketType",
            )),
        }
    }

    fn get_size_prediction(&self) -> usize {
        1 // TODO: can't be bothered doing this
    }
}
