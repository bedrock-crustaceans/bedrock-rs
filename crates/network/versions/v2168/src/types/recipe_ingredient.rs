use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct RecipeIngredient<V: ProtoVersion> {
    pub item_descriptor: V::ItemDescriptorType,
    #[endianness(le)]
    pub stack_size: i16,
}
