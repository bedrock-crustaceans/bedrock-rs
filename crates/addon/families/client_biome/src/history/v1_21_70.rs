//! Format 1.21.70: `dry_foliage`, `biome_music.music_definition` and the four
//! render-configuration identifiers; identifiers may still leave out the namespace.

use super::v1_21_40::Component;
use super::{ClientBiomePayload, change, find, has, insert, note_identifier, v1_21_90};
use crate::component::{
    AtmosphereIdentifier, BiomeMusic, ColorGradingIdentifier, LightingIdentifier, WaterIdentifier,
};
use bedrock_addon_core::diagnostic::DiagnosticCode;
use bedrock_addon_core::document::Component as _;
use bedrock_addon_core::id::{
    AtmosphereSettingsId, BiomeId, ColorGradingSettingsId, LightingSettingsId, MusicDefinitionId,
    WaterSettingsId,
};
use bedrock_addon_core::load::{BiomeRenderIdentifiers, LoadContext};
use bedrock_addon_core::version::{BaseGameVersion, FormatVersion, SemVersion};
use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};

/// A client biome of format `[1.21.70, 1.21.90)`.
pub type ClientBiome = Legacy<ClientBiomePayload<Component>, 1, 21, 70>;

/// The version below which a document gets the atmosphere, colour-grading and water identifiers.
const RENDER_UPGRADE_BEFORE: FormatVersion = FormatVersion::new(1, 21, 70);
/// The version below which a document gets the lighting identifier and the music definition.
const LIGHTING_AND_MUSIC_UPGRADE_BEFORE: FormatVersion = FormatVersion::new(1, 21, 80);

/// The identifiers the game's renderer gives a biome no render configuration names: the
/// values that nearly all of vanilla's own files carry.
fn default_render_identifiers() -> BiomeRenderIdentifiers {
    BiomeRenderIdentifiers {
        atmosphere: AtmosphereSettingsId::from_static("minecraft:default_atmospherics"),
        color_grading: ColorGradingSettingsId::from_static("minecraft:default_color_grading"),
        lighting: LightingSettingsId::from_static("minecraft:default_lighting"),
        water: WaterSettingsId::from_static("minecraft:default_water"),
    }
}

/// The renderer's identifiers for `biome`: the load context's map when it has one (a biome it
/// does not list gets none), else the defaults for every biome. Only a client has a registry
/// of its own for this; a server has none.
fn render_identifiers(cx: &LoadContext, biome: &BiomeId) -> Option<BiomeRenderIdentifiers> {
    match &cx.client_render_identifiers {
        Some(map) => map.get(biome).cloned(),
        None => Some(default_render_identifiers()),
    }
}

/// The base-game versions of the music table, named after the game updates that introduced
/// the music.
const BEE_UPDATE: SemVersion = SemVersion::new(1, 14, 0);
const NETHER_UPDATE: SemVersion = SemVersion::new(1, 16, 0);
const CAVES_AND_CLIFFS: SemVersion = SemVersion::new(1, 18, 0);
const WILD_UPDATE: SemVersion = SemVersion::new(1, 19, 0);
const WILD_UPDATE_U3: SemVersion = SemVersion::new(1, 19, 30);
const TRAILS_AND_TALES: SemVersion = SemVersion::new(1, 20, 0);
const SUMMER_24: SemVersion = SemVersion::new(1, 21, 0);

/// The music a vanilla biome's file left to code before 1.21.80, by the world version that
/// introduced it. The keys are `minecraft:` names; the definition is the key's name unless
/// [`music_name`] says otherwise. There are 43 keys. `endboss` and `nether` are plain keys;
/// no vanilla biome has those names.
const MUSIC: [(SemVersion, &[&str]); 7] = [
    (BEE_UPDATE, &["the_end", "endboss", "nether"]),
    (
        NETHER_UPDATE,
        &["crimson_forest", "hell", "soulsand_valley"],
    ),
    (
        CAVES_AND_CLIFFS,
        &[
            "dripstone_caves",
            "frozen_peaks",
            "grove",
            "jagged_peaks",
            "lush_caves",
            "meadow",
            "snowy_slopes",
            "stony_peaks",
        ],
    ),
    (
        WILD_UPDATE,
        &[
            "bamboo_jungle",
            "bamboo_jungle_hills",
            "birch_forest",
            "birch_forest_hills",
            "birch_forest_mutated",
            "deep_dark",
            "flower_forest",
            "forest",
            "jungle",
            "jungle_hills",
            "jungle_mutated",
            "mangrove_swamp",
            "mega_taiga",
            "mesa_plateau",
            "mesa_plateau_mutated",
            "redwood_taiga_hills_mutated",
            "redwood_taiga_mutated",
            "roofed_forest",
            "roofed_forest_mutated",
            "swampland",
            "swampland_mutated",
        ],
    ),
    (WILD_UPDATE_U3, &["basalt_deltas"]),
    (
        TRAILS_AND_TALES,
        &[
            "cherry_grove",
            "desert",
            "desert_hills",
            "desert_mutated",
            "jungle_edge",
            "mesa",
        ],
    ),
    (SUMMER_24, &["warped_forest"]),
];

/// The definition of a key of [`MUSIC`]: its own name, except the End's and the warped
/// forest's (whose definition is empty, and is inserted all the same).
fn music_name(key: &'static str) -> &'static str {
    match key {
        "the_end" => "end",
        "warped_forest" => "",
        other => other,
    }
}

/// The music definition of the biome `identifier` (namespaced), when the world's base-game
/// version includes the version that introduced it (`*` and an absent version include every
/// one).
fn music_definition(identifier: &str, world: &BaseGameVersion) -> Option<&'static str> {
    let name = identifier.strip_prefix("minecraft:")?;
    MUSIC.iter().find_map(|(since, keys)| {
        let key = keys.iter().find(|key| **key == name)?;
        world.includes(since).then(|| music_name(key))
    })
}

/// The step to 1.21.90: an identifier without a
/// namespace gets `minecraft:`; then the renderer's four identifiers and the music definition
/// that older vanilla files left to code are inserted.
impl LegacyStep<1, 21, 70> for ClientBiomePayload<Component> {
    type Next = v1_21_90::ClientBiome;

    fn step(this: ClientBiome, cx: &mut UpgradeCx<'_>) -> v1_21_90::ClientBiome {
        let biome = this.0;
        let mut description = biome.description;
        let mut components = biome.components;
        if !description.identifier.as_str().contains(':') {
            description.identifier =
                BiomeId::parse(format!("minecraft:{}", description.identifier.as_str()));
            note_identifier(cx, DiagnosticCode::IdentifierNamespaced);
        }
        if let Some(ids) = render_identifiers(cx.load, &description.identifier) {
            let early = cx.original_version < RENDER_UPGRADE_BEFORE;
            if early && !has(&components, AtmosphereIdentifier::NAME) {
                insert(
                    &mut components,
                    AtmosphereIdentifier::new(ids.atmosphere),
                    cx,
                );
            }
            if early && !has(&components, ColorGradingIdentifier::NAME) {
                insert(
                    &mut components,
                    ColorGradingIdentifier::new(ids.color_grading),
                    cx,
                );
            }
            if cx.original_version < LIGHTING_AND_MUSIC_UPGRADE_BEFORE
                && !has(&components, LightingIdentifier::NAME)
            {
                insert(&mut components, LightingIdentifier::new(ids.lighting), cx);
            }
            if early && !has(&components, WaterIdentifier::NAME) {
                insert(&mut components, WaterIdentifier::new(ids.water), cx);
            }
        }
        let definition = (cx.original_version < LIGHTING_AND_MUSIC_UPGRADE_BEFORE)
            .then(|| {
                music_definition(
                    description.identifier.as_str(),
                    &cx.load.world_base_game_version,
                )
            })
            .flatten()
            .map(MusicDefinitionId::parse);
        if let Some(definition) = definition {
            match find::<BiomeMusic>(&components) {
                // An existing definition is kept.
                Some((music, _)) if music.music_definition.is_some() => {}
                Some((mut music, synthesized)) => {
                    music.music_definition = Some(definition);
                    change(&mut components, music, synthesized, cx);
                }
                None if has(&components, BiomeMusic::NAME) => {}
                None => insert(
                    &mut components,
                    BiomeMusic {
                        music_definition: Some(definition),
                        volume_multiplier: None,
                        underwater_music: None,
                    },
                    cx,
                ),
            }
        }
        Legacy(ClientBiomePayload {
            format_version: biome.format_version,
            description,
            components,
        })
    }
}
