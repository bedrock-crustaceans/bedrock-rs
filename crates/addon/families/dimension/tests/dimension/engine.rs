//! The dimension loader of 1.26.60: its input kind, the
//! validator, the upgrade callback,
//! `minecraft:clouds` and the runtime defaults, tested on the engine's own test inputs
//! (`engine/`).
//!
//! Dimension documents the game's own unit tests feed its loader; the `.fmt.txt` files are
//! format-string templates the tests fill with bounds and heights before parsing.

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::version::FormatVersion;

use super::common::*;

// ---- input kind: `MinecraftDocumentInput` ------------------------------

#[test]
fn a_null_member_of_a_cereal_slice_is_a_type_error_that_rejects() {
    // The game's strict schema reader finds the member and its type
    // fails on `null`; an `std::optional` component is no exception.
    for (version, member) in [
        ("1.26.50", r#""minecraft:default_biome": null"#),
        ("1.26.50", r#""minecraft:dimension_height": null"#),
        ("1.21.60", r#""minecraft:dimension_bounds": null"#),
    ] {
        let d = rejects(&dimension(
            version,
            "minecraft:nether",
            &format!("{member}, {VOID}"),
        ));
        let found = with_code(&d, "InvalidType");
        assert_eq!(found.len(), 1, "{version} {member}: {d:#?}");
        assert_eq!(found[0].severity, Severity::Error);
        let name = member.split('"').nth(1).unwrap();
        assert_eq!(
            found[0].location.json_path.to_string(),
            format!("minecraft:dimension/components/{name}")
        );
    }
}

#[test]
fn below_1_21_60_the_legacy_parser_keeps_its_own_reading_of_null() {
    // The legacy slice is a hand-written parser: the first
    // run's reading stands, a `null` component is absent.
    let document = loads(&dimension(
        "1.20.0",
        "minecraft:nether",
        &format!(r#""minecraft:dimension_bounds": null, {VOID}"#),
    ));
    assert!(above_info(document.diagnostics()).is_empty());
}

#[test]
fn the_version_is_read_as_minecraft_document_input_reads_it() {
    // `initAsJson`: a number is read as its text.
    let text = dimension("X", "minecraft:nether", VOID).replace(r#""X""#, "1.21");
    let document = loads(&text);
    assert_eq!(
        document.version().written,
        Some(FormatVersion::new(1, 21, 0))
    );
}

// ---- the upgrade callback `v1_21_60 → v1_26_50` (`customizeDimensionUpgrade` ---

/// The height the step makes of `bounds` (`None`: no bounds written), on the struct.
fn upgraded_height(identifier: &str, bounds: Option<(i32, i32)>) -> Option<(i16, i16)> {
    use bedrock_addon_core::document::Extra;
    use bedrock_addon_core::json::StrictString;
    use bedrock_addon_core::load::LoadContext;
    use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};
    use bedrock_addon_family_dimension::history::v1_18_0::{Components, Dimension};
    use bedrock_addon_family_dimension::{DimensionBounds, DimensionDescription, Generation};

    let old = Dimension {
        description: DimensionDescription {
            identifier: bedrock_addon_core::id::DimensionId::parse(identifier.to_owned()),
            extra: Extra::default(),
        },
        components: Components {
            dimension_bounds: bounds.map(|(min, max)| DimensionBounds::new(min, max)),
            generation: Generation {
                generator_type: StrictString("void".to_owned()),
                extra: Extra::default(),
            },
            extra: Extra::default(),
        },
        source_format_version: FormatVersion::new(1, 21, 60),
        root_extra: Extra::default(),
    };
    let mut diagnostics = bedrock_addon_core::diagnostic::Diagnostics::default();
    let new = old.upgrade(&mut UpgradeCx {
        load: &LoadContext::detached(),
        original_version: FormatVersion::new(1, 21, 60),
        identifier,
        actor_type: bedrock_addon_core::vanilla::ActorType::Mob,
        skip_section_a: false,
        diagnostics: &mut diagnostics,
    });
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    new.components
        .dimension_height
        .map(|h| (h.min_y.0, h.height_range.0))
}

#[test]
fn the_upgrade_always_makes_a_height_min_and_max_minus_min() {
    // `min_y = (i16)min`, `height_range = (i16)(max − min)`, and the presence flag set even for
    // absent bounds (the presence flag of the height).
    let overworld = "minecraft:overworld";
    assert_eq!(
        upgraded_height(overworld, Some((-64, 320))),
        Some((-64, 384))
    );
    assert_eq!(upgraded_height(overworld, Some((16, 0))), Some((16, -16)));
    assert_eq!(upgraded_height(overworld, Some((64, 64))), Some((64, 0)));
    assert_eq!(upgraded_height(overworld, None), Some((0, 0)));
    assert_eq!(upgraded_height("minecraft:nether", None), Some((0, 0)));
    // Sixteen bits of each, as `bfi` stores them. No parsed file reaches a cut: the bind
    // constraints keep both bounds in [−512, 512], so `min` and `max − min`
    // (at most 1024 apart) always fit an i16; only a hand-built value shows the wrap.
    assert_eq!(
        upgraded_height(overworld, Some((-512, 40_000))),
        Some((-512, i16::try_from(40_512_i32 - 65_536).unwrap()))
    );
}

#[test]
fn the_engine_s_1_21_60_inputs_upgrade_to_a_height() {
    // main-1: the overworld's bounds {0, 16}.
    let document = loads(&engine("main-1.json"));
    let height = document.components.dimension_height.as_ref().unwrap();
    assert_eq!((height.min_y.0, height.height_range.0), (0, 16));
    // The nether without bounds: a present height {0, 0}.
    let document = loads(&render("case-9", &["1.21.60", "minecraft:nether"]));
    let height = document.components.dimension_height.as_ref().unwrap();
    assert_eq!((height.min_y.0, height.height_range.0), (0, 0));
    assert!(above_info(document.diagnostics()).is_empty());
}

// ---- part A of the validator: the height…) ----------------------------------------

const NETHER_END: &str =
    "Dimension height range: nether and the end do not support custom height values";
const RANGE_GRID: &str = "Dimension height range: height_range must be divisible by 16";
const MIN_Y_GRID: &str = "Dimension min y: min_y must be divisible by 16";
const NOT_POSITIVE: &str = "Dimension height range: height_range must be greater than zero";
const ABOVE_512: &str = "Dimension height range: min_y plus height_range must not exceed 512";

/// The two texts of a part-A failure.
fn height_failure(content_log: &str) -> String {
    validator(content_log, "Invalid dimension height.")
}

/// The one finding of a part-A failure, which must be at `at` and reject the file.
#[track_caller]
fn rejected_for_height(text: &str, content_log: &str, at: &str) {
    let d = rejects(text);
    assert_eq!(errors(&d), [height_failure(content_log)], "{text}");
    let found = with_code(&d, "InvalidValue");
    assert_eq!(found[0].location.json_path.to_string(), at);
}

const HEIGHT_AT: &str = "minecraft:dimension/components/minecraft:dimension_height";
const BOUNDS_AT: &str = "minecraft:dimension/components/minecraft:dimension_bounds";

#[test]
fn part_a_judges_a_1_26_50_height_with_the_engine_s_texts() {
    // case-16 (invalid bounds in the min_y and height format),
    // and case-2, the same document with another indentation: judged alike.
    for template in ["case-16", "case-2"] {
        let at = |id: &str, min_y: &str, range: &str| render(template, &[id, min_y, range]);
        rejected_for_height(
            &at("minecraft:overworld", "-60", "384"),
            MIN_Y_GRID,
            HEIGHT_AT,
        );
        rejected_for_height(
            &at("minecraft:overworld", "256", "512"),
            ABOVE_512,
            HEIGHT_AT,
        );
        rejected_for_height(&at("minecraft:nether", "0", "128"), NETHER_END, HEIGHT_AT);
        rejected_for_height(
            &at("minecraft:the_end", "-64", "384"),
            NETHER_END,
            HEIGHT_AT,
        );
        // `height_range` off the grid fails its bind constraint first; part A is not reached.
        let d = rejects(&at("minecraft:overworld", "-64", "390"));
        assert_eq!(errors(&d), ["390 is not multiple of 16"], "{template}");
        // The edges of a valid height.
        for (min_y, range) in [("-512", "1024"), ("496", "16"), ("0", "512")] {
            let document = loads(&at("minecraft:overworld", min_y, range));
            assert!(
                above_info(document.diagnostics()).is_empty(),
                "{template} {min_y} {range}"
            );
        }
    }
    let _ = RANGE_GRID; // the guard: the bind constraint is met first
}

#[test]
fn part_a_judges_the_height_the_upgrade_makes_of_1_21_60_bounds() {
    // case-1: incorrect height values, or none for a custom dimension.
    let at =
        |version: &str, id: &str, min: &str, max: &str| render("case-1", &[version, id, min, max]);
    for version in ["1.18.0", "1.21.60"] {
        let overworld = "minecraft:overworld";
        rejected_for_height(&at(version, overworld, "16", "0"), NOT_POSITIVE, BOUNDS_AT);
        rejected_for_height(&at(version, overworld, "0", "0"), NOT_POSITIVE, BOUNDS_AT);
        rejected_for_height(&at(version, overworld, "64", "64"), NOT_POSITIVE, BOUNDS_AT);
        rejected_for_height(
            &at(version, "minecraft:nether", "0", "16"),
            NETHER_END,
            BOUNDS_AT,
        );
        rejected_for_height(
            &at(version, "minecraft:the_end", "-64", "0"),
            NETHER_END,
            BOUNDS_AT,
        );
        // A nether's zero bounds are the height {0, 0}, which passes.
        assert!(
            loads(&at(version, "minecraft:nether", "0", "0"))
                .diagnostics()
                .iter()
                .all(|f| f.severity <= Severity::Info)
        );
        let document = loads(&at(version, overworld, "-64", "320"));
        assert!(above_info(document.diagnostics()).is_empty());
    }
    // An overworld without bounds is the height {0, 0}: rejected (`height_range` ≤ 0).
    rejected_for_height(
        &render("case-9", &["1.21.60", "minecraft:overworld"]),
        NOT_POSITIVE,
        BOUNDS_AT,
    );
}

#[test]
fn part_a_takes_the_default_height_when_there_is_none() {
    // case-9 (the overworld without a height component takes the default height)
    // and case-15 (the nether without a height component).
    let document = loads(&render("case-9", &["1.26.50", "minecraft:overworld"]));
    assert!(
        above_info(document.diagnostics()).is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    assert!(document.components.dimension_height.is_none());
    let document = loads(&engine("case-15.json"));
    assert!(
        above_info(document.diagnostics()).is_empty(),
        "{:#?}",
        document.diagnostics()
    );
}

// ---- part B of the validator: the identifier --------------------------

const VANILLA_BIOME: &str = "Cannot change the default value of vanilla dimensions biomes!";
const MINECRAFT_NAMESPACE: &str = "Custom dimensions cannot have the minecraft namespace!";
const NO_NAMESPACE: &str = "Custom dimensions need to have a namespace!";
const VANILLA: [&str; 3] = [
    "minecraft:overworld",
    "minecraft:nether",
    "minecraft:the_end",
];

#[test]
fn part_b_custom_dimensions_load() {
    // case-5 (a default biome is retained), -6, -10 (a namespaced custom dimension keeps its
    // JSON values), -11.
    for name in ["case-5.json", "case-6.json", "case-10.json", "case-11.json"] {
        let document = loads(&engine(name));
        assert!(
            above_info(document.diagnostics()).is_empty(),
            "{name}: {:#?}",
            document.diagnostics()
        );
        assert_eq!(
            document.description.identifier.as_str(),
            "test:custom_dimension"
        );
    }
    let document = loads(&engine("case-5.json"));
    let biome = document.components.default_biome.as_ref().unwrap();
    assert_eq!(biome.biome.as_str(), "test:my_biome");
    let document = loads(&engine("case-10.json"));
    let height = document.components.dimension_height.as_ref().unwrap();
    assert_eq!((height.min_y.0, height.height_range.0), (-32, 128));
}

#[test]
fn part_b_a_vanilla_dimension_takes_no_default_biome() {
    // case-19 and -13 (a vanilla dimension with a default biome is rejected,
    // parameterised over the three vanilla identifiers).
    for identifier in VANILLA {
        let d = rejects(&render("case-19", &[identifier, "minecraft:plains"]));
        assert_eq!(errors(&d), [custom_failure(VANILLA_BIOME)], "{identifier}");
        assert_eq!(
            with_code(&d, "InvalidValue")[0]
                .location
                .json_path
                .to_string(),
            "minecraft:dimension/components/minecraft:default_biome"
        );
    }
    let d = rejects(&render(
        "case-13",
        &["minecraft:overworld", "minecraft:plains"],
    ));
    assert_eq!(errors(&d), [custom_failure(VANILLA_BIOME)]);
    // Part A runs first and part B still runs: the nether's height and its biome.
    let d = rejects(&render(
        "case-13",
        &["minecraft:nether", "minecraft:plains"],
    ));
    assert_eq!(
        errors(&d),
        [
            height_failure(
                "Dimension height range: nether and the end do not support custom height values"
            ),
            custom_failure(VANILLA_BIOME)
        ]
    );
    // A custom dimension keeps it (case-3 / -19 with a custom identifier).
    for template in ["case-3", "case-19"] {
        let document = loads(&render(
            template,
            &["test:custom_dimension", "test:my_biome"],
        ));
        assert!(above_info(document.diagnostics()).is_empty(), "{template}");
    }
}

#[test]
fn part_b_a_custom_identifier_needs_a_namespace_other_than_minecraft() {
    // case-14 / -17 (a `minecraft`-namespaced and an unnamespaced custom dimension are
    // rejected). The game's namespace check lower-cases the identifier and wants exactly one `:` between two non-empty
    // parts, the name without white space.
    for template in ["case-14", "case-17"] {
        for (identifier, text) in [
            ("minecraft:custom_dimension", MINECRAFT_NAMESPACE),
            ("Minecraft:Custom", MINECRAFT_NAMESPACE),
            ("minecraft:Overworld", MINECRAFT_NAMESPACE),
            ("custom_dimension", NO_NAMESPACE),
            ("overworld", NO_NAMESPACE),
        ] {
            let d = rejects(&render(template, &[identifier]));
            assert_eq!(
                errors(&d),
                [custom_failure(text)],
                "{template} {identifier}"
            );
            assert_eq!(
                with_code(&d, "InvalidValue")[0]
                    .location
                    .json_path
                    .to_string(),
                "minecraft:dimension/description/identifier"
            );
        }
        // Any `:` is a namespace for the second rule, and the first wants a well-formed one.
        for identifier in [
            "test:custom_dimension",
            ":void",
            "a:b:c",
            "minecraft:a:b",
            "minecraft:a b",
        ] {
            let document = loads(&render(template, &[identifier]));
            assert!(
                above_info(document.diagnostics()).is_empty(),
                "{template} {identifier}"
            );
        }
    }
}

// ---- `minecraft:clouds`, and the runtime defaults --------------

const VANILLA_CLOUDS: &str = "Cannot change the default height of vanilla dimensions clouds!";

#[test]
fn clouds_in_range_load_and_set_the_cloud_height() {
    // case-4: an in-range clouds height is added.
    for height in [-512_i16, -64, 0, 192, 512] {
        let document = loads(&render(
            "case-4",
            &["test:custom_dimension", &height.to_string()],
        ));
        assert!(above_info(document.diagnostics()).is_empty(), "{height}");
        assert_eq!(
            document.components.clouds.as_ref().unwrap().height.0,
            height
        );
        assert_eq!(document.runtime_cloud_height(), height);
    }
}

#[test]
fn clouds_out_of_range_reject() {
    // case-8: an out-of-range clouds height is rejected; the range is −512 to 512.
    for height in ["-513", "513", "1000"] {
        let d = rejects(&render("case-8", &["test:custom_dimension", height]));
        assert_eq!(
            errors(&d),
            // `rangeMessage`: the bounds, not the value.
            ["the input is outside of the accepted range [-512, 512]".to_owned()]
        );
        assert_eq!(
            with_code(&d, "InvalidValue")[0]
                .location
                .json_path
                .to_string(),
            "minecraft:dimension/components/minecraft:clouds/height"
        );
    }
    // `height` is required, and `null` is no value.
    let clouds = |value: &str| {
        dimension(
            "1.26.50",
            "test:custom_dimension",
            &format!(r#""minecraft:clouds": {value}, {VOID}"#),
        )
    };
    assert_eq!(
        with_code(&rejects(&clouds("{}")), "MissingRequiredField").len(),
        1
    );
    assert_eq!(with_code(&rejects(&clouds("null")), "InvalidType").len(), 1);
}

#[test]
fn part_b_a_vanilla_dimension_takes_no_clouds() {
    for identifier in VANILLA {
        let text = dimension(
            "1.26.50",
            identifier,
            &format!(r#""minecraft:clouds": {{"height": 192}}, {VOID}"#),
        );
        let d = rejects(&text);
        assert_eq!(errors(&d), [custom_failure(VANILLA_CLOUDS)], "{identifier}");
        assert_eq!(
            with_code(&d, "InvalidValue")[0]
                .location
                .json_path
                .to_string(),
            "minecraft:dimension/components/minecraft:clouds"
        );
    }
    let d = rejects(&render("case-4", &["minecraft:overworld", "192"]));
    assert_eq!(errors(&d), [custom_failure(VANILLA_CLOUDS)]);
    // The default biome is checked first; one failure is reported.
    let both = dimension(
        "1.26.50",
        "minecraft:the_end",
        &format!(
            r#""minecraft:default_biome": {{"biome": "minecraft:plains"}}, "minecraft:clouds": {{"height": 192}}, {VOID}"#
        ),
    );
    assert_eq!(
        errors(&rejects(&both)),
        [custom_failure(
            "Cannot change the default value of vanilla dimensions biomes!"
        )]
    );
}

#[test]
fn the_runtime_definition_takes_the_defaults_of_absent_components() {
    // The runtime definition takes the height or {−64, 384}, the biome or
    // `minecraft:ocean`, the clouds or 128.
    // case-11 (no clouds take the default height), -5 (a default biome is retained), -10 (JSON
    // values are kept).
    let bare = loads(&engine("case-11.json"));
    assert!(bare.components.clouds.is_none());
    assert_eq!(bare.runtime_cloud_height(), 128);
    assert_eq!(bare.runtime_default_biome().as_str(), "minecraft:ocean");
    assert_eq!(bare.runtime_height(), (-64, 384));
    let biome = loads(&engine("case-5.json"));
    assert_eq!(biome.runtime_default_biome().as_str(), "test:my_biome");
    let height = loads(&engine("case-10.json"));
    assert_eq!(height.runtime_height(), (-32, 128));
}

// ---- part C of the validator: the generator -------------------------------------

#[test]
fn part_c_an_unknown_generator_is_told_and_the_document_kept() {
    // The game knows `void` alone as a generator; a miss is a content-log error
    // with no schema error after it, so the file loads: a kept error.
    // case-7 and -12 (the generator a parameter).
    for text in [
        render("case-7", &["1.26.50", "test:custom_dimension", "flat"]),
        render("case-7", &["1.26.50", "minecraft:nether", "Void"]),
        render(
            "case-12",
            &["1.21.60", "minecraft:overworld", "-64", "320", "overworld"],
        ),
    ] {
        let document = loads(&text);
        let found = above_info(document.diagnostics());
        assert_eq!(found.len(), 1, "{text}: {found:#?}");
        assert_eq!((found[0].severity, found[0].kept), (Severity::Error, true));
        assert_eq!(
            found[0].code.to_string(),
            "minecraft:generation: generator_type unsupported"
        );
        assert_eq!(
            found[0].location.json_path.to_string(),
            "minecraft:dimension/components/minecraft:generation/generator_type"
        );
        assert_eq!(document.components.generation.generator(), None);
    }
    let document = loads(&render("case-7", &["1.21.60", "minecraft:nether", "void"]));
    assert!(above_info(document.diagnostics()).is_empty());
}

// ---- structure ----------------------------------------------------------------------------

#[test]
fn a_document_without_minecraft_generation_is_rejected() {
    // case-18 (incorrect structure): `minecraft:generation` is a required member of `Components`.
    let d = rejects(&engine("case-18.json"));
    let found = with_code(&d, "MissingRequiredField");
    assert_eq!(found.len(), 1, "{d:#?}");
    assert_eq!(errors(&d).len(), 1, "{d:#?}");
    assert_eq!(
        found[0].location.json_path.to_string(),
        "minecraft:dimension/components/minecraft:generation"
    );
}

/// The game's loader performance input for dimensions of 1.26.60, verbatim: the same
/// document as `engine/main-1.json` but for white space. The 1.21.60 overworld loads
/// without a warning, upgraded to the height {0, 16}, and round-trips.
#[test]
fn loader_perf_tests_2_the_perf_dimension_loads_and_round_trips() {
    let document = loads(&fixture("loader_perf/LoaderPerfTests-2.json"));
    assert!(
        above_info(document.diagnostics()).is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    let height = document.components.dimension_height.as_ref().unwrap();
    assert_eq!((height.min_y.0, height.height_range.0), (0, 16));
    let written = document.to_json().unwrap();
    let again = loads(&written);
    assert_eq!(again.value().components, document.value().components);
    assert_eq!(again.to_json().unwrap(), written);
}

/// The texts of 1.26.20 and retail
/// 1.26.45 that 1.26.60 does not have are never reported, whatever input once raised them:
/// out-of-grid and inverted bounds, a missing overworld height, nether bounds, a custom or an
/// unnamespaced identifier, an unknown generator.
#[test]
fn no_text_gone_from_1_26_60_is_reported() {
    const GONE: [&str; 6] = [
        "Invalid dimension bounds.",
        "Invalid generator type.",
        "minecraft:dimension_bounds:",
        "currently only 'overworld', 'nether' and 'the_end' are supported",
        "Invalid dimension name '",
        "was ignored because it was already registered via scripting",
    ];
    let inputs = [
        dimension("1.21.60", "minecraft:overworld", &bounds(8, 64)),
        dimension("1.21.60", "minecraft:overworld", &bounds(16, 0)),
        dimension("1.21.60", "minecraft:overworld", &bounds(0, 0)),
        dimension("1.20.0", "minecraft:overworld", VOID),
        dimension("1.21.60", "minecraft:nether", &bounds(0, 16)),
        dimension("1.26.50", "custom:sky", VOID),
        dimension("1.26.50", "overworld", VOID),
        dimension("1.26.50", "Test:Void", VOID),
        dimension(
            "1.26.50",
            "custom:sky",
            r#""minecraft:generation": {"generator_type": "flat"}"#,
        ),
    ];
    for text in &inputs {
        let found = match load(text) {
            Ok(document) => document.diagnostics().clone(),
            Err(rejected) => rejected.diagnostics,
        };
        for finding in &found {
            let said = finding.code.to_string();
            assert!(
                GONE.iter().all(|gone| !said.contains(gone)),
                "{text}: {said}"
            );
        }
    }
}
