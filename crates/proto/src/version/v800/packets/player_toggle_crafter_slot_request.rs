use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 306)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerToggleCrafterSlotRequestPacket {
    #[endianness(le)]
    pub pos_x: i32,
    #[endianness(le)]
    pub pos_y: i32,
    #[endianness(le)]
    pub pos_z: i32,
    pub slot_index: i8,
    pub is_disabled: bool,
}