//! Every finding of the merge and of the merged value's read, pinned whole (severity, phase,
//! pack, file, path segment by segment, span and code, in order) with the value read, for the
//! cases `legacy.rs` pins only in part. Captured from the hand-walked reader before it moved
//! onto the reader kit; the kit's read must tell the same.

use bedrock_addon::document::Merged;
use bedrock_addon::document::RawLayer;
use bedrock_addon::families::biomes_client::LegacyClientBiomes;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{Pack, PackOrigin, PackStack, Resource};
use bedrock_addon::version::BaseGameVersion;

use super::common::pack;

/// The merged settings of files given lowest first, each in its own user pack.
fn merged(files: &[&str]) -> PackStack<Resource> {
    let packs: Vec<Pack> = files
        .iter()
        .enumerate()
        .map(|(index, text)| {
            pack(
                index as u128 + 1,
                &format!("pack{index}"),
                PackOrigin::User,
                vec![("biomes_client.json", (*text).to_owned())],
            )
        })
        .rev()
        .collect();
    let options = LoadOptions {
        base_game_version: BaseGameVersion::EMPTY,
        ..LoadOptions::default()
    };
    PackStack::<Resource>::of(packs, options).expect("a stack")
}

/// Each finding on a line, then the entries read.
fn told(files: &[&str]) -> Vec<String> {
    let stack = merged(files);
    let legacy: &Merged<LegacyClientBiomes, RawLayer> = stack.legacy_client_biomes();
    let mut lines: Vec<String> = legacy
        .diagnostics()
        .map(|d| {
            format!(
                "{:?}|{:?}|{}|{}|{:?}|{:?}|{:?}",
                d.severity,
                d.phase,
                d.location.pack.as_ref().map_or("-", |pack| pack.name()),
                d.location.path.as_ref().map_or("-", |path| path.as_str()),
                d.location.json_path,
                d.location.span,
                d.code,
            )
        })
        .collect();
    lines.push(format!("default: {:?}", legacy.default));
    for (id, entry) in &legacy.biomes {
        lines.push(format!("{}: {entry:?}", id.as_str()));
    }
    lines
}

/// Within one entry the findings follow the read's own order (the three colours, the
/// transparency and its clamp, the fog distance, the fog list, the back-compat flag), not the
/// order the members are written in; a bad element of the fog list is dropped alone.
#[test]
fn an_entrys_findings_follow_the_reads_order() {
    let lines = told(&[r##"{"biomes": {
        "minecraft:plains": {"water_fog_distance": "far", "fog_ids_to_merge": {}, "make_back_compat_fog": 1,
            "water_surface_transparency": "x", "water_fog_color": 5, "fog_color": "#12345",
            "water_surface_color": [1, 2, 3]},
        "minecraft:b": {"make_back_compat_fog": null, "fog_ids_to_merge": ["a:f", 3, null, "b:g"],
            "water_surface_transparency": 2, "water_fog_distance": null}}}"##]);
    assert_eq!(lines, EXPECTED_ORDER, "{lines:#?}");
}

/// A merged entry's findings go to the highest pack that has its key, whichever pack wrote the
/// bad member; a flag of the wrong type that the merge reads is told by the merge, at the
/// pack being merged.
#[test]
fn a_merged_entrys_findings_name_the_highest_pack_with_its_key() {
    let lines = told(&[
        r#"{"biomes": {"default": {}, "minecraft:plains": {"water_fog_distance": "far"},
            "minecraft:desert": {"water_surface_transparency": "x"}}}"#,
        r##"{"biomes": {"default": {"remove_all_prior_fog": "yes"},
            "minecraft:plains": {"fog_identifier": "a:f", "inherit_from_prior_fog": 1, "water_surface_color": "#000001"},
            "minecraft:desert": {"water_surface_color": "#000002"}}}"##,
    ]);
    assert_eq!(lines, EXPECTED_ORIGINS, "{lines:#?}");
}

/// Entries that are no object (`null` under a namespaced key, `default: null`), a `biomes`
/// that is no object, and a fog name that is no string under a back-compat fog.
#[test]
fn entries_of_the_wrong_type_and_silent_members() {
    let nulls = told(&[r#"{"biomes": {"a:x": null, "default": null, "plains": null}}"#]);
    assert_eq!(nulls, EXPECTED_NULLS, "{nulls:#?}");
    let array = told(&[r#"{"biomes": []}"#]);
    assert_eq!(array, EXPECTED_ARRAY, "{array:#?}");
    let fog = told(&[
        r#"{"biomes": {"minecraft:plains": {"fog_identifier": 5}}}"#,
        r##"{"biomes": {"minecraft:plains": {"fog_color": "#ff0000", "water_fog_distance": 1e300}}}"##,
    ]);
    assert_eq!(fog, EXPECTED_FOG, "{fog:#?}");
}

/// A key of digits is a path index (the root's, an entry member's), and a number no double
/// holds is the largest finite one, with core's warning.
#[test]
fn digit_keys_and_numbers_beyond_a_double() {
    let lines = told(&[
        r#"{"7": 1, "biomes": {"123": {"9": true}, "a:far": {"water_surface_transparency": 1e400}}}"#,
    ]);
    assert_eq!(lines, EXPECTED_DIGITS, "{lines:#?}");
}

const EXPECTED_ORDER: &[&str] = &[
    "Info|Parse|pack0|biomes_client.json|JsonPath { segments: [Key(\"biomes\"), Key(\"minecraft:plains\"), Key(\"water_surface_color\")] }|None|InvalidType { text: None }",
    "Info|Parse|pack0|biomes_client.json|JsonPath { segments: [Key(\"biomes\"), Key(\"minecraft:plains\"), Key(\"fog_color\")] }|None|InvalidType { text: None }",
    "Info|Parse|pack0|biomes_client.json|JsonPath { segments: [Key(\"biomes\"), Key(\"minecraft:plains\"), Key(\"water_fog_color\")] }|None|InvalidType { text: None }",
    "Info|Parse|pack0|biomes_client.json|JsonPath { segments: [Key(\"biomes\"), Key(\"minecraft:plains\"), Key(\"water_surface_transparency\")] }|None|InvalidType { text: None }",
    "Info|Parse|pack0|biomes_client.json|JsonPath { segments: [Key(\"biomes\"), Key(\"minecraft:plains\"), Key(\"water_fog_distance\")] }|None|InvalidType { text: None }",
    "Info|Parse|pack0|biomes_client.json|JsonPath { segments: [Key(\"biomes\"), Key(\"minecraft:plains\"), Key(\"fog_ids_to_merge\")] }|None|InvalidType { text: None }",
    "Info|Parse|pack0|biomes_client.json|JsonPath { segments: [Key(\"biomes\"), Key(\"minecraft:plains\"), Key(\"make_back_compat_fog\")] }|None|InvalidType { text: None }",
    "Info|Parse|pack0|biomes_client.json|JsonPath { segments: [Key(\"biomes\"), Key(\"minecraft:b\"), Key(\"water_surface_transparency\")] }|None|ClampedValue { text: None }",
    "Info|Parse|pack0|biomes_client.json|JsonPath { segments: [Key(\"biomes\"), Key(\"minecraft:b\"), Key(\"fog_ids_to_merge\"), Index(1)] }|None|InvalidType { text: None }",
    "Info|Parse|pack0|biomes_client.json|JsonPath { segments: [Key(\"biomes\"), Key(\"minecraft:b\"), Key(\"fog_ids_to_merge\"), Index(2)] }|None|InvalidType { text: None }",
    "default: None",
    "minecraft:plains: LegacyClientBiome { old_fog: None, fog_ids_to_merge: [], water_surface_color: None, water_surface_transparency: None }",
    "minecraft:b: LegacyClientBiome { old_fog: None, fog_ids_to_merge: [Id<fog>(\"a:f\"), Id<fog>(\"b:g\")], water_surface_color: None, water_surface_transparency: Some(1.0) }",
];
const EXPECTED_ORIGINS: &[&str] = &[
    "Info|Parse|pack1|biomes_client.json|JsonPath { segments: [Key(\"biomes\"), Key(\"default\"), Key(\"remove_all_prior_fog\")] }|None|InvalidType { text: None }",
    "Info|Parse|pack1|biomes_client.json|JsonPath { segments: [Key(\"biomes\"), Key(\"minecraft:plains\"), Key(\"inherit_from_prior_fog\")] }|None|InvalidType { text: None }",
    "Info|Parse|pack1|biomes_client.json|JsonPath { segments: [Key(\"biomes\"), Key(\"minecraft:desert\"), Key(\"water_surface_transparency\")] }|None|InvalidType { text: None }",
    "default: Some(LegacyClientBiome { old_fog: None, fog_ids_to_merge: [], water_surface_color: None, water_surface_transparency: None })",
    "minecraft:plains: LegacyClientBiome { old_fog: None, fog_ids_to_merge: [Id<fog>(\"a:f\")], water_surface_color: Some([0.0, 0.0, 0.003921569, 1.0]), water_surface_transparency: None }",
    "minecraft:desert: LegacyClientBiome { old_fog: None, fog_ids_to_merge: [], water_surface_color: Some([0.0, 0.0, 0.007843138, 1.0]), water_surface_transparency: None }",
];
const EXPECTED_NULLS: &[&str] = &[
    "Info|Parse|pack0|biomes_client.json|JsonPath { segments: [Key(\"biomes\"), Key(\"a:x\")] }|None|InvalidType { text: None }",
    "Info|Parse|pack0|biomes_client.json|JsonPath { segments: [Key(\"biomes\"), Key(\"default\")] }|None|InvalidType { text: None }",
    "default: None",
];
const EXPECTED_ARRAY: &[&str] = &[
    "Info|Validate|-|-|JsonPath { segments: [] }|None|NoEffect { text: None }",
    "default: None",
];
const EXPECTED_FOG: &[&str] = &[
    "Info|Parse|pack0|biomes_client.json|JsonPath { segments: [Key(\"biomes\"), Key(\"minecraft:plains\"), Key(\"fog_identifier\")] }|None|LegacyFogIgnored",
    "default: None",
    "minecraft:plains: LegacyClientBiome { old_fog: Some(OldFog { fog_color: Some([1.0, 0.0, 0.0, 1.0]), water_fog_color: None, water_fog_distance: Some(inf), fog_identifier: None }), fog_ids_to_merge: [], water_surface_color: None, water_surface_transparency: None }",
];
const EXPECTED_DIGITS: &[&str] = &[
    "Info|Parse|pack0|biomes_client.json|JsonPath { segments: [Key(\"biomes\"), Key(\"a:far\"), Key(\"water_surface_transparency\")] }|Some(Span { offset: 80, len: 5 })|InvalidValue { text: None }",
    "Info|Parse|pack0|biomes_client.json|JsonPath { segments: [Index(7)] }|None|UnrecognizedField { text: None }",
    "Info|Parse|pack0|biomes_client.json|JsonPath { segments: [Key(\"biomes\"), Key(\"minecraft:123\"), Index(9)] }|None|UnrecognizedField { text: None }",
    "Info|Parse|pack0|biomes_client.json|JsonPath { segments: [Key(\"biomes\"), Key(\"a:far\"), Key(\"water_surface_transparency\")] }|None|ClampedValue { text: None }",
    "default: None",
    "a:far: LegacyClientBiome { old_fog: None, fog_ids_to_merge: [], water_surface_color: None, water_surface_transparency: Some(1.0) }",
    "minecraft:123: LegacyClientBiome { old_fog: None, fog_ids_to_merge: [], water_surface_color: None, water_surface_transparency: None }",
];
