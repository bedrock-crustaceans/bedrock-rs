use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 334)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientBoundDataDrivenUICloseScreenPacket {
    #[endianness(le)]
    pub screen_id: u32
}