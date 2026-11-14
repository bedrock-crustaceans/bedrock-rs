use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u32)]
#[enum_endianness(var)]
#[repr(u32)]
pub enum EducationEditionOffer {
    None = 0,
    RestOfWorld = 1,
    #[deprecated]
    China = 2,
}
