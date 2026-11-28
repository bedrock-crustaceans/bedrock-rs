use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::{packet, ProtoCodec};

#[packet(id = 91, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ShowStoreOfferPacket<V: ProtoVersion> {
    pub product_id: String,
    pub redirect_type: V::ShowStoreOfferRedirectType,
}
