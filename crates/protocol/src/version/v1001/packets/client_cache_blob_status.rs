use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 135)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientCacheBlobStatusPacket {
    #[endianness(le)]
    pub missing_blobs: Vec<u64>,
    #[endianness(le)]
    pub obtained_blobs: Vec<u64>,
}
