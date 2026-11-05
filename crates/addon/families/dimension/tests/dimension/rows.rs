//! Loading one dimension file: dim01–dim10, dim13–dim17b.

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_family_dimension::GeneratorType;

use super::common::*;

// The 1.26.60 texts of part A: bounds without a positive range make a height whose
// `height_range` is not above zero.
const NOT_POSITIVE: &str = "Dimension height range: height_range must be greater than zero";
const NETHER_END: &str =
    "Dimension height range: nether and the end do not support custom height values";
const NO_NAMESPACE: &str = "Custom dimensions need to have a namespace!";
const MINECRAFT_NAMESPACE: &str = "Custom dimensions cannot have the minecraft namespace!";
const GENERATOR: &str = "minecraft:generation: generator_type unsupported";

#[test]
fn dim01_the_vanilla_unit_test_input_loads_at_1_18_0() {
    let document = loads(&fixture("dim01_1_18_0/dimensions/overworld.json"));
    assert!(
        above_info(document.diagnostics()).is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    assert_eq!(
        document.description.identifier.as_str(),
        "minecraft:overworld"
    );
    let height = document.components.dimension_height.as_ref().unwrap();
    assert_eq!((height.min_y, height.height_range), (-64, 384));
    assert_eq!(
        document.components.generation.generator(),
        Some(GeneratorType::Void)
    );
    assert_eq!(GeneratorType::Void.id(), 5);
    assert!(document.version().upgraded);
}

#[test]
fn dim02_below_1_18_0_is_unsupported_with_the_puv_text() {
    let d = rejects(&dimension(
        "1.17.40",
        "minecraft:overworld",
        &bounds(-64, 320),
    ));
    let found = with_code(&d, "UnsupportedVersion");
    assert_eq!(found.len(), 1, "{d:#?}");
    assert_eq!(
        found[0].code.to_string(),
        "unsupported version: no parser available for version 1.17.40"
    );
}

#[test]
fn dim03_bounds_constraints_and_the_overworld_rules() {
    for version in ["1.18.0", "1.21.60"] {
        let d = rejects(&dimension(version, "minecraft:overworld", &bounds(8, 320)));
        assert_eq!(errors(&d), ["8 is not multiple of 16"], "{version}");
        let d = rejects(&dimension(version, "minecraft:overworld", &bounds(16, 0)));
        assert_eq!(errors(&d), [height_failure(NOT_POSITIVE)], "{version}");
        let d = rejects(&dimension(version, "minecraft:overworld", &bounds(0, 0)));
        assert_eq!(errors(&d), [height_failure(NOT_POSITIVE)], "{version}");
        let d = rejects(&dimension(
            version,
            "minecraft:overworld",
            &bounds(-64, 528),
        ));
        let found = with_code(&d, "InvalidValue");
        assert_eq!(found.len(), 1, "{version}: {d:#?}");
        assert!(
            found[0]
                .code
                .to_string()
                .starts_with("the input is outside of the accepted range")
        );
    }
}

#[test]
fn dim04_there_is_no_lenient_mode_below_1_21_60() {
    for version in ["1.20.0", "1.21.60"] {
        let text = dimension(version, "minecraft:overworld", &bounds(-64, 320))
            .replace(r#""min": -64"#, r#""min": "-64""#);
        let d = rejects(&text);
        assert_eq!(with_code(&d, "InvalidType").len(), 1, "{version}: {d:#?}");
        let text = dimension(version, "minecraft:overworld", &bounds(-64, 320))
            .replace(r#""max": 320"#, r#""max": 320, "foo": 1"#);
        let document = loads(&text);
        let found = with_code(document.diagnostics(), "UnrecognizedField");
        assert_eq!(found.len(), 1, "{version}");
        // Level 2 from the cereal slice (1.21.60), silent below it.
        let expected = if version == "1.21.60" {
            Severity::Warning
        } else {
            Severity::Info
        };
        assert_eq!(found[0].severity, expected, "{version}");
        assert_eq!(
            found[0].location.json_path.to_string(),
            "minecraft:dimension/components/minecraft:dimension_bounds/foo"
        );
    }
}

#[test]
fn dim05_nether_and_the_end_take_no_custom_height() {
    for version in ["1.18.0", "1.21.60"] {
        for identifier in ["minecraft:nether", "minecraft:the_end"] {
            let d = rejects(&dimension(version, identifier, &bounds(0, 16)));
            assert_eq!(errors(&d), [height_failure(NETHER_END)], "{identifier}");
            // The step makes a height {0, 0} of absent bounds, which passes.
            let document = loads(&dimension(version, identifier, VOID));
            let height = document.components.dimension_height.as_ref().unwrap();
            assert_eq!((height.min_y, height.height_range), (0, 0));
            assert!(above_info(document.diagnostics()).is_empty());
        }
    }
}

#[test]
fn dim06_the_overworld_needs_bounds() {
    // Absent bounds upgrade to the height {0, 0}, which part A rejects.
    for version in ["1.18.0", "1.21.60"] {
        let d = rejects(&dimension(version, "minecraft:overworld", VOID));
        assert_eq!(errors(&d), [height_failure(NOT_POSITIVE)]);
    }
}

#[test]
fn dim07_custom_identifiers_load_in_a_namespace_other_than_minecraft() {
    // 1.26.60 part B replaces the three-identifier allowlist.
    assert!(above_info(loads(&dimension("1.26.50", "custom:sky", VOID)).diagnostics()).is_empty());
    for (identifier, text) in [
        ("overworld", NO_NAMESPACE),
        ("minecraft:Overworld", MINECRAFT_NAMESPACE),
    ] {
        let d = rejects(&dimension("1.26.50", identifier, VOID));
        assert_eq!(errors(&d), [custom_failure(text)], "{identifier}");
        let found = with_code(&d, "InvalidValue");
        assert_eq!(
            found[0].location.json_path.to_string(),
            "minecraft:dimension/description/identifier"
        );
    }
}

#[test]
fn dim08_only_void_is_a_generator() {
    // 1.26.60 part C: any other text is told and the document kept.
    for generator in ["flat", "Void"] {
        let components = format!(r#""minecraft:generation": {{"generator_type": "{generator}"}}"#);
        let document = loads(&dimension("1.21.60", "minecraft:nether", &components));
        let texts: Vec<String> = above_info(document.diagnostics())
            .iter()
            .map(|f| f.code.to_string())
            .collect();
        assert_eq!(texts, [GENERATOR], "{generator}");
    }
    let document = loads(&dimension("1.21.60", "minecraft:nether", VOID));
    assert_eq!(
        document.components.generation.generator(),
        Some(GeneratorType::Void)
    );
}

#[test]
fn dim09_the_generator_check_runs_after_a_failed_identifier() {
    let components = r#""minecraft:generation": {"generator_type": "flat"}"#;
    let d = rejects(&dimension("1.26.50", "x", components));
    assert_eq!(errors(&d), [custom_failure(NO_NAMESPACE)]);
    let found = with_code(&d, "InvalidValue");
    assert_eq!(found.len(), 2, "{d:#?}");
    // a kept error
    assert_eq!((found[1].severity, found[1].kept), (Severity::Error, true));
    assert_eq!(found[1].code.to_string(), GENERATOR);
}

#[test]
fn dim10_generation_and_its_generator_type_are_required() {
    let d = rejects(&dimension("1.21.60", "minecraft:nether", ""));
    assert_eq!(with_code(&d, "MissingRequiredField").len(), 1, "{d:#?}");
    let d = rejects(&dimension(
        "1.21.60",
        "minecraft:nether",
        r#""minecraft:generation": {}"#,
    ));
    assert_eq!(with_code(&d, "MissingRequiredField").len(), 1, "{d:#?}");
}

#[test]
fn dim13_the_empty_document_is_rejected() {
    // No `format_version`: the version check rejects it before any struct reads it.
    let d = rejects("{}");
    assert_eq!(with_code(&d, "MissingFormatVersion").len(), 1, "{d:#?}");
    let d = rejects(r#"{"format_version": "1.26.50"}"#);
    assert!(!with_code(&d, "MissingRequiredField").is_empty(), "{d:#?}");
}

#[test]
fn dim14_above_the_engine_is_unsupported() {
    let d = rejects(&dimension("1.27.50", "minecraft:nether", VOID));
    let found = with_code(&d, "UnsupportedVersion");
    assert_eq!(found.len(), 1);
    assert_eq!(
        found[0].code.to_string(),
        "unsupported version: no parser available for version 1.27.50"
    );
}

#[test]
fn dim15_the_1_26_50_step_turns_bounds_into_a_height() {
    let document = loads(&dimension(
        "1.21.60",
        "minecraft:overworld",
        &bounds(-64, 320),
    ));
    let height = document.components.dimension_height.as_ref().unwrap();
    assert_eq!((height.min_y, height.height_range), (-64, 384));
    assert!(document.components.default_biome.is_none());
    let document = loads(&dimension("1.21.60", "minecraft:nether", VOID));
    let height = document.components.dimension_height.as_ref().unwrap();
    assert_eq!((height.min_y, height.height_range), (0, 0));
    assert!(document.components.default_biome.is_none());
}

#[test]
fn dim16_a_1_26_50_document() {
    let height = r#""minecraft:dimension_height": {"min_y": -64, "height_range": 384}, "minecraft:default_biome": {"biome": "minecraft:plains"}, "minecraft:generation": {"generator_type": "void"}"#;
    // A custom dimension: a vanilla one takes no default biome.
    let document = loads(&dimension("1.26.50", "demo:sky", height));
    assert!(
        above_info(document.diagnostics()).is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    assert_eq!(
        document
            .components
            .default_biome
            .as_ref()
            .unwrap()
            .biome
            .as_str(),
        "minecraft:plains"
    );
    assert!(!document.version().upgraded);
    let d = rejects(&dimension(
        "1.26.50",
        "demo:sky",
        &height.replace("384", "8"),
    ));
    assert_eq!(with_code(&d, "InvalidValue").len(), 1, "{d:#?}");
    let leftover = format!(r#"{height}, "minecraft:dimension_bounds": {{"min": -64, "max": 320}}"#);
    let document = loads(&dimension("1.26.50", "demo:sky", &leftover));
    let found = with_code(document.diagnostics(), "UnrecognizedField");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].severity, Severity::Warning);
}

#[test]
fn dim16b_1_26_50_edges() {
    let at = |min_y: i32, range: i32| {
        dimension(
            "1.26.50",
            "minecraft:overworld",
            &format!(
                r#""minecraft:dimension_height": {{"min_y": {min_y}, "height_range": {range}}}, {VOID}"#
            ),
        )
    };
    let document = loads(&at(-512, 1024));
    assert!(
        above_info(document.diagnostics()).is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    for (min_y, range) in [(513, 16), (-64, 1040)] {
        let d = rejects(&at(min_y, range));
        assert_eq!(
            with_code(&d, "InvalidValue").len(),
            1,
            "{min_y} {range}: {d:#?}"
        );
    }
    let clouds = dimension(
        "1.26.50",
        "minecraft:nether",
        &format!(r#"{VOID}, "minecraft:clouds": {{"height": 192}}"#),
    );
    // 1.26.60 binds `minecraft:clouds`, and a vanilla dimension takes none.
    let d = rejects(&clouds);
    assert_eq!(
        errors(&d),
        [custom_failure(
            "Cannot change the default height of vanilla dimensions clouds!"
        )]
    );
    assert!(above_info(loads(&dimension("1.26.50", "custom:x", VOID)).diagnostics()).is_empty());
}

#[test]
fn dim17_a_missing_format_version_is_rejected() {
    // The loader passes a zeroed version as the caller's version:
    // no key and no caller version is 0x8 "no valid version passed by
    // the caller or found in the input", rejected.
    let text = dimension("X", "minecraft:nether", VOID).replace(r#""format_version": "X", "#, "");
    let d = rejects(&text);
    let found = with_code(&d, "MissingFormatVersion");
    assert_eq!(found.len(), 1, "{d:#?}");
    assert_eq!(found[0].severity, Severity::Error);
    assert_eq!(
        found[0].code.to_string(),
        "no valid version passed by the caller or found in the input"
    );
}

#[test]
fn dim17b_envelope_edges() {
    // A number is read as its text (`MinecraftDocumentInput`): engine.rs.
    for version in [r#""abc""#, "true"] {
        let text = dimension("X", "minecraft:nether", VOID).replace(r#""X""#, version);
        let d = rejects(&text);
        assert_eq!(
            with_code(&d, "MalformedFormatVersion").len(),
            1,
            "{version}"
        );
    }
    let text = dimension("X", "minecraft:nether", VOID).replace(r#""X""#, "[1, 21, 60]");
    assert_eq!(
        loads(&text).version().written,
        Some(FormatVersion::new(1, 21, 60))
    );
    let text = dimension("1.21.60", "minecraft:nether", VOID).replacen('{', r#"{"extra": 1, "#, 1);
    let document = loads(&text);
    let found = with_code(document.diagnostics(), "UnrecognizedField");
    assert_eq!(found.len(), 1, "{:#?}", document.diagnostics());
    assert_eq!(found[0].location.json_path.to_string(), "extra");
    assert!(
        document
            .unknown()
            .get(&bedrock_addon_core::diagnostic::JsonPath::new(), "extra")
            .is_some()
    );
}

#[test]
fn a_member_named_like_a_skipped_field_is_an_unknown_member() {
    // facet rc.7 matches a skipped field by its name: `extra`, the one skipped field
    // of the dimension structs, is an unknown member wherever it is written, and so is a
    // member of no struct.
    let text = dimension(
        "1.26.50",
        "minecraft:nether",
        &format!(r#"{VOID}, "extra": 1"#),
    )
    .replace(
        r#""identifier": "minecraft:nether""#,
        r#""identifier": "minecraft:nether", "extra": {"min": 0}"#,
    );
    let document = loads(&text);
    let found: Vec<String> = with_code(document.diagnostics(), "UnrecognizedField")
        .iter()
        .map(|f| f.location.json_path.to_string())
        .collect();
    assert_eq!(
        found,
        [
            "minecraft:dimension/description/extra",
            "minecraft:dimension/components/extra"
        ]
    );
}

#[test]
fn rule_5_equal_non_zero_overworld_bounds_are_not_a_range() {
    for version in ["1.18.0", "1.21.60"] {
        let d = rejects(&dimension(version, "minecraft:overworld", &bounds(64, 64)));
        assert_eq!(errors(&d), [height_failure(NOT_POSITIVE)], "{version}");
    }
}

#[test]
fn rule_5_a_non_zero_minimum_alone_is_a_custom_height_for_nether_and_the_end() {
    for identifier in ["minecraft:nether", "minecraft:the_end"] {
        let d = rejects(&dimension("1.21.60", identifier, &bounds(-64, 0)));
        assert_eq!(errors(&d), [height_failure(NETHER_END)], "{identifier}");
    }
}
