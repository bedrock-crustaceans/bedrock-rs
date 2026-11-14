use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 196, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct UpdateClientInputLocksPacket {
    #[endianness(var)]
    pub input_lock_component_data: u32,
}
