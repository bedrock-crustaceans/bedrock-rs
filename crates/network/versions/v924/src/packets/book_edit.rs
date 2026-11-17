use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::{ProtoCodec, packet};

#[packet(id = 97, direction = "client_to_server")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct BookEditPacket<V: ProtoVersion> {
    #[endianness(var)]
    pub book_slot: i32,
    pub action: V::BookEditAction,
}
