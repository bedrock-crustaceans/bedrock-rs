//! Characterization: the text a document below a chain's floor is told today, for the version
//! spellings that render apart (two parts, three integers, a pre-release, a bare number), over
//! three families that each render the version their own way: the camera preset (the raw
//! envelope text, `chain_file::puv_below_floor`), POI (the version as the input parsed it,
//! `versioning::no_parser_version`) and feature rules (`FormatVersion::to_string`,
//! `support::puv`). The rows of voxel shape, the jigsaw processor list, spawn rules, the
//! lighting settings of `deferred_settings`, block culling and the crafting catalog pin the
//! renderings and texts the rest of the families give today (spawn groups and the other
//! jigsaw and deferred documents are not pinned here).
//!
//! This pins the behaviour `Family::BELOW_FLOOR` will take over; the change that picks one
//! rendering updates these rows with its reasons and deletes this file.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(any(
    feature = "camera",
    feature = "poi",
    feature = "feature_rules",
    feature = "voxel_shape",
    feature = "jigsaw",
    feature = "spawn_rules",
    feature = "deferred_settings",
    feature = "block_culling",
    feature = "crafting_catalog",
))]

use bedrock_addon::diagnostic::{DiagnosticCode, Diagnostics, Severity, VanillaText};
use bedrock_addon::document::{Content, Document, Rejected};
use bedrock_addon::load::{Experiment, Experiments, LoadContext, LoadOptions};
use bedrock_addon::pack::PackType;
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::MinEngineVersion;
use uuid::Uuid;

/// What a load told of the version: the floor or no-parser text with its version, or the
/// malformed or loaded outcome.
fn told<T>(outcome: &Result<Document<T>, Rejected>) -> String {
    let findings: &Diagnostics = match outcome {
        Ok(document) => document.diagnostics(),
        Err(rejected) => &rejected.diagnostics,
    };
    let mut lines = Vec::new();
    for finding in findings {
        match &finding.code {
            DiagnosticCode::UnsupportedVersion { text } => lines.push(match text {
                Some(VanillaText::NoParserForVersion { version }) => {
                    format!("no parser {version:?}")
                }
                Some(VanillaText::UseFormatVersion { version, .. }) => {
                    format!("use format version {version}")
                }
                Some(VanillaText::Fixed(text)) => format!("fixed {text:?}"),
                Some(VanillaText::Formatted { template, .. }) => format!("formatted {template:?}"),
                other => format!("unsupported {other:?}"),
            }),
            DiagnosticCode::MalformedFormatVersion { .. } => lines.push("malformed".to_owned()),
            _ => {}
        }
        assert!(
            !matches!(finding.code, DiagnosticCode::UnsupportedVersion { .. })
                || finding.severity == Severity::Error
        );
    }
    if outcome.is_ok() {
        lines.push("loaded".to_owned());
    }
    lines.join("; ")
}

fn context() -> LoadContext {
    let mut experiments = Experiments::new();
    experiments.insert(Experiment::Poi);
    let options = LoadOptions {
        experiments,
        ..LoadOptions::default()
    };
    LoadContext::for_pack(
        &options,
        Uuid::from_u128(7),
        PackType::Behavior,
        Some(MinEngineVersion::new(1, 26, 40)),
        false,
    )
}

fn load<T: Content>(path: &'static str, text: &str) -> Result<Document<T>, Rejected> {
    T::parse(&PackPath::from_static(path), text.as_bytes(), &context())
}

/// Each version spelling, as written in the file, and what the family tells of it.
type Rows = [(&'static str, &'static str); 4];

fn check<T: Content>(path: &'static str, document: &str, rows: Rows) {
    let found: Vec<(&str, String)> = rows
        .iter()
        .map(|(version, _)| {
            let text = document.replace("VERSION", version);
            (*version, told(&load::<T>(path, &text)))
        })
        .collect();
    let expected: Vec<(&str, String)> = rows
        .iter()
        .map(|(version, told)| (*version, (*told).to_owned()))
        .collect();
    assert_eq!(found, expected);
}

#[cfg(feature = "camera")]
#[test]
fn camera_preset_below_its_floor_is_told_the_raw_envelope_text() {
    // Floor 1.19.50.
    check::<bedrock_addon::families::camera::CameraPreset>(
        "cameras/presets/test.json",
        r#"{"format_version": VERSION, "minecraft:camera_preset": {"identifier": "demo:p"}}"#,
        [
            (r#""1.19""#, r#"no parser "1.19""#),
            ("[1, 19, 0]", r#"no parser "1.19.0""#),
            (r#""1.19.50-beta.1""#, r#"no parser "1.19.50-beta.1""#),
            ("1", r#"no parser """#),
        ],
    );
}

#[cfg(feature = "poi")]
#[test]
fn poi_below_its_floor_is_told_the_version_as_parsed() {
    // Floor 1.26.40.
    check::<bedrock_addon::families::poi::PoiType>(
        "poi/blocks/test.json",
        r#"{"format_version": VERSION, "minecraft:poi_block": {"description": {"identifier": "test:forge"}, "blocks": ["minecraft:anvil"]}}"#,
        [
            (r#""1.26""#, r#"no parser "1.26.0""#),
            ("[1, 26, 0]", r#"no parser "1.26.0""#),
            (r#""1.26.40-beta.1""#, r#"no parser "1.26.40-beta.1""#),
            ("1", r#"no parser "1.0.0""#),
        ],
    );
}

#[cfg(feature = "feature_rules")]
#[test]
fn feature_rules_below_their_floor_are_told_the_version_struct() {
    // Floor 1.12.0.
    check::<bedrock_addon::families::feature_rules::FeatureRule>(
        "feature_rules/test.json",
        r#"{"format_version": VERSION, "minecraft:feature_rules": {"description": {"identifier": "demo:r", "places_feature": "demo:f"}, "conditions": {"placement_pass": "surface_pass"}}}"#,
        [
            (r#""1.11""#, r#"no parser "1.11.0""#),
            ("[1, 11, 0]", r#"no parser "1.11.0""#),
            (r#""1.12.0-beta.1""#, r#"no parser "1.12.0""#),
            ("1", r#"no parser "1.0.0""#),
        ],
    );
}

#[cfg(feature = "voxel_shape")]
#[test]
fn voxel_shape_below_its_floor_is_told_the_version_as_written() {
    // Floor 1.21.110.
    check::<bedrock_addon::families::voxel_shape::VoxelShape>(
        "shapes/test.json",
        r#"{"format_version": VERSION, "minecraft:voxel_shape": {"description": {"identifier": "a:shape"}, "shape": {"boxes": [{"min": [0, 0, 0], "max": [16, 8, 16]}]}}}"#,
        [
            (r#""1.21""#, r#"no parser "1.21""#),
            ("[1, 21, 100]", r#"no parser "1.21.100""#),
            (r#""1.21.110-beta.1""#, r#"no parser "1.21.110-beta.1""#),
            ("1", r#"no parser "1""#),
        ],
    );
}

#[cfg(feature = "jigsaw")]
#[test]
fn jigsaw_processor_list_below_its_floor_is_told_the_raw_envelope_text() {
    // Floor 1.21.20.
    check::<bedrock_addon::families::jigsaw::ProcessorList>(
        "worldgen/processors/test.json",
        r#"{"format_version": VERSION, "minecraft:processor_list": {"description": {"identifier": "demo:l"}, "processors": []}}"#,
        [
            (r#""1.21""#, r#"no parser "1.21""#),
            ("[1, 21, 0]", r#"no parser "1.21.0""#),
            (r#""1.21.20-beta.1""#, r#"no parser "1.21.20-beta.1""#),
            ("1", r#"no parser """#),
        ],
    );
}

#[cfg(feature = "spawn_rules")]
#[test]
fn spawn_rules_below_their_floor_are_told_the_version_struct() {
    // Floor 1.8.0.
    check::<bedrock_addon::families::spawn_rules::SpawnRules>(
        "spawn_rules/test.json",
        r#"{"format_version": VERSION, "minecraft:spawn_rules": {"description": {"identifier": "x:mob", "population_control": "animal"}, "conditions": []}}"#,
        [
            (r#""1.7""#, r#"no parser "1.7.0""#),
            ("[1, 7, 0]", r#"no parser "1.7.0""#),
            (r#""1.8.0-beta.1""#, r#"no parser "1.8.0""#),
            ("1", "malformed"),
        ],
    );
}

#[cfg(feature = "deferred_settings")]
#[test]
fn lighting_settings_below_their_floor_are_told_the_version_as_parsed() {
    // Floor 1.21.40.
    check::<bedrock_addon::families::deferred_settings::LightingSettings>(
        "lighting/test.json",
        r#"{"format_version": VERSION, "minecraft:lighting_settings": {"description": {"identifier": "demo:l"}, "emissive": {"desaturation": 0.5}}}"#,
        [
            (r#""1.21""#, r#"no parser "1.21.0""#),
            ("[1, 21, 30]", r#"no parser "1.21.30""#),
            (r#""1.21.40-beta.1""#, r#"no parser "1.21.40-beta.1""#),
            ("1", "malformed"),
        ],
    );
}

#[cfg(feature = "block_culling")]
#[test]
fn block_culling_below_its_floor_is_told_missing_object() {
    // Floor 1.20.60.
    check::<bedrock_addon::families::block_culling::CullingRules>(
        "block_culling/test.json",
        r#"{"format_version": VERSION, "minecraft:block_culling_rules": {"description": {"identifier": "demo:lamp_culling"}, "rules": []}}"#,
        [
            (
                r#""1.20""#,
                "formatted \"Failed to parse block culling file '{}'. Missing object \\\"minecraft:block_culling_rules\\\"\"",
            ),
            (
                "[1, 20, 50]",
                "formatted \"Failed to parse block culling file '{}'. Missing object \\\"minecraft:block_culling_rules\\\"\"",
            ),
            (
                r#""1.20.60-beta.1""#,
                "formatted \"Failed to parse block culling file '{}'. Missing object \\\"minecraft:block_culling_rules\\\"\"",
            ),
            ("1", "malformed"),
        ],
    );
}

#[cfg(feature = "crafting_catalog")]
#[test]
fn crafting_catalog_below_its_floor_is_told_no_valid_format_version() {
    // Floor 1.21.60.
    check::<bedrock_addon::families::crafting_catalog::CraftingCatalog>(
        "item_catalog/crafting_item_catalog.json",
        r#"{"format_version": VERSION, "minecraft:crafting_items_catalog": {}}"#,
        [
            (
                r#""1.21""#,
                "fixed \"The file cannot be processed without a valid format_version\"",
            ),
            (
                "[1, 21, 50]",
                "fixed \"The file cannot be processed without a valid format_version\"",
            ),
            (
                r#""1.21.60-beta.1""#,
                "fixed \"The file cannot be processed without a valid format_version\"",
            ),
            (
                "1",
                "fixed \"The file cannot be processed without a valid format_version\"",
            ),
        ],
    );
}
