use super::super::enums::PackType;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 82)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ResourcePackDataInfoPacket {
    pub resource_name: String,
    #[endianness(le)]
    pub chunk_size: u32,
    #[endianness(le)]
    pub chunk_amount: u32,
    #[endianness(le)]
    pub file_size: u64,
    pub file_hash: String,
    pub is_premium: bool,
    pub pack_type: PackType,
}