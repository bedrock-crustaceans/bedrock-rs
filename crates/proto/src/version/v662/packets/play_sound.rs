use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 86)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlaySoundPacket<V: ProtoVersion> {
    pub name: String,
    pub position: V::NetworkBlockPosition,
    #[endianness(le)]
    pub volume: f32,
    #[endianness(le)]
    pub pitch: f32,
}
