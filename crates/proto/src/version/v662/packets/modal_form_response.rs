use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 101)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ModalFormResponsePacket<V: ProtoVersion> {
    #[endianness(var)]
    pub form_id: u32,
    pub json_response: Option<String>,
    pub form_cancel_reason: Option<V::ModalFormCancelReason>,
}