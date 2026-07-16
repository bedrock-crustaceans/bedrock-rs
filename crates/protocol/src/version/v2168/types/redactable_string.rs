use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct RedactableString {
    pub unredacted: String,
    pub redacted: Option<String>,
}
