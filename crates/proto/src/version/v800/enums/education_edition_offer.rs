use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i32)]
#[enum_endianness(var)]
#[repr(i32)]
pub enum EducationEditionOffer {
    None = 0,
    RestOfWorld = 1,
    #[deprecated] China = 2,
}