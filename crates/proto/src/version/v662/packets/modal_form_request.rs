use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 100)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ModalFormRequestPacket {
    #[endianness(var)]
    pub form_id: u32,
    pub form_ui_json: String,
}
