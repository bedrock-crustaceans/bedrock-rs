use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, gamepacket};

#[gamepacket(id = 29)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct UpdateAttributesPacket<V: ProtoVersion> {
    pub target_runtime_id: V::ActorRuntimeID,

    pub attribute_list: Vec<AttributeData<V>>,
    #[endianness(var)]
    pub ticks_since_sim_started: u64,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct AttributeModifier<V: ProtoVersion> {
    pub id: String,
    pub name: String,
    #[endianness(le)]
    pub amount: f32,
    pub operation: V::AttributeModifierOperation,
    pub operand: V::AttributeOperands,
    pub is_serializable: bool,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct AttributeData<V: ProtoVersion> {
    #[endianness(le)]
    pub min_value: f32,
    #[endianness(le)]
    pub max_value: f32,
    #[endianness(le)]
    pub current_value: f32,
    #[endianness(le)]
    pub default_min: f32,
    #[endianness(le)]
    pub default_max: f32,
    #[endianness(le)]
    pub default_value: f32,
    pub attribute_name: String,

    pub attribute_modifiers: Vec<AttributeModifier<V>>,
}
