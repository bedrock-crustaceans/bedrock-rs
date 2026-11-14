use bedrock_protocol_v944::packets::EnvironmentAttributeData;
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
    #[endianness(var)]
    pub dimension: i32,
    pub settings: AttributeLayerSettings,
    pub attributes: Vec<EnvironmentAttributeData>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct AttributeLayerSettings {
    #[endianness(le)]
    pub priority: i32,
    #[endianness(le)]
    pub weight: f32,
    pub enabled: bool,
    pub transitions_paused: bool,
}
