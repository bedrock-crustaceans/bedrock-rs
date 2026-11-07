//! The text a document below a chain's floor is told: the one rendering of
//! `Family::ENVELOPE.below_floor`, pinned per family for the version spellings that once rendered
//! apart (two parts, three integers, a pre-release, a bare number). The families with
//! `BelowFloor::NoParser` (the camera preset and spline, both aim-assist files, POI, feature
//! rules, voxel shape, the jigsaw structure, template pool, structure set and processor list,
//! spawn rules and spawn groups, the lighting settings of `deferred_settings`, biome,
//! dimension, the server sound definitions and the pack settings) all print the version as
//! `versioning::no_parser_version` does: the version as parsed, a pre-release as written; a
//! bare number under the root member's reading is malformed. Block culling and the crafting
//! catalog keep texts of their own below the floor.

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
    feature = "aim_assist",
    feature = "biome",
    feature = "dimension",
    feature = "spawn_groups",
    feature = "server_sound_definitions",
))]

use bedrock_addon::diagnostic::{DiagnosticCode, Diagnostics, Severity, codes};
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
            DiagnosticCode::Specific { code } if code.is(&codes::NO_PARSER_FOR_VERSION) => {
                lines.push(format!("no parser {:?}", code.args()[0]));
            }
            DiagnosticCode::Specific { code } if code.is(&codes::USE_FORMAT_VERSION) => {
                lines.push(format!("use format version {}", code.args()[2]));
            }
            DiagnosticCode::Specific { code } if code.is(&codes::UNSUPPORTED_VERSION_WORDED) => {
                lines.push(format!("fixed {:?}", code.args()[0]));
            }
            // Block culling's own line below its floor.
            #[cfg(feature = "block_culling")]
            DiagnosticCode::Specific { code }
                if code.is(
                    &bedrock_addon::families::block_culling::codes::MISSING_OBJECT_UNSUPPORTED_VERSION,
                ) =>
            {
                lines.push(format!(
                    "formatted {:?}",
                    bedrock_addon::families::block_culling::codes::MISSING_OBJECT_UNSUPPORTED_VERSION
                        .template()
                ));
            }
            // By legacy name: a family's own definition of a fixed sentence that was the
            // variant's `Fixed` text.
            DiagnosticCode::Specific { code }
                if code.name() == "UnsupportedVersion" && code.args().is_empty() =>
            {
                lines.push(format!("fixed {:?}", code.vanilla().unwrap_or_default()));
            }
            // By legacy name: the variant and the definitions that were it.
            code if code.name() == "MalformedFormatVersion" => lines.push("malformed".to_owned()),
            _ => {}
        }
        assert!(finding.code.name() != "UnsupportedVersion" || finding.severity == Severity::Error);
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
type Rows<'a> = &'a [(&'static str, &'static str)];

fn check<T: Content>(path: &'static str, document: &str, rows: Rows<'_>) {
    check_with(|text| load::<T>(path, text), document, rows);
}

/// [`check`] of a file `parse` reads.
fn check_with<T>(
    parse: impl Fn(&str) -> Result<Document<T>, Rejected>,
    document: &str,
    rows: Rows<'_>,
) {
    let found: Vec<(&str, String)> = rows
        .iter()
        .map(|(version, _)| {
            let text = document.replace("VERSION", version);
            (*version, told(&parse(&text)))
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
fn camera_preset_below_its_floor_is_told_the_version_as_parsed() {
    // Floor 1.19.50.
    check::<bedrock_addon::families::camera::CameraPreset>(
        "cameras/presets/test.json",
        r#"{"format_version": VERSION, "minecraft:camera_preset": {"identifier": "demo:p"}}"#,
        &[
            (r#""1.19""#, r#"no parser "1.19.0""#),
            ("[1, 19, 0]", r#"no parser "1.19.0""#),
            (r#""1.19.50-beta.1""#, r#"no parser "1.19.50-beta.1""#),
            ("1", r#"no parser "1.0.0""#),
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
        &[
            (r#""1.26""#, r#"no parser "1.26.0""#),
            ("[1, 26, 0]", r#"no parser "1.26.0""#),
            (r#""1.26.40-beta.1""#, r#"no parser "1.26.40-beta.1""#),
            ("1", r#"no parser "1.0.0""#),
        ],
    );
}

#[cfg(feature = "feature_rules")]
#[test]
fn feature_rules_below_their_floor_are_told_the_version_as_parsed() {
    // Floor 1.12.0.
    check::<bedrock_addon::families::feature_rules::FeatureRule>(
        "feature_rules/test.json",
        r#"{"format_version": VERSION, "minecraft:feature_rules": {"description": {"identifier": "demo:r", "places_feature": "demo:f"}, "conditions": {"placement_pass": "surface_pass"}}}"#,
        &[
            (r#""1.11""#, r#"no parser "1.11.0""#),
            ("[1, 11, 0]", r#"no parser "1.11.0""#),
            (r#""1.12.0-beta.1""#, r#"no parser "1.12.0-beta.1""#),
            ("1", r#"no parser "1.0.0""#),
        ],
    );
}

#[cfg(feature = "voxel_shape")]
#[test]
fn voxel_shape_below_its_floor_is_told_the_version_as_parsed() {
    // Floor 1.21.110.
    check::<bedrock_addon::families::voxel_shape::VoxelShape>(
        "shapes/test.json",
        r#"{"format_version": VERSION, "minecraft:voxel_shape": {"description": {"identifier": "a:shape"}, "shape": {"boxes": [{"min": [0, 0, 0], "max": [16, 8, 16]}]}}}"#,
        &[
            (r#""1.21""#, r#"no parser "1.21.0""#),
            ("[1, 21, 100]", r#"no parser "1.21.100""#),
            (r#""1.21.110-beta.1""#, r#"no parser "1.21.110-beta.1""#),
            ("1", r#"no parser "1.0.0""#),
        ],
    );
}

#[cfg(feature = "jigsaw")]
#[test]
fn jigsaw_processor_list_below_its_floor_is_told_the_version_as_parsed() {
    // Floor 1.21.20.
    check::<bedrock_addon::families::jigsaw::ProcessorList>(
        "worldgen/processors/test.json",
        r#"{"format_version": VERSION, "minecraft:processor_list": {"description": {"identifier": "demo:l"}, "processors": []}}"#,
        &[
            (r#""1.21""#, r#"no parser "1.21.0""#),
            ("[1, 21, 0]", r#"no parser "1.21.0""#),
            (r#""1.21.20-beta.1""#, r#"no parser "1.21.20-beta.1""#),
            ("1", r#"no parser "1.0.0""#),
        ],
    );
}

#[cfg(feature = "spawn_rules")]
#[test]
fn spawn_rules_below_their_floor_are_told_the_version_as_parsed() {
    // Floor 1.8.0.
    check::<bedrock_addon::families::spawn_rules::SpawnRules>(
        "spawn_rules/test.json",
        r#"{"format_version": VERSION, "minecraft:spawn_rules": {"description": {"identifier": "x:mob", "population_control": "animal"}, "conditions": []}}"#,
        &[
            (r#""1.7""#, r#"no parser "1.7.0""#),
            ("[1, 7, 0]", r#"no parser "1.7.0""#),
            (r#""1.8.0-beta.1""#, r#"no parser "1.8.0-beta.1""#),
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
        &[
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
        &[
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
        &[
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

#[cfg(feature = "camera")]
#[test]
fn camera_spline_below_its_floor_is_told_the_version_as_parsed() {
    // Floor 1.26.0.
    check::<bedrock_addon::families::camera::CameraSpline>(
        "cameras/splines/test.json",
        r#"{"format_version": VERSION, "minecraft:camera_custom_splines": {"splines": []}}"#,
        &[
            (r#""1.25""#, r#"no parser "1.25.0""#),
            ("[1, 25, 0]", r#"no parser "1.25.0""#),
            (r#""1.26.0-beta.1""#, r#"no parser "1.26.0-beta.1""#),
            ("1", r#"no parser "1.0.0""#),
        ],
    );
}

/// The four spellings below aim assist's floor, 1.21.50.
#[cfg(feature = "aim_assist")]
const AIM_ASSIST_ROWS: Rows<'static> = &[
    (r#""1.21""#, r#"no parser "1.21.0""#),
    ("[1, 21, 40]", r#"no parser "1.21.40""#),
    (r#""1.21.50-beta.1""#, r#"no parser "1.21.50-beta.1""#),
    ("1", r#"no parser "1.0.0""#),
];

#[cfg(feature = "aim_assist")]
#[test]
fn aim_assist_preset_below_its_floor_is_told_the_version_as_parsed() {
    check::<bedrock_addon::families::aim_assist::AimAssistPreset>(
        "aim_assist/presets/test.json",
        r#"{"format_version": VERSION, "minecraft:aim_assist_preset": {"identifier": "test:p"}}"#,
        AIM_ASSIST_ROWS,
    );
}

#[cfg(feature = "aim_assist")]
#[test]
fn aim_assist_categories_below_their_floor_are_told_the_version_as_parsed() {
    check::<bedrock_addon::families::aim_assist::AimAssistCategories>(
        "aim_assist/categories/test.json",
        r#"{"format_version": VERSION, "minecraft:aim_assist_categories": {"categories": []}}"#,
        AIM_ASSIST_ROWS,
    );
}

#[test]
fn pack_settings_below_their_floor_are_told_the_version_as_parsed() {
    // Floor 1.21.100.
    check_with(
        |text| bedrock_addon::pack::settings::SavedSettings::parse(text.as_bytes()),
        r#"{"format_version": VERSION, "minecraft:pack_settings": {"settings": []}}"#,
        &[
            (r#""1.21""#, r#"no parser "1.21.0""#),
            ("[1, 21, 90]", r#"no parser "1.21.90""#),
            (r#""1.21.100-beta.1""#, r#"no parser "1.21.100-beta.1""#),
            ("1", "malformed"),
        ],
    );
}

/// The four spellings below the jigsaw floor, 1.21.20.
#[cfg(feature = "jigsaw")]
const JIGSAW_ROWS: Rows<'static> = &[
    (r#""1.21""#, r#"no parser "1.21.0""#),
    ("[1, 21, 0]", r#"no parser "1.21.0""#),
    (r#""1.21.20-beta.1""#, r#"no parser "1.21.20-beta.1""#),
    ("1", r#"no parser "1.0.0""#),
];

#[cfg(feature = "jigsaw")]
#[test]
fn jigsaw_structure_below_its_floor_is_told_the_version_as_parsed() {
    check::<bedrock_addon::families::jigsaw::JigsawStructure>(
        "worldgen/structures/test.json",
        r#"{"format_version": VERSION, "minecraft:jigsaw": {"description": {"identifier": "demo:j"}}}"#,
        JIGSAW_ROWS,
    );
}

#[cfg(feature = "jigsaw")]
#[test]
fn jigsaw_template_pool_below_its_floor_is_told_the_version_as_parsed() {
    check::<bedrock_addon::families::jigsaw::TemplatePool>(
        "worldgen/template_pools/test.json",
        r#"{"format_version": VERSION, "minecraft:template_pool": {"description": {"identifier": "demo:p"}, "elements": []}}"#,
        JIGSAW_ROWS,
    );
}

#[cfg(feature = "jigsaw")]
#[test]
fn jigsaw_structure_set_below_its_floor_is_told_the_version_as_parsed() {
    check::<bedrock_addon::families::jigsaw::StructureSet>(
        "worldgen/structure_sets/test.json",
        r#"{"format_version": VERSION, "minecraft:structure_set": {"description": {"identifier": "demo:s"}, "structures": []}}"#,
        JIGSAW_ROWS,
    );
}

#[cfg(feature = "biome")]
#[test]
fn biome_below_its_floor_is_told_the_version_as_parsed() {
    // Floor 1.21.110; a pre-release of the floor is below it.
    check::<bedrock_addon::families::biome::Biome>(
        "biomes/test.json",
        r#"{"format_version": VERSION, "minecraft:biome": {"description": {"identifier": "test:x"}, "components": {}}}"#,
        &[
            (r#""1.21""#, r#"no parser "1.21.0""#),
            ("[1, 21, 100]", r#"no parser "1.21.100""#),
            (r#""1.20.0-beta.1""#, r#"no parser "1.20.0-beta.1""#),
            (r#""1.21.110-beta.1""#, r#"no parser "1.21.110-beta.1""#),
            ("1", r#"no parser "1.0.0""#),
        ],
    );
}

#[cfg(feature = "dimension")]
#[test]
fn dimension_below_its_floor_is_told_the_version_as_parsed() {
    // Floor 1.18.0.
    check::<bedrock_addon::families::dimension::Dimension>(
        "dimensions/test.json",
        r#"{"format_version": VERSION, "minecraft:dimension": {"description": {"identifier": "test:void"}, "components": {}}}"#,
        &[
            (r#""1.17""#, r#"no parser "1.17.0""#),
            ("[1, 17, 0]", r#"no parser "1.17.0""#),
            (r#""1.18.0-beta.1""#, r#"no parser "1.18.0-beta.1""#),
            ("1", r#"no parser "1.0.0""#),
        ],
    );
}

#[cfg(feature = "spawn_groups")]
#[test]
fn spawn_groups_below_their_floor_are_told_the_version_as_parsed() {
    // Floor 1.8.0.
    check::<bedrock_addon::families::spawn_groups::SpawnGroup>(
        "definitions/spawn_groups/test.json",
        r#"{"format_version": VERSION, "minecraft:spawn_groups": {"description": {"identifier": "x:g"}, "conditions": [{}]}}"#,
        &[
            (r#""1.7""#, r#"no parser "1.7.0""#),
            ("[1, 7, 0]", r#"no parser "1.7.0""#),
            (r#""1.8.0-beta.1""#, r#"no parser "1.8.0-beta.1""#),
            ("1", "malformed"),
        ],
    );
}

#[cfg(feature = "server_sound_definitions")]
#[test]
fn server_sound_definitions_below_their_floor_are_told_the_version_as_parsed() {
    // Floor 1.26.60.
    check::<bedrock_addon::families::server_sound_definitions::SoundMetadataFile>(
        "sounds/sound_definitions.json",
        r#"{"format_version": VERSION, "minecraft:server_sound_definitions": []}"#,
        &[
            (r#""1.26""#, r#"no parser "1.26.0""#),
            ("[1, 26, 50]", r#"no parser "1.26.50""#),
            (r#""1.26.60-beta.1""#, r#"no parser "1.26.60-beta.1""#),
            ("1", r#"no parser "1.0.0""#),
        ],
    );
}
