//! The dimension chain: the step from 1.18.0, the chain's versions and the history path.

use bedrock_addon_core::document::{Content, JsonDocument};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::testing::{Checks, Run, step_output};
use bedrock_addon_core::versioning::{VersionChain, VersionStruct};
use bedrock_addon_family_dimension::Dimension;
use bedrock_addon_family_dimension::history::{self, DimensionHistory};

use super::common::*;

/// The JSON at `path` (`/`-separated) of `value`.
fn at<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('/')
        .try_fold(value, |v, key| v.as_object().and_then(|o| o.get(key)))
}

/// `base` with `minecraft:dimension_bounds` `{min, max}`.
fn with_bounds(base: &Value, min: i32, max: i32) -> Value {
    let text = facet_json::to_string(base).unwrap().replace(
        r#""components":{"#,
        &format!(r#""components":{{"minecraft:dimension_bounds":{{"min":{min},"max":{max}}},"#),
    );
    facet_json::from_str(&text).unwrap()
}

/// The whole document the step reads: the dimension of `bounds`, if any.
fn overworld_input(bounds: Option<(i32, i32)>) -> Value {
    let base: Value = facet_json::from_str(
        r#"{"format_version": "1.21.60", "minecraft:dimension": {"description": {"identifier": "minecraft:overworld"}, "components": {"minecraft:generation": {"generator_type": "void"}}}}"#,
    )
    .unwrap();
    match bounds {
        Some((min, max)) => with_bounds(&base, min, max),
        None => base,
    }
}

/// The `(min_y, height_range)` the step makes of `bounds`, or `None` when it writes no height.
fn height_of(bounds: Option<(i32, i32)>) -> Option<(i64, i64)> {
    let checks = Checks {
        identifier: "minecraft:overworld",
        ..Checks::default()
    };
    let out = step_output::<history::v1_18_0::Dimension>(
        &overworld_input(bounds),
        Run::default(),
        &checks,
    )
    .expect("the step runs");
    let height = at(
        &out.payload,
        "minecraft:dimension/components/minecraft:dimension_height",
    )?;
    let member = |key| at(height, key).and_then(|v| v.as_number()?.to_i64());
    Some((member("min_y")?, member("height_range")?))
}

#[test]
fn the_bounds_of_the_file_become_a_height_of_min_and_the_distance_to_max() {
    assert_eq!(height_of(Some((-64, 320))), Some((-64, 384)));
    assert_eq!(height_of(Some((-512, 512))), Some((-512, 1024)));
    // Every pair makes a height; `{0, 0}` is the one the newest format cannot spell,
    // and is written as nothing.
    assert_eq!(height_of(Some((16, 0))), Some((16, -16)));
    assert_eq!(height_of(Some((0, 0))), None);
    assert_eq!(height_of(None), None);
}

#[test]
fn the_chain_is_1_18_0_then_1_26_50() {
    let versions = <DimensionHistory as VersionChain<_>>::VERSIONS;
    assert_eq!(
        versions,
        [FormatVersion::new(1, 18, 0), FormatVersion::new(1, 26, 50)]
    );
}

#[test]
fn the_history_path_writes_as_read_and_upgrades_to_the_same_dimension() {
    let text = dimension("1.21.60", "minecraft:overworld", &bounds(-64, 320));
    let path = PackPath::from_static("dimensions/overworld.json");
    let cx = LoadContext::detached();
    let written = Dimension::parse_as_written(&path, text.as_bytes(), &cx).unwrap();
    assert_eq!(written.parsed_version(), FormatVersion::new(1, 18, 0));
    let json = written.to_json().unwrap();
    assert!(json.contains(r#""format_version": "1.21.60""#), "{json}");
    assert!(json.contains("minecraft:dimension_bounds"), "{json}");
    let upgraded = history::upgrade(written, &cx).unwrap();
    let direct = <Dimension as Content>::parse(&path, text.as_bytes(), &cx).unwrap();
    assert_eq!(upgraded.value(), direct.value());
    let _ = VersionStruct::write(direct.value()).unwrap();
}

/// A `minecraft:dimension_height` of a document below
/// 1.26.50 is no member of its format (the game reads `minecraft:dimension_bounds` there); the
/// step makes the height of the bounds, and what the file has under the name is noted as
/// dropped, whatever it holds, and not written beside or over the step's.
#[test]
fn a_dimension_height_of_the_file_the_step_makes_of_the_bounds_is_noted() {
    for (height, written_height) in [
        (r#"{"min_y": -48, "height_range": 320}"#, r#""min_y": -64"#),
        (r#"{"min_y": [7], "height_range": 320}"#, r#""min_y": -64"#),
        ("[7]", r#""min_y": -64"#),
    ] {
        let text = format!(
            r#"{{"format_version": "1.21.60", "minecraft:dimension": {{"description": {{"identifier": "minecraft:overworld"}}, "components": {{"minecraft:dimension_bounds": {{"min": -64, "max": 320}}, "minecraft:dimension_height": {height}, "minecraft:generation": {{"generator_type": "void"}}}}}}}}"#
        );
        let document = loads(&text);
        let told: Vec<String> = document
            .diagnostics()
            .iter()
            .filter(|f| f.code.id() == "core/dropped_field")
            .map(|f| format!("{:?} {}", f.severity, f.location.json_path))
            .collect();
        assert_eq!(
            told,
            ["Info minecraft:dimension/components/minecraft:dimension_height"],
            "{height}"
        );
        let written = document.to_json().expect("the dimension writes");
        let written = written.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(written.contains(written_height), "{height}: {written}");
        assert!(
            !written.contains("-48") && !written.contains("[ 7 ]"),
            "{written}"
        );
    }
}

/// The height `{0, 0}` the step makes of zero
/// bounds is written as nothing (the newest format cannot spell it), neither bounds nor
/// height; where the file has the member that is noted at it (the nether and the end load
/// with it). Bounds that become another height, and no bounds, say nothing.
#[test]
fn bounds_of_the_file_that_become_no_height_are_noted_as_dropped() {
    let told = |identifier: &str, version: &str, bounds: &str| {
        let text = format!(
            r#"{{"format_version": "{version}", "minecraft:dimension": {{"description": {{"identifier": "{identifier}"}}, "components": {{{bounds}"minecraft:generation": {{"generator_type": "void"}}}}}}}}"#
        );
        let document = loads(&text);
        let written = document.to_json().expect("the dimension writes");
        assert!(!written.contains("dimension_bounds"), "{written}");
        document
            .diagnostics()
            .iter()
            .filter(|f| f.code.id() == "core/dropped_field")
            .map(|f| format!("{:?} {}", f.severity, f.location.json_path))
            .collect::<Vec<String>>()
    };
    for (identifier, version, bounds) in [
        ("minecraft:nether", "1.18.0", r#"{"min": 0, "max": 0}"#),
        ("minecraft:the_end", "1.21.60", r#"{"min": 0, "max": 0}"#),
    ] {
        assert_eq!(
            told(
                identifier,
                version,
                &format!(r#""minecraft:dimension_bounds": {bounds}, "#)
            ),
            ["Info minecraft:dimension/components/minecraft:dimension_bounds"],
            "the bounds that become no height are not told: {identifier} {version} {bounds}"
        );
    }
    let none: [&str; 0] = [];
    assert_eq!(
        told(
            "minecraft:overworld",
            "1.18.0",
            r#""minecraft:dimension_bounds": {"min": -64, "max": 320}, "#
        ),
        none
    );
    assert_eq!(told("minecraft:nether", "1.18.0", ""), none);
}
