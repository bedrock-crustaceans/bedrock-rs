use bedrock_macros::ProtoCodec;

#[expect(clippy::struct_excessive_bools, reason = "mirrors the packet layout")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct AdventureSettings {
    pub no_pvm: bool,
    pub no_mvp: bool,
    pub immutable_world: bool,
    pub show_name_tags: bool,
    pub auto_jump: bool,
}
