use crate::version::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, packet};
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::{ProtoCodec, ProtoCodecLE, ProtoCodecVAR};
use std::cmp::PartialEq;
use std::io::{Read, Write};
use std::mem::size_of;

#[packet(id = 174)]
#[derive(Clone, Debug)]
pub struct SubChunkPacket<V: ProtoVersion> {
    pub cache_enabled: bool,
    pub dimension_type: i32,
    pub center_pos: V::SubChunkPos,
    pub sub_chunk_data: Vec<SubChunkDataEntry<V>>,
}

#[derive(ProtoCodec, Clone, Debug, PartialEq)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum HeightMapDataType {
    NoData = 0,
    HasData = 1,
    AllTooHigh = 2,
    AllTooLow = 3,
}

#[derive(ProtoCodec, Clone, Debug, PartialEq)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum SubChunkRequestResult {
    Undefined = 0,
    Success = 1,
    LevelChunkDoesntExist = 2,
    WrongDimension = 3,
    PlayerDoesntExist = 4,
    IndexOutOfBounds = 5,
    SuccessAllAir = 6,
}

#[derive(Clone, Debug)]
pub struct SubChunkDataEntry<V: ProtoVersion> {
    pub sub_chunk_pos_offset: V::SubChunkPosOffset,
    pub sub_chunk_request_result: SubChunkRequestResult,
    pub serialized_sub_chunk: Option<String>, // If sub_chunk_request_result != SuccessAllAir, or cache_enabled == false
    pub height_map_data_type: HeightMapDataType,
    pub height_map_data: Option<[[i8; 16]; 16]>, // If height_map_data_type == HasData (vec sizes are i8)
    pub render_height_map_data_type: HeightMapDataType,
    pub render_height_map_data: Option<[[i8; 16]; 16]>, // If height_map_data_type == HasData (vec sizes are i8)
    pub blob_id: Option<u64>,                           // If cache_enabled == true
}

impl<V: ProtoVersion> ProtoCodec for SubChunkPacket<V> {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        self.cache_enabled.serialize(stream)?;
        <i32 as ProtoCodecVAR>::serialize(&self.dimension_type, stream)?;
        self.center_pos.serialize(stream)?;
        <u32 as ProtoCodecLE>::serialize(&self.sub_chunk_data.len().try_into()?, stream)?;
        for i in &self.sub_chunk_data {
            i.sub_chunk_pos_offset.serialize(stream)?;
            i.sub_chunk_request_result.serialize(stream)?;
            if i.sub_chunk_request_result != SubChunkRequestResult::SuccessAllAir
                || !self.cache_enabled
            {
                i.serialized_sub_chunk.as_ref().unwrap().serialize(stream)?;
            }
            i.height_map_data_type.serialize(stream)?;
            if i.height_map_data_type == HeightMapDataType::HasData {
                let height_map = i.height_map_data.as_ref().unwrap();
                for x in height_map {
                    for y in x {
                        y.serialize(stream)?;
                    }
                }
            }
            i.render_height_map_data_type.serialize(stream)?;
            if i.render_height_map_data_type == HeightMapDataType::HasData {
                let render_height_map = i.render_height_map_data.as_ref().unwrap();
                for x in render_height_map {
                    for y in x {
                        y.serialize(stream)?;
                    }
                }
            }
            if self.cache_enabled {
                <u64 as ProtoCodecLE>::serialize(i.blob_id.as_ref().unwrap(), stream)?;
            }
        }

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let cache_enabled = bool::deserialize(stream)?;
        let dimension_type = ProtoCodecVAR::deserialize(stream)?;
        let center_pos = V::SubChunkPos::deserialize(stream)?;
        let sub_chunk_data = {
            let len = <u32 as ProtoCodecLE>::deserialize(stream)?;
            let mut vec = Vec::with_capacity(len.try_into()?);
            for _ in 0..len {
                let sub_chunk_pos_offset = V::SubChunkPosOffset::deserialize(stream)?;
                let sub_chunk_request_result = SubChunkRequestResult::deserialize(stream)?;
                let serialized_sub_chunk = match sub_chunk_request_result
                    != SubChunkRequestResult::SuccessAllAir
                    || !cache_enabled
                {
                    true => Some(String::deserialize(stream)?),
                    false => None,
                };
                let height_map_data_type = HeightMapDataType::deserialize(stream)?;
                let height_map_data = match height_map_data_type == HeightMapDataType::HasData {
                    true => {
                        let mut height_map: [[i8; 16]; 16] = [[0; 16]; 16];
                        for row in &mut height_map {
                            for value in row {
                                *value = i8::deserialize(stream)?;
                            }
                        }

                        Some(height_map)
                    }
                    false => None,
                };
                let render_height_map_data_type = HeightMapDataType::deserialize(stream)?;
                let render_height_map_data =
                    match render_height_map_data_type == HeightMapDataType::HasData {
                        true => {
                            let mut render_height_map: [[i8; 16]; 16] = [[0; 16]; 16];
                            for row in &mut render_height_map {
                                for value in row {
                                    *value = i8::deserialize(stream)?;
                                }
                            }

                            Some(render_height_map)
                        }
                        false => None,
                    };
                let blob_id = match cache_enabled {
                    true => Some(ProtoCodecLE::deserialize(stream)?),
                    false => None,
                };

                vec.push(SubChunkDataEntry {
                    sub_chunk_pos_offset,
                    sub_chunk_request_result,
                    serialized_sub_chunk,
                    height_map_data_type,
                    height_map_data,
                    render_height_map_data_type,
                    render_height_map_data,
                    blob_id,
                })
            }
            vec
        };

        Ok(Self {
            cache_enabled,
            dimension_type,
            center_pos,
            sub_chunk_data,
        })
    }

    fn size_hint(&self) -> usize {
        self.cache_enabled.size_hint()
            + <i32 as ProtoCodecVAR>::size_hint(&self.dimension_type)
            + self.center_pos.size_hint()
            + size_of::<u32>()
            + self
                .sub_chunk_data
                .iter()
                .map(|i| {
                    i.sub_chunk_pos_offset.size_hint()
                        + i.sub_chunk_request_result.size_hint()
                        + match i.sub_chunk_request_result != SubChunkRequestResult::SuccessAllAir
                            || !self.cache_enabled
                        {
                            true => i.serialized_sub_chunk.as_ref().unwrap().size_hint(),
                            false => 0,
                        }
                        + i.height_map_data_type.size_hint()
                        + match i.height_map_data_type == HeightMapDataType::HasData {
                            true => {
                                let height_map = i.height_map_data.as_ref().unwrap();
                                height_map.len() * height_map[0].len() * size_of::<i8>()
                            }
                            false => 0,
                        }
                        + i.render_height_map_data_type.size_hint()
                        + match i.render_height_map_data_type == HeightMapDataType::HasData {
                            true => {
                                let render_height_map = i.render_height_map_data.as_ref().unwrap();
                                render_height_map.len()
                                    * render_height_map[0].len()
                                    * size_of::<i8>()
                            }
                            false => 0,
                        }
                        + match self.cache_enabled {
                            true => size_of::<u64>(),
                            false => 0,
                        }
                })
                .sum::<usize>()
    }
}

// VERIFY: ProtoCodec impl
