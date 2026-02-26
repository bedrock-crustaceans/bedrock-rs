use super::super::enums::PhotoType;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 99)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PhotoTransferPacket {
    pub photo_name: String,
    pub photo_data: String,
    pub book_id: String,
    pub photo_type: PhotoType,
    pub source_type: PhotoType,
    #[endianness(le)]
    pub owner_id: i64,
    pub new_photo_name: String,
}