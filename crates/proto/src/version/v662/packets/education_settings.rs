use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 137)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct EducationSettingsPacket<V: ProtoVersion> {
    pub education_level_settings: V::EducationLevelSettings,
}