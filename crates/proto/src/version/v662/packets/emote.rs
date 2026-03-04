use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 138)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct EmotePacket<V: ProtoVersion> {
    pub actor_runtime_id: V::ActorRuntimeID,
    pub emote_id: String,
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