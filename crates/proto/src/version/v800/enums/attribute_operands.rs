use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug, Eq, PartialEq)]
#[enum_repr(i32)]
#[enum_endianness(le)]
#[repr(i32)]
pub enum AttributeOperands {
    Min = 0,
    Max = 1,
    Current = 2,
    TotalOperands = 3,
}

impl AttributeOperands {
    pub const OPERAND_INVALID: AttributeOperands = AttributeOperands::TotalOperands;
}