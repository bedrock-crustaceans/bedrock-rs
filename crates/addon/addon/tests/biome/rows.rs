//! Every row of the biome fixtures, one test each, named after the row.

use bedrock_addon::diagnostic::{DiagnosticCode, Diagnostics, Severity, VanillaText};
use bedrock_addon::document::{ComponentRef, Content, JsonDocument as _};
use bedrock_addon::families::biome::component::{
    Builder, Climate, NoiseType, OverworldHeight, ReplaceBiomes, SubsurfaceBuilder, SurfaceBuilder,
    Tags, VillageKind, VillageType,
};
use bedrock_addon::families::biome::{Beta, Biome, history};
use bedrock_addon::load::LoadContext;
use bedrock_addon::version::{BaseGameVersion, FormatVersion, SemVersion};
use std::fmt::Write as _;

use super::common::{
    base_game, beta_apis, biome, context, document, errors, fixture, fixtures, has, load,
    load_beta, loads, rejected, texts, with,
};

/// Writes `biome` and reads the text back strictly: the read holds the same components and
/// description (a synthesized component reads back as written), and writing it again gives
/// the same text (value level).
#[track_caller]
pub fn round_trip(biome: &Biome) {
    let written = biome.to_json().expect("the biome writes");
    let again = loads(&written, &context());
    assert!(
        errors(again.diagnostics()).is_empty()
            && texts(again.diagnostics(), Severity::Warning).is_empty(),
        "the rewrite reads back cleanly: {:?}",
        again.diagnostics()
    );
    let entries = |b: &Biome| -> Vec<String> {
        b.components
            .iter()
            .map(|entry| match entry {
                ComponentRef::Typed { name, value, .. } => format!("{name} {value:?}"),
                other => format!("{other:?}"),
            })
            .collect()
    };
    assert_eq!(
        entries(again.value()),
        entries(biome),
        "the rewrite reads back equal"
    );
    assert_eq!(again.value().description, biome.description);
    assert_eq!(again.value().extra, biome.extra);
    assert_eq!(again.to_json().unwrap(), written, "the rewrite is stable");
}

/// `b04_1_21_110_floor`: vanilla `plains.biome.json` of format 1.21.110 loads, upgraded to the
/// newest struct.
// The file is the BDS 1.26.36.1 `behavior_packs/vanilla/biomes/plains.biome.json` (format
// 1.21.110).
#[test]
fn b04_1_21_110_floor() {
    let document = loads(&fixture("b04_1_21_110_floor/plains.biome.json"), &context());
    assert!(errors(document.diagnostics()).is_empty());
    assert!(
        texts(document.diagnostics(), Severity::Warning).is_empty(),
        "{:?}",
        texts(document.diagnostics(), Severity::Warning)
    );
    assert_eq!(
        document.version().written,
        Some(FormatVersion::new(1, 21, 110))
    );
    assert_eq!(document.version().parsed_as, FormatVersion::new(1, 21, 110));
    assert!(document.version().upgraded);
    let biome = document.value();
    assert_eq!(biome.description.identifier.as_str(), "minecraft:plains");
    let climate = biome.components.get::<Climate>().expect("climate");
    assert_eq!(climate.downfall.map(|d| d.0), Some(0.4));
    assert_eq!(
        climate.snow_accumulation.map(|[a, b]| (a.0, b.0)),
        Some((0.0, 0.125))
    );
    assert_eq!(
        biome
            .components
            .get::<OverworldHeight>()
            .unwrap()
            .noise_type,
        Some(NoiseType::Lowlands)
    );
    let Builder::Overworld(surface) = &biome.components.get::<SurfaceBuilder>().unwrap().builder
    else {
        panic!("an overworld builder")
    };
    assert_eq!(surface.sea_floor_depth.0, 7);
    assert_eq!(surface.top_material.name.as_str(), "minecraft:grass_block");
    assert!(
        biome
            .components
            .get::<Tags>()
            .unwrap()
            .contains("bee_habitat")
    );
    // The 1.26.0 step gives the vanilla plains its village type, synthesized.
    assert_eq!(
        biome.components.get::<VillageType>().map(|v| v.kind),
        Some(VillageKind::Default)
    );
    assert!(biome.components.iter().any(|entry| matches!(
        entry,
        ComponentRef::Typed {
            name: "minecraft:village_type",
            synthesized: true,
            ..
        }
    )));
    round_trip(biome);
}

/// The `UnsupportedVersion` "no parser available" error of `version`, then our hint.
#[track_caller]
fn below_floor(diagnostics: &Diagnostics, version: &str) -> DiagnosticCode {
    let unsupported = with(diagnostics, |c| {
        matches!(
            c,
            DiagnosticCode::UnsupportedVersion {
                text: Some(VanillaText::NoParserForVersion { .. })
            }
        )
    });
    assert_eq!(unsupported.len(), 1, "{diagnostics:#?}");
    assert_eq!(unsupported[0].severity, Severity::Error);
    assert_eq!(
        unsupported[0].code.to_string(),
        format!("unsupported version: no parser available for version {version}")
    );
    let hints = with(diagnostics, |c| {
        matches!(c, DiagnosticCode::BiomeFormatHint { .. })
    });
    assert_eq!(hints.len(), 1, "one hint: {diagnostics:#?}");
    assert_eq!(hints[0].severity, Severity::Info);
    hints[0].code.clone()
}

/// `b01_1_12_0_name_keyed`: the 1.12 shape (one root member named after the biome, its
/// `format_version` inside) is below the floor, with the 1.12 hint; nothing loads.
// The input of the game's own test for a 1.12-format biome file, rebuilt from its description.
#[test]
fn b01_1_12_0_name_keyed() {
    let found = rejected(&fixture("b01_1_12_0_name_keyed/plains.json"), &context());
    let hint = below_floor(&found, "1.12.0");
    assert_eq!(
        hint,
        DiagnosticCode::BiomeFormatHint {
            version: None,
            surface_components: false
        }
    );
}

/// `b02_1_13_0_tag_components`: format 1.13.0 (tags as empty components) is below the floor,
/// with the experimental-format hint.
#[test]
fn b02_1_13_0_tag_components() {
    let found = rejected(
        &fixture("b02_1_13_0_tag_components/plains.json"),
        &context(),
    );
    let hint = below_floor(&found, "1.13.0");
    assert_eq!(
        hint,
        DiagnosticCode::BiomeFormatHint {
            version: Some(FormatVersion::new(1, 13, 0)),
            surface_components: false
        }
    );
}

/// `b03_1_20_60_tags`: format 1.20.60 (`minecraft:tags`) is below the floor, with the hint.
#[test]
fn b03_1_20_60_tags() {
    let found = rejected(&fixture("b03_1_20_60_tags/plains.json"), &context());
    let hint = below_floor(&found, "1.20.60");
    assert_eq!(
        hint,
        DiagnosticCode::BiomeFormatHint {
            version: Some(FormatVersion::new(1, 20, 60)),
            surface_components: false
        }
    );
}

/// `b04b_envelope_edges`: no `format_version`, an unreadable one, an array, a third root
/// member.
#[test]
fn b04b_envelope_edges() {
    let plains = fixture("b04_1_21_110_floor/plains.biome.json");
    // Without `format_version`.
    let text = plains.replacen(r#""format_version": "1.21.110","#, "", 1);
    assert!(!text.contains("format_version"));
    let found = rejected(&text, &context());
    assert!(has(
        &found,
        |c| matches!(c, DiagnosticCode::MissingFormatVersion { .. }),
        Severity::Error,
        "no valid version passed by the caller or found in the input"
    ));
    // `"abc"` is malformed.
    let text = plains.replacen(r#""1.21.110""#, r#""abc""#, 1);
    let found = rejected(&text, &context());
    assert!(
        has(
            &found,
            |c| matches!(c, DiagnosticCode::MalformedFormatVersion { .. }),
            Severity::Error,
            "Malformed format version"
        ),
        "{found:#?}"
    );
    // The number `1.21` is read as its text, 1.21.0, which
    // is below the floor.
    let text = plains.replacen(r#""1.21.110""#, "1.21", 1);
    let found = rejected(&text, &context());
    assert!(
        found.iter().any(
            |f| matches!(f.code, DiagnosticCode::UnsupportedVersion { .. })
                && f.severity == Severity::Error
                && f.code.to_string().ends_with("1.21.0")
        ),
        "{found:#?}"
    );
    // A 1.26.0 biome with `[1, 26, 0]` loads.
    let at_1_26_0 = biome("1.26.0", "test:arrayed", "");
    let arrayed = at_1_26_0.replacen(r#""1.26.0""#, "[1, 26, 0]", 1);
    let document = loads(&arrayed, &context());
    assert_eq!(document.version().parsed_as, FormatVersion::new(1, 26, 0));
    // A third root member at ≥ 1.21.120.
    let extra = at_1_26_0.replacen('{', r#"{"extra": 1, "#, 1);
    let found = rejected(&extra, &context());
    assert!(has(
        &found,
        |c| matches!(c, DiagnosticCode::MalformedInput { .. }),
        Severity::Error,
        "A Minecraft Document must have exactly two members: \"format_version\" and a payload member"
    ));
}

/// `b05_world_gate`: a 1.21.110 biome in a pack other than a base-game pack, in a world of
/// base-game version 1.21.100, is skipped with vanilla's message; from a base-game pack it
/// loads.
#[test]
fn b05_world_gate() {
    let world = |mut cx: LoadContext| {
        cx.world_base_game_version = BaseGameVersion::new(SemVersion::new(1, 21, 100));
        cx
    };
    let plains = fixture("b04_1_21_110_floor/plains.biome.json");
    let found = rejected(&plains, &world(context()));
    assert!(has(
        &found,
        |c| matches!(c, DiagnosticCode::IgnoredFile { .. }),
        Severity::Error,
        "Custom Biomes can only be added for world versions 1.21.110 and up!"
    ));
    assert_eq!(found.len(), 1, "skipped before it is read: {found:#?}");
    loads(&plains, &world(base_game()));
    // A world at 1.21.110 takes it.
    let mut at_floor = context();
    at_floor.world_base_game_version = BaseGameVersion::new(SemVersion::new(1, 21, 110));
    loads(&plains, &at_floor);
}

/// `b06_1_21_120_village_type`: the 1.26.0 step inserts `village_type` for a vanilla biome's
/// override below 1.26.0 only.
#[test]
fn b06_1_21_120_village_type() {
    let climate = r#""minecraft:climate": {"temperature": 0.8}"#;
    let plains = loads(&biome("1.21.120", "minecraft:plains", climate), &context());
    assert_eq!(
        plains.components.get::<VillageType>().map(|v| v.kind),
        Some(VillageKind::Default)
    );
    assert!(plains.components.iter().any(|e| matches!(
        e,
        ComponentRef::Typed {
            name: "minecraft:village_type",
            synthesized: true,
            ..
        }
    )));
    assert!(plains.diagnostics().is_empty(), "nothing is reported");
    let custom = loads(&biome("1.21.120", "test:plains", climate), &context());
    assert!(custom.components.get::<VillageType>().is_none());
    let at_1_26_0 = loads(&biome("1.26.0", "minecraft:plains", climate), &context());
    assert!(at_1_26_0.components.get::<VillageType>().is_none());
}

/// The tags of a loaded biome.
pub fn tags(document: &Biome) -> Vec<String> {
    document
        .components
        .get::<Tags>()
        .map(|t| t.tags.iter().map(|t| t.0.clone()).collect())
        .unwrap_or_default()
}

/// `b07_1_21_110_tag_appends`: the 1.26.0 step appends the tags vanilla moved into data,
/// creating `minecraft:tags` when absent and skipping a tag already present (ours, pinned by
/// `chain::tag_appends_skip_present_tags_deviation`).
#[test]
fn b07_1_21_110_tag_appends() {
    let swamp = loads(
        &biome(
            "1.21.110",
            "minecraft:swampland",
            r#""minecraft:tags": {"tags": ["swamp"]}"#,
        ),
        &context(),
    );
    assert_eq!(
        tags(swamp.value()),
        ["swamp", "slime", "swamp_water_huge_mushroom"]
    );
    let mesa = loads(&biome("1.21.110", "minecraft:mesa", ""), &context());
    assert_eq!(tags(mesa.value()), ["surface_mineshaft"]);
    assert!(mesa.components.iter().any(|e| matches!(
        e,
        ComponentRef::Typed {
            name: "minecraft:tags",
            synthesized: true,
            ..
        }
    )));
    let ocean = loads(
        &biome(
            "1.21.110",
            "minecraft:ocean",
            r#""minecraft:tags": {"tags": ["high_seas"]}"#,
        ),
        &context(),
    );
    assert_eq!(tags(ocean.value()), ["high_seas", "fast_fishing"]);
}

/// A biome replacing `crimson_forest` in `dimension`, with `builder` as its surface builder
/// (none when empty).
pub fn replacing(version: &str, dimension: &str, builder: &str) -> String {
    let mut components = format!(
        r#""minecraft:replace_biomes": {{"replacements": [{{"dimension": "{dimension}", "targets": ["crimson_forest"], "amount": 0.5, "noise_frequency_scale": 1.0}}]}}"#
    );
    if !builder.is_empty() {
        write!(
            components,
            r#", "minecraft:surface_builder": {{"builder": {builder}}}"#
        )
        .unwrap();
    }
    biome(version, "test:replacer", &components)
}

/// `b08_replace_biomes_nether`: the Nether is a replacement dimension from 1.26.0 on.
#[test]
fn b08_replace_biomes_nether() {
    let found = rejected(&replacing("1.21.120", "minecraft:nether", ""), &context());
    assert!(
        has(
            &found,
            |c| matches!(c, DiagnosticCode::InvalidValue { .. }),
            Severity::Error,
            "Dimension must be 'minecraft:overworld' but is 'minecraft:nether'"
        ),
        "{found:#?}"
    );
    let loaded = loads(&replacing("1.26.0", "minecraft:nether", ""), &context());
    assert!(loaded.components.contains::<ReplaceBiomes>());
}

/// `b09_replace_biomes_constraints`: an empty list, a repeated pair, an amount of 0.
#[test]
fn b09_replace_biomes_constraints() {
    let empty = biome(
        "1.26.0",
        "test:replacer",
        r#""minecraft:replace_biomes": {"replacements": []}"#,
    );
    assert!(has(
        &rejected(&empty, &context()),
        |c| matches!(c, DiagnosticCode::InvalidValue { .. }),
        Severity::Error,
        "Expected array with at least 1 element"
    ));
    let repeated = biome(
        "1.26.0",
        "test:replacer",
        r#""minecraft:replace_biomes": {"replacements": [
            {"dimension": "minecraft:overworld", "targets": ["plains"], "amount": 0.5, "noise_frequency_scale": 1.0},
            {"dimension": "minecraft:overworld", "targets": ["plains"], "amount": 0.5, "noise_frequency_scale": 1.0}
        ]}"#,
    );
    assert!(has(
        &rejected(&repeated, &context()),
        |c| matches!(c, DiagnosticCode::InvalidValue { .. }),
        Severity::Error,
        "Dimension and target biome pairing already defined: 'minecraft:overworld', 'plains'"
    ));
    let zero = biome(
        "1.26.0",
        "test:replacer",
        r#""minecraft:replace_biomes": {"replacements": [
            {"dimension": "minecraft:overworld", "targets": ["plains"], "amount": 0, "noise_frequency_scale": 1.0}
        ]}"#,
    );
    let found = rejected(&zero, &context());
    assert_eq!(
        errors(&found)[0].location.json_path.to_string(),
        "minecraft:biome/components/minecraft:replace_biomes/replacements/0/amount"
    );
}

/// A mesa builder's members.
pub const MESA: &str = r#"{"type": "minecraft:mesa", "sea_floor_depth": 7, "sea_floor_material": "minecraft:gravel", "foundation_material": "minecraft:stone", "mid_material": "minecraft:hardened_clay", "top_material": "minecraft:red_sand", "sea_material": "minecraft:water", "clay_material": "minecraft:white_terracotta", "hard_clay_material": "minecraft:hardened_clay", "bryce_pillars": false, "has_forest": false}"#;
/// A capped builder's members.
pub const CAPPED: &str = r#"{"type": "minecraft:capped", "floor_materials": ["minecraft:netherrack"], "ceiling_materials": ["minecraft:netherrack"], "sea_material": "minecraft:lava", "foundation_material": "minecraft:netherrack"}"#;

/// `b10_nether_mesa_builder`: a Nether replacement needs a builder the Nether takes.
#[test]
fn b10_nether_mesa_builder() {
    let found = rejected(&replacing("1.26.0", "minecraft:nether", MESA), &context());
    assert!(has(
        &found,
        |c| matches!(c, DiagnosticCode::InvalidValue { .. }),
        Severity::Error,
        "Biomes in Nether are only compatible with following Surface Builders: overworld, frozen_ocean, capped, the_end"
    ));
    let capped = loads(&replacing("1.26.0", "minecraft:nether", CAPPED), &context());
    assert!(matches!(
        capped.components.get::<SurfaceBuilder>().unwrap().builder,
        Builder::Capped(_)
    ));
}

/// `b11_glue_errors`: both height forms, a reversed snow cover, an empty noise range.
#[test]
fn b11_glue_errors() {
    for (components, text, at) in [
        (
            r#""minecraft:overworld_height": {"noise_type": "default", "noise_params": [0.1, 0.2]}"#,
            "Cannot specify both noise_type and noise_params",
            "minecraft:overworld_height",
        ),
        (
            r#""minecraft:climate": {"snow_accumulation": [0.5, 0.25]}"#,
            "Incorrect values for snow_accumulation - first element (minimum) cannot be larger than second element (maximum)",
            "minecraft:climate/snow_accumulation",
        ),
        (
            r#""minecraft:surface_material_adjustments": {"adjustments": [{"materials": {"top_material": "minecraft:sand"}, "noise_range": [0.5, 0.5]}]}"#,
            "Incorrect values for noise_range bounds - upper bound (second value) should be strictly greater than lower bound (first value)",
            "minecraft:surface_material_adjustments/adjustments/0/noise_range",
        ),
    ] {
        let found = rejected(&biome("1.21.110", "test:glue", components), &context());
        let errors = errors(&found);
        assert_eq!(errors.len(), 1, "{found:#?}");
        assert_eq!(errors[0].code.to_string(), text);
        assert_eq!(
            errors[0].location.json_path.to_string(),
            format!("minecraft:biome/components/{at}")
        );
    }
}

/// `b13_unknown_component`: a later struct's component and a 1.20.60-era component in a
/// 1.21.110 biome are unknown components. The legacy parser's `StrictJsonLoader` is built with
/// 0xfffc, so each is an error and the biome is refused, as from 1.21.120
/// (`chain::unknown_components_and_members_refuse_a_cereal_biome`).
#[test]
fn b13_unknown_component() {
    let text = biome(
        "1.21.110",
        "test:unknown",
        r#""minecraft:village_type": {"type": "desert"}, "minecraft:surface_parameters": {"sea_floor_depth": 7}"#,
    );
    let found = rejected(&text, &context());
    let unrecognized = with(&found, |c| {
        matches!(c, DiagnosticCode::UnrecognizedComponent { .. })
    });
    let paths: Vec<String> = unrecognized
        .iter()
        .map(|d| d.location.json_path.to_string())
        .collect();
    assert_eq!(
        paths,
        [
            "minecraft:biome/components/minecraft:village_type",
            "minecraft:biome/components/minecraft:surface_parameters"
        ]
    );
    assert!(unrecognized.iter().all(|d| d.severity == Severity::Error));
}

/// Rule N2: every biome slice reads the document through the game's document input
/// with a JSON reader that has no null test for a member (the strict reader for the cereal
/// slices; the legacy parser's reader is the same code), so a member written `null`, an optional one included,
/// is present, fails its type (`0x8`) and refuses the biome. The member is not read as
/// the defaults.
#[test]
fn a_null_member_refuses_the_biome_on_every_slice() {
    for (version, components, at) in [
        (
            "1.21.110",
            r#""minecraft:climate": null"#,
            "minecraft:biome/components/minecraft:climate",
        ),
        (
            "1.21.110",
            r#""minecraft:surface_material_adjustments": {"adjustments": [{"materials": null}]}"#,
            "minecraft:biome/components/minecraft:surface_material_adjustments/adjustments/0/materials",
        ),
        (
            "1.21.120",
            r#""minecraft:climate": {"temperature": null}"#,
            "minecraft:biome/components/minecraft:climate/temperature",
        ),
        (
            "1.26.0",
            r#""minecraft:overworld_height": null"#,
            "minecraft:biome/components/minecraft:overworld_height",
        ),
        (
            "1.26.30",
            r#""minecraft:climate": {"downfall": null}"#,
            "minecraft:biome/components/minecraft:climate/downfall",
        ),
    ] {
        let found = match load(&biome(version, "test:null", components), &context()) {
            Ok(document) => panic!(
                "{version} {components}: loaded, {:#?}",
                document.diagnostics()
            ),
            Err(refused) => refused.diagnostics,
        };
        let typed: Vec<_> = with(&found, |c| matches!(c, DiagnosticCode::InvalidType { .. }));
        assert_eq!(typed.len(), 1, "{version} {components}: {found:#?}");
        assert_eq!(typed[0].severity, Severity::Error);
        assert_eq!(typed[0].location.json_path.to_string(), at, "{version}");
    }
    // The beta slice too.
    let beta = load_beta(
        &biome(
            "beta",
            "test:null",
            r#""minecraft:climate": {"temperature": null}"#,
        ),
        &base_game(),
    )
    .expect_err("refused");
    assert!(
        beta.diagnostics
            .iter()
            .any(|d| matches!(d.code, DiagnosticCode::InvalidType { .. })
                && d.severity == Severity::Error
                && d.location.json_path.to_string()
                    == "minecraft:biome/components/minecraft:climate/temperature"),
        "{:#?}",
        beta.diagnostics
    );
    // A well-formed component says nothing.
    let document = loads(
        &biome(
            "1.21.110",
            "test:null",
            r#""minecraft:climate": {"temperature": 0.8}"#,
        ),
        &context(),
    );
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
}

/// A noise-gradient builder's members.
pub const NOISE_GRADIENT: &str = r#"{"type": "minecraft:noise_gradient", "noise_descriptor": {"name": "test:gradient", "first_octave": -5, "amplitudes": [1.0, 0.0, 1.0]}, "non_replaceable_blocks": ["minecraft:bedrock"], "noise_block_specifiers": [{"range": {"min": -0.4, "max": -0.1}, "block": "minecraft:stone"}, {"range": {"min": 0.0, "max": 0.4}, "block": "minecraft:tuff"}]}"#;

/// `b14_1_26_30_noise_gradient`: the noise gradient and the subsurface builder from 1.26.30;
/// at 1.26.0 `minecraft:noise_gradient` is no builder type and rejects the document.
#[test]
fn b14_1_26_30_noise_gradient() {
    let components = format!(
        r#""minecraft:surface_builder": {{"builder": {NOISE_GRADIENT}}}, "minecraft:subsurface_builder": {{"builder": {NOISE_GRADIENT}}}"#
    );
    let document = loads(&biome("1.26.30", "test:gradient", &components), &context());
    assert!(matches!(
        document.components.get::<SurfaceBuilder>().unwrap().builder,
        Builder::NoiseGradient(_)
    ));
    assert!(matches!(
        document
            .components
            .get::<SubsurfaceBuilder>()
            .unwrap()
            .builder,
        Builder::NoiseGradient(_)
    ));
    round_trip(document.value());
    let found = rejected(&biome("1.26.0", "test:gradient", &components), &context());
    let rejection = with(&found, |c| {
        matches!(c, DiagnosticCode::InvalidEnumValue { .. })
    });
    assert_eq!(rejection.len(), 1, "{found:#?}");
    assert_eq!(rejection[0].severity, Severity::Error);
    assert_eq!(
        rejection[0].location.json_path.to_string(),
        "minecraft:biome/components/minecraft:surface_builder/builder/type"
    );
}

/// `b15_beta_environment_attributes`: a beta biome with an environment attribute loads from a
/// base-game pack and is refused elsewhere without Beta APIs.
#[test]
fn b15_beta_environment_attributes() {
    let text = document(
        "beta",
        r#"{"description": {"identifier": "test:beta"}, "components": {}, "environment_attributes": {"minecraft:experimental_float_attr_1": {"operation": "add", "value": 1.0}}}"#,
    );
    let beta = load_beta(&text, &base_game()).expect("a beta biome");
    let attribute = beta
        .environment_attributes
        .as_ref()
        .and_then(|a| a.experimental_float_attr_1.as_ref())
        .expect("the attribute");
    assert_eq!(attribute.value.0, 1.0);
    assert_eq!(
        attribute.operation,
        bedrock_addon::families::biome::FloatOperation::Add
    );
    assert!(beta.version().beta);
    let with_toggle = Beta::parse(&super::common::path(), text.as_bytes(), &beta_apis())
        .expect("Beta APIs allow it from any pack");
    assert_eq!(with_toggle.value(), beta.value());
    let found = load_beta(&text, &context()).expect_err("refused");
    assert!(has(
        &found.diagnostics,
        |c| matches!(c, DiagnosticCode::BetaFeaturesNotEnabled { .. }),
        Severity::Error,
        "Using a beta format_version is only allowed for base packs or when the BetaApis toggle is enabled."
    ));
    // `Biome::parse` takes no beta document.
    assert!(load(&text, &base_game()).is_err());
    let written = beta.to_json().unwrap();
    assert_eq!(
        load_beta(&written, &base_game()).unwrap().value(),
        beta.value()
    );
}

/// `b17_overrides`: retail 1.26.45's 17 `"beta"` overrides load unchanged
/// from a base-game pack, and are refused from another pack without Beta APIs.
// The files are `behavior_packs/experimental_y_2026_drop_3/biomes/*.biome.json` of the retail
// 1.26.45 GDK build (format "beta"; `dappled_forest`, at 1.26.30, is b18's).
#[test]
fn b17_overrides() {
    let files = fixtures("b17_overrides");
    assert_eq!(files.len(), 17);
    for (name, text) in &files {
        let beta = load_beta(text, &base_game()).unwrap_or_else(|r| panic!("{name}: {r:#?}"));
        assert!(
            beta.diagnostics()
                .iter()
                .all(|d| d.severity < Severity::Warning),
            "{name}: {:#?}",
            beta.diagnostics()
        );
        // Unchanged: no step runs on a beta document, so nothing is synthesized.
        assert!(
            beta.components.iter().all(|e| !matches!(
                e,
                ComponentRef::Typed {
                    synthesized: true,
                    ..
                }
            )),
            "{name}"
        );
        let tags = beta.components.get::<Tags>().expect("tags");
        assert!(tags.contains("has_structure_abandoned_camp"), "{name}");
        // Written back and read again, the document is the same.
        let written = beta.to_json().unwrap();
        let again = load_beta(&written, &base_game()).unwrap();
        assert_eq!(again.value(), beta.value(), "{name}");
        assert_eq!(again.to_json().unwrap(), written, "{name}");
        let refused = load_beta(text, &context()).expect_err("refused");
        assert!(
            has(
                &refused.diagnostics,
                |c| matches!(c, DiagnosticCode::BetaFeaturesNotEnabled { .. }),
                Severity::Error,
                "Using a beta format_version is only allowed for base packs or when the BetaApis toggle is enabled."
            ),
            "{name}"
        );
    }
    let savanna = files
        .iter()
        .find(|(n, _)| n == "savanna.biome.json")
        .unwrap();
    let savanna = load_beta(&savanna.1, &base_game()).unwrap();
    assert_eq!(
        savanna.components.get::<VillageType>().map(|v| v.kind),
        Some(VillageKind::Savanna)
    );
}

/// `b18_1_26_30_files`: retail `sulfur_caves` (the noise gradient in both builders,
/// specifiers with only `range` and `block`, a negative bound) and `dappled_forest` load as
/// the newest struct without findings; the absent specifier members take their defaults.
// `behavior_packs/vanilla_1.26.30/biomes/sulfur_caves.biome.json` and
// `behavior_packs/experimental_y_2026_drop_3/biomes/dappled_forest.biome.json` of the retail
// 1.26.45 GDK build.
#[test]
fn b18_1_26_30_files() {
    for (name, text) in fixtures("b18_1_26_30_files") {
        let document = loads(&text, &base_game());
        assert!(
            document.diagnostics().is_empty(),
            "{name}: {:#?}",
            document.diagnostics()
        );
        assert_eq!(document.version().parsed_as, FormatVersion::new(1, 26, 30));
        round_trip(document.value());
    }
    let sulfur = loads(
        &fixture("b18_1_26_30_files/sulfur_caves.biome.json"),
        &base_game(),
    );
    for builder in [
        &sulfur.components.get::<SurfaceBuilder>().unwrap().builder,
        &sulfur
            .components
            .get::<SubsurfaceBuilder>()
            .unwrap()
            .builder,
    ] {
        let Builder::NoiseGradient(gradient) = builder else {
            panic!("a noise gradient")
        };
        let specifiers = &gradient.noise_block_specifiers;
        assert_eq!(specifiers.len(), 3);
        assert_eq!(specifiers[0].range(), (-0.4, -0.1));
        assert!(
            specifiers
                .iter()
                .all(|s| s.noise().is_empty() && s.threshold() == 0.0)
        );
        assert!(
            specifiers
                .iter()
                .all(|s| s.noise.is_none() && s.threshold.is_none())
        );
    }
}

/// `b19_1_26_20_noise_gradient`: the dev 1.26.20 beta `sulfur_caves` uses the noise
/// gradient's first shape, which our beta struct (the 1.26.51 one) does not take (ours;
/// vanilla 1.26.20 loads it, 1.26.45 does not).
// `behavior_packs/experimental_y_2026_drop_2/biomes/sulfur_caves.biome.json` of the GDK
// Dev-1.26.2004 build.
#[test]
fn b19_1_26_20_noise_gradient_deviation() {
    let text = fixture("b19_1_26_20_noise_gradient/sulfur_caves.biome.json");
    let found = load_beta(&text, &base_game()).expect_err("rejected");
    let missing = with(&found.diagnostics, |c| {
        matches!(c, DiagnosticCode::MissingRequiredField { .. })
    });
    assert_eq!(missing.len(), 1, "{:#?}", found.diagnostics);
    assert_eq!(missing[0].severity, Severity::Error);
    assert_eq!(
        missing[0].location.json_path.to_string(),
        "minecraft:biome/components/minecraft:subsurface_builder/builder/noise_descriptor"
    );
    let unrecognized: Vec<String> = with(&found.diagnostics, |c| {
        matches!(c, DiagnosticCode::UnrecognizedField { .. })
    })
    .iter()
    .map(|d| d.location.json_path.to_string())
    .collect();
    for member in ["noise_seed_string", "gradient_blocks"] {
        assert!(
            unrecognized.contains(&format!(
                "minecraft:biome/components/minecraft:subsurface_builder/builder/{member}"
            )),
            "{member}: {unrecognized:#?}"
        );
    }
}

/// `b16_1_20_60_surface_components`: a 1.20.60 vanilla `mesa` with `minecraft:mesa_surface` is
/// below the floor; the hint names the surface components.
// Rebuilt from the vanilla 1.21.110 `mesa.biome.json` (the 1.21.90 file is not in the corpus).
#[test]
fn b16_1_20_60_surface_components() {
    let found = rejected(
        &fixture("b16_1_20_60_surface_components/mesa.biome.json"),
        &context(),
    );
    let hint = below_floor(&found, "1.20.60");
    assert_eq!(
        hint,
        DiagnosticCode::BiomeFormatHint {
            version: Some(FormatVersion::new(1, 20, 60)),
            surface_components: true
        }
    );
    assert!(hint.to_string().ends_with(
        "and replace minecraft:surface_parameters / minecraft:*_surface with minecraft:surface_builder)"
    ));
}

/// The parse-as-written stage keeps the struct the version selects, and the upgrade gives
/// what `parse` gives.
#[test]
fn b04_parse_as_written_then_upgrade_equals_parse() {
    let text = fixture("b04_1_21_110_floor/plains.biome.json");
    let written = Biome::parse_as_written(&super::common::path(), text.as_bytes(), &context())
        .expect("parses");
    assert!(matches!(written.value(), history::Any::V1_21_110(_)));
    let upgraded = history::upgrade(written, &context()).expect("upgrades");
    let parsed = loads(&text, &context());
    assert_eq!(upgraded.value(), parsed.value());
}

/// A 1.26.30 biome whose surface builder is a noise gradient with `specifiers`.
pub fn gradient(specifiers: &str) -> String {
    let components = format!(
        r#""minecraft:surface_builder": {{"builder": {{"type": "minecraft:noise_gradient", "noise_descriptor": {{"name": "test:gradient", "first_octave": -5, "amplitudes": [1.0]}}, "noise_block_specifiers": [{specifiers}]}}}}"#
    );
    biome("1.26.30", "test:gradient", &components)
}

/// One specifier of `block` over `[min, max]`, with `noise` when given.
fn specifier(min: f32, max: f32, noise: Option<&str>) -> String {
    let noise = noise.map_or(String::new(), |n| format!(r#""noise": "{n}", "#));
    format!(r#"{{{noise}"range": {{"min": {min}, "max": {max}}}, "block": "minecraft:stone"}}"#)
}

/// A `range` in a form other than `{min, max}`: the range is a composite in the game's schema,
/// and its loader logs "expected an
/// object" with code 8, which the strict mask 0xfffc
/// of every biome slice counts: the biome is refused.
#[test]
fn b20a_a_range_in_another_form_refuses_the_biome() {
    let at = "minecraft:biome/components/minecraft:surface_builder/builder/noise_block_specifiers/0/range";
    for range in ["[0.0, 0.4]", "0.4"] {
        let found = rejected(
            &gradient(&format!(
                r#"{{"range": {range}, "block": "minecraft:stone"}}"#
            )),
            &context(),
        );
        let errors_found = errors(&found);
        assert_eq!(errors_found.len(), 1, "{range}: {found:#?}");
        assert!(matches!(
            errors_found[0].code,
            DiagnosticCode::InvalidType { .. }
        ));
        assert!(!errors_found[0].kept);
        assert_eq!(errors_found[0].location.json_path.to_string(), at);
    }
}

/// `b20_noise_gradient_specifier_checks`: the specifiers' list, `noise` and range checks.
#[test]
fn b20_noise_gradient_specifier_checks() {
    let at = "minecraft:biome/components/minecraft:surface_builder/builder/noise_block_specifiers";
    // An empty list: the member's `min_len(1)`.
    let found = rejected(&gradient(""), &context());
    let errors_found = errors(&found);
    assert_eq!(errors_found.len(), 1, "{found:#?}");
    assert!(matches!(
        errors_found[0].code,
        DiagnosticCode::InvalidValue { text: None }
    ));
    assert_eq!(errors_found[0].location.json_path.to_string(), at);
    // `noise` by pack area.
    let one = |noise: &str| gradient(&specifier(0.0, 1.0, Some(noise)));
    loads(&one("test:grad"), &context());
    assert!(has(
        &rejected(&one("minecraft:grad"), &context()),
        |c| matches!(c, DiagnosticCode::NamespaceRule { .. }),
        Severity::Error,
        "string has invalid namespace (minecraft)"
    ));
    assert!(has(
        &rejected(&one("grad"), &context()),
        |c| matches!(c, DiagnosticCode::NamespaceRule { .. }),
        Severity::Error,
        "string must be prefixed with a namespace (eg. namespace:value)"
    ));
    loads(&one("minecraft:grad"), &base_game());
    let spaced = rejected(&one("a b:c"), &context());
    let spaced = errors(&spaced);
    assert_eq!(spaced.len(), 1);
    assert!(matches!(
        spaced[0].code,
        DiagnosticCode::InvalidValue { text: None }
    ));
    assert_eq!(
        spaced[0].location.json_path.to_string(),
        format!("{at}/0/noise")
    );
    // An inverted range passes vanilla's pairwise test.
    let inverted = format!(
        "{}, {}",
        specifier(0.5, 0.6, None),
        specifier(0.55, 0.2, None)
    );
    loads(&gradient(&inverted), &context());
    // Overlapping ranges.
    let overlapping = format!(
        "{}, {}",
        specifier(0.0, 0.5, None),
        specifier(0.4, 1.0, None)
    );
    let found = rejected(&gradient(&overlapping), &context());
    let texts_found: Vec<String> = errors(&found).iter().map(|d| d.code.to_string()).collect();
    assert_eq!(
        texts_found,
        [
            "The provided NoiseBlockSpecifiers have overlapping ranges. They should be specified as closed intervals [a, b] with optionally overlapping endpoints.",
            "The SurfaceBuilderComponent could not be resolved."
        ]
    );
    // Touching at an endpoint passes.
    let touching = format!(
        "{}, {}",
        specifier(0.0, 0.4, None),
        specifier(0.4, 1.0, None)
    );
    loads(&gradient(&touching), &context());
}
