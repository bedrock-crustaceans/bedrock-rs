use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 137)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct EducationSettingsPacket<V: ProtoVersion> {
    pub education_level_settings: V::EducationLevelSettings,
}
