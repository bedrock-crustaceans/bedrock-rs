use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone, Copy)]
pub struct BiomeMesaSurfaceData {
    #[endianness(le)]
    pub clay_material: i32,
    #[endianness(le)]
    pub hard_clay_material: i32,
    pub bryce_pillars: bool,
    pub has_forest: bool,
}
