use super::super::enums::ModalFormCancelReason;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 101)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ModalFormResponsePacket {
    #[endianness(var)]
    pub form_id: u32,
    pub json_response: Option<String>,
    pub form_cancel_reason: Option<ModalFormCancelReason>,
}