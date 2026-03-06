use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 184)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct RequestAbilityPacket<V: ProtoVersion> {
    pub ability: V::AbilitiesIndex,
    pub value_type: Type,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum Type {
    Unset = 0,
    Bool {
        variable_value: bool,
        #[endianness(le)]
        default_value: f32,
    } = 1,
    Float {
        #[endianness(le)]
        variable_value: f32,
        default_value: bool,
    } = 2,
}

// VERIFY: default_values. They seem to be incorrectly documented.
