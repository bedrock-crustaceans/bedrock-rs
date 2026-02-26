use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 136)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientCacheMissResponsePacket {
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub missing_blobs: Vec<MissingBlobEntry>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct MissingBlobEntry {
    #[endianness(le)]
    pub blob_id: u64,
    pub blob_data: String,
}
