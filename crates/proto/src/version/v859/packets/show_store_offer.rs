use crate::version::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, packet};
use uuid::Uuid;

#[packet(id = 91)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ShowStoreOfferPacket<V: ProtoVersion> {
    pub product_id: Uuid,
    pub redirect_type: V::ShowStoreOfferRedirectType,
}
