//! Format 1.21.90: the identifier must have a namespace; the components are 1.21.70's.

use super::v1_21_40::Component;
use super::{change, find, has, insert, legacy_document, v1_21_100};
use crate::component::GrassAppearance;
use bedrock_addon_core::document::{Component as _, Extra};
use bedrock_addon_core::json::StrictBool;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};

legacy_document! {
    /// A client biome of format `[1.21.90, 1.21.100)`: the 1.21.70 component map with the
    /// namespaced description.
    version = (1, 21, 90), set = Component
}

/// The version below which a document gets the grass shading: the next struct's version, so
/// every document that reaches the step is below it.
const GRASS_SHADE_UPGRADE_BEFORE: FormatVersion = FormatVersion::new(1, 21, 100);

/// The one biome whose grass the game shaded in code before 1.21.100
/// (`minecraft:roofed_forest_mutated` is not shaded).
const SHADED: &str = "minecraft:roofed_forest";

/// The step to 1.21.100: `grass_appearance` moves into the type with `grass_is_shaded`; the
/// dark forest's grass is shaded, whatever its file says. Every document this struct reads, or
/// an older step hands on, is below 1.21.100, so the shading always applies.
impl Upgrade for ClientBiome {
    type Next = v1_21_100::ClientBiome;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> v1_21_100::ClientBiome {
        let mut components = self.components;
        if cx.original_version < GRASS_SHADE_UPGRADE_BEFORE
            && self.description.identifier.as_str() == SHADED
        {
            let shaded = Some(StrictBool(true));
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
                        extra: Extra::default(),
                    },
                    cx,
                ),
            }
        }
        v1_21_100::ClientBiome {
            format_version: self.format_version,
            description: self.description,
            components,
            extra: self.extra,
            root_extra: self.root_extra,
        }
    }
}
