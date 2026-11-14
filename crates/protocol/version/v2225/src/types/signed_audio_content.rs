use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct SignedAudioContent {
    pub compact_jwt: String,
}
