use strum_macros::{Display, EnumString};
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 345)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientBoundAttributeLayerSyncPacket {
    pub data: ClientBoundAttributeLayerSyncData
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_endianness(var)]
#[enum_repr(u32)]
#[repr(u32)]
pub enum ClientBoundAttributeLayerSyncData {
    UpdateAttributeLayersData {
        layers: Vec<AttributeLayerData>
    } = 0,
    UpdateAttributeLayerSettingsData {
        name: String,
        #[endianness(var)]
        dimension: i32,
        settings: AttributeLayerSettings,
    } = 1,
    UpdateEnvironmentAttributesData {
        name: String,
        #[endianness(var)]
        dimension: i32,
        attributes: Vec<EnvironmentAttributeData>,
    } = 2,
    RemoveEnvironmentAttributesData {
        name: String,
        #[endianness(var)]
        dimension: i32,
        attributes: Vec<String>,
    } = 3,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct AttributeLayerData {
    pub name: String,
    #[endianness(var)]
    pub dimension: i32,
    pub settings: AttributeLayerSettings,
    pub attributes: Vec<EnvironmentAttributeData>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct AttributeLayerSettings {
    #[endianness(le)]
    pub priority: i32,
    pub weight: AttributeLayerWeight,
    pub enabled: bool,
    pub transitions_paused: bool,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_endianness(var)]
#[enum_repr(u32)]
#[repr(u32)]
pub enum AttributeLayerWeight {
    Float(#[endianness(le)] f64) = 0,
    String(String) = 1,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct EnvironmentAttributeData {
    pub attribute_name: String,
    pub from_attribute: Option<AttributeData>,
    pub attribute: AttributeData,
    pub to_attribute: Option<AttributeData>,
    #[endianness(le)]
    pub current_transition_ticks: u32,
    #[endianness(le)]
    pub total_transition_ticks: u32,
    // TODO: EaseType enum as string value
    pub easing: String,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_endianness(var)]
#[enum_repr(u32)]
#[repr(u32)]
pub enum AttributeData {
    Bool {
        value: bool,
        #[str]
        operation: BoolAttributeOperation
    } = 0,
    Float {
        #[endianness(le)]
        value: f32,
        #[str]
        operation: FloatAttributeOperation,
        #[endianness(le)]
        constraint_min: f32,
        #[endianness(le)]
        constraint_max: f32,
    } = 1,
    Color {
        value: Color255RGBA,
        #[str]
        operation: ColorAttributeOperation,
    } = 2,
}

#[derive(Clone, Debug, EnumString, Display)]
#[strum(serialize_all = "snake_case")]
pub enum BoolAttributeOperation {
    Override,
    AlphaBlend,
    AND,
    NAND,
    OR,
    NOR,
    XOR,
    XNOR,
}

#[derive(Clone, Debug, EnumString, Display)]
#[strum(serialize_all = "snake_case")]
pub enum FloatAttributeOperation {
    Override,
    AlphaBlend,
    Add,
    Subtract,
    Multiply,
    Minimum,
    Maximum
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_endianness(var)]
#[enum_repr(u32)]
#[repr(u32)]
pub enum Color255RGBA {
    String(String) = 0,
    Array(#[endianness(le)] [i32; 4]) = 1,
}

#[derive(Clone, Debug, EnumString, Display)]
#[strum(serialize_all = "snake_case")]
pub enum ColorAttributeOperation {
    Override,
    AlphaBlend,
    Add,
    Subtract,
    Multiply,
}
