use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemEnchants<V: ProtoVersion> {
    #[endianness(le)]
    pub slot: i32,

    pub enchants_for_given_activation: Vec<ItemEnchant<V>>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemEnchant<V: ProtoVersion> {
    pub enchant_type: V::EnchantType,
    pub enchant_level: i8,
}
