//! Constructors of the client-biome documents and of the components with required members.

use super::color::Color255Rgb;
use super::component::{DryFoliageColor, SkyColor, SoundAddition, WaterAppearance};
use super::{Beta, ClientBiome, Description};
use bedrock_addon_core::document::{ComponentMap, Extra};
use bedrock_addon_core::id::{BiomeId, SoundEventName};
use bedrock_addon_core::json::StrictFloat;

impl Description {
    /// The description of `identifier`.
    pub fn new(identifier: BiomeId) -> Self {
        Self {
            identifier,
            extra: Extra::default(),
        }
    }
}

impl ClientBiome {
    /// A client biome named `identifier` without components.
    pub fn new(identifier: BiomeId) -> Self {
        Self {
            description: Description::new(identifier),
            components: ComponentMap::new(),
            extra: Extra::default(),
        }
    }

    /// The surface colour (RGBA channels from 0 to 1) and opacity its
    /// `minecraft:water_appearance` sets; `None` for a member it leaves out. The start of the
    /// water surface `biomes_client.json` then overrides.
    pub fn water_surface(&self) -> (Option<[f32; 4]>, Option<f32>) {
        let Some(water) = self.components.get::<WaterAppearance>() else {
            return (None, None);
        };
        let color = water
            .surface_color
            .as_ref()
            .and_then(Color255Rgb::rgb)
            .map(|[r, g, b]| [r, g, b].map(|channel| f32::from(channel) / 255.0))
            .map(|[r, g, b]| [r, g, b, 1.0]);
        (color, water.surface_opacity.map(|opacity| opacity.0))
    }
}

impl Beta {
    /// A beta client biome named `identifier` without components or environment attributes.
    pub fn new(identifier: BiomeId) -> Self {
        Self {
            description: Description::new(identifier),
            components: ComponentMap::new(),
            environment_attributes: None,
            extra: Extra::default(),
        }
    }
}

impl SkyColor {
    /// `minecraft:sky_color` with `sky_color`.
    pub fn new(sky_color: Color255Rgb) -> Self {
        Self {
            sky_color,
            extra: Extra::default(),
        }
    }
}

impl DryFoliageColor {
    /// `minecraft:dry_foliage_color` with `color`.
    pub fn new(color: Color255Rgb) -> Self {
        Self {
            color,
            extra: Extra::default(),
        }
    }
}

impl SoundAddition {
    /// The sound `asset` at `chance` (from 0 to 1).
    pub fn new(asset: SoundEventName, chance: f32) -> Self {
        Self {
            asset,
            chance: StrictFloat(chance),
            extra: Extra::default(),
        }
    }
}
