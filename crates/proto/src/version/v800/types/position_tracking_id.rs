use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct PositionTrackingId {
    #[endianness(var)]
    pub value: i32,
}