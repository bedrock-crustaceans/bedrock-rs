use crate::version::v1001::packets::{
    BoolAttributeOperation, ColorAttributeOperation, FloatAttributeOperation,
};
use bedrock_macros::{ProtoCodec, packet};

#[packet(id = 345)]
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
    pub weight: AttributeLayerWeight,
    pub enabled: bool,
    pub transitions_paused: bool,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_endianness(var)]
#[enum_repr(u32)]
#[repr(u32)]
pub enum AttributeLayerWeight {
    Float(#[endianness(le)] f32) = 0,
    String(String) = 1,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct EnvironmentAttributeData {
    pub attribute_name: String,
    pub payload: EnvironmentAttributePayload,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_endianness(var)]
#[enum_repr(u32)]
#[repr(u32)]
pub enum EnvironmentAttributePayload {
    Constant {
        attribute: AttributeData,
    } = 0,
    Transition {
        from_attribute: AttributeData,
        to_attribute: AttributeData,
        settings: AttributeTransitionSettings,
    } = 1,
    NoiseTransition {
        from_attribute: AttributeData,
        to_attribute: AttributeData,
        settings: AttributeNoiseTransitionSettings,
    } = 2,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct AttributeTransitionSettings {
    #[endianness(var)]
    pub total_transition_ticks: u32,
    #[endianness(var)]
    pub current_transition_ticks: u32,
    #[endianness(var)]
    pub ease_type: i32,
    pub clock_name: String,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct AttributeNoiseTransitionSettings {
    #[endianness(var)]
    pub total_transition_ticks: u32,
    #[endianness(var)]
    pub current_transition_ticks: u32,
    #[endianness(var)]
    pub ease_type: i32,
    pub clock_name: String,
    #[endianness(var)]
    pub local_transition_ticks: u32,
    pub noise_name: String,
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

#[derive(ProtoCodec, Clone, Debug)]
#[enum_endianness(var)]
#[enum_repr(u32)]
#[repr(u32)]
pub enum AttributeData {
    Bool {
        value: bool,
        #[str]
        operation: BoolAttributeOperation,
    } = 0,
    Float {
        #[endianness(le)]
        value: f32,
        #[str]
        operation: FloatAttributeOperation,
    } = 1,
    Color {
        value: Color255RGBA,
        #[str]
        operation: ColorAttributeOperation,
    } = 2,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_endianness(var)]
#[enum_repr(u32)]
#[repr(u32)]
pub enum Color255RGBA {
    String(String) = 0,
    Array(#[endianness(le)] [u32; 4]) = 1,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_protocol_core::ProtoCodec;
    use std::io::Cursor;

    #[test]
    fn update_environment_reads_a_typed_payload() {
        let bytes = [
            2, 1, b'a', 0, 1, 1, b'b', 0, 1, 0, 0, 0x80, 0x3f, 3, b'a', b'd', b'd',
        ];
        let packet =
            ClientBoundAttributeLayerSyncPacket::deserialize(&mut Cursor::new(&bytes[..])).unwrap();
        let mut encoded = Vec::new();
        packet.serialize(&mut encoded).unwrap();
        assert_eq!(encoded, bytes, "encoding differs from the input");

        let ClientBoundAttributeLayerSyncData::UpdateEnvironmentAttributesData { attributes, .. } =
            packet.data
        else {
            panic!("expected UpdateEnvironmentAttributesData");
        };
        assert!(matches!(
            &attributes[0].payload,
            EnvironmentAttributePayload::Constant {
                attribute: AttributeData::Float {
                    operation: FloatAttributeOperation::Add,
                    ..
                }
            }
        ));
    }
}
