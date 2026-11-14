use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 38, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct HurtArmorPacket {
    #[endianness(var)]
    pub cause: i32,
    #[endianness(var)]
    pub damage: i32,
    #[endianness(var)]
    pub armor_slots: u64,
}
