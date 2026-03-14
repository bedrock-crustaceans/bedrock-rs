use bedrockrs_macros::{ProtoCodec, packet};

#[packet(id = 149)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerArmorDamagePacket {
    pub list: Vec<PlayerArmorDamageEntry>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerArmorDamageEntry {
    pub slot: i8,
    #[endianness(le)]
    pub damage: i16,
}
