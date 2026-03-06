use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 103)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ServerSettingsResponsePacket {
    #[endianness(var)]
    pub form_id: u32,
    pub form_ui_json: String,
}

// TODO: this doesn't seem right. Probably an error in the proto docs. Check gopher or cloudburst
