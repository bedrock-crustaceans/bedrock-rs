use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct EducationLevelSettings {
    pub code_builder_default_uri: String,
    pub code_builder_title: String,
    pub code_builder_resizable: bool,
    pub disable_legacy_title_bar: bool,
    pub post_process_filter: String,
    pub screenshot_border_resource_path: String,
    pub agent_capabilities: Option<()>, // TODO: AgentCapabilities
    pub code_builder_override_uri: Option<String>,
    pub has_quiz: bool,
    pub external_link_settings: Option<()>, // TODO: ExternalLinkSettings
}