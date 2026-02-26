use super::super::types::NetworkBlockPosition;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 86)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlaySoundPacket {
    pub name: String,
    pub position: NetworkBlockPosition,
    #[endianness(le)]
    pub volume: f32,
    #[endianness(le)]
    pub pitch: f32,
}