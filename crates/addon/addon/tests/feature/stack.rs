//! `bp.features()`: the walk of `features/` top-down, the app's `definitions/features` after the
//! packs, the first definition claiming its identifier (fixtures f10, f20).

use bedrock_addon::diagnostic::{Kind, Severity, codes};
use bedrock_addon::families::feature::Feature;
use bedrock_addon::id::FeatureId;
use bedrock_addon::load::{AppData, LoadOptions};
use bedrock_addon::pack::{Behavior, PackStack};
use bedrock_addon::storage::PackPath;

use super::pack_common::bp;

const HIGH: &[u8] = br#"{"format_version": "1.13.0", "minecraft:ore_feature": {"description": {"identifier": "demo:vein"}, "count": 16}}"#;
const LOW: &[u8] = br#"{"format_version": "1.13.0", "minecraft:ore_feature": {"description": {"identifier": "demo:vein"}, "count": 4}}"#;
const BROKEN: &[u8] = br#"{"format_version": "1.13.0", "minecraft:ore_feature": {"description": {"identifier": "demo:claimed"}, "count": 0}}"#;
const VALID: &[u8] = br#"{"format_version": "1.13.0", "minecraft:ore_feature": {"description": {"identifier": "demo:claimed"}, "count": 2}}"#;

fn id(text: &str) -> FeatureId {
    FeatureId::new(text).unwrap()
}

/// The count of an ore feature.
fn count(feature: &Feature) -> i32 {
    match feature {
        Feature::Ore(ore) => ore.count.0,
        other => panic!("not an ore: {other:?}"),
    }
}

#[test]
fn f10_the_top_pack_definition_wins_and_the_lower_one_is_reported() {
    let top = bp(1).file("features/vein.json", HIGH).open();
    let bottom = bp(2).file("features/sub/vein.json", LOW).open();
    let stack = PackStack::<Behavior>::of([top, bottom], LoadOptions::default()).unwrap();
    let features = stack.features();
    assert_eq!(count(features.get(&id("demo:vein")).unwrap()), 16);
    let duplicates: Vec<_> = features
        .diagnostics()
        .filter(|f| {
            // An information: the game is silent.
            f.is(&codes::DUPLICATE_IDENTIFIER) && f.severity == Severity::Info
        })
        .collect();
    assert_eq!(duplicates.len(), 1);
    assert_eq!(
        duplicates[0].location.path.as_ref().unwrap().as_str(),
        "features/sub/vein.json"
    );
}

#[test]
fn f20_a_second_definition_in_one_pack_is_refused_with_vanillas_text() {
    let pack = bp(1)
        .file("features/a.json", HIGH)
        .file("features/b.json", LOW)
        .open();
    let stack = PackStack::<Behavior>::of([pack], LoadOptions::default()).unwrap();
    let features = stack.features();
    // The first in walk order (bytewise path order within a pack) is kept.
    assert_eq!(count(features.get(&id("demo:vein")).unwrap()), 16);
    // The game's level-3 line, and the second document is dropped: a refusal, an error that
    // is not kept.
    assert!(
        features.diagnostics().any(|f| f.rejects()
            && f.code.to_string() == "Feature with identifier 'demo:vein' already exists"),
        "{:#?}",
        features.diagnostics().collect::<Vec<_>>()
    );
}

#[test]
fn a_failed_top_definition_still_claims_its_identifier() {
    let top = bp(1).file("features/claimed.json", BROKEN).open();
    let bottom = bp(2).file("features/claimed.json", VALID).open();
    let stack = PackStack::<Behavior>::of([top, bottom], LoadOptions::default()).unwrap();
    let features = stack.features();
    assert!(features.get(&id("demo:claimed")).is_none());
    assert_eq!(features.rejected().len(), 1);
}

#[test]
fn the_apps_definitions_come_after_every_pack() {
    let app = AppData::from_files([
        (
            PackPath::from_static("definitions/features/vein.json"),
            LOW.to_vec(),
        ),
        (
            PackPath::from_static("definitions/features/own.json"),
            br#"{"format_version": "1.13.0", "minecraft:ore_feature": {"description": {"identifier": "minecraft:own"}, "count": 3}}"#.to_vec(),
        ),
        (
            PackPath::from_static("definitions/features/custom.json"),
            br#"{"format_version": "1.13.0", "minecraft:ore_feature": {"description": {"identifier": "demo:custom"}, "count": 3}}"#.to_vec(),
        ),
    ]);
    let options = LoadOptions {
        app_data: Some(app),
        ..LoadOptions::default()
    };
    let pack = bp(1).file("features/vein.json", HIGH).open();
    let stack = PackStack::<Behavior>::of([pack], options).unwrap();
    let features = stack.features();
    assert_eq!(count(features.get(&id("demo:vein")).unwrap()), 16);
    assert!(features.contains(&id("minecraft:own")));
    // A built-in definition must be in the `minecraft` namespace.
    assert!(!features.contains(&id("demo:custom")));
    assert!(features.diagnostics().any(|f| f.code.to_string()
        == "The feature identifer 'demo:custom' should be in the 'minecraft' namespace"));
    // The pack's definition replacing a built-in one is not a duplicate.
    assert!(
        !features
            .diagnostics()
            .any(|f| f.code.kind() == Kind::Duplicate)
    );
}
