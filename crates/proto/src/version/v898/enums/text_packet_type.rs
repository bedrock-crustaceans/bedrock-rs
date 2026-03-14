use bedrockrs_proto_core::ProtoCodec;
use bedrockrs_proto_core::error::ProtoCodecError;
use std::io::{Read, Write};

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
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
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

        u8::serialize(&category, stream)?;

        match category {
            0 => {
                String::serialize(&"raw".to_string(), stream)?;
                String::serialize(&"tip".to_string(), stream)?;
                String::serialize(&"systemMessage".to_string(), stream)?;
                String::serialize(&"textObjectWhisper".to_string(), stream)?;
                String::serialize(&"textObjectAnnouncement".to_string(), stream)?;
                String::serialize(&"textObject".to_string(), stream)?;
            }
            1 => {
                String::serialize(&"chat".to_string(), stream)?;
                String::serialize(&"whisper".to_string(), stream)?;
                String::serialize(&"announcement".to_string(), stream)?;
            }
            _ => {
                String::serialize(&"translate".to_string(), stream)?;
                String::serialize(&"popup".to_string(), stream)?;
                String::serialize(&"jukeboxPopup".to_string(), stream)?;
            }
        }

        match self {
            TextPacketType::Raw(message) => {
                u8::serialize(&0, stream)?;
                message.serialize(stream)?;
            }
            TextPacketType::Chat {
                player_name,
                message,
            } => {
                u8::serialize(&1, stream)?;
                player_name.serialize(stream)?;
                message.serialize(stream)?;
            }
            TextPacketType::Translate {
                message,
                parameter_list,
            } => {
                u8::serialize(&2, stream)?;
                message.serialize(stream)?;
                parameter_list.serialize(stream)?;
            }
            TextPacketType::Popup {
                message,
                parameter_list,
            } => {
                u8::serialize(&3, stream)?;
                message.serialize(stream)?;
                parameter_list.serialize(stream)?;
            }
            TextPacketType::JukeboxPopup {
                message,
                parameter_list,
            } => {
                u8::serialize(&4, stream)?;
                message.serialize(stream)?;
                parameter_list.serialize(stream)?;
            }
            TextPacketType::Tip(message) => {
                u8::serialize(&5, stream)?;
                message.serialize(stream)?;
            }
            TextPacketType::SystemMessage(message) => {
                u8::serialize(&6, stream)?;
                message.serialize(stream)?;
            }
            TextPacketType::Whisper {
                player_name,
                message,
            } => {
                u8::serialize(&7, stream)?;
                player_name.serialize(stream)?;
                message.serialize(stream)?;
            }
            TextPacketType::Announcement {
                player_name,
                message,
            } => {
                u8::serialize(&8, stream)?;
                player_name.serialize(stream)?;
                message.serialize(stream)?;
            }
            TextPacketType::TextObjectWhisper(message) => {
                u8::serialize(&9, stream)?;
                message.serialize(stream)?;
            }
            TextPacketType::TextObject(message) => {
                u8::serialize(&10, stream)?;
                message.serialize(stream)?;
            }
            TextPacketType::TextObjectAnnouncement(message) => {
                u8::serialize(&11, stream)?;
                message.serialize(stream)?;
            }
        }

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let category = u8::deserialize(stream)?;

        match category {
            0 => {
                String::deserialize(stream)?;
                String::deserialize(stream)?;
                String::deserialize(stream)?;
                String::deserialize(stream)?;
                String::deserialize(stream)?;
                String::deserialize(stream)?;
            }
            _ => {
                String::deserialize(stream)?;
                String::deserialize(stream)?;
                String::deserialize(stream)?;
            }
        }

        let discriminant = u8::deserialize(stream)?;

        match discriminant {
            0 => Ok(Self::Raw(String::deserialize(stream)?)),
            1 => Ok(Self::Chat {
                player_name: String::deserialize(stream)?,
                message: String::deserialize(stream)?,
            }),
            2 => Ok(Self::Translate {
                message: String::deserialize(stream)?,
                parameter_list: Vec::deserialize(stream)?,
            }),
            3 => Ok(Self::Popup {
                message: String::deserialize(stream)?,
                parameter_list: Vec::deserialize(stream)?,
            }),
            4 => Ok(Self::JukeboxPopup {
                message: String::deserialize(stream)?,
                parameter_list: Vec::deserialize(stream)?,
            }),
            5 => Ok(Self::Tip(String::deserialize(stream)?)),
            6 => Ok(Self::SystemMessage(String::deserialize(stream)?)),
            7 => Ok(Self::Whisper {
                player_name: String::deserialize(stream)?,
                message: String::deserialize(stream)?,
            }),
            8 => Ok(Self::Announcement {
                player_name: String::deserialize(stream)?,
                message: String::deserialize(stream)?,
            }),
            9 => Ok(Self::TextObjectWhisper(String::deserialize(stream)?)),
            10 => Ok(Self::TextObject(String::deserialize(stream)?)),
            11 => Ok(Self::TextObjectAnnouncement(String::deserialize(stream)?)),
            invalid => Err(ProtoCodecError::InvalidEnumID(
                format!("{invalid}"),
                "TextPacketType",
            )),
        }
    }

    fn size_hint(&self) -> usize {
        1 // TODO: can't be bothered doing this
    }
}
