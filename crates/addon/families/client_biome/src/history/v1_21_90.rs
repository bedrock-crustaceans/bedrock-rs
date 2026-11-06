//! Format 1.21.90: the identifier must have a namespace; the components are 1.21.70's.

use super::v1_21_40::Component;
use super::{ClientBiomePayload, change, find, has, insert, v1_21_100};
use crate::component::GrassAppearance;
use bedrock_addon_core::document::Component as _;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};

/// A client biome of format `[1.21.90, 1.21.100)`: the 1.21.70 component map with the
/// namespaced description.
pub type ClientBiome = Legacy<ClientBiomePayload<Component>, 1, 21, 90>;

/// The version below which a document gets the grass shading: the next struct's version, so
/// every document that reaches the step is below it.
const GRASS_SHADE_UPGRADE_BEFORE: FormatVersion = known::V1_21_100;

/// The one biome whose grass the game shaded in code before 1.21.100
/// (`minecraft:roofed_forest_mutated` is not shaded).
const SHADED: &str = "minecraft:roofed_forest";

/// The step to 1.21.100: `grass_appearance` moves into the type with `grass_is_shaded`; the
/// dark forest's grass is shaded, whatever its file says. Every document this struct reads, or
/// an older step hands on, is below 1.21.100, so the shading always applies.
impl LegacyStep<1, 21, 90> for ClientBiomePayload<Component> {
    type Next = v1_21_100::ClientBiome;

    fn step(this: ClientBiome, cx: &mut UpgradeCx<'_>) -> v1_21_100::ClientBiome {
        let biome = this.0;
        let mut components = biome.components;
        if cx.original_version < GRASS_SHADE_UPGRADE_BEFORE
            && biome.description.identifier.as_str() == SHADED
        {
            let shaded = Some(true);
            match find::<GrassAppearance>(&components) {
                Some((mut grass, synthesized)) => {
                    grass.grass_is_shaded = shaded;
                    change(&mut components, grass, synthesized, cx);
                }
                None if has(&components, GrassAppearance::NAME) => {}
                None => insert(
                    &mut components,
                    GrassAppearance {
                        color: None,
                        grass_is_shaded: shaded,
                    },
                    cx,
                ),
            }
        }
        Legacy(ClientBiomePayload {
            format_version: biome.format_version,
            description: biome.description,
            components,
        })
    }
}
