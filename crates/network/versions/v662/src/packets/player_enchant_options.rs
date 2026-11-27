use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::{packet, ProtoCodec};

#[packet(id = 146, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerEnchantOptionsPacket<V: ProtoVersion> {
    pub options: Vec<OptionsEntry<V>>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct OptionsEntry<V: ProtoVersion> {
    #[endianness(var)]
    pub cost: u32,
    pub enchants: V::ItemEnchants,
    pub enchant_name: String,
    #[endianness(var)]
    pub enchant_net_id: u32,
}
