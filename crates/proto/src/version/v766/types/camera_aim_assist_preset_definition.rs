use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraAimAssistPresetDefinition<V: ProtoVersion> {
    pub identifier: String,
    pub categories: String,

    pub exclusion_list: Vec<String>,

    pub liquid_targeting_list: Vec<String>,

    pub item_settings: Vec<V::CameraAimAssistItemSettings>,
    pub default_item_settings: Option<String>,
    pub hand_settings: Option<String>,
}
