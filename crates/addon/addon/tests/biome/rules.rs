//! The validation rules that no fixture row covers on its own: one
//! test per rule (the rows cover rules 1, 2, 5, 7, 9a, 10, 11, 14 and 15 too). References to
//! other biomes and to blocks are the link pass's: their tests run through
//! `StackSet::link`.

use bedrock_addon::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon::version::{FormatVersion, MinEngineVersion};

use super::common::{base_game, biome, context, document, errors, has, loads, rejected, with};
use super::rows::{CAPPED, NOISE_GRADIENT, replacing};

/// The texts of the errors.
fn error_texts(found: &bedrock_addon::diagnostic::Diagnostics) -> Vec<String> {
    errors(found).iter().map(|d| d.code.to_string()).collect()
}

/// The path of the first error.
fn first_error_at(found: &bedrock_addon::diagnostic::Diagnostics) -> String {
    errors(found)[0].location.json_path.to_string()
}

/// Rule 3: the payload member is `minecraft:biome`.
#[test]
fn rule_3_the_payload_key_is_minecraft_biome() {
    let wrong = r#"{"format_version": "1.26.0", "minecraft:block": {"description": {"identifier": "test:x"}, "components": {}}}"#;
    let found = rejected(wrong, &context());
    assert!(
        // By legacy name: the variant and the definitions that were it.
        !with(&found, |c| matches!(c.name(), "MissingPayload")).is_empty()
    );
    // `description` and `components` are required members of the payload.
    for payload in [
        r#"{"description": {"identifier": "test:x"}}"#,
        r#"{"components": {}}"#,
    ] {
        let found = rejected(&document("1.26.30", payload), &context());
        assert!(
            // By legacy name: the variant and the definitions that were it.
            !with(&found, |c| matches!(c.name(), "MissingRequiredField")).is_empty(),
            "{payload}: {found:#?}"
        );
    }
    // On the raw path the legacy struct misses its member.
    let legacy = wrong.replace("1.26.0", "1.21.110");
    let found = rejected(&legacy, &context());
    assert!(
        // By legacy name: the variant and the definitions that were it.
        !with(&found, |c| matches!(c.name(), "MissingRequiredField")).is_empty(),
        "{found:#?}"
    );
}

/// Rule 4: the identifier is lower case and namespaced.
#[test]
fn rule_4_the_identifier_is_lower_case_and_namespaced() {
    for identifier in ["Test:Upper", "plains", "test:a b"] {
        let found = rejected(&biome("1.26.30", identifier, ""), &context());
        assert!(
            has(
                &found,
                // By legacy name: the variant and the definitions that were it.
                |c| matches!(c.name(), "NamespaceRule"),
                Severity::Error,
                "identifier has an upper case letter or does not specify any namespace"
            ),
            "{identifier}"
        );
        assert_eq!(
            first_error_at(&found),
            "minecraft:biome/description/identifier"
        );
    }
    loads(&biome("1.26.30", "test:a.b_c-d%e+f", ""), &context());
}

/// Rule 6: in `[1.21.110, 1.21.120)` a strict-load failure closes with "Invalid Biome file";
/// from 1.21.120 the deserialiser's finding stands alone.
#[test]
fn rule_6_a_legacy_strict_failure_is_invalid_biome_file() {
    let arrayed = biome("1.21.110", "test:x", "").replacen(r#""1.21.110""#, "[1, 21, 110]", 1);
    let found = rejected(&arrayed, &context());
    // By legacy name: the variant and the definitions that were it.
    assert!(!with(&found, |c| matches!(c.name(), "InvalidType")).is_empty());
    assert!(has(
        &found,
        // By legacy name: the variant and the definitions that were it.
        |c| matches!(c.name(), "MalformedInput"),
        Severity::Error,
        "Invalid Biome file"
    ));
    let humidity = r#""minecraft:humidity": {}"#;
    let found = rejected(&biome("1.21.110", "test:x", humidity), &context());
    assert_eq!(
        error_texts(&found).last().map(String::as_str),
        Some("Invalid Biome file")
    );
    let found = rejected(&biome("1.21.120", "test:x", humidity), &context());
    let errors_found = errors(&found);
    assert_eq!(errors_found.len(), 1);
    // By legacy name: the variant and the definitions that were it.
    assert!(matches!(
        errors_found[0].code.name(),
        "MissingRequiredField"
    ));
    assert_eq!(
        errors_found[0].location.json_path.to_string(),
        "minecraft:biome/components/minecraft:humidity/is_humid"
    );
}

/// Rule 7: the 1.21.x constraint's texts, and the 1.26.0 dimension set.
#[test]
fn rule_7_replace_biomes_constraint_by_struct() {
    let empty = r#""minecraft:replace_biomes": {"replacements": []}"#;
    assert!(has(
        &rejected(&biome("1.21.120", "test:x", empty), &context()),
        // By legacy name: the variant and the definitions that were it.
        |c| matches!(c.name(), "InvalidValue"),
        Severity::Error,
        "expected array with at least 1 element"
    ));
    assert!(has(
        &rejected(&replacing("1.26.0", "minecraft:the_end", ""), &context()),
        // By legacy name: the variant and the definitions that were it.
        |c| matches!(c.name(), "InvalidValue"),
        Severity::Error,
        "Dimension must be either 'minecraft:overworld' or 'minecraft:nether' but is 'minecraft:the_end'"
    ));
}

/// Rule 8: the member constraints.
#[test]
fn rule_8_member_constraints() {
    let overworld = |depth: i32| {
        format!(
            r#""minecraft:surface_builder": {{"builder": {{"type": "minecraft:overworld", "sea_floor_depth": {depth}, "sea_floor_material": "minecraft:gravel", "foundation_material": "minecraft:stone", "mid_material": "minecraft:dirt", "top_material": "minecraft:grass_block", "sea_material": "minecraft:water"}}}}"#
        )
    };
    let swamp = |puddles: i32| {
        format!(
            r#""minecraft:surface_builder": {{"builder": {{"type": "minecraft:swamp", "sea_floor_depth": 7, "sea_floor_material": "minecraft:gravel", "foundation_material": "minecraft:stone", "mid_material": "minecraft:dirt", "top_material": "minecraft:grass_block", "sea_material": "minecraft:water", "max_puddle_depth_below_sea_level": {puddles}}}}}"#
        )
    };
    let replacement = |targets: &str, amount: f32, scale: f32| {
        format!(
            r#""minecraft:replace_biomes": {{"replacements": [{{"dimension": "minecraft:overworld", "targets": {targets}, "amount": {amount}, "noise_frequency_scale": {scale}}}]}}"#
        )
    };
    let amplitudes = |n: usize| {
        let list = vec!["1.0"; n].join(", ");
        format!(
            r#""minecraft:surface_builder": {{"builder": {{"type": "minecraft:noise_gradient", "noise_descriptor": {{"name": "test:n", "first_octave": 0, "amplitudes": [{list}]}}, "noise_block_specifiers": [{{"block": "minecraft:stone"}}]}}}}"#
        )
    };
    let cases: Vec<(String, &str)> = vec![
        (
            r#""minecraft:creature_spawn_probability": {"probability": 0.8}"#.to_owned(),
            "minecraft:creature_spawn_probability/probability",
        ),
        (
            overworld(128),
            "minecraft:surface_builder/builder/sea_floor_depth",
        ),
        (
            overworld(-1),
            "minecraft:surface_builder/builder/sea_floor_depth",
        ),
        (
            swamp(128),
            "minecraft:surface_builder/builder/max_puddle_depth_below_sea_level",
        ),
        (
            CAPPED.replace(
                r#""floor_materials": ["minecraft:netherrack"]"#,
                r#""floor_materials": []"#,
            ),
            "minecraft:surface_builder/builder/floor_materials",
        ),
        (
            replacement("[]", 0.5, 1.0),
            "minecraft:replace_biomes/replacements/0/targets",
        ),
        (
            replacement(r#"["plains"]"#, 1.5, 1.0),
            "minecraft:replace_biomes/replacements/0/amount",
        ),
        (
            replacement(r#"["plains"]"#, 0.5, 0.0),
            "minecraft:replace_biomes/replacements/0/noise_frequency_scale",
        ),
        (
            replacement(r#"["plains"]"#, 0.5, 101.0),
            "minecraft:replace_biomes/replacements/0/noise_frequency_scale",
        ),
        (
            amplitudes(0),
            "minecraft:surface_builder/builder/noise_descriptor/amplitudes",
        ),
        (
            amplitudes(101),
            "minecraft:surface_builder/builder/noise_descriptor/amplitudes",
        ),
    ];
    for (components, at) in cases {
        let components = if components.starts_with('{') {
            format!(r#""minecraft:surface_builder": {{"builder": {components}}}"#)
        } else {
            components
        };
        let found = rejected(&biome("1.26.30", "test:x", &components), &context());
        let errors_found = errors(&found);
        assert_eq!(errors_found.len(), 1, "{at}: {found:#?}");
        assert!(matches!(
            errors_found[0].code,
            DiagnosticCode::InvalidValue { text: None }
        ));
        assert_eq!(
            errors_found[0].location.json_path.to_string(),
            format!("minecraft:biome/components/{at}"),
        );
    }
    // The bounds themselves pass.
    loads(&biome("1.26.30", "test:x", &overworld(127)), &context());
    loads(&biome("1.26.30", "test:x", &swamp(0)), &context());
    loads(&biome("1.26.30", "test:x", &amplitudes(100)), &context());
    let _ = NOISE_GRADIENT;
}

/// Rule 9: the transformation and climate entries are a name (category) and a weight, in
/// that order; replacements in two dimensions are a warning; constant height bounds.
#[test]
fn rule_9_glue_checks() {
    let found = rejected(
        &biome(
            "1.26.30",
            "test:x",
            r#""minecraft:overworld_generation_rules": {"hills_transformation": [[1, "minecraft:forest"]]}"#,
        ),
        &context(),
    );
    assert_eq!(
        error_texts(&found),
        [
            "Weighted biome name entries must be arrays with 2 elements, the first being a string name and the second being a positive weight integer"
        ]
    );
    let found = rejected(
        &biome(
            "1.26.30",
            "test:x",
            r#""minecraft:overworld_generation_rules": {"generate_for_climates": [[3, "medium"]]}"#,
        ),
        &context(),
    );
    assert_eq!(
        error_texts(&found),
        [
            "Weighted temperature categories must be arrays with 2 elements, the first being a string temperature category and the second being a positive weight integer"
        ]
    );
    let both = biome(
        "1.26.0",
        "test:x",
        r#""minecraft:replace_biomes": {"replacements": [
            {"dimension": "minecraft:overworld", "targets": ["plains"], "amount": 0.5, "noise_frequency_scale": 1.0},
            {"dimension": "minecraft:nether", "targets": ["crimson_forest"], "amount": 0.5, "noise_frequency_scale": 1.0}
        ]}"#,
    );
    let document = loads(&both, &context());
    assert!(has(
        document.diagnostics(),
        // By legacy name: the variant and the definitions that were it.
        |c| matches!(c.name(), "InvalidValue"),
        Severity::Warning,
        "Replacement present in multiple Dimensions"
    ));
    let found = rejected(
        &biome(
            "1.26.30",
            "test:x",
            r#""minecraft:surface_material_adjustments": {"adjustments": [{"materials": {}, "height_range": [64, true]}]}"#,
        ),
        &context(),
    );
    assert_eq!(
        error_texts(&found),
        [
            "Incorrect values for height_range bounds - upper bound (second value) should be strictly greater than lower bound (first value)"
        ]
    );
}

// `StackSet::link`, the document loaded; they were written against the load of one document,
// which reads no other definition and cannot tell these)
/// The link findings of a world whose one behaviour pack has the biome `text`.
fn linked(text: &str) -> bedrock_addon::diagnostic::Diagnostics {
    use bedrock_addon::load::LoadOptions;
    use bedrock_addon::pack::{PackOrigin, PackStack, StackSet};
    let files = vec![
        (
            "manifest.json",
            super::common::manifest(1, "bp").into_bytes(),
        ),
        ("biomes/x.json", text.as_bytes().to_vec()),
    ];
    let pack = super::common::pack(files, PackOrigin::World);
    let set = StackSet {
        behavior: PackStack::of([pack], LoadOptions::default()).expect("a stack"),
        resource: PackStack::of([], LoadOptions::default()).expect("a stack"),
    };
    // The document loads: the game finds these when it resolves the biome against the others.
    assert_eq!(
        set.behavior.biomes().len(),
        1,
        "{:#?}",
        set.behavior.biomes().rejected()
    );
    set.link()
}

/// Rule 9 (link): an unknown transformation biome is "No biome found with name '%s'".
#[test]
fn rule_9_unknown_transformation_biome_is_a_link_error() {
    let found = linked(&biome(
        "1.26.30",
        "test:x",
        r#""minecraft:overworld_generation_rules": {"mutate_transformation": "test:nowhere"}"#,
    ));
    assert!(has(
        &found,
        // By legacy name: the variant and the definitions that were it.
        |c| matches!(c.name(), "UnresolvedReference"),
        Severity::Error,
        "No biome found with name 'test:nowhere'"
    ));
}

/// Rule 9 (link): replacement targets must be vanilla biomes.
#[test]
fn rule_9_replacement_targets_are_link_errors() {
    let unknown =
        replacing("1.26.0", "minecraft:overworld", "").replace("crimson_forest", "nowhere");
    assert!(has(
        &linked(&unknown),
        // By legacy name: the variant and the definitions that were it.
        |c| matches!(c.name(), "UnresolvedReference"),
        Severity::Error,
        "No biome found with name: 'nowhere'"
    ));
}

/// Rule 9 (link): a block that does not resolve.
#[cfg(feature = "block")]
#[test]
fn rule_9_unresolved_blocks_are_link_errors() {
    let found = linked(&biome(
        "1.26.30",
        "test:x",
        r#""minecraft:surface_material_adjustments": {"adjustments": [{"materials": {"top_material": "test:nothing"}}]}"#,
    ));
    assert!(has(
        &found,
        // By legacy name: the variant and the definitions that were it.
        |c| matches!(c.name(), "UnresolvedReference"),
        Severity::Error,
        "'top_material' block with name 'test:nothing', failed to resolve."
    ));
}

/// Rule 10: a subsurface builder has no effect in the Nether (a warning).
#[test]
fn rule_10_a_subsurface_builder_has_no_effect_in_the_nether() {
    let text = replacing("1.26.30", "minecraft:nether", CAPPED).replace(
        r#""minecraft:surface_builder""#,
        &format!(r#""minecraft:subsurface_builder": {{"builder": {CAPPED}}}, "minecraft:surface_builder""#),
    );
    let document = loads(&text, &context());
    assert!(has(
        document.diagnostics(),
        // By legacy name: the variant and the definitions that were it.
        |c| matches!(c.name(), "NoEffect"),
        Severity::Warning,
        "minecraft:subsurface_builder will have no effect on Biomes in the Nether"
    ));
}

/// Rule 12: `multinoise_generation_rules` outside the `minecraft` namespace is unused;
/// the game logs it at level 3 and returns without the component,
/// the biome kept: a kept error.
#[test]
fn rule_12_multinoise_outside_minecraft_is_a_kept_error() {
    let rules = r#""minecraft:multinoise_generation_rules": {"weight": 0.4}"#;
    let document = loads(&biome("1.26.30", "test:x", rules), &context());
    assert!(has(
        document.diagnostics(),
        // By legacy name: the variant and the definitions that were it.
        |c| matches!(c.name(), "InvalidValue"),
        Severity::Error,
        "Biome named: test:x contains minecraft:multinoise_generation_rules component which is a pre-caves and cliffs component and is unused for custom biomes"
    ));
    assert!(document.diagnostics().iter().all(|d| d.kept));
    let vanilla = loads(&biome("1.26.30", "minecraft:x", rules), &context());
    assert!(vanilla.diagnostics().is_empty());
}

/// Rule 13: no gate reads the pack's `min_engine_version`.
#[test]
fn rule_13_the_pack_engine_version_gates_nothing() {
    let text = biome(
        "1.26.30",
        "test:x",
        r#""minecraft:subsurface_builder": {"builder": {"type": "minecraft:the_end"}}"#,
    );
    for engine in [(1, 13, 0), (1, 21, 110), (1, 26, 50)] {
        let mut cx = context();
        cx.min_engine_version = Some(MinEngineVersion::new(engine.0, engine.1, engine.2));
        let document = loads(&text, &cx);
        assert!(document.diagnostics().is_empty(), "{engine:?}");
    }
}

/// Rule 14: `"beta"` from a base-game pack or with Beta APIs (also `b15`, `b17`).
#[test]
fn rule_14_the_beta_literal_needs_a_base_game_pack_or_beta_apis() {
    let text = document(
        "beta",
        r#"{"description": {"identifier": "test:b"}, "components": {}}"#,
    );
    assert!(super::common::load_beta(&text, &base_game()).is_ok());
    assert!(super::common::load_beta(&text, &super::common::beta_apis()).is_ok());
    let found = super::common::load_beta(&text, &context()).expect_err("refused");
    assert_eq!(found.diagnostics.len(), 1);
}

/// Rule 16: the custom-biome service texts of retail 1.26.45 are not document rules; nothing
/// in a document raises them.
#[test]
fn rule_16_service_texts_are_not_document_rules() {
    let document = loads(
        &super::common::fixture("b04_1_21_110_floor/plains.biome.json"),
        &context(),
    );
    assert!(
        document
            .diagnostics()
            .iter()
            .all(|d| !d.code.to_string().contains("config for biome"))
    );
    assert_eq!(document.version().parsed_as, FormatVersion::new(1, 21, 110));
}

/// Our hint texts.
#[test]
fn the_below_floor_hints_render_the_readme_texts() {
    assert_eq!(
        DiagnosticCode::BiomeFormatHint {
            version: None,
            surface_components: false
        }
        .to_string(),
        "biome files in the 1.12 format are no longer supported; use the minecraft:biome document format with format_version 1.21.110 or higher"
    );
    assert_eq!(
        DiagnosticCode::BiomeFormatHint {
            version: Some(FormatVersion::new(1, 20, 60)),
            surface_components: false
        }
        .to_string(),
        "biome format 1.20.60 was experimental and is not loaded; set format_version to 1.21.110 (move climate ash/white_ash/red_spores/blue_spores to the client biome's minecraft:precipitation)"
    );
    assert_eq!(
        DiagnosticCode::BiomeFormatHint {
            version: Some(FormatVersion::new(1, 20, 60)),
            surface_components: true
        }
        .to_string(),
        "biome format 1.20.60 was experimental and is not loaded; set format_version to 1.21.110 (move climate ash/white_ash/red_spores/blue_spores to the client biome's minecraft:precipitation and replace minecraft:surface_parameters / minecraft:*_surface with minecraft:surface_builder)"
    );
}

/// The path of the one `InvalidValue` constraint error `text` is rejected with, or `None`
/// when it loads.
fn constraint_at(text: &str) -> Option<String> {
    let found = match super::common::load(text, &context()) {
        Ok(_) => return None,
        Err(rejected) => rejected.diagnostics,
    };
    let errors_found = errors(&found);
    assert_eq!(errors_found.len(), 1, "{found:#?}");
    assert!(matches!(
        errors_found[0].code,
        DiagnosticCode::InvalidValue { text: None }
    ));
    Some(errors_found[0].location.json_path.to_string())
}

/// Rule 8 at the ends of each range: `probability` in `[0, 0.75]`, `amount` in `(0, 1]`,
/// `noise_frequency_scale` in `(0, 100]`, `ceiling_materials` at least one element.
#[test]
fn rule_8_range_ends() {
    let probability = |p: &str| {
        biome(
            "1.26.30",
            "test:x",
            &format!(r#""minecraft:creature_spawn_probability": {{"probability": {p}}}"#),
        )
    };
    let at = "minecraft:biome/components/minecraft:creature_spawn_probability/probability";
    for p in ["0.0", "0.75"] {
        assert_eq!(constraint_at(&probability(p)), None, "probability {p}");
    }
    for p in ["-0.1", "-1.0", "-0.000001", "0.7500001", "1.0"] {
        assert_eq!(
            constraint_at(&probability(p)).as_deref(),
            Some(at),
            "probability {p}"
        );
    }
    let replacement = |amount: &str, scale: &str| {
        biome(
            "1.26.0",
            "test:x",
            &format!(
                r#""minecraft:replace_biomes": {{"replacements": [{{"dimension": "minecraft:overworld", "targets": ["plains"], "amount": {amount}, "noise_frequency_scale": {scale}}}]}}"#
            ),
        )
    };
    let member =
        |m: &str| format!("minecraft:biome/components/minecraft:replace_biomes/replacements/0/{m}");
    assert_eq!(constraint_at(&replacement("1.0", "100.0")), None);
    assert_eq!(constraint_at(&replacement("0.000001", "0.000001")), None);
    for amount in ["0", "0.0", "-0.5", "1.000001", "2"] {
        assert_eq!(
            constraint_at(&replacement(amount, "1.0")),
            Some(member("amount")),
            "amount {amount}"
        );
    }
    for scale in ["0", "-1.0", "100.00001", "101"] {
        assert_eq!(
            constraint_at(&replacement("0.5", scale)),
            Some(member("noise_frequency_scale")),
            "noise_frequency_scale {scale}"
        );
    }
    let ceiling = CAPPED.replace(
        r#""ceiling_materials": ["minecraft:netherrack"]"#,
        r#""ceiling_materials": []"#,
    );
    assert_eq!(
        constraint_at(&biome(
            "1.26.30",
            "test:x",
            &format!(r#""minecraft:surface_builder": {{"builder": {ceiling}}}"#)
        ))
        .as_deref(),
        Some("minecraft:biome/components/minecraft:surface_builder/builder/ceiling_materials")
    );
}

// Facet reports an object given to an identifier
// as `Unsupported`, which would trip a debug assertion; it is a wrong type at the member.
#[test]
fn an_identifier_that_is_an_object_is_a_type_error_and_no_panic() {
    let d = rejected(
        &document(
            "1.26.30",
            r#"{"description": {"identifier": {}}, "components": {}}"#,
        ),
        &context(),
    );
    let found: Vec<_> = d
        .iter()
        // By legacy name: the variant and the definitions that were it.
        .filter(|f| matches!(f.code.name(), "InvalidType"))
        .collect();
    assert_eq!(found.len(), 1, "{d:#?}");
    assert_eq!(found[0].severity, Severity::Error);
    assert_eq!(
        found[0].location.json_path.to_string(),
        "minecraft:biome/description/identifier"
    );
}

// The format hint reads the copy that stands. A doubled
// payload whose first copy alone has surface components gets the hint without the surface
// sentence.
#[test]
fn the_format_hint_reads_the_last_copy_of_a_doubled_payload() {
    let text = r#"{"format_version": "1.13.0", "minecraft:biome": {"description": {"identifier": "demo:x"}, "components": {"minecraft:surface_parameters": {}}}, "minecraft:biome": {"description": {"identifier": "demo:x"}, "components": {}}}"#;
    let d = rejected(text, &context());
    let hints: Vec<_> = d
        .iter()
        .filter_map(|f| match &f.code {
            DiagnosticCode::BiomeFormatHint {
                surface_components, ..
            } => Some(*surface_components),
            _ => None,
        })
        .collect();
    assert_eq!(hints, [false], "{d:#?}");
}

/// biome checks no colour: a `map_tints` colour that is no hex text, or an array channel
/// outside 0..=255, loads and is told nothing (`client_biome`'s colour rule, `is_rgb` and
/// `clamped_channels`, is that family's own).
#[test]
fn a_map_tints_colour_is_not_checked() {
    let text = biome(
        "1.26.30",
        "test:tinted",
        r##""minecraft:map_tints": {"grass": {"type": "tint", "tint": "#zz"}, "foliage": [300, -1, 2]}"##,
    );
    let document = loads(&text, &context());
    let told: Vec<String> = document
        .diagnostics()
        .iter()
        .map(|d| format!("{:?} {}", d.code, d.location.json_path))
        .collect();
    assert!(told.is_empty(), "{told:#?}");
}
