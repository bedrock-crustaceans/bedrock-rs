use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 176, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerStartItemCooldownPacket {
    pub item_category: String,
    #[endianness(var)]
    pub duration_ticks: i32,
}
