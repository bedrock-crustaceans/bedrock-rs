use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u32)]
#[enum_endianness(var)]
#[repr(u32)]
pub enum CommandBlockMode {
    Normal = 0,
    Chain = 2,
    Repeating = 1,
}