//! Format 1.21.120: the first format read from the `minecraft:biome` payload alone; the
//! component set is 1.21.110's.

use super::v1_26_0;
use crate::Description;
use crate::component::{BiomeComponent, Tags, VillageKind, VillageType};
use crate::parse::payload;
use crate::write::object;
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::document::{ComponentMap, Extra};
use bedrock_addon_core::json::{EncodeError, JsonSlice, StrictString, Value};
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use bedrock_addon_core::versioning::{ParseCx, Upgrade, UpgradeCx, VersionStruct, Versioned};

/// A biome of format `[1.21.120, 1.26.0)`: the payload of `minecraft:biome`.
#[derive(Clone, Debug, PartialEq)]
pub struct Biome {
    /// The biome's name (JSON `description`).
    pub description: Description,
    /// The components (JSON `components`).
    pub components: ComponentMap<BiomeComponent>,
    /// Unknown members of `minecraft:biome`.
    pub extra: Extra,
}

impl Versioned for Biome {
    const VERSION: FormatVersion = known::V1_21_120;
}

impl VersionStruct for Biome {
    fn parse(slice: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let payload = payload(slice, Self::VERSION, cx, d)?;
        Some(Self {
            description: payload.description,
            components: payload.components,
            extra: payload.extra,
        })
    }

    fn write(&self) -> Result<Value, EncodeError> {
        object(&self.description, &self.components, &self.extra)
    }
}

impl MolangFields for Biome {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visitor.nested("components", &mut self.components);
    }
}

/// The vanilla biomes whose overrides get `minecraft:village_type`, and the type: five
/// identifier sets, one per village style.
const VILLAGE_TYPES: [(&[&str], VillageKind); 5] = [
    (
        &[
            "minecraft:plains",
            "minecraft:sunflower_plains",
            "minecraft:meadow",
        ],
        VillageKind::Default,
    ),
    (&["minecraft:desert"], VillageKind::Desert),
    (&["minecraft:ice_plains"], VillageKind::Ice),
    (
        &[
            "minecraft:taiga",
            "minecraft:taiga_hills",
            "minecraft:cold_taiga",
            "minecraft:cold_taiga_hills",
        ],
        VillageKind::Taiga,
    ),
    (&["minecraft:savanna"], VillageKind::Savanna),
];

/// The vanilla biomes whose overrides get tags appended, and the tags, in vanilla's order:
/// four identifier sets, each with its tags.
const TAGS: [(&[&str], &[&str]); 4] = [
    (
        &[
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
        ],
        &["fast_fishing", "high_seas"],
    ),
    (
        &[
            "minecraft:swampland",
            "minecraft:swampland_mutated",
            "minecraft:mangrove_swamp",
        ],
        &["slime"],
    ),
    (
        &["minecraft:swampland", "minecraft:swampland_mutated"],
        &["swamp_water_huge_mushroom"],
    ),
    (
        &[
            "minecraft:mesa",
            "minecraft:mesa_plateau_stone",
            "minecraft:mesa_plateau",
            "minecraft:mesa_bryce",
            "minecraft:mesa_plateau_stone_mutated",
            "minecraft:mesa_plateau_mutated",
        ],
        &["surface_mineshaft"],
    ),
];

/// The step to 1.26.0: every component moves (`minecraft:replace_biomes` keeps its members;
/// 1.26.0 only widens its constraint), then an override of a vanilla biome gets the
/// `minecraft:village_type` and the tags that 1.26.0 moved from code into data. Nothing is
/// reported; inserted components are synthesized.
impl Upgrade for Biome {
    type Next = v1_26_0::Biome;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> v1_26_0::Biome {
        let mut components = self.components;
        add_missing_components(&mut components, cx.identifier);
        v1_26_0::Biome {
            description: self.description,
            components,
            extra: self.extra,
        }
    }
}

/// Inserts the components the game adds to an override of the vanilla biome named
/// `identifier` (the full identifier).
fn add_missing_components(components: &mut ComponentMap<BiomeComponent>, identifier: &str) {
    for (identifiers, kind) in VILLAGE_TYPES {
        // Vanilla sets it without checking; a 1.21.x document cannot hold it (the struct does
        // not bind it), so set and insert-if-absent coincide.
        if identifiers.contains(&identifier) && !components.contains::<VillageType>() {
            components.insert_typed(VillageType::new(kind).into(), true);
        }
    }
    for (identifiers, tags) in TAGS {
        if !identifiers.contains(&identifier) {
            continue;
        }
        if !components.contains::<Tags>() {
            components.insert_typed(
                Tags {
                    tags: Vec::new(),
                    extra: bedrock_addon_core::document::Extra::default(),
                }
                .into(),
                true,
            );
        }
        if let Some(present) = components.get_mut::<Tags>() {
            for tag in tags {
                // The crate does not append a tag that is already present; vanilla appends it
                // again without checking for a duplicate.
                if !present.contains(tag) {
                    present.tags.push(StrictString((*tag).to_owned()));
                }
            }
        }
    }
}
