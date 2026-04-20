use bedrockrs_macros::ProtoCodec;
use crate::ProtoVersion;

#[derive(ProtoCodec, Clone, Debug)]
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
