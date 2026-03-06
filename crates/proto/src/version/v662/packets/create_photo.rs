use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 171)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CreatePhotoPacket {
    #[endianness(le)]
    pub raw_id: u64,
    pub photo_name: String,
    pub photo_item_name: String,
}
