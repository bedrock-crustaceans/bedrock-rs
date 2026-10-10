use crate::ProtoVersion;
use bedrock_macros::{ProtoCodec, packet};
use uuid::Uuid;

#[packet(id = 91, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ShowStoreOfferPacket<V: ProtoVersion> {
    pub product_id: Uuid,
    pub redirect_type: V::ShowStoreOfferRedirectType,
}
