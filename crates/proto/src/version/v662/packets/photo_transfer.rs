use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 99)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PhotoTransferPacket<V: ProtoVersion> {
    pub photo_name: String,
    pub photo_data: String,
    pub book_id: String,
    pub photo_type: V::PhotoType,
    pub source_type: V::PhotoType,
    #[endianness(le)]
    pub owner_id: i64,
    pub new_photo_name: String,
}