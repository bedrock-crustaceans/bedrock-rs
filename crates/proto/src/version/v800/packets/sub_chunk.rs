use super::super::types::{SubChunkPos, SubChunkPosOffset};
use bedrockrs_macros::{gamepacket, ProtoCodec};
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::{ProtoCodec, ProtoCodecLE, ProtoCodecVAR};
use std::cmp::PartialEq;
use std::io::Cursor;
use std::mem::size_of;

#[gamepacket(id = 174)]
#[derive(Clone, Debug)]
pub struct SubChunkPacket {
    pub cache_enabled: bool,
    pub dimension_type: i32,
    pub center_pos: SubChunkPos,
    pub sub_chunk_data: Vec<SubChunkDataEntry>,
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
pub struct SubChunkDataEntry {
    pub sub_chunk_pos_offset: SubChunkPosOffset,
    pub sub_chunk_request_result: SubChunkRequestResult,
    pub serialized_sub_chunk: Option<String>, // If sub_chunk_request_result == SuccessAllAir, or cache_enabled == false
    pub height_map_data_type: HeightMapDataType,
    pub sub_chunk_height_map: Option<[[i8; 16]; 16]>, // If height_map_data_type == HasData (vec sizes are i8)
    pub blob_id: Option<u64>,                         // If cache_enabled == true
}

impl ProtoCodec for SubChunkPacket {
    fn proto_serialize(&self, stream: &mut Vec<u8>) -> Result<(), ProtoCodecError> {
        self.cache_enabled.proto_serialize(stream)?;
        <i32 as ProtoCodecVAR>::proto_serialize(&self.dimension_type, stream)?;
        self.center_pos.proto_serialize(stream)?;
        <u32 as ProtoCodecLE>::proto_serialize(&self.sub_chunk_data.len().try_into()?, stream)?;
        for i in &self.sub_chunk_data {
            i.sub_chunk_pos_offset.proto_serialize(stream)?;
            i.sub_chunk_request_result.proto_serialize(stream)?;
            if i.sub_chunk_request_result == SubChunkRequestResult::SuccessAllAir
                || !self.cache_enabled
            {
                i.serialized_sub_chunk
                    .as_ref()
                    .unwrap()
                    .proto_serialize(stream)?;
            }
            i.height_map_data_type.proto_serialize(stream)?;
            if i.height_map_data_type == HeightMapDataType::HasData {
                let height_map = i.sub_chunk_height_map.as_ref().unwrap();
                for x in height_map {
                    for y in x {
                        y.proto_serialize(stream)?;
                    }
                }
            }
            if self.cache_enabled {
                <u64 as ProtoCodecLE>::proto_serialize(&i.blob_id.as_ref().unwrap(), stream)?;
            }
        }

        Ok(())
    }

    fn proto_deserialize(stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError> {
        let cache_enabled = bool::proto_deserialize(stream)?;
        let dimension_type = ProtoCodecVAR::proto_deserialize(stream)?;
        let center_pos = SubChunkPos::proto_deserialize(stream)?;
        let sub_chunk_data = {
            let len = <u32 as ProtoCodecLE>::proto_deserialize(stream)?;
            let mut vec = Vec::with_capacity(len.try_into()?);
            for _ in 0..len {
                let sub_chunk_pos_offset = SubChunkPosOffset::proto_deserialize(stream)?;
                let sub_chunk_request_result = SubChunkRequestResult::proto_deserialize(stream)?;
                let serialized_sub_chunk = match sub_chunk_request_result
                    == SubChunkRequestResult::SuccessAllAir
                    || cache_enabled
                {
                    true => Some(String::proto_deserialize(stream)?),
                    false => None,
                };
                let height_map_data_type = HeightMapDataType::proto_deserialize(stream)?;
                let sub_chunk_height_map =
                    match height_map_data_type == HeightMapDataType::HasData {
                        true => {
                            let mut height_map: [[i8; 16]; 16] = [[0; 16]; 16];
                            for x in 0..16 {
                                for y in 0..16 {
                                    height_map[x][y] = i8::proto_deserialize(stream)?;
                                }
                            }

                            Some(height_map)
                        }
                        false => None,
                    };
                let blob_id = match cache_enabled {
                    true => Some(ProtoCodecLE::proto_deserialize(stream)?),
                    false => None,
                };

                vec.push(SubChunkDataEntry {
                    sub_chunk_pos_offset,
                    sub_chunk_request_result,
                    serialized_sub_chunk,
                    height_map_data_type,
                    sub_chunk_height_map,
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

    fn get_size_prediction(&self) -> usize {
        self.cache_enabled.get_size_prediction()
            + <i32 as ProtoCodecVAR>::get_size_prediction(&self.dimension_type)
            + self.center_pos.get_size_prediction()
            + size_of::<u32>()
            + self
            .sub_chunk_data
            .iter()
            .map(|i| {
                i.sub_chunk_pos_offset.get_size_prediction()
                    + i.sub_chunk_request_result.get_size_prediction()
                    + match i.sub_chunk_request_result == SubChunkRequestResult::SuccessAllAir
                    || self.cache_enabled
                {
                    true => i
                        .serialized_sub_chunk
                        .as_ref()
                        .unwrap()
                        .get_size_prediction(),
                    false => 0,
                }
                    + i.height_map_data_type.get_size_prediction()
                    + match i.height_map_data_type == HeightMapDataType::HasData {
                    true => {
                        let height_map = i.sub_chunk_height_map.as_ref().unwrap();
                        height_map.len() * height_map[0].len() * size_of::<i8>()
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
