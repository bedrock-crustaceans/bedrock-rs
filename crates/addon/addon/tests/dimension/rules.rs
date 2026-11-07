//! Rows dim18 and dim19 and the validation rules 2, 10 and 10a; the listed deviations; the
//! round trip.

#[cfg(feature = "biome")]
use bedrock_addon::diagnostic::Severity;
use bedrock_addon::families::dimension::Dimension;

use super::common::*;

/// A 1.26.50 overworld with `min_y` and `height_range`.
fn overworld_height(min_y: i32, range: i32) -> String {
    dimension(
        "1.26.50",
        "minecraft:overworld",
        &format!(
            r#""minecraft:dimension_height": {{"min_y": {min_y}, "height_range": {range}}}, {VOID}"#
        ),
    )
}

#[test]
fn rule_10_an_overworld_without_a_height_takes_the_default_and_the_nether_must_not_have_one() {
    // 1.26.60 part A: an absent height is {−64, 384}; the nether and the end take
    // none but {0, 0}.
    let document = loads(&dimension("1.26.50", "minecraft:overworld", VOID));
    assert!(above_info(document.diagnostics()).is_empty());
    let height = r#""minecraft:dimension_height": {"min_y": 0, "height_range": 128}"#;
    for identifier in ["minecraft:nether", "minecraft:the_end"] {
        let d = rejects(&dimension(
            "1.26.50",
            identifier,
            &format!("{height}, {VOID}"),
        ));
        assert_eq!(
            errors(&d),
            [height_failure(
                "Dimension height range: nether and the end do not support custom height values"
            )]
        );
    }
}

#[test]
fn rule_10_an_off_grid_min_y_and_a_top_above_512_reject() {
    // Errors in 1.26.60 as errors, not warnings.
    let d = rejects(&overworld_height(-60, 384));
    assert_eq!(
        errors(&d),
        [height_failure(
            "Dimension min y: min_y must be divisible by 16"
        )]
    );
    let d = rejects(&overworld_height(256, 512));
    assert_eq!(
        errors(&d),
        [height_failure(
            "Dimension height range: min_y plus height_range must not exceed 512"
        )]
    );
    assert!(above_info(loads(&overworld_height(-64, 384)).diagnostics()).is_empty());
}

#[cfg(feature = "biome")]
#[test]
fn rule_10_a_default_biome_no_biome_defines_is_an_info() {
    use bedrock_addon::load::LoadOptions;
    use bedrock_addon::pack::{PackOrigin, PackRepository, PackSelection, WorldPacks};
    // A custom dimension: a vanilla one takes no default biome.
    let text: &'static [u8] = dimension(
        "1.26.50",
        "demo:moon",
        &format!(r#""minecraft:dimension_height": {{"min_y": -64, "height_range": 384}}, "minecraft:default_biome": {{"biome": "demo:nowhere"}}, {VOID}"#),
    )
    .leak()
    .as_bytes();
    let spec = super::pack_common::bp(1).file("dimensions/demo_moon.json", text);
    let mut repository = PackRepository::new();
    repository.add_pack(spec.open(), PackOrigin::World);
    let id = spec.id();
    let world = WorldPacks {
        behavior: vec![PackSelection {
            pack_id: id.uuid,
            version: id.version,
            subpack: None,
        }],
        resource: Vec::new(),
    };
    let d = repository.stacks(&world, LoadOptions::default()).link();
    assert!(
        d.iter()
            .any(|f| f.severity == Severity::Info && f.code.name() == "UnresolvedReference")
    );
}

#[test]
fn dim18_without_a_registration_a_custom_dimension_loads() {
    // 1.26.60: no allowlist; the collision with a script registration is
    // `stack.rs::dim18_*`.
    let document = loads(&dimension("1.26.50", "test:void", VOID));
    assert!(above_info(document.diagnostics()).is_empty());
}

#[test]
fn d14_a_custom_dimension_the_world_could_not_create_is_a_warning_for_the_host() {
    // 1.26.60 `DimensionManager::serverInitialize(…)::$_0`: the runtime's
    // failure to create a pack's dimension, a warning naming it and its pack. The
    // 1.26.45 script-name check (`check_custom_name`) does not exist.
    use bedrock_addon::id::DimensionId;
    use bedrock_addon::load::LoadOptions;
    use bedrock_addon::pack::{Behavior, PackStack};
    let text: &'static [u8] = dimension("1.26.50", "test:void", VOID).leak().as_bytes();
    let pack = super::pack_common::bp(1)
        .file("dimensions/test_void.json", text)
        .open();
    let stack = PackStack::<Behavior>::of([pack], LoadOptions::default()).unwrap();
    let document = stack
        .dimensions()
        .get(&DimensionId::new("test:void").unwrap())
        .unwrap();
    let finding = Dimension::creation_failed(document);
    assert_eq!(finding.severity, Severity::Warning);
    assert_eq!(
        finding.code.to_string(),
        "Dimension 'test:void' from pack id '00000000-0000-0000-0000-000000000001' was invalid and could not be created!"
    );
    assert_eq!(
        finding
            .location
            .path
            .as_ref()
            .map(bedrock_addon_core::pack_identity::PackPath::as_str),
        Some("dimensions/test_void.json")
    );
    assert_eq!(
        finding.location.json_path.to_string(),
        "minecraft:dimension/description/identifier"
    );
}

#[test]
fn rule_2_every_parse_error_is_its_own_finding_below_1_21_60_deviation() {
    // Vanilla folds them into "DimensionLegacyParser expects jsonString input format: …" (TOML
    // `vanilla = "one joined Error text …"`); facet stops at the first, which is ours.
    let text = dimension("1.20.0", "minecraft:overworld", &bounds(-64, 320))
        .replace(r#""min": -64"#, r#""min": "-64""#);
    let d = rejects(&text);
    assert_eq!(errors(&d).len(), 1);
    assert!(!errors(&d)[0].starts_with("DimensionLegacyParser"));
}

#[test]
fn unknown_members_warn_below_1_21_60_deviation() {
    // Vanilla is silent below 1.21.60 (TOML `vanilla = "silent below 1.21.60"`).
    let text = dimension("1.18.0", "minecraft:nether", VOID).replace(
        r#""identifier": "minecraft:nether""#,
        r#""identifier": "minecraft:nether", "foo": 1"#,
    );
    let document = loads(&text);
    let found = with_code(document.diagnostics(), "UnrecognizedField");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].severity, Severity::Info);
}

#[test]
fn the_format_version_member_is_not_an_unknown_member_deviation() {
    // No dimension struct binds `format_version`, so vanilla's cereal slice reports it as an
    // unrecognised member (inferred); ours reads it as the envelope.
    let document = loads(&dimension("1.21.60", "minecraft:nether", VOID));
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
}

#[test]
fn every_document_round_trips() {
    for text in [
        overworld_height(-64, 384)
            .replace("minecraft:overworld", "demo:sky")
            .replace(
                VOID,
                &format!(
                    r#"{VOID}, "minecraft:default_biome": {{"biome": "minecraft:plains", "x": 1}}"#
                ),
            ),
        dimension("1.26.50", "minecraft:nether", VOID).replacen('{', r#"{"note": "kept", "#, 1),
        dimension("1.21.60", "minecraft:overworld", &bounds(-64, 320)),
    ] {
        let first = loads(&text);
        let written = first.to_json().unwrap();
        let second = loads(&written);
        let (mut a, b) = (first.value().clone(), second.value().clone());
        a.source_format_version = b.source_format_version;
        assert_eq!(a, b, "{written}");
        assert_eq!(second.to_json().unwrap(), written);
    }
    let written = loads(&dimension(
        "1.21.60",
        "minecraft:overworld",
        &bounds(-64, 320),
    ))
    .to_json()
    .unwrap();
    assert!(
        written.contains(r#""format_version": "1.26.50""#),
        "{written}"
    );
    assert!(written.contains("minecraft:dimension_height"), "{written}");
    assert!(!written.contains("dimension_bounds"), "{written}");
}

// Facet reports an object given to an identifier
// as `Unsupported`; it is a wrong type at the member and must not panic.
#[test]
fn an_identifier_that_is_an_object_is_a_type_error_and_no_panic() {
    let json =
        dimension("1.26.50", "minecraft:overworld", "").replace(r#""minecraft:overworld""#, "{}");
    assert!(json.contains(r#""identifier": {}"#));
    let d = rejects(&json);
    let found = with_code(&d, "InvalidType");
    assert_eq!(found.len(), 1, "{d:#?}");
    assert_eq!(found[0].severity, Severity::Error);
    assert_eq!(
        found[0].location.json_path.to_string(),
        "minecraft:dimension/description/identifier"
    );
}
