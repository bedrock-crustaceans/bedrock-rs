use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug, Eq, PartialEq)]
#[enum_repr(i32)]
#[enum_endianness(le)]
#[repr(i32)]
pub enum AttributeModifierOperation {
    Addition = 0,
    MultiplyBase = 1,
    MultiplyTotal = 2,
    Cap = 3,
    TotalOperations = 4,
}

impl AttributeModifierOperation {
    pub const OPERATION_INVALID: AttributeModifierOperation = AttributeModifierOperation::TotalOperations;
}

