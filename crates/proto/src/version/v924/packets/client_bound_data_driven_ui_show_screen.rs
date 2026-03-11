use bedrockrs_macros::{ProtoCodec, packet};

#[packet(id = 333)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientBoundDataDrivenUIShowScreenPacket {
    pub screen_id: String,
}
