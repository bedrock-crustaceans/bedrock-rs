use crate::version::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraAimAssistPresetDefinition<V: ProtoVersion> {
    pub identifier: String,

    pub block_exclusion_list: Vec<String>,

    pub entity_exclusion_list: Vec<String>,

    pub block_tag_exclusion_list: Vec<String>,

    pub liquid_targeting_list: Vec<String>,

    pub item_settings: Vec<V::CameraAimAssistItemSettings>,
    pub default_item_settings: Option<String>,
    pub hand_settings: Option<String>,
}
