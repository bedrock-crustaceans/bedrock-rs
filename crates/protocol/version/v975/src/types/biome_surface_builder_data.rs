use bedrock_macros::ProtoCodec;
use bedrock_protocol_version::ProtoVersion;

#[derive(ProtoCodec, Clone, Debug)]
#[expect(clippy::struct_excessive_bools, reason = "mirrors the packet layout")]
pub struct BiomeSurfaceBuilderData<V: ProtoVersion> {
    pub surface_material: Option<V::BiomeSurfaceMaterialData>,
    pub has_default_overworld_surface: bool,
    pub has_swamp_surface: bool,
    pub has_frozen_ocean_surface: bool,
    pub has_the_end_surface: bool,
    pub mesa_surface: Option<V::BiomeMesaSurfaceData>,
    pub capped_surface: Option<V::BiomeCappedSurfaceData>,
    pub noise_gradient_surface: Option<V::BiomeNoiseGradientSurfaceData>,
}
