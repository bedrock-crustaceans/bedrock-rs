//! The 1.26.0 step of `families::biome::history`: its insertions key on the identifier, so each
//! test runs the step for the identifiers of its table and for one outside it.

use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
use bedrock_addon_core::json::JsonSlice;
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::version::FormatVersionInput;
use bedrock_addon_core::versioning::{ParseCx, Upgrade, UpgradeCx, VersionStruct, Versioned};
use bedrock_addon_family_biome::component::{Tags, VillageKind, VillageType};
use bedrock_addon_family_biome::history::{v1_21_120, v1_26_0};

/// The 1.26.0 step run on `components` for the biome `identifier`.
fn step_1_26_0(identifier: &str, components: &str) -> v1_26_0::Biome {
    let text = format!(
        r#"{{"description": {{"identifier": "{identifier}"}}, "components": {{{components}}}}}"#
    );
    let load = LoadContext::detached();
    let version = <v1_21_120::Biome as Versioned>::VERSION;
    let parse = ParseCx {
        load: &load,
        written: FormatVersionInput {
            version: Some(version),
            beta_literal: false,
            prerelease: false,
        },
        original_version: version,
        at: JsonPath::new(),
    };
    let mut diagnostics = Diagnostics::default();
    let biome = v1_21_120::Biome::parse(JsonSlice::new(&text), &parse, &mut diagnostics)
        .expect("the input parses");
    let mut out = Diagnostics::default();
    let next = biome.upgrade(&mut UpgradeCx {
        load: &load,
        original_version: version,
        identifier,
        actor_type: ActorType::Mob,
        skip_section_a: false,
        diagnostics: &mut out,
    });
    assert!(out.is_empty(), "the step reports nothing");
    next
}

/// Each identifier of `identifiers` gets `kind`; a custom biome and an existing village type
/// are left alone.
fn village(identifiers: &[&str], kind: VillageKind) {
    for identifier in identifiers {
        let next = step_1_26_0(identifier, "");
        assert!(
            next.components.get::<VillageType>().map(|v| v.kind) == Some(kind),
            "the village type is inserted",
        );
    }
    let custom = step_1_26_0("test:plains", "");
    assert!(
        custom.components.get::<VillageType>().is_none(),
        "a custom biome gets none",
    );
}

/// Each identifier of `identifiers` gets `tags` appended, `minecraft:tags` created when absent,
/// and a tag already present is not repeated (ours).
fn tags(identifiers: &[&str], tags: &[&str]) {
    for identifier in identifiers {
        let created = step_1_26_0(identifier, "");
        let created: Vec<&str> = created
            .components
            .get::<Tags>()
            .map(|t| t.tags.iter().map(|t| t.0.as_str()).collect())
            .unwrap_or_default();
        let wanted: Vec<&str> = tags.to_vec();
        assert!(
            created.iter().filter(|t| wanted.contains(t)).count() == wanted.len(),
            "the tags are appended to a created component",
        );
        let present = step_1_26_0(
            identifier,
            &format!(r#""minecraft:tags": {{"tags": ["own", "{}"]}}"#, tags[0]),
        );
        let present: Vec<String> = present
            .components
            .get::<Tags>()
            .unwrap()
            .tags
            .iter()
            .map(|t| t.0.clone())
            .collect();
        assert!(present[0] == "own", "the author's tags stay first");
        assert!(
            present.iter().filter(|t| *t == tags[0]).count() == 1,
            "a present tag is not appended again",
        );
    }
    let custom = step_1_26_0("test:ocean", "");
    assert!(
        custom.components.get::<Tags>().is_none(),
        "a custom biome gets none",
    );
}

#[test]
fn village_type_default() {
    village(
        &[
            "minecraft:plains",
            "minecraft:sunflower_plains",
            "minecraft:meadow",
        ],
        VillageKind::Default,
    )
}

#[test]
fn village_type_desert() {
    village(&["minecraft:desert"], VillageKind::Desert)
}

#[test]
fn village_type_ice() {
    village(&["minecraft:ice_plains"], VillageKind::Ice)
}

#[test]
fn village_type_taiga() {
    village(
        &[
            "minecraft:taiga",
            "minecraft:taiga_hills",
            "minecraft:cold_taiga",
            "minecraft:cold_taiga_hills",
        ],
        VillageKind::Taiga,
    )
}

#[test]
fn village_type_savanna() {
    village(&["minecraft:savanna"], VillageKind::Savanna)
}

#[test]
fn tags_oceans() {
    tags(
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
    )
}

#[test]
fn tags_swamps_slime() {
    tags(
        &[
            "minecraft:swampland",
            "minecraft:swampland_mutated",
            "minecraft:mangrove_swamp",
        ],
        &["slime"],
    )
}

#[test]
fn tags_swamps_mushroom() {
    tags(
        &["minecraft:swampland", "minecraft:swampland_mutated"],
        &["swamp_water_huge_mushroom"],
    )
}

#[test]
fn tags_mesas() {
    tags(
        &[
            "minecraft:mesa",
            "minecraft:mesa_plateau_stone",
            "minecraft:mesa_plateau",
            "minecraft:mesa_bryce",
            "minecraft:mesa_plateau_stone_mutated",
            "minecraft:mesa_plateau_mutated",
        ],
        &["surface_mineshaft"],
    )
}
