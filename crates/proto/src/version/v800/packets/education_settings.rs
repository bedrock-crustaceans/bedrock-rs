use super::super::types::EducationLevelSettings;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 137)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct EducationSettingsPacket {
    pub education_level_settings: EducationLevelSettings,
}