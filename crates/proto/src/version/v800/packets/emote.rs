use super::super::types::ActorRuntimeID;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 138)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct EmotePacket {
    pub actor_runtime_id: ActorRuntimeID,
    pub emote_id: String,
    #[endianness(var)]
    pub emote_duration: i32,
    pub xuid: String,
    pub platform_id: String,
    pub flags: Flags,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum Flags {
    ServerSide = 0x0,
    MuteEmoteChat = 0x2,
}
