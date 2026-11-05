//! The engine's jigsaw inputs of 1.26.60 not used elsewhere (copied under
//! `fixtures/worldgen/jigsaw/engine/`;
//! `on_placed.rs` and `registry.rs` have parser-7 and -9 and case-4). Each
//! payload is put in a 1.21.20 document, the format of every jigsaw struct. The parser tests'
//! names give their expectations (`…_Parse_Succeeds`, `…_HasExpectedValues`, `…_AreEqual`: the
//! document loads; `…_ValidationFails`: it is refused); a document that loads has no finding and
//! round-trips (value equality, a stable second write).
//!
//! Jigsaw documents the game's own unit tests feed its parser: jigsaw structures,
//! structure sets and template pools.

use bedrock_addon::diagnostic::{Diagnostics, Severity};
use bedrock_addon::document::Content;
use bedrock_addon::families::jigsaw::{
    JigsawStructure, MaxDistanceFromCenter, ProcessorList, StructureSet, TemplatePool,
};

use super::common::{document, fixture, load, loads};

/// A payload of `engine/` in a 1.21.20 document under `key`.
fn payload(name: &str, key: &str) -> String {
    let text = String::from_utf8(fixture(&format!("engine/{name}.json"))).unwrap();
    document("1.21.20", key, &text)
}

/// Loads without a finding, writes, reads back equal, writes the same again.
#[track_caller]
fn round_trips<T: Content + PartialEq + std::fmt::Debug>(path: &str, text: &str) -> T {
    let first = loads::<T>(path, text);
    assert!(
        first.diagnostics().is_empty(),
        "{path}: {:#?}",
        first.diagnostics()
    );
    let written = first.to_json().unwrap();
    let again = loads::<T>(path, &written);
    assert_eq!(again.value(), first.value(), "{written}");
    assert_eq!(again.to_json().unwrap(), written);
    first.into_value()
}

/// The findings at warning level or above: `(code name, severity, path)`.
fn found(d: &Diagnostics) -> Vec<(&'static str, Severity, String)> {
    d.iter()
        .filter(|f| f.severity >= Severity::Warning)
        .map(|f| (f.code.name(), f.severity, f.location.json_path.to_string()))
        .collect()
}

const STRUCTURE: &str = "worldgen/structures/trail_ruins.json";

/// `…TemplatePoolJsonWithFeaturePoolElement_Parse_Succeeds` (parser-1, with
/// its trailing commas, which the game's reader takes).
#[test]
fn the_parser_test_s_feature_pool_parses() {
    round_trips::<TemplatePool>(
        "worldgen/template_pools/feature_pool_structure.json",
        &payload("parser-1", "minecraft:template_pool"),
    );
}

/// `…StructureSetJson_ProcessSet_HasExpectedValues` (-2, triangular) and
/// `…LoadDataDrivenStructureSet_CompareWithHardcodedSet_StructureSetsAreEqual` (-10, linear):
/// the trail ruins' set, spacing 34 and separation 8.
#[test]
fn the_parser_test_s_structure_sets_load() {
    for name in ["parser-2", "parser-10"] {
        let set = round_trips::<StructureSet>(
            "worldgen/structure_sets/trail_ruins.json",
            &payload(name, "minecraft:structure_set"),
        );
        assert_eq!(
            (set.placement.spacing.0, set.placement.separation.0),
            (34, 8),
            "{name}"
        );
    }
}

/// The three `max_distance_from_center` forms of the `…ProcessJigsawStructure_HasExpectedValues`
/// tests: an object with the horizontal distance only (-3), with both (-4), and a number (-5).
#[test]
fn the_parser_test_s_max_distance_forms_load() {
    let horizontal =
        round_trips::<JigsawStructure>(STRUCTURE, &payload("parser-3", "minecraft:jigsaw"));
    assert!(matches!(
        horizontal.max_distance_from_center,
        Some(MaxDistanceFromCenter::Object { .. })
    ));
    let both = round_trips::<JigsawStructure>(STRUCTURE, &payload("parser-4", "minecraft:jigsaw"));
    assert!(matches!(
        both.max_distance_from_center,
        Some(MaxDistanceFromCenter::Object { .. })
    ));
    let number =
        round_trips::<JigsawStructure>(STRUCTURE, &payload("parser-5", "minecraft:jigsaw"));
    assert_eq!(
        number.max_distance_from_center,
        Some(MaxDistanceFromCenter::Horizontal(7))
    );
}

/// `…LoadDataDrivenJigsaw_CompareWithHardcodedJigsaw_JigsawStructuresAreEqual` (-8): the trail
/// ruins loads.
#[test]
fn the_parser_test_s_trail_ruins_loads() {
    let ruins = round_trips::<JigsawStructure>(STRUCTURE, &payload("parser-8", "minecraft:jigsaw"));
    assert_eq!(ruins.max_depth.0, 7);
}

/// `…BlockRulesJsonInvalidVersion_ValidateBlockRules_ValidationFails` (-6, a whole document):
/// `"format_version": "invalid"` is refused at the version.
#[test]
fn the_parser_test_s_processor_list_with_an_invalid_version_is_refused() {
    let text = String::from_utf8(fixture("engine/parser-6.json")).unwrap();
    let refused = load::<ProcessorList>("worldgen/processors/block_rules_parsing.json", &text)
        .expect_err("refused");
    let errors = found(&refused.diagnostics);
    assert!(!errors.is_empty());
    assert_eq!(
        (errors[0].0, errors[0].1, errors[0].2.as_str()),
        ("MalformedFormatVersion", Severity::Error, "format_version"),
        "{errors:?}"
    );
    assert!(errors.iter().all(|e| e.1 == Severity::Error), "{errors:?}");
}

/// The unit-test library's three structures (owning tests not traced): case-2 and -3
/// load; case-1, whose `max_depth` 21 is above the bound range {0, 20} (`v1_21_20`
/// `Contents`, unchanged in 1.26.60), is refused at it.
#[test]
fn the_unit_test_library_s_structures() {
    for name in ["case-2", "case-3"] {
        round_trips::<JigsawStructure>(STRUCTURE, &payload(name, "minecraft:jigsaw"));
    }
    let refused = load::<JigsawStructure>(STRUCTURE, &payload("case-1", "minecraft:jigsaw"))
        .expect_err("refused");
    let errors = found(&refused.diagnostics);
    assert!(
        errors.iter().any(|e| e.0 == "InvalidValue"
            && e.1 == Severity::Error
            && e.2 == "minecraft:jigsaw/max_depth"),
        "{errors:?}"
    );
}
