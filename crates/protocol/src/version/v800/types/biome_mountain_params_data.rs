use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone, Copy)]
#[expect(clippy::struct_excessive_bools, reason = "mirrors the packet layout")]
pub struct BiomeMountainParamsData {
    #[endianness(le)]
    pub steep_block: i32,
    pub north_slopes: bool,
    pub south_slopes: bool,
    pub west_slopes: bool,
    pub east_slopes: bool,
    pub top_slide_enabled: bool,
}
