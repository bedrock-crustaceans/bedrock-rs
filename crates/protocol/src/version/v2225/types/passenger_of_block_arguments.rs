use crate::ProtoVersion;
use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct PassengerOfBlockArguments<V: ProtoVersion> {
    pub block_position: V::NetworkBlockPosition,
    #[endianness(le)]
    pub offset: (f32, f32, f32),
    #[endianness(le)]
    pub rotation: f32,
    #[endianness(le)]
    pub rotation_limit: f32,
    pub emote_type: PassengerOfBlockEmoteType,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u8)]
#[repr(u8)]
pub enum PassengerOfBlockEmoteType {
    Standing = 0,
    Riding = 1,
    Laying = 2,
}
