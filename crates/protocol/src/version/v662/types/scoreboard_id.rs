use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct ScoreboardId {
    #[endianness(var)]
    pub id: i64,
}
