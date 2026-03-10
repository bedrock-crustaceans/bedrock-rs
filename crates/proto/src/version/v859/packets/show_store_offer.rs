use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, gamepacket};
use uuid::Uuid;

#[gamepacket(id = 91)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ShowStoreOfferPacket<V: ProtoVersion> {
    pub product_id: Uuid,
    pub redirect_type: V::ShowStoreOfferRedirectType,
}
