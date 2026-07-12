use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};
use bedrock_protocol_core::error::ProtoCodecError;
use bedrock_protocol_core::{ProtoCodec, ProtoCodecVAR};
use std::io::{Read, Write};
use std::mem::size_of;
use uuid::Uuid;

#[packet(id = 63)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerListPacket<V: ProtoVersion> {
    pub entries: Vec<PlayerListEntry<V>>,
}

#[derive(Clone, Debug)]
pub enum PlayerListEntry<V: ProtoVersion> {
    Remove { uuid: Uuid },
    Add(AddPlayerListEntry<V>),
}

impl<V: ProtoVersion> ProtoCodec for PlayerListEntry<V> {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        match self {
            PlayerListEntry::Remove { uuid } => {
                <u32 as ProtoCodecVAR>::serialize(&0, stream)?;
                u8::serialize(&1, stream)?;
                uuid.serialize(stream)?;
            }
            PlayerListEntry::Add(entry) => {
                <u32 as ProtoCodecVAR>::serialize(&1, stream)?;
                u8::serialize(&0, stream)?;
                entry.serialize(stream)?;
            }
        }

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let packet_type = <u32 as ProtoCodecVAR>::deserialize(stream)?;
        u8::deserialize(stream)?;

        Ok(match packet_type {
            0 => PlayerListEntry::Remove {
                uuid: Uuid::deserialize(stream)?,
            },
            1 => PlayerListEntry::Add(AddPlayerListEntry::deserialize(stream)?),
            other => {
                return Err(ProtoCodecError::InvalidEnumID(
                    format!("{other}"),
                    "PlayerListEntry",
                ));
            }
        })
    }

    fn size_hint(&self) -> usize {
        size_of::<u32>()
            + size_of::<u8>()
            + match self {
                PlayerListEntry::Remove { uuid } => uuid.size_hint(),
                PlayerListEntry::Add(entry) => entry.size_hint(),
            }
    }
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct AddPlayerListEntry<V: ProtoVersion> {
    pub uuid: Uuid,
    pub target_actor_id: V::ActorUniqueID,
    pub player_name: String,
    pub xbl_xuid: String,
    pub platform_chat_id: String,
    pub build_platform: V::BuildPlatform,
    pub serialized_skin: V::SerializedSkin,
    pub is_teacher: bool,
    pub is_host: bool,
    pub is_sub_client: bool,
    pub color: V::Color,
}
