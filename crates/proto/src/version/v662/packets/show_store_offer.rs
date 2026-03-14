use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 91)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ShowStoreOfferPacket<V: ProtoVersion> {
    pub product_id: String,
    pub redirect_type: V::ShowStoreOfferRedirectType,
}
