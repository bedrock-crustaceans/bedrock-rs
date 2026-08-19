use crate::ProtoVersion;
use bedrock_macros::{ProtoCodec, packet};
use bedrock_protocol_core::error::ProtoCodecError;
use bedrock_protocol_core::{ProtoCodec, ProtoCodecLE};
use std::io::{Read, Write};
use std::mem::size_of;
use varint_rs::{VarintReader, VarintWriter};

#[packet(id = 108)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetScorePacket<V: ProtoVersion> {
    pub score_info: Vec<ScorePacketEntry<V>>,
}

#[derive(Clone, Debug)]
pub enum ScorePacketEntry<V: ProtoVersion> {
    Remove {
        scoreboard_id: V::ScoreboardId,
        objective_name: Option<Option<String>>,
    },
    ChangePlayer {
        scoreboard_id: V::ScoreboardId,
        objective_name: String,
        score_value: i32,
        player_unique_id: i64,
    },
    ChangeEntity {
        scoreboard_id: V::ScoreboardId,
        objective_name: String,
        score_value: i32,
        actor_id: i64,
    },
    ChangeFakePlayer {
        scoreboard_id: V::ScoreboardId,
        objective_name: String,
        score_value: i32,
        fake_player_name: String,
    },
}

impl<V: ProtoVersion> ProtoCodec for ScorePacketEntry<V> {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        match self {
            Self::Remove {
                scoreboard_id,
                objective_name,
            } => {
                stream.write_u32_varint(0)?;
                <String as ProtoCodec>::serialize(&String::from("remove"), stream)?;
                <V::ScoreboardId as ProtoCodec>::serialize(scoreboard_id, stream)?;
                <Option<Option<String>> as ProtoCodec>::serialize(objective_name, stream)?;
            }
            Self::ChangePlayer {
                scoreboard_id,
                objective_name,
                score_value,
                player_unique_id,
            } => {
                stream.write_u32_varint(1)?;
                <String as ProtoCodec>::serialize(&String::from("changeplayer"), stream)?;
                <V::ScoreboardId as ProtoCodec>::serialize(scoreboard_id, stream)?;
                <String as ProtoCodec>::serialize(objective_name, stream)?;
                <i32 as ProtoCodecLE>::serialize(score_value, stream)?;
                stream.write_i64_varint(*player_unique_id)?;
            }
            Self::ChangeEntity {
                scoreboard_id,
                objective_name,
                score_value,
                actor_id,
            } => {
                stream.write_u32_varint(2)?;
                <String as ProtoCodec>::serialize(&String::from("changeentity"), stream)?;
                <V::ScoreboardId as ProtoCodec>::serialize(scoreboard_id, stream)?;
                <String as ProtoCodec>::serialize(objective_name, stream)?;
                <i32 as ProtoCodecLE>::serialize(score_value, stream)?;
                stream.write_i64_varint(*actor_id)?;
            }
            Self::ChangeFakePlayer {
                scoreboard_id,
                objective_name,
                score_value,
                fake_player_name,
            } => {
                stream.write_u32_varint(3)?;
                <String as ProtoCodec>::serialize(&String::from("changefakeplayer"), stream)?;
                <V::ScoreboardId as ProtoCodec>::serialize(scoreboard_id, stream)?;
                <String as ProtoCodec>::serialize(objective_name, stream)?;
                <i32 as ProtoCodecLE>::serialize(score_value, stream)?;
                <String as ProtoCodec>::serialize(fake_player_name, stream)?;
            }
        }

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let action = stream.read_u32_varint()?;
        let _action_id = <String as ProtoCodec>::deserialize(stream)?;

        Ok(match action {
            0 => Self::Remove {
                scoreboard_id: <V::ScoreboardId as ProtoCodec>::deserialize(stream)?,
                objective_name: <Option<Option<String>> as ProtoCodec>::deserialize(stream)?,
            },
            1 => Self::ChangePlayer {
                scoreboard_id: <V::ScoreboardId as ProtoCodec>::deserialize(stream)?,
                objective_name: <String as ProtoCodec>::deserialize(stream)?,
                score_value: <i32 as ProtoCodecLE>::deserialize(stream)?,
                player_unique_id: stream.read_i64_varint()?,
            },
            2 => Self::ChangeEntity {
                scoreboard_id: <V::ScoreboardId as ProtoCodec>::deserialize(stream)?,
                objective_name: <String as ProtoCodec>::deserialize(stream)?,
                score_value: <i32 as ProtoCodecLE>::deserialize(stream)?,
                actor_id: stream.read_i64_varint()?,
            },
            3 => Self::ChangeFakePlayer {
                scoreboard_id: <V::ScoreboardId as ProtoCodec>::deserialize(stream)?,
                objective_name: <String as ProtoCodec>::deserialize(stream)?,
                score_value: <i32 as ProtoCodecLE>::deserialize(stream)?,
                fake_player_name: <String as ProtoCodec>::deserialize(stream)?,
            },
            other => {
                return Err(ProtoCodecError::InvalidEnumID(
                    other.to_string(),
                    "ScorePacketEntry",
                ));
            }
        })
    }

    fn size_hint(&self) -> usize {
        let header = size_of::<u32>() + size_of::<u32>() + 16;

        header
            + match self {
                Self::Remove {
                    scoreboard_id,
                    objective_name,
                } => scoreboard_id.size_hint() + objective_name.size_hint(),
                Self::ChangePlayer {
                    scoreboard_id,
                    objective_name,
                    ..
                }
                | Self::ChangeEntity {
                    scoreboard_id,
                    objective_name,
                    ..
                } => {
                    scoreboard_id.size_hint()
                        + objective_name.size_hint()
                        + size_of::<i32>()
                        + size_of::<i64>()
                }
                Self::ChangeFakePlayer {
                    scoreboard_id,
                    objective_name,
                    fake_player_name,
                    ..
                } => {
                    scoreboard_id.size_hint()
                        + objective_name.size_hint()
                        + size_of::<i32>()
                        + fake_player_name.size_hint()
                }
            }
    }
}
