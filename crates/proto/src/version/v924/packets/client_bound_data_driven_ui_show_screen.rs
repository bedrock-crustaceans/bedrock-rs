use bedrockrs_macros::{ProtoCodec, gamepacket};

#[gamepacket(id = 333)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientBoundDataDrivenUIShowScreenPacket {
    pub screen_id: String,
}
