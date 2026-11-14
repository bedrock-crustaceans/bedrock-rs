use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::{packet, ProtoCodec};

#[packet(id = 114, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct UpdateSoftEnumPacket<V: ProtoVersion> {
    pub enum_name: String,
    pub values: Vec<String>,
    pub update_type: V::SoftEnumUpdateType,
}
