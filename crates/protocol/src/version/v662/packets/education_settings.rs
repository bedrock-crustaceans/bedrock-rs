use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 137, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct EducationSettingsPacket<V: ProtoVersion> {
    pub education_level_settings: V::EducationLevelSettings,
}
