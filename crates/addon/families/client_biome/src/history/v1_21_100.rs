//! Format 1.21.100: `grass_appearance.grass_is_shaded`.

use super::v1_21_40::Component;
use super::{change, find, has, insert, legacy_document, v1_21_110};
use crate::component::{BiomeMusic, Precipitation};
use bedrock_addon_core::document::{Component as _, Extra};
use bedrock_addon_core::json::{StrictBool, StrictFloat};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};

legacy_document! {
    /// A client biome of format `[1.21.100, 1.21.110)`.
    version = (1, 21, 100), set = Component
}

/// The version below which a document gets the precipitation component and the underwater
/// music: the next struct's version, so every document that reaches the step is below it.
const PARTICLES_AND_UNDERWATER_MUSIC_BEFORE: FormatVersion = known::V1_21_110;

/// The particles of the four nether biomes, which the server biome's `minecraft:climate`
/// carried before 1.21.110. Only the one member is written: of the four densities (`ash`,
/// `blue_spores`, `red_spores`, `white_ash`) the game writes each only when it is not 0.
fn precipitation(identifier: &str) -> Option<Precipitation> {
    let density = |value: f32| Some(StrictFloat(value));
    let mut particles = Precipitation::default();
    match identifier {
        "minecraft:basalt_deltas" => particles.white_ash = density(2.0),
        "minecraft:crimson_forest" => particles.red_spores = density(0.25),
        "minecraft:soulsand_valley" => particles.ash = density(0.05),
        "minecraft:warped_forest" => particles.blue_spores = density(0.25),
        _ => return None,
    }
    Some(particles)
}

/// The 13 water biomes whose underwater music the game chose in code before 1.21.110.
const UNDERWATER_MUSIC: [&str; 13] = [
    "minecraft:legacy_frozen_ocean",
    "minecraft:ocean",
    "minecraft:deep_ocean",
    "minecraft:warm_ocean",
    "minecraft:deep_warm_ocean",
    "minecraft:lukewarm_ocean",
    "minecraft:deep_lukewarm_ocean",
    "minecraft:cold_ocean",
    "minecraft:deep_cold_ocean",
    "minecraft:frozen_ocean",
    "minecraft:deep_frozen_ocean",
    "minecraft:river",
    "minecraft:frozen_river",
];

/// The step to 1.21.110: `biome_music` moves into the type with `underwater_music`; the nether
/// biomes get their particles and the water biomes their underwater music. Every document that
/// reaches this step is below 1.21.110, so both always apply.
impl Upgrade for ClientBiome {
    type Next = v1_21_110::ClientBiome;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> v1_21_110::ClientBiome {
        let mut components = self.components;
        let identifier = self.description.identifier.as_str();
        let early = cx.original_version < PARTICLES_AND_UNDERWATER_MUSIC_BEFORE;
        if early
            && !has(&components, Precipitation::NAME)
            && let Some(particles) = precipitation(identifier)
        {
            insert(&mut components, particles, cx);
        }
        if early && UNDERWATER_MUSIC.contains(&identifier) {
            let underwater = Some(StrictBool(true));
            match find::<BiomeMusic>(&components) {
                Some((mut music, synthesized)) => {
                    music.underwater_music = underwater;
                    change(&mut components, music, synthesized, cx);
                }
                None if has(&components, BiomeMusic::NAME) => {}
                None => insert(
                    &mut components,
                    BiomeMusic {
                        volume_multiplier: None,
                        music_definition: None,
                        underwater_music: underwater,
                        extra: Extra::default(),
                    },
                    cx,
                ),
            }
        }
        v1_21_110::ClientBiome {
            format_version: self.format_version,
            description: self.description,
            components,
        }
    }
}
