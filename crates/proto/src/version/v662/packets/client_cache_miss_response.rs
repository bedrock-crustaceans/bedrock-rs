use bedrockrs_macros::{ProtoCodec, gamepacket};

#[gamepacket(id = 136)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientCacheMissResponsePacket {
    pub missing_blobs: Vec<MissingBlobEntry>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct MissingBlobEntry {
    #[endianness(le)]
    pub blob_id: u64,
    pub blob_data: String,
}
