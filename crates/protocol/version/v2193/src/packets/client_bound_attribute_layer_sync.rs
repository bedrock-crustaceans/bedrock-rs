use bedrock_protocol_v1001::packets::{AttributeData, AttributeLayerSettings};
use bedrock_macros::{ProtoCodec, packet};

#[packet(id = 345, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientBoundAttributeLayerSyncPacket {
    pub data: ClientBoundAttributeLayerSyncData,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_endianness(var)]
#[enum_repr(u32)]
#[repr(u32)]
pub enum ClientBoundAttributeLayerSyncData {
    UpdateAttributeLayersData {
        layers: Vec<AttributeLayerData>,
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
    pub noise_name: Option<String>,
    #[endianness(var)]
    pub dimension: i32,
    pub settings: AttributeLayerSettings,
    pub attributes: Vec<EnvironmentAttributeData>,
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
    pub easing: String,
    #[endianness(le)]
    pub local_transition_ticks: u32,
    pub noise_transition: bool,
    pub noise_alignment: NoiseAlignment,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct NoiseAlignment {
    pub alignment_type: NoiseAlignmentType,
    #[endianness(var)]
    pub value: u32,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u8)]
#[repr(u8)]
pub enum NoiseAlignmentType {
    MinLocalTransitionEnd = 0,
}
