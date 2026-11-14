use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 334, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientBoundDataDrivenUICloseScreenPacket {
    #[endianness(le)]
    pub form_id: u32
}