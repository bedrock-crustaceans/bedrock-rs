use crate::ProtoVersion;
use bedrock_macros::{ProtoCodec, packet};
use bedrock_protocol_core::error::ProtoCodecError;
use bedrock_protocol_core::{ProtoCodec, ProtoCodecVAR};
use std::io::{Read, Write};

#[packet(id = 174)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SubChunkPacket<V: ProtoVersion> {
    pub cache_enabled: bool,
    #[endianness(var)]
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

#[derive(ProtoCodec, Clone, Debug)]
pub struct SubChunkDataEntry<V: ProtoVersion> {
    pub sub_chunk_pos_offset: V::SubChunkPosOffset,
    pub sub_chunk_request_result: SubChunkRequestResult,
    pub serialized_sub_chunk: Option<Vec<u8>>,
    pub height_map_data_type: HeightMapDataType,
    pub height_map_data: Option<HeightMapData>,
    pub render_height_map_data_type: HeightMapDataType,
    pub render_height_map_data: Option<HeightMapData>,
    #[endianness(le)]
    pub blob_id: Option<u64>,
}

/// Heights are sent as fixed length runs, each prefixed by its own length.
#[derive(Clone, Debug)]
pub struct HeightMapData {
    pub runs: [[i8; HeightMapData::RUN_LENGTH]; HeightMapData::RUN_COUNT],
}

impl HeightMapData {
    pub const RUN_LENGTH: usize = 16;
    pub const RUN_COUNT: usize = 17;
}

impl ProtoCodec for HeightMapData {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        let run_length: u32 = Self::RUN_LENGTH as u32;

        for run in &self.runs {
            <u32 as ProtoCodecVAR>::serialize(&run_length, stream)?;

            for height in run {
                <i8 as ProtoCodec>::serialize(height, stream)?;
            }
        }

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let mut runs = [[0i8; Self::RUN_LENGTH]; Self::RUN_COUNT];

        for run in runs.iter_mut() {
            let run_length = <u32 as ProtoCodecVAR>::deserialize(stream)?;

            if run_length as usize != Self::RUN_LENGTH {
                return Err(ProtoCodecError::FormatMismatch(
                    "unexpected height map run length",
                ));
            }

            for height in run.iter_mut() {
                *height = <i8 as ProtoCodec>::deserialize(stream)?;
            }
        }

        Ok(Self { runs })
    }

    fn size_hint(&self) -> usize {
        Self::RUN_COUNT
            * (<u32 as ProtoCodecVAR>::size_hint(&(Self::RUN_LENGTH as u32)) + Self::RUN_LENGTH)
    }
}
