use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 20, direction = "client_to_server")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PassengerJumpPacket {
    #[endianness(var)]
    pub jump_scale: i32,
}
