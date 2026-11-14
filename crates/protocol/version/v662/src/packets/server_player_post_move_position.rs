use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 16, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ServerPlayerPostMovePositionPacket {
    #[endianness(le)]
    pub pos: (f32, f32, f32),
}
