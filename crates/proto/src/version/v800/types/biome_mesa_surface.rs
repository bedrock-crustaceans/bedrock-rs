use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone, Copy)]
pub struct BiomeMesaSurfaceData {
    #[endianness(le)]
    pub clay_material: i64,
    #[endianness(le)]
    pub hard_clay_material: i64,
    pub bryce_pillars: bool,
    pub has_forest: bool,
}