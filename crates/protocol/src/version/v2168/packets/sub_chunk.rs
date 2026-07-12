use crate::ProtoVersion;
use bedrock_macros::{ProtoCodec, packet};

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
    pub height_map_data: Option<[[i8; 16]; 16]>,
    pub render_height_map_data_type: HeightMapDataType,
    pub render_height_map_data: Option<[[i8; 16]; 16]>,
    #[endianness(le)]
    pub blob_id: Option<u64>,
}
