//! The upgrade path and the versions and validation of client biomes, one test per row.

use bedrock_addon::diagnostic::{Code, Severity};
use bedrock_addon::document::{ComponentRef, Content, Document};
use bedrock_addon::families::client_biome::component::{
    AmbientSounds, BiomeMusic, GrassAppearance, WaterAppearance,
};
use bedrock_addon::families::client_biome::{Beta, ClientBiome};
use bedrock_addon::load::{BiomeRenderIdentifiers, LoadContext};
use bedrock_addon::version::{BaseGameVersion, ENGINE_VERSION};
use indexmap::IndexMap;
use std::fmt::Write as _;
use std::sync::Arc;

use super::common::{
    PATH, base_game, beta_apis, biome, component, components, context, count, document, errors,
    fixture, fixtures, has, has_at, json, load, load_beta, loads, rejected, texts, world, written,
};

/// The names of the components a step inserted.
fn synthesized(document: &Document<ClientBiome>) -> Vec<&str> {
    document
        .components
        .iter()
        .filter_map(|entry| match entry {
            ComponentRef::Typed {
                name,
                synthesized: true,
                ..
            } => Some(name),
            _ => None,
        })
        .collect()
}

/// The component names a document writes back, in order.
fn names(document: &Document<ClientBiome>) -> Vec<String> {
    components(document)
        .as_object()
        .unwrap()
        .keys()
        .map(|key| key.as_str().to_owned())
        .collect()
}

const WATER: &str = r##""minecraft:water_appearance": {"surface_color": "#617B64"}"##;
const RENDER_IDS: [&str; 4] = [
    "minecraft:atmosphere_identifier",
    "minecraft:color_grading_identifier",
    "minecraft:lighting_identifier",
    "minecraft:water_identifier",
];

// ---- F.1 -----------------------------------------------------------------------------------

/// `cb01_plains_1_21_40_oracle`: vanilla's 1.21.40 plains file upgrades to the payload of
/// vanilla's current one. The inserted `ambient_sounds.addition` reads as the constructor's
/// default through its accessor and is not written.
#[test]
fn cb01_plains_1_21_40_oracle() {
    let loaded = loads(
        &fixture("client_biome/cb01_plains_1_21_40/input.json"),
        &context(),
    );
    let expected = json(&fixture("client_biome/cb01_plains_1_21_40/expected.json"));
    let expected = expected.as_object().unwrap().get("minecraft:client_biome");
    let payload = written(&loaded);
    assert_eq!(
        payload.as_object().unwrap().get("minecraft:client_biome"),
        expected
    );
    assert_eq!(loaded.description.identifier.as_str(), "minecraft:plains");
    let sounds = loaded.components.get::<AmbientSounds>().unwrap();
    assert_eq!(sounds.addition().asset.as_str(), "");
    assert_eq!(sounds.addition().chance, 1.0);
    assert_eq!(sounds.underwater_mood(), "");
    assert!(loaded.version().upgraded);
    assert_eq!(
        // By legacy name: the variant and the definitions that were it.
        count(loaded.diagnostics(), |c| matches!(
            c.name(),
            "IdentifierNamespaced"
        )),
        1
    );
    assert!(
        loaded
            .diagnostics()
            .iter()
            .all(|d| d.severity == Severity::Info)
    );
}

/// `cb02_swampland_tables`: all three colour tables of the 1.21.70 step.
#[test]
fn cb02_swampland_tables() {
    let loaded = loads(&biome("1.21.40", "minecraft:swampland", WATER), &context());
    assert_eq!(
        component(&loaded, "minecraft:foliage_appearance"),
        Some(json(r#"{"color": {"color_map": "swamp_foliage"}}"#))
    );
    assert_eq!(
        component(&loaded, "minecraft:grass_appearance"),
        Some(json(r#"{"color": {"color_map": "swamp_grass"}}"#))
    );
    assert!(
        !loaded
            .components
            .get::<GrassAppearance>()
            .unwrap()
            .grass_is_shaded()
    );
    assert_eq!(
        component(&loaded, "minecraft:dry_foliage_color"),
        Some(json(r##"{"color": "#7b5334"}"##))
    );
    let inserted = synthesized(&loaded);
    for name in [
        "minecraft:foliage_appearance",
        "minecraft:grass_appearance",
        "minecraft:dry_foliage_color",
    ] {
        assert!(inserted.contains(&name), "{name} is synthesized");
        assert!(has_at(
            loaded.diagnostics(),
            // By legacy name: the variant and the definitions that were it.
            |c| matches!(c.name(), "SynthesizedComponent"),
            Severity::Info,
            &format!("/minecraft:client_biome/components/{name}"),
        ));
    }
    assert!(!inserted.contains(&"minecraft:water_appearance"));
}

/// `cb03_swampland_bare`: the tables compare the identifier as written, so a bare `swampland`
/// gets none of the three; the identifier is namespaced by the next step.
#[test]
fn cb03_swampland_bare() {
    let loaded = loads(&biome("1.21.40", "swampland", WATER), &context());
    assert_eq!(
        loaded.description.identifier.as_str(),
        "minecraft:swampland"
    );
    for name in [
        "minecraft:foliage_appearance",
        "minecraft:grass_appearance",
        "minecraft:dry_foliage_color",
    ] {
        assert_eq!(component(&loaded, name), None, "{name}");
        assert!(has_at(
            loaded.diagnostics(),
            // By legacy name: the variant and the definitions that were it.
            |c| matches!(c.name(), "UpgradeTableMissedBareIdentifier"),
            Severity::Info,
            &format!("/minecraft:client_biome/components/{name}"),
        ));
    }
    assert_eq!(
        // By legacy name: the variant and the definitions that were it.
        count(loaded.diagnostics(), |c| matches!(
            c.name(),
            "UpgradeTableMissedBareIdentifier"
        )),
        3
    );
    // A bare name no table has misses nothing.
    let plain = loads(&biome("1.21.40", "plains", WATER), &context());
    assert_eq!(
        // By legacy name: the variant and the definitions that were it.
        count(plain.diagnostics(), |c| matches!(
            c.name(),
            "UpgradeTableMissedBareIdentifier"
        )),
        0
    );
}

/// `cb04_mesa_tables`: a component the document has is kept; an absent one is inserted.
#[test]
fn cb04_mesa_tables() {
    let loaded = loads(
        &biome(
            "1.21.50",
            "minecraft:mesa",
            r##""minecraft:grass_appearance": {"color": "#112233"}"##,
        ),
        &context(),
    );
    assert_eq!(
        component(&loaded, "minecraft:foliage_appearance"),
        Some(json(r##"{"color": "#aea42a"}"##))
    );
    assert_eq!(
        component(&loaded, "minecraft:grass_appearance"),
        Some(json(r##"{"color": "#112233"}"##))
    );
    // The other mesas, the cherry grove and the pale garden, by their table rows.
    for (identifier, foliage, grass) in [
        ("minecraft:mesa_bryce", "#aea42a", "#90814d"),
        ("minecraft:mesa_plateau_mutated", "#aea42a", "#90814d"),
        ("minecraft:mesa_plateau_stone", "#aea42a", "#90814d"),
        ("minecraft:mesa_plateau_stone_mutated", "#aea42a", "#90814d"),
        ("minecraft:cherry_grove", "#b6db61", "#b6db61"),
        ("minecraft:pale_garden", "#878d76", "#778272"),
    ] {
        let loaded = loads(&biome("1.21.40", identifier, ""), &context());
        assert_eq!(
            component(&loaded, "minecraft:foliage_appearance"),
            Some(json(&format!(r#"{{"color": "{foliage}"}}"#))),
            "{identifier}"
        );
        assert_eq!(
            component(&loaded, "minecraft:grass_appearance"),
            Some(json(&format!(r#"{{"color": "{grass}"}}"#))),
            "{identifier}"
        );
    }
    let mangrove = loads(
        &biome("1.21.40", "minecraft:mangrove_swamp", ""),
        &context(),
    );
    assert_eq!(
        component(&mangrove, "minecraft:foliage_appearance"),
        Some(json(
            r#"{"color": {"color_map": "mangrove_swamp_foliage"}}"#
        ))
    );
    assert_eq!(
        component(&mangrove, "minecraft:grass_appearance"),
        Some(json(r#"{"color": {"color_map": "swamp_grass"}}"#))
    );
    let mutated = loads(
        &biome("1.21.40", "minecraft:swampland_mutated", ""),
        &context(),
    );
    assert_eq!(
        component(&mutated, "minecraft:foliage_appearance"),
        Some(json(r#"{"color": {"color_map": "swamp_foliage"}}"#))
    );
}

/// `cb05_mesa_plateau_not_in_table`.
#[test]
fn cb05_mesa_plateau_not_in_table() {
    let loaded = loads(&biome("1.21.40", "minecraft:mesa_plateau", ""), &context());
    assert_eq!(component(&loaded, "minecraft:foliage_appearance"), None);
    assert_eq!(component(&loaded, "minecraft:grass_appearance"), None);
    assert_eq!(component(&loaded, "minecraft:dry_foliage_color"), None);
}

/// `cb06_dry_foliage_gate`: 1.21.60 is not below 1.21.60, so only the dry foliage (below
/// 1.21.70) is inserted; 1.21.59 still gets all three.
#[test]
fn cb06_dry_foliage_gate() {
    let loaded = loads(&biome("1.21.60", "minecraft:pale_garden", ""), &context());
    assert_eq!(
        component(&loaded, "minecraft:dry_foliage_color"),
        Some(json(r##"{"color": "#a0a69c"}"##))
    );
    assert_eq!(component(&loaded, "minecraft:foliage_appearance"), None);
    assert_eq!(component(&loaded, "minecraft:grass_appearance"), None);
    // 1.21.69 is the last version below `DRY_FOLIAGE_UPGRADE_BEFORE`.
    let last = loads(&biome("1.21.69", "minecraft:pale_garden", ""), &context());
    assert_eq!(
        component(&last, "minecraft:dry_foliage_color"),
        Some(json(r##"{"color": "#a0a69c"}"##))
    );
    let below = loads(&biome("1.21.59", "minecraft:pale_garden", ""), &context());
    assert!(component(&below, "minecraft:foliage_appearance").is_some());
    assert!(component(&below, "minecraft:grass_appearance").is_some());
    // The other dry-foliage rows, and a 1.21.70 document, which is past the gate.
    for identifier in [
        "minecraft:swampland",
        "minecraft:swampland_mutated",
        "minecraft:mangrove_swamp",
        "minecraft:roofed_forest",
        "minecraft:roofed_forest_mutated",
    ] {
        let loaded = loads(&biome("1.21.60", identifier, ""), &context());
        assert_eq!(
            component(&loaded, "minecraft:dry_foliage_color"),
            Some(json(r##"{"color": "#7b5334"}"##)),
            "{identifier}"
        );
    }
    let later = loads(&biome("1.21.70", "minecraft:pale_garden", ""), &context());
    assert_eq!(component(&later, "minecraft:dry_foliage_color"), None);
    let kept = loads(
        &biome(
            "1.21.40",
            "minecraft:pale_garden",
            r#""minecraft:foliage_appearance": {"color": [1, 2, 3]}"#,
        ),
        &context(),
    );
    assert_eq!(
        component(&kept, "minecraft:foliage_appearance"),
        Some(json(r#"{"color": [1, 2, 3]}"#))
    );
}

/// A context whose renderer knows `myns:glade`.
fn glade_render_identifiers() -> LoadContext {
    let mut cx = context();
    let mut map = IndexMap::new();
    map.insert(
        bedrock_addon::id::Id::parse("myns:glade"),
        BiomeRenderIdentifiers {
            atmosphere: bedrock_addon::id::Id::parse("myns:a"),
            color_grading: bedrock_addon::id::Id::parse("myns:c"),
            lighting: bedrock_addon::id::Id::parse("myns:l"),
            water: bedrock_addon::id::Id::parse("myns:w"),
        },
    );
    cx.client_render_identifiers = Some(Arc::new(map));
    cx
}

/// `cb07_render_ids_custom`: each identifier has its own gate; the values come from the load
/// context (ours; vanilla reads the client's `DeferredRenderingBiomeRegistry`).
#[test]
fn cb07_render_ids_custom_deviation() {
    let cx = glade_render_identifiers();
    let loaded = loads(&biome("1.21.70", "myns:glade", ""), &cx);
    assert_eq!(
        component(&loaded, "minecraft:lighting_identifier"),
        Some(json(r#"{"lighting_identifier": "myns:l"}"#))
    );
    for name in [
        "minecraft:atmosphere_identifier",
        "minecraft:color_grading_identifier",
        "minecraft:water_identifier",
    ] {
        assert_eq!(
            component(&loaded, name),
            None,
            "1.21.70 is not below 1.21.70: {name}"
        );
    }
    // 1.21.80 is not below 1.21.80.
    let later = loads(&biome("1.21.80", "myns:glade", ""), &cx);
    assert_eq!(component(&later, "minecraft:lighting_identifier"), None);
    let last = loads(&biome("1.21.79", "myns:glade", ""), &cx);
    assert!(component(&last, "minecraft:lighting_identifier").is_some());
    // Below 1.21.70 all four, in vanilla's order, each from its own field.
    let old = loads(&biome("1.21.60", "myns:glade", ""), &cx);
    assert_eq!(names(&old), RENDER_IDS);
    assert_eq!(
        components(&old),
        json(
            r#"{"minecraft:atmosphere_identifier": {"atmosphere_identifier": "myns:a"},
                "minecraft:color_grading_identifier": {"color_grading_identifier": "myns:c"},
                "minecraft:lighting_identifier": {"lighting_identifier": "myns:l"},
                "minecraft:water_identifier": {"water_identifier": "myns:w"}}"#
        )
    );
    // A biome the supplied map does not list gets none; one already present is kept.
    let unlisted = loads(&biome("1.21.60", "myns:other", ""), &cx);
    assert!(names(&unlisted).is_empty());
    let kept = loads(
        &biome(
            "1.21.70",
            "myns:glade",
            r#""minecraft:lighting_identifier": {"lighting_identifier": "mine:l"}"#,
        ),
        &cx,
    );
    assert!(synthesized(&kept).is_empty());
    assert_eq!(
        component(&kept, "minecraft:lighting_identifier"),
        Some(json(r#"{"lighting_identifier": "mine:l"}"#))
    );
    // Without a map every biome gets vanilla's defaults.
    let defaults = loads(&biome("1.21.60", "myns:glade", ""), &context());
    assert_eq!(
        components(&defaults),
        json(
            r#"{"minecraft:atmosphere_identifier": {"atmosphere_identifier": "minecraft:default_atmospherics"},
                "minecraft:color_grading_identifier": {"color_grading_identifier": "minecraft:default_color_grading"},
                "minecraft:lighting_identifier": {"lighting_identifier": "minecraft:default_lighting"},
                "minecraft:water_identifier": {"water_identifier": "minecraft:default_water"}}"#
        )
    );
}

/// The `music_definition` an upgraded document of `identifier` has in a world.
fn music(identifier: &str, cx: &LoadContext) -> Option<String> {
    let loaded = loads(&biome("1.21.70", identifier, ""), cx);
    loaded
        .components
        .get::<BiomeMusic>()
        .and_then(|music| music.music_definition.clone())
        .map(|definition| definition.as_str().to_owned())
}

/// `cb08_music_world_gate`: `deep_dark` came with 1.19.0; the world's version is the load
/// context's (ours; vanilla: the joined world's).
#[test]
fn cb08_music_world_gate_deviation() {
    assert_eq!(music("minecraft:deep_dark", &world(1, 0, 0)), None);
    assert_eq!(music("minecraft:deep_dark", &world(1, 18, 30)), None);
    let deep_dark = Some("deep_dark".to_owned());
    assert_eq!(music("minecraft:deep_dark", &world(1, 19, 0)), deep_dark);
    let mut any = context();
    any.world_base_game_version = BaseGameVersion::ANY;
    assert_eq!(music("minecraft:deep_dark", &any), deep_dark);
    let engine = world(
        ENGINE_VERSION.major,
        ENGINE_VERSION.minor,
        ENGINE_VERSION.patch,
    );
    assert_eq!(music("minecraft:deep_dark", &engine), deep_dark);
    // No world version: every entry applies (ours).
    assert_eq!(music("minecraft:deep_dark", &context()), deep_dark);
    let mut never = context();
    never.world_base_game_version = BaseGameVersion::INCOMPATIBLE;
    assert_eq!(music("minecraft:deep_dark", &never), None);
    // 1.21.80 is not below `MUSIC_UPGRADE_BEFORE`.
    let later = loads(&biome("1.21.80", "minecraft:deep_dark", ""), &context());
    assert_eq!(component(&later, "minecraft:biome_music"), None);
    let last = loads(&biome("1.21.79", "minecraft:deep_dark", ""), &context());
    assert!(component(&last, "minecraft:biome_music").is_some());
}

/// `cb08b_music_since_boundaries`: each group of the table at its own world version.
#[test]
fn cb08b_music_since_boundaries() {
    for (identifier, before, since, definition) in [
        ("minecraft:the_end", (1, 13, 0), (1, 14, 0), "end"),
        ("minecraft:hell", (1, 15, 0), (1, 16, 0), "hell"),
        ("minecraft:meadow", (1, 17, 40), (1, 18, 0), "meadow"),
        ("minecraft:forest", (1, 18, 30), (1, 19, 0), "forest"),
        (
            "minecraft:basalt_deltas",
            (1, 19, 20),
            (1, 19, 30),
            "basalt_deltas",
        ),
        (
            "minecraft:cherry_grove",
            (1, 19, 80),
            (1, 20, 0),
            "cherry_grove",
        ),
    ] {
        let (a, b, c) = before;
        assert_eq!(music(identifier, &world(a, b, c)), None, "{identifier}");
        let (a, b, c) = since;
        assert_eq!(
            music(identifier, &world(a, b, c)),
            Some(definition.to_owned()),
            "{identifier}"
        );
    }
}

/// Every key of the table: 43, each with its definition (its own name but for the End's and
/// the warped forest's).
#[test]
fn the_music_table_has_vanillas_43_keys() {
    let keys = [
        "the_end",
        "endboss",
        "nether",
        "crimson_forest",
        "hell",
        "soulsand_valley",
        "dripstone_caves",
        "frozen_peaks",
        "grove",
        "jagged_peaks",
        "lush_caves",
        "meadow",
        "snowy_slopes",
        "stony_peaks",
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
        "basalt_deltas",
        "cherry_grove",
        "desert",
        "desert_hills",
        "desert_mutated",
        "jungle_edge",
        "mesa",
        "warped_forest",
    ];
    assert_eq!(keys.len(), 43);
    let cx = context();
    for key in keys {
        let expected = match key {
            "the_end" => "end",
            "warped_forest" => "",
            other => other,
        };
        assert_eq!(
            music(&format!("minecraft:{key}"), &cx).as_deref(),
            Some(expected),
            "{key}"
        );
        // The table is keyed by the `minecraft:` name.
        assert_eq!(music(&format!("myns:{key}"), &cx), None, "{key}");
    }
    assert_eq!(music("minecraft:plains", &cx), None);
    assert_eq!(music("minecraft:sulfur_caves", &cx), None);
}

/// `cb08c_music_literal_keys`.
#[test]
fn cb08c_music_literal_keys() {
    let engine = world(
        ENGINE_VERSION.major,
        ENGINE_VERSION.minor,
        ENGINE_VERSION.patch,
    );
    assert_eq!(
        music("minecraft:endboss", &engine),
        Some("endboss".to_owned())
    );
    assert_eq!(
        music("minecraft:nether", &engine),
        Some("nether".to_owned())
    );
}

/// `cb08d_warped_empty_definition`: an empty definition is inserted all the same.
#[test]
fn cb08d_warped_empty_definition() {
    let loaded = loads(
        &biome("1.21.70", "minecraft:warped_forest", ""),
        &world(1, 21, 0),
    );
    assert_eq!(
        component(&loaded, "minecraft:biome_music"),
        Some(json(r#"{"music_definition": ""}"#))
    );
    let before = loads(
        &biome("1.21.70", "minecraft:warped_forest", ""),
        &world(1, 20, 80),
    );
    assert_eq!(component(&before, "minecraft:biome_music"), None);
}

/// `cb09_music_keeps_existing`; a component without a definition gets one.
#[test]
fn cb09_music_keeps_existing() {
    let loaded = loads(
        &biome(
            "1.21.70",
            "minecraft:forest",
            r#""minecraft:biome_music": {"music_definition": "mine"}"#,
        ),
        &context(),
    );
    assert_eq!(
        component(&loaded, "minecraft:biome_music"),
        Some(json(r#"{"music_definition": "mine"}"#))
    );
    assert!(!synthesized(&loaded).contains(&"minecraft:biome_music"));
    let filled = loads(
        &biome(
            "1.21.70",
            "minecraft:forest",
            r#""minecraft:biome_music": {"volume_multiplier": 0.5}"#,
        ),
        &context(),
    );
    assert_eq!(
        component(&filled, "minecraft:biome_music"),
        Some(json(
            r#"{"volume_multiplier": 0.5, "music_definition": "forest"}"#
        ))
    );
    assert!(has_at(
        filled.diagnostics(),
        // By legacy name: the variant and the definitions that were it.
        |c| matches!(c.name(), "SynthesizedComponent"),
        Severity::Info,
        "/minecraft:client_biome/components/minecraft:biome_music",
    ));
    // The author's component stays the author's.
    assert!(!synthesized(&filled).contains(&"minecraft:biome_music"));
    // A biome without a row keeps its component as written.
    let untouched = loads(
        &biome(
            "1.21.70",
            "minecraft:plains",
            r#""minecraft:biome_music": {"volume_multiplier": 0.5}"#,
        ),
        &context(),
    );
    assert_eq!(
        component(&untouched, "minecraft:biome_music"),
        Some(json(r#"{"volume_multiplier": 0.5}"#))
    );
}

/// `cb10_roofed_forest_shade`: both branches.
#[test]
fn cb10_roofed_forest_shade() {
    let loaded = loads(
        &biome(
            "1.21.90",
            "minecraft:roofed_forest",
            r##""minecraft:grass_appearance": {"color": "#507a32"}"##,
        ),
        &context(),
    );
    assert_eq!(
        component(&loaded, "minecraft:grass_appearance"),
        Some(json(r##"{"color": "#507a32", "grass_is_shaded": true}"##))
    );
    let inserted = loads(&biome("1.21.90", "minecraft:roofed_forest", ""), &context());
    assert_eq!(
        component(&inserted, "minecraft:grass_appearance"),
        Some(json(r#"{"grass_is_shaded": true}"#))
    );
    assert!(synthesized(&inserted).contains(&"minecraft:grass_appearance"));
    assert!(
        inserted
            .components
            .get::<GrassAppearance>()
            .unwrap()
            .grass_is_shaded()
    );
    // A 1.21.100 document is past the step.
    let later = loads(
        &biome("1.21.100", "minecraft:roofed_forest", ""),
        &context(),
    );
    assert_eq!(component(&later, "minecraft:grass_appearance"), None);
}

/// `cb11_roofed_forest_mutated_no_shade`.
#[test]
fn cb11_roofed_forest_mutated_no_shade() {
    let loaded = loads(
        &biome(
            "1.21.90",
            "minecraft:roofed_forest_mutated",
            r##""minecraft:grass_appearance": {"color": "#507a32"}"##,
        ),
        &context(),
    );
    assert_eq!(
        component(&loaded, "minecraft:grass_appearance"),
        Some(json(r##"{"color": "#507a32"}"##))
    );
    let absent = loads(
        &biome("1.21.90", "minecraft:roofed_forest_mutated", ""),
        &context(),
    );
    assert_eq!(component(&absent, "minecraft:grass_appearance"), None);
}

/// `cb12_nether_precipitation`: exactly one member each.
#[test]
fn cb12_nether_precipitation() {
    for (identifier, expected) in [
        ("minecraft:basalt_deltas", r#"{"white_ash": 2.0}"#),
        ("minecraft:crimson_forest", r#"{"red_spores": 0.25}"#),
        ("minecraft:soulsand_valley", r#"{"ash": 0.05}"#),
        ("minecraft:warped_forest", r#"{"blue_spores": 0.25}"#),
    ] {
        let loaded = loads(&biome("1.21.100", identifier, ""), &context());
        assert_eq!(
            component(&loaded, "minecraft:precipitation"),
            Some(json(expected)),
            "{identifier}"
        );
    }
    let other = loads(&biome("1.21.100", "minecraft:hell", ""), &context());
    assert_eq!(component(&other, "minecraft:precipitation"), None);
    let later = loads(
        &biome("1.21.110", "minecraft:basalt_deltas", ""),
        &context(),
    );
    assert_eq!(component(&later, "minecraft:precipitation"), None);
}

/// `cb13_ocean_underwater_music`: both branches, and the thirteen water biomes.
#[test]
fn cb13_ocean_underwater_music() {
    let loaded = loads(
        &biome(
            "1.21.100",
            "minecraft:ocean",
            r#""minecraft:biome_music": {"volume_multiplier": 0.5}"#,
        ),
        &context(),
    );
    assert_eq!(
        component(&loaded, "minecraft:biome_music"),
        Some(json(
            r#"{"volume_multiplier": 0.5, "underwater_music": true}"#
        ))
    );
    for identifier in [
        "legacy_frozen_ocean",
        "ocean",
        "deep_ocean",
        "warm_ocean",
        "deep_warm_ocean",
        "lukewarm_ocean",
        "deep_lukewarm_ocean",
        "cold_ocean",
        "deep_cold_ocean",
        "frozen_ocean",
        "deep_frozen_ocean",
        "river",
        "frozen_river",
    ] {
        let inserted = loads(
            &biome("1.21.100", &format!("minecraft:{identifier}"), ""),
            &context(),
        );
        assert_eq!(
            component(&inserted, "minecraft:biome_music"),
            Some(json(r#"{"underwater_music": true}"#)),
            "{identifier}"
        );
        assert!(
            inserted
                .components
                .get::<BiomeMusic>()
                .unwrap()
                .underwater_music()
        );
    }
    let land = loads(
        &biome(
            "1.21.100",
            "minecraft:plains",
            r#""minecraft:biome_music": {"volume_multiplier": 0.5}"#,
        ),
        &context(),
    );
    assert!(
        !land
            .components
            .get::<BiomeMusic>()
            .unwrap()
            .underwater_music()
    );
    let later = loads(&biome("1.21.110", "minecraft:ocean", ""), &context());
    assert_eq!(component(&later, "minecraft:biome_music"), None);
}

/// `cb14_custom_ambient_copy`: the example of The members the example writes
/// as `""` stay absent and read `""`.
#[test]
fn cb14_custom_ambient_copy() {
    let loaded = loads(
        &biome(
            "1.21.110",
            "myns:glade",
            r#""minecraft:ambient_sounds": {"addition": "ambient.cave", "loop": "ambient.glade.loop"}"#,
        ),
        &context(),
    );
    assert_eq!(
        component(&loaded, "minecraft:ambient_sounds"),
        Some(json(
            r#"{"addition": {"asset": "ambient.cave", "chance": 0.0111}, "loop": "ambient.glade.loop",
                "underwater_addition": {"asset": "ambient.cave", "chance": 0.0111},
                "underwater_loop": "ambient.glade.loop"}"#
        ))
    );
    let sounds = loaded.components.get::<AmbientSounds>().unwrap();
    assert_eq!(sounds.mood(), "");
    assert_eq!(sounds.underwater_mood(), "");
    assert_eq!(sounds.loop_sound(), "ambient.glade.loop");
    // `mood` is copied too; a custom biome without the component gets none.
    let mood = loads(
        &biome(
            "1.21.110",
            "myns:glade",
            r#""minecraft:ambient_sounds": {"mood": "ambient.m"}"#,
        ),
        &context(),
    );
    assert_eq!(
        component(&mood, "minecraft:ambient_sounds"),
        Some(json(
            r#"{"mood": "ambient.m", "underwater_mood": "ambient.m"}"#
        ))
    );
    let none = loads(&biome("1.21.110", "myns:glade", ""), &context());
    assert_eq!(component(&none, "minecraft:ambient_sounds"), None);
}

/// `cb15_vanilla_ambient_overwrite`.
#[test]
fn cb15_vanilla_ambient_overwrite() {
    let loaded = loads(
        &biome(
            "1.21.110",
            "minecraft:river",
            r#""minecraft:ambient_sounds": {"addition": "x", "loop": "y"}"#,
        ),
        &context(),
    );
    assert_eq!(
        component(&loaded, "minecraft:ambient_sounds"),
        Some(json(
            r#"{"addition": {"asset": "x", "chance": 0.0111}, "loop": "y",
                "underwater_addition": {"asset": "ambient.underwater.additions", "chance": 0.01},
                "underwater_loop": "ambient.underwater.loop"}"#
        ))
    );
    assert_eq!(
        loaded
            .components
            .get::<AmbientSounds>()
            .unwrap()
            .underwater_mood(),
        ""
    );
    // Absent: the component is inserted with the underwater ambience only.
    let inserted = loads(&biome("1.21.110", "minecraft:river", ""), &context());
    assert_eq!(
        component(&inserted, "minecraft:ambient_sounds"),
        Some(json(
            r#"{"underwater_addition": {"asset": "ambient.underwater.additions", "chance": 0.01},
                "underwater_loop": "ambient.underwater.loop"}"#
        ))
    );
    assert!(synthesized(&inserted).contains(&"minecraft:ambient_sounds"));
}

/// An `addition` the older document left out or wrote empty stays absent (ours; vanilla keeps
/// chance 0.0111 beside the empty name, which has no 1.21.120 JSON).
#[test]
fn an_empty_addition_stays_absent_deviation() {
    let loaded = loads(
        &biome(
            "1.21.110",
            "myns:glade",
            r#""minecraft:ambient_sounds": {"addition": "", "loop": "l"}"#,
        ),
        &context(),
    );
    assert_eq!(
        component(&loaded, "minecraft:ambient_sounds"),
        Some(json(r#"{"loop": "l", "underwater_loop": "l"}"#))
    );
    // What is written loads again.
    let again = loads(&loaded.to_json().unwrap(), &context());
    assert_eq!(again.value(), loaded.value());
}

/// The 1.21.100 grass type has `grass_is_shaded = false` in vanilla, whether the 1.21.70 step
/// inserted the component or the 1.21.100 step converted the author's; ours leaves the
/// member absent, and the accessor reads the `false` vanilla stores.
#[test]
fn an_upgraded_grass_appearance_has_no_grass_is_shaded_member_deviation() {
    // Inserted by the 1.21.70 step (swampland's table).
    let inserted = loads(&biome("1.21.40", "minecraft:swampland", WATER), &context());
    // Converted by the 1.21.100 step (any biome but `roofed_forest`).
    let converted = loads(
        &biome(
            "1.21.90",
            "myns:glade",
            r##""minecraft:grass_appearance": {"color": "#507a32"}"##,
        ),
        &context(),
    );
    assert_eq!(
        component(&converted, "minecraft:grass_appearance"),
        Some(json(r##"{"color": "#507a32"}"##))
    );
    for loaded in [&inserted, &converted] {
        let written = component(loaded, "minecraft:grass_appearance").unwrap();
        assert!(
            !written.as_object().unwrap().contains_key("grass_is_shaded"),
            "{written:?}"
        );
        let grass = loaded.components.get::<GrassAppearance>().unwrap();
        assert!(!grass.grass_is_shaded());
        // What is written loads again, the member still absent.
        let again = loads(&loaded.to_json().unwrap(), &context());
        assert_eq!(
            component(&again, "minecraft:grass_appearance"),
            Some(written)
        );
        let grass = again.components.get::<GrassAppearance>().unwrap();
        assert!(!grass.grass_is_shaded());
    }
}

/// The 1.21.110 music type has `underwater_music = false` in vanilla for a biome that is no
/// water biome; ours leaves the member absent, and the accessor reads that `false`.
#[test]
fn an_upgraded_biome_music_has_no_underwater_music_member_deviation() {
    let loaded = loads(
        &biome(
            "1.21.100",
            "minecraft:warped_forest",
            r#""minecraft:biome_music": {"volume_multiplier": 0.5, "music_definition": "m"}"#,
        ),
        &context(),
    );
    assert_eq!(
        component(&loaded, "minecraft:biome_music"),
        Some(json(
            r#"{"volume_multiplier": 0.5, "music_definition": "m"}"#
        ))
    );
    assert!(
        !loaded
            .components
            .get::<BiomeMusic>()
            .unwrap()
            .underwater_music()
    );
    // What is written loads again, the member still absent.
    let again = loads(&loaded.to_json().unwrap(), &context());
    assert_eq!(
        component(&again, "minecraft:biome_music"),
        component(&loaded, "minecraft:biome_music")
    );
    assert!(
        !again
            .components
            .get::<BiomeMusic>()
            .unwrap()
            .underwater_music()
    );
}

/// `cb16_1_21_120_no_steps`.
#[test]
fn cb16_1_21_120_no_steps() {
    let loaded = loads(
        &biome(
            "1.21.120",
            "minecraft:river",
            r#""minecraft:ambient_sounds": {"addition": {"asset": "a", "chance": 0.5}}"#,
        ),
        &context(),
    );
    assert_eq!(
        components(&loaded),
        json(r#"{"minecraft:ambient_sounds": {"addition": {"asset": "a", "chance": 0.5}}}"#)
    );
    assert!(synthesized(&loaded).is_empty());
    assert!(
        loaded.diagnostics().is_empty(),
        "{:?}",
        texts(loaded.diagnostics())
    );
    assert!(loaded.version().upgraded);
}

/// `cb17_all_vanilla_samples`: every client biome of the game's 1.26.50 sample packs loads without
/// a warning, and writes back what it read.
#[test]
fn cb17_all_vanilla_samples() {
    let files = fixtures("client_biome/cb17_all_vanilla_samples");
    assert_eq!(files.len(), 89);
    let mut stamps = Vec::new();
    for (name, text) in &files {
        let loaded = load(text, &base_game()).unwrap_or_else(|r| panic!("{name}: {r}"));
        assert!(
            loaded
                .diagnostics()
                .iter()
                .all(|d| d.severity < Severity::Warning),
            "{name}: {:?}",
            texts(loaded.diagnostics())
        );
        let source = json(text);
        assert_eq!(
            written(&loaded)
                .as_object()
                .unwrap()
                .get("minecraft:client_biome"),
            source.as_object().unwrap().get("minecraft:client_biome"),
            "{name}"
        );
        if loaded.version().written.is_some_and(|v| v.minor == 26) {
            stamps.push(name.as_str());
        }
    }
    assert_eq!(
        stamps,
        [
            "dappled_forest.client_biome.json",
            "sulfur_caves.client_biome.json"
        ]
    );
}

/// `cb18_all_vanilla_1_21_80`: vanilla's first client biomes (1.21.40, identifiers without a
/// namespace), here from retail's `vanilla_1.21.40` slice, load through the whole chain.
#[test]
fn cb18_all_vanilla_1_21_40() {
    let files = fixtures("client_biome/cb18_all_vanilla_1_21_40");
    assert_eq!(files.len(), 86);
    for (name, text) in &files {
        let loaded = load(text, &base_game()).unwrap_or_else(|r| panic!("{name}: {r}"));
        assert!(
            loaded
                .diagnostics()
                .iter()
                .all(|d| d.severity < Severity::Warning),
            "{name}: {:?}",
            texts(loaded.diagnostics())
        );
        let identifier = loaded.description.identifier.as_str();
        assert_eq!(
            format!(
                "{}.client_biome.json",
                identifier.trim_start_matches("minecraft:")
            ),
            *name
        );
        // The slice holds 1.21.40, 1.21.60, 1.21.70 and 1.21.100 files: each gets the
        // identifiers its own version is below the gate of.
        let version = loaded.version().written.unwrap();
        let present = names(&loaded);
        for id in RENDER_IDS {
            let gate = if id == "minecraft:lighting_identifier" {
                80
            } else {
                70
            };
            if version.patch < gate {
                assert!(present.iter().any(|n| n == id), "{name}: {id}");
            } else {
                assert!(!synthesized(&loaded).contains(&id), "{name}: {id}");
            }
        }
        let sounds = loaded.components.get::<AmbientSounds>().expect(name);
        assert_eq!(sounds.underwater_loop(), "ambient.underwater.loop");
        assert_eq!(sounds.underwater_addition().chance, 0.01);
        // What the chain gave is a document the newest struct reads back unchanged.
        let again = loads(&loaded.to_json().unwrap(), &base_game());
        assert_eq!(written(&again), written(&loaded), "{name}");
        assert!(again.diagnostics().is_empty(), "{name}");
    }
}

// ---- F.2 -----------------------------------------------------------------------------------

fn unsupported(code: &Code) -> bool {
    // By legacy name: the variant and the definitions that were it.
    matches!(code.name(), "UnsupportedVersion")
}

/// `cb20_below_floor`: from 1.0.0 to the floor the legacy parser refuses in its own words.
#[test]
fn cb20_below_floor() {
    for version in ["1.21.30", "1.0.0", "1.21.39"] {
        let found = rejected(&biome(version, "minecraft:plains", ""), &context());
        assert!(
            has(
                &found,
                unsupported,
                Severity::Error,
                &format!("{PATH}: Cannot load client_biome with format_version {version}"),
            ),
            "{:?}",
            texts(&found)
        );
    }
    loads(&biome("1.21.40", "minecraft:plains", ""), &context());
}

/// `cb21_below_one`: below the legacy slice Puv has no parser.
#[test]
fn cb21_below_one() {
    let found = rejected(&biome("0.9.0", "minecraft:plains", ""), &context());
    assert!(
        has(
            &found,
            unsupported,
            Severity::Error,
            "unsupported version: no parser available for version 0.9.0",
        ),
        "{:?}",
        texts(&found)
    );
    let found = rejected(&biome("0.99.99", "minecraft:plains", ""), &context());
    assert!(has(
        &found,
        unsupported,
        Severity::Error,
        "unsupported version: no parser available for version 0.99.99",
    ));
}

/// `cb22_above_engine`: the last range ends at the engine version, inclusive.
#[test]
fn cb22_above_engine() {
    let engine = ENGINE_VERSION;
    loads(
        &biome(&engine.to_string(), "minecraft:plains", ""),
        &context(),
    );
    let above = format!("{}.{}.{}", engine.major, engine.minor, engine.patch + 1);
    let found = rejected(&biome(&above, "minecraft:plains", ""), &context());
    assert!(has(
        &found,
        unsupported,
        Severity::Error,
        &format!("unsupported version: no parser available for version {above}"),
    ));
}

const BETA: &str = r##"{"description": {"identifier": "myns:glade"},
    "components": {"minecraft:sky_color": {"sky_color": "#102030"}},
    "environment_attributes": {"minecraft:experimental_float_attr_1": {"operation": "add", "value": 1.0}}}"##;

/// `cb23_beta_literal` (ours follows retail 1.26.45; vanilla 1.26.20 has no beta parser).
#[test]
fn cb23_beta_literal_deviation() {
    let text = document("beta", BETA);
    let refused = load_beta(&text, &context()).unwrap_err().diagnostics;
    assert!(
        has(
            &refused,
            // By legacy name: the variant and the definitions that were it.
            |c| matches!(c.name(), "BetaFeaturesNotEnabled"),
            Severity::Error,
            "Using a beta format_version is only allowed for base packs or when the BetaApis toggle is enabled.",
        ),
        "{:?}",
        texts(&refused)
    );
    for cx in [beta_apis(), base_game()] {
        let loaded = load_beta(&text, &cx).unwrap_or_else(|r| panic!("{r}"));
        assert!(loaded.version().beta);
        assert!(!loaded.version().upgraded);
        let attributes = loaded.environment_attributes.as_ref().unwrap();
        assert_eq!(
            attributes.experimental_float_attr_1.as_ref().unwrap().value,
            1.0
        );
        // Unchanged: written back as read, and read back equal.
        let payload = json(&loaded.to_json().unwrap());
        assert_eq!(
            payload.as_object().unwrap().get("minecraft:client_biome"),
            Some(&json(BETA))
        );
        assert_eq!(
            payload.as_object().unwrap().get("format_version"),
            Some(&"beta".into())
        );
        let again = load_beta(&loaded.to_json().unwrap(), &cx).unwrap();
        assert_eq!(again.value(), loaded.value());
        // The stable document's parser has none for it.
        let stable = rejected(&text, &cx);
        assert!(has(
            &stable,
            unsupported,
            Severity::Error,
            "unsupported version: no parser available for version beta",
        ));
    }
    // A stable document is no beta document.
    assert!(load_beta(&biome("1.21.130", "myns:glade", ""), &beta_apis()).is_err());
}

/// A beta document gets none of the steps' insertions.
#[test]
fn a_beta_document_is_never_upgraded() {
    let text = document(
        "beta",
        r#"{"description": {"identifier": "minecraft:swampland"}, "components": {}}"#,
    );
    let loaded = load_beta(&text, &beta_apis()).unwrap();
    assert!(loaded.components.is_empty());
    assert!(loaded.diagnostics().is_empty());
}

/// `cb23a_beta_attr_invalid`.
#[test]
fn cb23a_beta_attr_invalid() {
    let with = |attributes: &str| {
        document(
            "beta",
            &format!(
                r#"{{"description": {{"identifier": "myns:glade"}}, "components": {{}}, "environment_attributes": {{{attributes}}}}}"#
            ),
        )
    };
    let failure = |attributes: &str| {
        load_beta(&with(attributes), &beta_apis())
            .unwrap_err()
            .diagnostics
    };
    let missing = failure(r#""minecraft:experimental_bool_attr": {"value": true}"#);
    assert!(
        has_at(
            &missing,
            // By legacy name: the variant and the definitions that were it.
            |c| matches!(c.name(), "MissingRequiredField"),
            Severity::Error,
            "/minecraft:client_biome/environment_attributes/minecraft:experimental_bool_attr/operation",
        ),
        "{:?}",
        texts(&missing)
    );
    let color = failure(
        r##""minecraft:experimental_color_attr": {"value": "#FFFFFFFF", "operation": "minimum"}"##,
    );
    assert!(
        // By legacy name: the variant and the definitions that were it.
        count(&color, |c| matches!(c.name(), "InvalidEnumValue")) == 1,
        "{:?}",
        texts(&color)
    );
    let float =
        failure(r#""minecraft:experimental_float_attr_1": {"value": 0.5, "operation": "xor"}"#);
    assert_eq!(
        // By legacy name: the variant and the definitions that were it.
        count(&float, |c| matches!(c.name(), "InvalidEnumValue")),
        1
    );
    // `minimum` is a float operation and `xor` a bool one.
    load_beta(
        &with(
            r#""minecraft:experimental_float_attr_2": {"value": 0.5, "operation": "minimum"},
               "minecraft:experimental_bool_attr": {"value": true, "operation": "xor"}"#,
        ),
        &beta_apis(),
    )
    .unwrap();
    // An unknown attribute is an unknown member of the beta struct: an error under its
    // 0xfffc, and the document is refused.
    let unknown = load_beta(
        &with(r#""minecraft:other": {"value": 1, "operation": "add"}"#),
        &beta_apis(),
    )
    .expect_err("refused");
    assert!(has_at(
        &unknown.diagnostics,
        // By legacy name: the variant and the definitions that were it.
        |c| matches!(c.name(), "UnrecognizedField"),
        Severity::Error,
        "/minecraft:client_biome/environment_attributes/minecraft:other",
    ));
}

/// `cb23b_1_26_stamps`: retail's 1.26.0 and 1.26.30 files are read by the 1.21.130
/// struct. (Their fogs and the server's biome list are the linking pass's.)
#[test]
fn cb23b_1_26_stamps() {
    for (name, text) in fixtures("client_biome/cb23b_1_26_stamps") {
        let loaded = load(&text, &base_game()).unwrap_or_else(|r| panic!("{name}: {r}"));
        assert!(loaded.diagnostics().is_empty(), "{name}");
        assert_eq!(
            loaded.version().parsed_as,
            bedrock_addon::version::FormatVersion::new(1, 21, 130)
        );
        assert!(!loaded.version().upgraded);
        if name.starts_with("dappled_forest") {
            let water = loaded.components.get::<WaterAppearance>().unwrap();
            assert_eq!(water.surface_opacity, Some(1.0));
        }
    }
}

/// `cb24_missing_version`.
#[test]
fn cb24_missing_version() {
    let found = rejected(
        r#"{"minecraft:client_biome": {"description": {"identifier": "a:b"}, "components": {}}}"#,
        &context(),
    );
    assert!(
        has(
            &found,
            // By legacy name: the variant and the definitions that were it.
            |c| matches!(c.name(), "MissingFormatVersion"),
            Severity::Error,
            "no valid version passed by the caller or found in the input",
        ),
        "{:?}",
        texts(&found)
    );
}

/// `cb25_numeric_version` and `cb26_invalid_version`.
#[test]
fn cb25_cb26_malformed_version() {
    // The number `1.21` is read as its text, 1.21.0: below
    // the floor, the legacy parser's "Cannot load client_biome with format_version 1.21.0".
    let numeric = rejected(
        r#"{"format_version": 1.21, "minecraft:client_biome": {"description": {"identifier": "a:b"}, "components": {}}}"#,
        &context(),
    );
    assert!(
        has(
            &numeric,
            // By legacy name: the variant and the definitions that were it.
            |c| matches!(c.name(), "UnsupportedVersion"),
            Severity::Error,
            "biomes/demo.client_biome.json: Cannot load client_biome with format_version 1.21.0",
        ),
        "{:?}",
        texts(&numeric)
    );
    for version in [r#""1.21.x""#, "true", r#""""#] {
        let text = format!(
            r#"{{"format_version": {version}, "minecraft:client_biome": {{"description": {{"identifier": "a:b"}}, "components": {{}}}}}}"#
        );
        let found = rejected(&text, &context());
        assert!(
            has(
                &found,
                // By legacy name: the variant and the definitions that were it.
                |c| matches!(c.name(), "MalformedFormatVersion"),
                Severity::Error,
                "no valid version passed by the caller or found in the input",
            ),
            "{version}: {:?}",
            texts(&found)
        );
    }
}

/// `cb26b_legacy_syntax_error`: below 1.21.130 the legacy parser's text; from it, Puv's.
#[test]
fn cb26b_legacy_syntax_error() {
    let broken = |version: &str| {
        format!(
            r#"{{"format_version": "{version}" "minecraft:client_biome": {{"description": {{"identifier": "a:b"}}, "components": {{}}}}}}"#
        )
    };
    let found = rejected(&broken("1.21.40"), &context());
    let error = errors(&found)[0];
    // By legacy name: the variant and the definitions that were it.
    assert!(matches!(error.code.name(), "MalformedInput"));
    assert!(
        error
            .code
            .to_string()
            .starts_with("failed to parse file into a json document: "),
        "{}",
        error.code
    );
    assert!(error.code.to_string().len() > "failed to parse file into a json document: ".len());
    let last = rejected(&broken("1.21.129"), &context());
    assert!(
        errors(&last)[0]
            .code
            .to_string()
            .starts_with("failed to parse file into a json")
    );
    let modern = rejected(&broken("1.21.130"), &context());
    assert!(
        errors(&modern)[0]
            .code
            .to_string()
            .starts_with("Failed to parse the Minecraft Document JSON: "),
        "{}",
        errors(&modern)[0].code
    );
}

/// `cb26c_legacy_comments`.
#[test]
fn cb26c_legacy_comments() {
    let text = r#"// a line comment
    {"format_version": "1.21.40", /* a block comment */
     "minecraft:client_biome": {"description": {"identifier": "plains"}, "components": {}}}"#;
    let loaded = loads(text, &context());
    assert_eq!(loaded.description.identifier.as_str(), "minecraft:plains");
}

/// `cb26d_legacy_root_not_object`.
#[test]
fn cb26d_legacy_root_not_object() {
    let text = format!("[{}]", biome("1.21.40", "plains", ""));
    let found = rejected(&text, &context());
    assert!(
        has(
            &found,
            // By legacy name: the variant and the definitions that were it.
            |c| matches!(c.name(), "MalformedInput"),
            Severity::Error,
            &format!(
                "failed to parse client biome file {PATH} into a json document, root value needs to be a json object"
            ),
        ),
        "{:?}",
        texts(&found)
    );
}

/// `cb27_between_120_and_130`.
#[test]
fn cb27_between_120_and_130() {
    for version in ["1.21.125", "1.21.121", "1.21.129"] {
        let loaded = loads(
            &biome(
                version,
                "myns:glade",
                r#""minecraft:ambient_sounds": {"addition": {"asset": "a", "chance": 0.5}}"#,
            ),
            &context(),
        );
        assert_eq!(
            loaded.version().parsed_as,
            bedrock_addon::version::FormatVersion::new(1, 21, 120)
        );
        assert!(has_at(
            loaded.diagnostics(),
            // By legacy name: the variant and the definitions that were it.
            |c| matches!(c.name(), "UnusualFormatVersion"),
            Severity::Info,
            "/format_version",
        ));
    }
    for version in ["1.21.120", "1.21.130", "1.21.110"] {
        let loaded = loads(&biome(version, "myns:glade", ""), &context());
        assert_eq!(
            // By legacy name: the variant and the definitions that were it.
            count(loaded.diagnostics(), |c| matches!(
                c.name(),
                "UnusualFormatVersion"
            )),
            0,
            "{version}"
        );
    }
}

/// A pre-release orders before its release: `1.21.130-beta.1` is a legacy document.
#[test]
fn a_prerelease_takes_the_legacy_path() {
    let loaded = loads(&biome("1.21.130-beta.1", "myns:glade", ""), &context());
    assert_eq!(
        loaded.version().parsed_as,
        bedrock_addon::version::FormatVersion::new(1, 21, 120)
    );
    assert_eq!(
        // By legacy name: the variant and the definitions that were it.
        count(loaded.diagnostics(), |c| matches!(
            c.name(),
            "PrereleaseFormatVersion"
        )),
        1
    );
}

const NAMESPACE_TEXT: &str =
    "identifier has an upper case letter or does not specify any namespace";

/// `cb28_bare_identifier_1_21_90`: from 1.21.90 the identifier needs a namespace.
#[test]
fn cb28_bare_identifier_1_21_90() {
    for (version, identifier) in [
        ("1.21.90", "glade"),
        ("1.21.130", "glade"),
        ("1.21.90", "Myns:glade"),
        ("1.21.90", "myns:"),
        ("1.21.90", ":glade"),
        ("1.21.90", "a,b:c"),
        ("1.21.120", "a/b:c"),
    ] {
        let found = rejected(&biome(version, identifier, ""), &context());
        assert!(
            has_at(
                &found,
                // By legacy name: the variant and the definitions that were it.
                |c| matches!(c.name(), "NamespaceRule"),
                Severity::Error,
                "/minecraft:client_biome/description/identifier",
            ) && has(&found, |_| true, Severity::Error, NAMESPACE_TEXT),
            "{version} {identifier}: {:?}",
            texts(&found)
        );
    }
    loads(&biome("1.21.90", "my.n_s%+-:gl.a_d%e+-", ""), &context());
    loads(&biome("1.21.89", "glade", ""), &context());
}

/// `cb29_uppercase_1_21_40`.
#[test]
fn cb29_uppercase_1_21_40() {
    for (version, identifier) in [("1.21.40", "Glade"), ("1.21.70", "gl ade"), ("1.21.40", "")] {
        let found = rejected(&biome(version, identifier, ""), &context());
        assert!(
            has(
                &found,
                // By legacy name: the variant and the definitions that were it.
                |c| matches!(c.name(), "InvalidValue"),
                Severity::Error,
                "identifier has an upper case letter",
            ),
            "{:?}",
            texts(&found)
        );
    }
}

/// `cb30_slash_allowed_1_21_40`: in the older regex `+-:` is a range, which has `,` and `/`.
#[test]
fn cb30_slash_allowed_1_21_40() {
    for identifier in ["a/b", "a,b", "a:b:c", "a+b-c.d_e%f0"] {
        loads(&biome("1.21.40", identifier, ""), &context());
    }
    // `*` (below `+`) and `;` (above `:`) are outside the range.
    for identifier in ["a*b", "a;b"] {
        rejected(&biome("1.21.40", identifier, ""), &context());
    }
}

/// `cb31_water_empty`, in every version.
#[test]
fn cb31_water_empty() {
    for version in ["1.21.40", "1.21.120", "1.21.130"] {
        let found = rejected(
            &biome(version, "myns:glade", r#""minecraft:water_appearance": {}"#),
            &context(),
        );
        assert!(
            has(
                &found,
                // By legacy name: the variant and the definitions that were it.
                |c| matches!(c.name(), "InvalidValue"),
                Severity::Error,
                "No surface color or opacity specified. Specify one or remove the component.",
            ),
            "{:?}",
            texts(&found)
        );
    }
    for members in [
        r##""surface_color": "#102030""##,
        r#""surface_opacity": 0.5"#,
    ] {
        loads(
            &biome(
                "1.21.130",
                "myns:glade",
                &format!(r#""minecraft:water_appearance": {{{members}}}"#),
            ),
            &context(),
        );
    }
}

/// `cb32_precipitation_too_early`, and the other components by their first version (ours
/// names the version; vanilla's strict load calls the component unknown; the document is not
/// loaded either way).
#[test]
fn cb32_precipitation_too_early_deviation() {
    for (version, component, body, since) in [
        (
            "1.21.100",
            "minecraft:precipitation",
            r#"{"ash": 1.0}"#,
            "1.21.110",
        ),
        (
            "1.21.109",
            "minecraft:precipitation",
            r#"{"ash": 1.0}"#,
            "1.21.110",
        ),
        (
            "1.21.69",
            "minecraft:atmosphere_identifier",
            r#"{"atmosphere_identifier": "a"}"#,
            "1.21.70",
        ),
        (
            "1.21.40",
            "minecraft:color_grading_identifier",
            r#"{"color_grading_identifier": "a"}"#,
            "1.21.70",
        ),
        (
            "1.21.40",
            "minecraft:dry_foliage_color",
            r##"{"color": "#102030"}"##,
            "1.21.70",
        ),
        (
            "1.21.40",
            "minecraft:lighting_identifier",
            r#"{"lighting_identifier": "a"}"#,
            "1.21.70",
        ),
        (
            "1.21.40",
            "minecraft:water_identifier",
            r#"{"water_identifier": "a"}"#,
            "1.21.70",
        ),
        (
            "1.21.120",
            "minecraft:cubemap_identifier",
            r#"{"cubemap_identifier": "a"}"#,
            "1.21.130",
        ),
    ] {
        let found = rejected(
            &biome(version, "myns:glade", &format!(r#""{component}": {body}"#)),
            &context(),
        );
        assert!(
            has(
                &found,
                unsupported,
                Severity::Error,
                &format!(
                    "To use component '{component}', use json format version {since} or higher"
                ),
            ),
            "{version} {component}: {:?}",
            texts(&found)
        );
        // Raised while the struct is read: a finding of the parse.
        assert!(
            found
                .iter()
                .filter(|f| unsupported(&f.code))
                .all(|f| f.phase == bedrock_addon::diagnostic::Phase::Parse),
            "{version} {component}"
        );
    }
    loads(
        &biome(
            "1.21.110",
            "myns:glade",
            r#""minecraft:precipitation": {"ash": 1.0}"#,
        ),
        &context(),
    );
    loads(
        &biome(
            "1.21.70",
            "myns:glade",
            r##""minecraft:atmosphere_identifier": {"atmosphere_identifier": "a"},
               "minecraft:color_grading_identifier": {"color_grading_identifier": "a"},
               "minecraft:dry_foliage_color": {"color": "#102030"},
               "minecraft:lighting_identifier": {"lighting_identifier": "a"},
               "minecraft:water_identifier": {"water_identifier": "a"}"##,
        ),
        &context(),
    );
}

/// `cb33_cubemap_1_21_130`.
#[test]
fn cb33_cubemap_1_21_130() {
    let loaded = loads(
        &biome(
            "1.21.130",
            "myns:glade",
            r#""minecraft:cubemap_identifier": {"cubemap_identifier": "myns:sky"}"#,
        ),
        &context(),
    );
    assert_eq!(
        component(&loaded, "minecraft:cubemap_identifier"),
        Some(json(r#"{"cubemap_identifier": "myns:sky"}"#))
    );
    assert!(loaded.diagnostics().is_empty());
}

/// `cb34_unknown_component`: every client-biome slice reads with 0xfffc, so an unknown
/// component is an error and the document is refused, in the legacy slice as in the 1.21.130
/// one.
#[test]
fn cb34_unknown_component() {
    for version in ["1.21.130", "1.21.40"] {
        let found = rejected(
            &biome(version, "myns:glade", r#""minecraft:foo": {"a": [1, 2]}"#),
            &context(),
        );
        assert!(
            has_at(
                &found,
                // By legacy name: the variant and the definitions that were it.
                |c| matches!(c.name(), "UnrecognizedComponent"),
                Severity::Error,
                "/minecraft:client_biome/components/minecraft:foo",
            ),
            "{version}: {:?}",
            texts(&found)
        );
    }
}

/// Unknown members are errors and the document is refused, as vanilla's strict loaders do.
#[test]
fn unknown_members_refuse_the_client_biome() {
    let text = r##"{"format_version": "1.21.130", "minecraft:client_biome": {
        "description": {"identifier": "myns:glade", "note": 1},
        "components": {"minecraft:sky_color": {"sky_color": "#102030", "extra": true}},
        "tag": "x"}}"##;
    let found = rejected(text, &context());
    for at in [
        "/minecraft:client_biome/description/note",
        "/minecraft:client_biome/components/minecraft:sky_color/extra",
        "/minecraft:client_biome/tag",
    ] {
        assert!(
            has_at(
                &found,
                // By legacy name: the variant and the definitions that were it.
                |c| matches!(c.name(), "UnrecognizedField"),
                Severity::Error,
                at
            ),
            "{at}: {:?}",
            texts(&found)
        );
    }
    // A third root member of a document below 1.21.130 is an unknown member of its struct.
    let legacy = rejected(
        r#"{"format_version": "1.21.40", "minecraft:client_biome": {"description": {"identifier": "a:b"}, "components": {}}, "note": 1}"#,
        &context(),
    );
    assert!(has_at(
        &legacy,
        // By legacy name: the variant and the definitions that were it.
        |c| matches!(c.name(), "UnrecognizedField"),
        Severity::Error,
        "/note",
    ));
}

/// `cb35_string_addition_1_21_120`: from 1.21.120 `addition` is `{asset, chance}`; before, a
/// plain sound.
#[test]
fn cb35_string_addition_1_21_120() {
    let found = rejected(
        &biome(
            "1.21.120",
            "myns:glade",
            r#""minecraft:ambient_sounds": {"addition": "x"}"#,
        ),
        &context(),
    );
    assert!(
        has_at(
            &found,
            // By legacy name: the variant and the definitions that were it.
            |c| matches!(c.name(), "InvalidType"),
            Severity::Error,
            "/minecraft:client_biome/components/minecraft:ambient_sounds/addition",
        ),
        "{:?}",
        texts(&found)
    );
    let older = rejected(
        &biome(
            "1.21.110",
            "myns:glade",
            r#""minecraft:ambient_sounds": {"addition": {"asset": "x", "chance": 0.5}}"#,
        ),
        &context(),
    );
    assert!(
        has_at(
            &older,
            // By legacy name: the variant and the definitions that were it.
            |c| matches!(c.name(), "InvalidType"),
            Severity::Error,
            "/minecraft:client_biome/components/minecraft:ambient_sounds/addition",
        ),
        "{:?}",
        texts(&older)
    );
}

/// `cb36_chance_range`: `chance` lies in `[0, 1]`, both ends included.
#[test]
fn cb36_chance_range() {
    let with = |member: &str, chance: &str| {
        biome(
            "1.21.120",
            "myns:glade",
            &format!(
                r#""minecraft:ambient_sounds": {{"{member}": {{"asset": "x", "chance": {chance}}}}}"#
            ),
        )
    };
    for member in ["addition", "underwater_addition"] {
        for chance in ["1.5", "-0.1", "1.0001"] {
            let found = rejected(&with(member, chance), &context());
            assert!(
                has_at(
                    &found,
                    // By legacy name: the variant and the definitions that were it.
                    |c| matches!(c.name(), "InvalidValue"),
                    Severity::Error,
                    &format!(
                        "/minecraft:client_biome/components/minecraft:ambient_sounds/{member}/chance"
                    ),
                ),
                "{member} {chance}: {:?}",
                texts(&found)
            );
        }
        for chance in ["0", "1", "0.0", "1.0", "0.5"] {
            loads(&with(member, chance), &context());
        }
    }
}

/// `cb37_empty_asset`.
#[test]
fn cb37_empty_asset() {
    for member in ["addition", "underwater_addition"] {
        let found = rejected(
            &biome(
                "1.21.130",
                "myns:glade",
                &format!(
                    r#""minecraft:ambient_sounds": {{"{member}": {{"asset": "", "chance": 0.5}}}}"#
                ),
            ),
            &context(),
        );
        assert!(
            has_at(
                &found,
                // By legacy name: the variant and the definitions that were it.
                |c| matches!(c.name(), "InvalidValue"),
                Severity::Error,
                &format!(
                    "/minecraft:client_biome/components/minecraft:ambient_sounds/{member}/asset"
                ),
            ),
            "{:?}",
            texts(&found)
        );
    }
    // Both members are required.
    for body in [r#"{"asset": "x"}"#, r#"{"chance": 0.5}"#] {
        let found = rejected(
            &biome(
                "1.21.130",
                "myns:glade",
                &format!(r#""minecraft:ambient_sounds": {{"addition": {body}}}"#),
            ),
            &context(),
        );
        assert_eq!(
            // By legacy name: the variant and the definitions that were it.
            count(&found, |c| matches!(c.name(), "MissingRequiredField")),
            1
        );
    }
}

/// `cb38_envelope_extra_member`.
#[test]
fn cb38_envelope_extra_member() {
    let found = rejected(
        r#"{"format_version": "1.21.130", "minecraft:client_biome": {"description": {"identifier": "a:b"}, "components": {}}, "note": 1}"#,
        &context(),
    );
    assert!(
        errors(&found)[0]
            .code
            .to_string()
            .starts_with("A Minecraft Document must have exactly two members"),
        "{:?}",
        texts(&found)
    );
}

/// The newest document on its own: `Beta::parse` and `ClientBiome::parse` name their kinds.
#[test]
fn the_content_names_and_sides() {
    use bedrock_addon::pack::PackSide;
    assert_eq!(<ClientBiome as Content>::NAME, "client biome");
    assert_eq!(<ClientBiome as Content>::SIDES, &[PackSide::Resource]);
    assert_eq!(<Beta as Content>::SIDES, &[PackSide::Resource]);
}

/// The findings of cb23a (variants 2 and 3), cb29, cb31, cb32 and cb38 by code, severity and
/// path.
#[test]
fn the_rejections_name_their_code_severity_and_member() {
    let one =
        |found: &bedrock_addon::diagnostic::Diagnostics, code: fn(&Code) -> bool, at: &str| {
            let hits: Vec<_> = found.iter().filter(|d| code(&d.code)).collect();
            assert_eq!(hits.len(), 1, "{at}: {:?}", texts(found));
            assert_eq!(hits[0].severity, Severity::Error, "{at}");
            assert_eq!(hits[0].location.json_path.to_string(), at);
        };
    // cb23a (2), (3): the operation is not one of the attribute's enum.
    for (attribute, body) in [
        (
            "minecraft:experimental_color_attr",
            r##"{"value": "#FFFFFFFF", "operation": "minimum"}"##,
        ),
        (
            "minecraft:experimental_float_attr_1",
            r#"{"value": 0.5, "operation": "xor"}"#,
        ),
    ] {
        let text = document(
            "beta",
            &format!(
                r#"{{"description": {{"identifier": "myns:glade"}}, "components": {{}}, "environment_attributes": {{"{attribute}": {body}}}}}"#
            ),
        );
        let found = load_beta(&text, &beta_apis()).unwrap_err().diagnostics;
        one(
            &found,
            // By legacy name: the variant and the definitions that were it.
            |c| matches!(c.name(), "InvalidEnumValue"),
            &format!("minecraft:client_biome/environment_attributes/{attribute}/operation"),
        );
    }
    // cb29
    let found = rejected(&biome("1.21.40", "Glade", ""), &context());
    one(
        &found,
        // By legacy name: the variant and the definitions that were it.
        |c| matches!(c.name(), "InvalidValue"),
        "minecraft:client_biome/description/identifier",
    );
    // cb31
    let found = rejected(
        &biome(
            "1.21.130",
            "myns:glade",
            r#""minecraft:water_appearance": {}"#,
        ),
        &context(),
    );
    one(
        &found,
        // By legacy name: the variant and the definitions that were it.
        |c| matches!(c.name(), "InvalidValue"),
        "minecraft:client_biome/components/minecraft:water_appearance",
    );
    // cb32
    let found = rejected(
        &biome(
            "1.21.100",
            "myns:glade",
            r#""minecraft:precipitation": {"ash": 1.0}"#,
        ),
        &context(),
    );
    one(
        &found,
        // By legacy name: the variant and the definitions that were it.
        |c| matches!(c.name(), "UnsupportedVersion"),
        "minecraft:client_biome/components/minecraft:precipitation",
    );
    // cb38: the envelope's finding is the document's.
    let found = rejected(
        r#"{"format_version": "1.21.130", "minecraft:client_biome": {"description": {"identifier": "a:b"}, "components": {}}, "note": 1}"#,
        &context(),
    );
    assert_eq!(errors(&found).len(), 1, "{:?}", texts(&found));
    // By legacy name: the variant and the definitions that were it.
    one(&found, |c| matches!(c.name(), "MalformedInput"), "");
}

/// cb17 / cb23b, the other half: vanilla's 1.26 stamps load because the last range ends at
/// the engine version. `ENGINE_VERSION` is a constant of core, so the row's "with
/// `ENGINE_VERSION` below the stamp" is shown from the other side: vanilla's
/// `dappled_forest` is stamped 1.26.50 and loads, restamped with
/// the engine version itself it loads, and one patch above it has no parser.
#[test]
fn a_stamp_above_the_engine_version_has_no_parser() {
    let sample = fixture("client_biome/cb17_all_vanilla_samples/dappled_forest.client_biome.json");
    assert!(sample.contains(r#""format_version": "1.26.50""#));
    loads(&sample, &base_game());
    let engine = ENGINE_VERSION.to_string();
    let text = sample.replace("1.26.50", &engine);
    loads(&text, &base_game());
    let above = format!(
        "{}.{}.{}",
        ENGINE_VERSION.major,
        ENGINE_VERSION.minor,
        ENGINE_VERSION.patch + 1
    );
    let found = rejected(&text.replace(&engine, &above), &base_game());
    let hits: Vec<_> = found.iter().filter(|d| unsupported(&d.code)).collect();
    assert_eq!(hits.len(), 1, "{:?}", texts(&found));
    assert_eq!(hits[0].severity, Severity::Error);
    assert_eq!(hits[0].location.json_path.to_string(), "format_version");
    assert_eq!(
        hits[0].code.to_string(),
        format!("unsupported version: no parser available for version {above}")
    );
    // Retail's 1.26.0 and 1.26.30 stamps are below the engine version.
    for (_, text) in fixtures("client_biome/cb23b_1_26_stamps") {
        let version = loads(&text, &base_game()).version().written.unwrap();
        assert!(version <= ENGINE_VERSION && version.minor == 26);
    }
}

/// cb18's golden output: per file, the identifier and each component the chain gave, with
/// `+` before one a step inserted. `BEDROCK_ADDON_BLESS=1` rewrites the file.
#[test]
fn cb18_golden_output() {
    let mut lines = String::new();
    for (name, text) in fixtures("client_biome/cb18_all_vanilla_1_21_40") {
        let loaded = loads(&text, &base_game());
        let written = loaded.version().written.unwrap();
        writeln!(
            lines,
            "{name} {written} -> {}",
            loaded.description.identifier.as_str()
        )
        .unwrap();
        for entry in loaded.components.iter() {
            if let ComponentRef::Typed {
                name, synthesized, ..
            } = entry
            {
                let json = component(&loaded, name).unwrap();
                writeln!(
                    lines,
                    "  {}{name} {}",
                    if synthesized { "+" } else { " " },
                    facet_json::to_string(&json).unwrap()
                )
                .unwrap();
            }
        }
    }
    let path = super::common::fixture_dir("client_biome/cb18_all_vanilla_1_21_40.golden");
    if std::env::var_os("BEDROCK_ADDON_BLESS").is_some() {
        std::fs::write(&path, &lines).unwrap();
    }
    let golden = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(
        lines == golden,
        "the chain's output differs from {}",
        path.display()
    );
    assert_eq!(golden.lines().filter(|l| !l.starts_with(' ')).count(), 86);
}

/// Every `*_BEFORE` constant of the chain, at the last version below it and at
/// the version itself: a document one patch below the gate gets the component, a document at
/// the gate does not.
#[test]
fn every_gate_opens_one_patch_below_its_constant_and_closes_at_it() {
    // (constant, its version's patch in 1.21.x, the biome, the component a step inserts)
    let gates = [
        (
            "FOLIAGE_UPGRADE_BEFORE",
            60,
            "minecraft:pale_garden",
            "minecraft:foliage_appearance",
        ),
        (
            "GRASS_UPGRADE_BEFORE",
            60,
            "minecraft:pale_garden",
            "minecraft:grass_appearance",
        ),
        (
            "DRY_FOLIAGE_UPGRADE_BEFORE",
            70,
            "minecraft:pale_garden",
            "minecraft:dry_foliage_color",
        ),
        (
            "ATMOSPHERE_IDENTIFIER_UPGRADE_BEFORE",
            70,
            "myns:glade",
            "minecraft:atmosphere_identifier",
        ),
        (
            "COLOR_GRADING_IDENTIFIER_UPGRADE_BEFORE",
            70,
            "myns:glade",
            "minecraft:color_grading_identifier",
        ),
        (
            "WATER_IDENTIFIER_UPGRADE_BEFORE",
            70,
            "myns:glade",
            "minecraft:water_identifier",
        ),
        (
            "LIGHTING_IDENTIFIER_UPGRADE_BEFORE",
            80,
            "myns:glade",
            "minecraft:lighting_identifier",
        ),
        (
            "MUSIC_UPGRADE_BEFORE",
            80,
            "minecraft:cherry_grove",
            "minecraft:biome_music",
        ),
        (
            "GRASS_SHADE_UPGRADE_BEFORE",
            100,
            "minecraft:roofed_forest",
            "minecraft:grass_appearance",
        ),
        (
            "PRECIPITATION_COMPONENT_BEFORE",
            110,
            "minecraft:basalt_deltas",
            "minecraft:precipitation",
        ),
        (
            "UNDERWATER_MUSIC_UPGRADE_BEFORE",
            110,
            "minecraft:ocean",
            "minecraft:biome_music",
        ),
        (
            "UNDERWATER_AMBIENT_SOUNDS_COMPONENT_BEFORE",
            120,
            "minecraft:river",
            "minecraft:ambient_sounds",
        ),
    ];
    assert_eq!(gates.len(), 12);
    for (constant, gate, identifier, name) in gates {
        let last = loads(
            &biome(&format!("1.21.{}", gate - 1), identifier, ""),
            &context(),
        );
        assert!(
            synthesized(&last).contains(&name),
            "{constant}: 1.21.{} is below the gate",
            gate - 1
        );
        let at = loads(&biome(&format!("1.21.{gate}"), identifier, ""), &context());
        assert_eq!(
            component(&at, name),
            None,
            "{constant}: 1.21.{gate} is the gate"
        );
    }
}
