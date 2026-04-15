use std::io::{Read, Write};
use uuid::Uuid;
use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::ProtoCodec;

#[packet(id = 63)]
#[derive(Clone, Debug)]
#[repr(u8)]
pub enum PlayerListPacket<V: ProtoVersion> {
    Add {
        add_player_list: Vec<AddPlayerListEntry<V>>,
        is_trusted_skin: Vec<bool>,
    } = 0,
    Remove {
        remove_player_list: Vec<Uuid>,
    } = 1,
}

impl <V: ProtoVersion> ProtoCodec for PlayerListPacket<V> {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        match self {
            PlayerListPacket::Add { add_player_list, is_trusted_skin } => {
                u8::serialize(&0, stream)?;
                <Vec<AddPlayerListEntry<V>>>::serialize(add_player_list, stream)?;
                for i in 0..add_player_list.len() {
                    bool::serialize(is_trusted_skin.get(i).unwrap_or(&false), stream)?;
                }
            }
            PlayerListPacket::Remove { remove_player_list } => {
                u8::serialize(&1, stream)?;
                <Vec<Uuid>>::serialize(remove_player_list, stream)?;
            }
        }

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        Ok(match u8::deserialize(stream)? {
            0 => {
                let add_player_list = Vec::<AddPlayerListEntry<V>>::deserialize(stream)?;
                let mut is_trusted_skin = Vec::with_capacity(add_player_list.len());

                for _ in 0..add_player_list.len() {
                    is_trusted_skin.push(bool::deserialize(stream)?);
                }

                PlayerListPacket::Add {
                    add_player_list,
                    is_trusted_skin,
                }
            },
            1 => PlayerListPacket::Remove {
                remove_player_list: <Vec<Uuid>>::deserialize(stream)?,
            },
            other => return Err(ProtoCodecError::InvalidEnumID(format!("{other}"), "PlayerListPacket"))
        })
    }

    fn size_hint(&self) -> usize {
        u8::size_hint(&0)
        + match self {
            PlayerListPacket::Add { add_player_list, .. } => {
                add_player_list.size_hint()
                + add_player_list.len() * bool::size_hint(&false)
            },
            PlayerListPacket::Remove { remove_player_list } => {
                remove_player_list.size_hint()
            }
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