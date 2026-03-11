use bedrockrs_macros::{ProtoCodec, gamepacket};

#[gamepacket(id = 336)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientBoundTextureShiftPacket {
    pub action: ClientBoundTextureShiftAction,
    pub collection_name: String,
    pub from_step: String,
    pub to_step: String,
    pub all_steps: Vec<String>,
    #[endianness(var)]
    pub current_length_ticks: u64,
    #[endianness(var)]
    pub total_length_ticks: u64,
    pub enabled: bool,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u8)]
#[repr(u8)]
pub enum ClientBoundTextureShiftAction {
    Invalid = 0,
    Initialize = 1,
    Start = 2,
    SetEnabled = 3,
    Sync = 4,
}
