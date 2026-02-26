use super::super::enums::ShowStoreOfferRedirectType;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 91)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ShowStoreOfferPacket {
    pub product_id: String,
    pub redirect_type: ShowStoreOfferRedirectType,
}