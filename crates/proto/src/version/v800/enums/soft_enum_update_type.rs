use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u32)]
#[enum_endianness(le)]
#[repr(u32)]
pub enum SoftEnumUpdateType {
    Add = 0,
    Remove = 1,
    Replace = 2,
}