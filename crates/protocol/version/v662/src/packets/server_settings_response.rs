use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 103, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ServerSettingsResponsePacket {
    #[endianness(var)]
    pub form_id: u32,
    pub form_ui_json: String,
}
