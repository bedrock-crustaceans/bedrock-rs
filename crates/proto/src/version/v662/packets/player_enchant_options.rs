use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, gamepacket};

#[gamepacket(id = 146)]
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
