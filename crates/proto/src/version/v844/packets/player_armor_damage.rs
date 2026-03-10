use bedrockrs_macros::{ProtoCodec, gamepacket};

#[gamepacket(id = 149)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerArmorDamagePacket {
    #[vec_repr(i32)]
    #[vec_endianness(var)]
    pub list: Vec<PlayerArmorDamageEntry>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerArmorDamageEntry {
    pub slot: i8,
    #[endianness(le)]
    pub damage: i16,
}
