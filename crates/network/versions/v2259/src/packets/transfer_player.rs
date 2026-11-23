use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::{ProtoCodec, packet};

#[packet(id = 85, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct TransferPlayerPacket<V: ProtoVersion> {
    pub server_address: String,
    #[endianness(le)]
    pub server_port: u16,
    pub reload_world: bool,
    pub gatherings_config: Option<V::GatheringsConfig>,
    pub ui_options: Option<TransferPlayerUiOptions>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct TransferPlayerUiOptions {
    pub title_text: Option<String>,
    pub body_text: Option<String>,
    pub title_visible: Option<bool>,
    pub body_visible: Option<bool>,
    pub cancel_button_text: Option<String>,
    pub cancel_button_visible: Option<bool>,
    #[endianness(le)]
    pub cancel_button_delay_seconds: Option<f32>,
}
