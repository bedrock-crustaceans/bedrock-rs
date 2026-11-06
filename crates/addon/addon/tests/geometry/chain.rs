//! The chain of geometry files: the struct each version selects, what each upgrade step does
//! (the callbacks of the steps run one at a time), and the upgrade rows u01–u15.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::families::geometry::history::{
    self, GeometryHistory, VERSIONS, v0_0_0, v1_8_0, v1_12_0, v1_14_0, v1_16_0, v1_19_30, v1_21_0,
};
use bedrock_addon::families::geometry::{CubeUv, GeometryFile, Locator};
use bedrock_addon::json::Value;
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::FormatVersion;
use bedrock_addon::versioning::testing::{Checks, Run, step_output};
use bedrock_addon::versioning::{Upgrade, VersionStruct, Versioned};

use crate::common::{clean, count, engine, expect, expect_at, first, load, loaded, rp};

fn json(text: &str) -> Value {
    facet_json::from_str(text).unwrap_or_else(|e| panic!("{text}: {e}"))
}

/// The checks every step of the chain runs under: a resource pack at the newest engine.
fn checks() -> Checks<'static> {
    Checks {
        load: rp(),
        ..Checks::default()
    }
}

/// Runs the step `S` on `input` and requires the output `expected`.
#[track_caller]
fn gives<S>(input: &str, expected: &str)
where
    S: Upgrade + VersionStruct,
    S::Next: VersionStruct,
{
    let out = step_output::<S>(&json(input), Run::default(), &checks())
        .unwrap_or_else(|_| panic!("the input did not upgrade"));
    assert_eq!(
        out.payload,
        json(expected),
        "the output differs: {}",
        bedrock_addon::json::to_json(&out.payload).unwrap_or_default()
    );
}

#[test]
fn mirror_strings_become_bools() {
    gives::<v0_0_0::GeometryFile>(
        r#"{"format_version": "0.0.0", "geometry.x": {"bones": [
            {"name": "a", "mirror": "true", "cubes": [{"size": [1, 1, 1], "mirror": "false"}]},
            {"name": "b", "mirror": false, "cubes": [{"size": [1, 1, 1], "mirror": true}]}]}}"#,
        r#"{"format_version": "0.0.0", "geometry.x": {"bones": [
            {"name": "a", "mirror": true, "cubes": [{"size": [1, 1, 1], "mirror": false}]},
            {"name": "b", "mirror": false, "cubes": [{"size": [1, 1, 1], "mirror": true}]}]}}"#,
    );
}

#[test]
fn reset_true_is_kept() {
    gives::<v0_0_0::GeometryFile>(
        r#"{"format_version": "0.0.0", "geometry.x": {"bones": [
            {"name": "a", "reset": true}, {"name": "b", "reset": "true"},
            {"name": "c", "reset": "false"}, {"name": "d", "reset": "yes"}]}}"#,
        r#"{"format_version": "0.0.0", "geometry.x": {"bones": [
            {"name": "a", "reset": true}, {"name": "b", "reset": true},
            {"name": "c"}, {"name": "d"}]}}"#,
    );
}

/// The removed element is replaced by the last one, which is examined next.
#[test]
fn empty_cubes_are_swap_removed() {
    gives::<v0_0_0::GeometryFile>(
        r#"{"format_version": "0.0.0", "geometry.x": {"bones": [{"name": "a", "cubes": [
            {}, {"size": [1, 1, 1]}, {"size": [2, 2, 2]}, {}, {"size": [3, 3, 3]}, {}]}]}}"#,
        r#"{"format_version": "0.0.0", "geometry.x": {"bones": [{"name": "a", "cubes": [
            {"size": [3, 3, 3]}, {"size": [1, 1, 1]}, {"size": [2, 2, 2]}]}]}}"#,
    );
}

#[test]
fn the_key_becomes_the_identifier() {
    gives::<v1_8_0::GeometryFile>(
        r#"{"format_version": "1.8.0", "geometry.pig": {}}"#,
        r#"{"format_version": "1.8.0", "minecraft:geometry": [{"description": {"identifier": "geometry.pig"}}]}"#,
    );
}

#[test]
fn the_key_is_split_at_its_last_colon() {
    gives::<v1_8_0::GeometryFile>(
        r#"{"format_version": "1.8.0", "geometry.a:geometry.b:geometry.c": {}}"#,
        r#"{"format_version": "1.8.0", "minecraft:geometry": [{"description": {"identifier": "geometry.a:geometry.b"}, "__legacy": {"parent": "geometry.c"}}]}"#,
    );
}

#[test]
fn geometries_are_in_key_order() {
    gives::<v1_8_0::GeometryFile>(
        r#"{"format_version": "1.8.0", "geometry.b": {}, "geometry.c": {}, "geometry.a": {}}"#,
        r#"{"format_version": "1.8.0", "minecraft:geometry": [
            {"description": {"identifier": "geometry.a"}},
            {"description": {"identifier": "geometry.b"}},
            {"description": {"identifier": "geometry.c"}}]}"#,
    );
}

const CUBE_HEAD: &str = r#"{"format_version": "1.12.0", "minecraft:geometry": [{"description": {"identifier": "geometry.x"}, "bones": [{"name": "body", "cubes": ["#;

#[test]
fn an_explicit_pivot_is_kept() {
    let cube = r#"{"origin": [-4, 0, -2], "size": [8, 12, 4], "pivot": [1, 2, 3]}"#;
    let file = format!("{CUBE_HEAD}{cube}]}}]}}]}}");
    gives::<v1_12_0::GeometryFile>(&file, &file);
}

#[test]
fn a_missing_origin_counts_as_zero() {
    gives::<v1_12_0::GeometryFile>(
        &format!(r#"{CUBE_HEAD}{{"size": [8, 12, 5]}}, {{"origin": [1, 2, 3]}}]}}]}}]}}"#),
        &format!(
            r#"{CUBE_HEAD}{{"size": [8, 12, 5], "pivot": [4, 6, 2.5]}}, {{"origin": [1, 2, 3], "pivot": [1, 2, 3]}}]}}]}}]}}"#
        ),
    );
}

const SHELF: &str = r#"{"format_version": "1.21.0", "minecraft:geometry": [
    {"description": {"identifier": "geometry.a"}, "item_display_transforms": {"fixed": {"scale": [1, 1, 1]}}},
    {"description": {"identifier": "geometry.b"}, "item_display_transforms": {"shelf": {}, "fixed": {"scale": [2, 2, 2]}}},
    {"description": {"identifier": "geometry.c"}, "item_display_transforms": {"fixed": {"scale": [3, 3, 3]}}}]}"#;

/// The loop stops at the first geometry with a `shelf`: the one before it has its copy, the one
/// after it is not examined.
#[test]
fn a_shelf_in_an_older_file_fails_the_file() {
    let out = step_output::<v1_21_0::GeometryFile>(&json(SHELF), Run::default(), &checks())
        .unwrap_or_else(|_| panic!("the input did not upgrade"));
    let text = "\"item_display_transform\" for \"geometry.b\" is invalid - \"shelf\" can only be used in a file version 1.26.40 or higher";
    let errors: Vec<String> = out
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .map(|d| format!("{} {}", d.code.name(), d.code))
        .collect();
    assert_eq!(
        errors,
        [format!("InvalidValue {text}")],
        "not the one `InvalidValue` error with vanilla's text"
    );
    let shelf = |index: usize| {
        out.payload
            .as_object()
            .and_then(|o| o.get("minecraft:geometry"))
            .and_then(|g| g.as_array()?.get(index))
            .and_then(|g| g.as_object()?.get("item_display_transforms"))
            .and_then(|t| t.as_object()?.get("shelf"))
            .cloned()
    };
    assert_eq!(
        shelf(0),
        Some(json(r#"{"scale": [1, 1, 1]}"#)),
        "the geometry before the offender has no copy"
    );
    assert!(
        shelf(2).is_none(),
        "the geometry after the offender was examined"
    );
}

/// The 1.26.20 game has neither the step nor the transform: `fixed` is not copied.
#[test]
fn an_engine_below_1_26_40_does_not_copy_fixed_to_shelf() {
    let input = json(&transforms("1.21.0", r#"{"fixed": {"scale": [1, 1, 1]}}"#));
    let shelf_of = |load| {
        let checks = Checks {
            load,
            ..Checks::default()
        };
        let out = step_output::<v1_21_0::GeometryFile>(&input, Run::default(), &checks)
            .unwrap_or_else(|_| panic!("the input did not upgrade"));
        out.payload
            .as_object()
            .and_then(|o| o.get("minecraft:geometry"))
            .and_then(|g| g.as_array()?.first())
            .and_then(|g| g.as_object()?.get("item_display_transforms"))
            .and_then(|t| t.as_object()?.get("shelf"))
            .cloned()
    };
    assert!(shelf_of(engine(1, 26, 40)).is_some(), "copied from 1.26.40");
    assert!(shelf_of(engine(1, 26, 39)).is_none(), "not copied below it");
}

// ---- the chain ----------------------------------------------------------------------------------

#[test]
fn the_chain_is_the_eight_structs_of_the_plan() {
    let v = FormatVersion::new;
    assert_eq!(
        VERSIONS,
        [
            v(0, 0, 0),
            v(1, 8, 0),
            v(1, 12, 0),
            v(1, 14, 0),
            v(1, 16, 0),
            v(1, 19, 30),
            v(1, 21, 0),
            v(1, 26, 40)
        ]
    );
    assert_eq!(<v0_0_0::GeometryFile as Versioned>::VERSION, VERSIONS[0]);
    assert_eq!(<v1_8_0::GeometryFile as Versioned>::VERSION, VERSIONS[1]);
    assert_eq!(<v1_12_0::GeometryFile as Versioned>::VERSION, VERSIONS[2]);
    assert_eq!(<v1_14_0::GeometryFile as Versioned>::VERSION, VERSIONS[3]);
    assert_eq!(<v1_16_0::GeometryFile as Versioned>::VERSION, VERSIONS[4]);
    assert_eq!(<v1_19_30::GeometryFile as Versioned>::VERSION, VERSIONS[5]);
    assert_eq!(<v1_21_0::GeometryFile as Versioned>::VERSION, VERSIONS[6]);
    assert_eq!(<GeometryFile as Versioned>::VERSION, VERSIONS[7]);
    assert_eq!(
        <history::v1_26_40::GeometryFile as Versioned>::VERSION,
        VERSIONS[7]
    );
    assert_eq!(history::STRICT_FROM, v(1, 19, 30));
}

fn written(version: &str) -> bedrock_addon::document::Document<GeometryHistory> {
    let legacy = FormatVersion::new(1, 12, 0)
        > bedrock_addon::version::SemVersion::parse(version)
            .unwrap()
            .format_version();
    let text = if legacy {
        format!(r#"{{"format_version": "{version}", "geometry.x": {{}}}}"#)
    } else {
        format!(
            r#"{{"format_version": "{version}", "minecraft:geometry": [{{"description": {{"identifier": "geometry.x"}}}}]}}"#
        )
    };
    loaded(GeometryFile::parse_as_written(
        &PackPath::from_static("models/entity/x.json"),
        text.as_bytes(),
        &rp(),
    ))
}

/// Selection is `VERSIONS[i] <= version < VERSIONS[i + 1]`, both ends of every range.
#[test]
fn a_version_selects_the_struct_whose_range_holds_it() {
    let cases = [
        ("0.0.1", 0),
        ("1.7.99", 0),
        ("1.8.0", 1),
        ("1.11.0", 1),
        ("1.12.0", 2),
        ("1.13.99", 2),
        ("1.14.0", 3),
        ("1.15.99", 3),
        ("1.16.0", 4),
        ("1.19.29", 4),
        ("1.19.30", 5),
        ("1.20.99", 5),
        ("1.21.0", 6),
        ("1.26.39", 6),
        ("1.26.40", 7),
        ("1.26.50", 7),
    ];
    for (version, index) in cases {
        let document = written(version);
        assert_eq!(
            document.value().parsed_version(),
            VERSIONS[index],
            "{version}"
        );
        assert_eq!(document.version().parsed_as, VERSIONS[index], "{version}");
        assert!(!document.version().upgraded, "not upgraded yet");
    }
}

#[test]
fn a_file_parsed_as_written_writes_its_own_shape_and_upgrades_to_the_loaded_file() {
    let text = r#"{"format_version": "1.8.0", "geometry.pig:geometry.quadruped": {"texturewidth": 64, "bones": [{"name": "body", "neverRender": true}]}}"#;
    let path = PackPath::from_static("models/entity/x.json");
    let document = loaded(GeometryFile::parse_as_written(
        &path,
        text.as_bytes(),
        &rp(),
    ));
    assert!(matches!(document.value(), GeometryHistory::V1_8_0(_)));
    assert_eq!(
        json(&document.to_json().unwrap()),
        json(text),
        "the 1.8 shape is written back"
    );
    let upgraded = loaded(history::upgrade(document, &rp()));
    let direct = loaded(load(text));
    assert_eq!(upgraded.value(), direct.value());
    assert_eq!(upgraded.version(), direct.version());
    assert!(direct.version().upgraded);
    assert_eq!(direct.version().written, Some(FormatVersion::new(1, 8, 0)));
    assert_eq!(direct.version().parsed_as, VERSIONS[1]);
}

#[test]
fn the_newest_struct_is_not_marked_upgraded() {
    let text = r#"{"format_version": "1.26.40", "minecraft:geometry": [{"description": {"identifier": "geometry.x"}}]}"#;
    let document = loaded(load(text));
    assert!(!document.version().upgraded);
    assert_eq!(document.version().parsed_as, VERSIONS[7]);
    let older = loaded(load(&text.replace("1.26.40", "1.26.39")));
    assert!(older.version().upgraded);
}

// ---- the fixtures ----------------------------------------------------------------

/// A number is not a declared type of the pre-1.8 `mirror`, so
/// the cube's `mirror: 1` is removed by the lenient validation, where the fixtures u01 expects
/// `true`.
#[test]
fn u01_pre18_mirror_reset() {
    let text = r#"{"geometry.pig": {"texturewidth": 64, "preserve_model_pose": true, "bones": [
        {"name": "body", "mirror": "true", "reset": "false",
         "cubes": [{}, {"origin": [0, 0, 0], "size": [1, 1, 1], "mirror": 1}, {"size": [2, 2, 2], "mirror": "true"}]}]}}"#;
    let document = loaded(load(text));
    let d = document.diagnostics();
    expect(
        d,
        "MissingFormatVersion",
        Severity::Info,
        "no format_version",
    );
    expect_at(
        d,
        "InvalidType",
        Severity::Info,
        "geometry.pig/bones/0/cubes/1/mirror",
        "unknown child schema option type.  Allowed types: bool, string",
    );
    expect_at(
        d,
        "DroppedField",
        Severity::Info,
        "geometry.pig/bones/0/reset",
        "field dropped",
    );
    assert_eq!(document.version().written, None);
    assert_eq!(document.version().effective, FormatVersion::new(1, 0, 0));
    assert_eq!(document.value().format_version, FormatVersion::new(1, 0, 0));
    let geometry = &document.value().geometries[0];
    assert_eq!(geometry.description.texture_width, Some(64));
    let body = &geometry.bones()[0];
    assert_eq!(body.mirror, Some(true));
    assert_eq!(body.legacy.reset, None);
    // The empty cube is replaced by the last one: the order changes.
    let cubes = body.cubes();
    assert_eq!(cubes.len(), 2);
    assert_eq!(cubes[0].size, Some([2.0, 2.0, 2.0]));
    assert_eq!(cubes[0].mirror, Some(true));
    assert_eq!(cubes[1].size, Some([1.0, 1.0, 1.0]));
    assert_eq!(cubes[1].mirror, None);
}

/// The upgrade steps, item 4: bone `parent` is not part of the schema below 1.8, so the
/// lenient validation removes it (ours: with the warning of every removed member); the bone
/// reaches the newest shape without a parent, and from 1.8.0 the member is read.
#[test]
fn a_bone_parent_below_1_8_is_removed_by_the_lenient_validation() {
    let file = |version: &str| {
        format!(
            r#"{{{version}"geometry.pig": {{"bones": [{{"name": "body"}}, {{"name": "leg", "parent": "body"}}]}}}}"#
        )
    };
    for version in [
        "",
        r#""format_version": "1.0.0", "#,
        r#""format_version": "1.7.0", "#,
    ] {
        let document = loaded(load(&file(version)));
        expect_at(
            document.diagnostics(),
            "UnrecognizedField",
            Severity::Info,
            "geometry.pig/bones/1/parent",
            "child 'parent' not valid here (removed; format_version < 1.19.30)",
        );
        assert_eq!(
            count(document.diagnostics(), "UnrecognizedField", Severity::Info),
            1,
            "{version}"
        );
        assert_eq!(
            count(
                document.diagnostics(),
                "UnresolvedReference",
                Severity::Info
            ),
            0,
            "{version}"
        );
        let geometry = &document.value().geometries[0];
        let leg = geometry.bone("leg").expect("the bone is kept");
        assert_eq!(leg.parent, None, "{version}");
        // The written text has no `parent` either.
        assert!(!document.to_json().unwrap().contains("parent"), "{version}");
    }
    let document = loaded(load(&file(r#""format_version": "1.8.0", "#)));
    clean(document.diagnostics());
    let geometry = &document.value().geometries[0];
    assert_eq!(
        geometry.bone("leg").and_then(|b| b.parent.as_deref()),
        Some("body")
    );
}

#[test]
fn u02_pre18_preserve_model_pose() {
    let geometry =
        first(r#"{"format_version": "1.1", "geometry.x": {"preserve_model_pose": true}}"#);
    assert_eq!(geometry.legacy.preserve_model_pose, Some(true));
    assert!(!geometry.legacy.is_empty());
    // From 1.8 the member is undeclared.
    let document = loaded(load(
        r#"{"format_version": "1.8.0", "geometry.x": {"preserve_model_pose": true}}"#,
    ));
    assert!(document.value().geometries[0].legacy.is_empty());
    expect(
        document.diagnostics(),
        "UnrecognizedField",
        Severity::Info,
        "child 'preserve_model_pose' not valid here (removed; format_version < 1.19.30)",
    );
}

#[test]
fn u03_pre18_locator_object() {
    let text = |version: &str| {
        format!(
            r#"{{"format_version": "{version}", "geometry.x": {{"bones": [{{"name": "body", "locators": {{"lead": {{"offset": [0, 1, 0]}}, "hand": [1, 2, 3]}}}}]}}}}"#
        )
    };
    let document = loaded(load(&text("1.7.0")));
    clean(document.diagnostics());
    let locators = document.value().geometries[0].bones()[0]
        .locators
        .clone()
        .unwrap();
    assert_eq!(
        locators["lead"],
        Locator::Placed {
            offset: [0.0, 1.0, 0.0],
            rotation: None,
            ignore_inherited_scale: None
        }
    );
    assert_eq!(locators["hand"], Locator::Position([1.0, 2.0, 3.0]));
    // 1.8 declares the vector only: the object is a wrong type, removed.
    let document = loaded(load(&text("1.8.0")));
    expect_at(
        document.diagnostics(),
        "InvalidType",
        Severity::Info,
        "geometry.x/bones/0/locators/lead",
        "unknown child schema option type.  Allowed types: array",
    );
    let locators = document.value().geometries[0].bones()[0]
        .locators
        .clone()
        .unwrap();
    assert_eq!(locators.len(), 1);
    assert!(locators.contains_key("hand"));
}

#[test]
fn u04_18_to_112_shape() {
    let document = loaded(load(
        r#"{"format_version": "1.8.0",
            "geometry.b": {"texturewidth": 64, "textureheight": 32, "visible_bounds_width": 2, "visible_bounds_height": 3, "visible_bounds_offset": [0, 1, 0]},
            "geometry.a": {"texturewidth": 16}}"#,
    ));
    clean(document.diagnostics());
    let geometries = &document.value().geometries;
    assert_eq!(geometries.len(), 2);
    assert_eq!(geometries[0].description.identifier, "geometry.a");
    assert_eq!(geometries[0].description.texture_width, Some(16));
    assert_eq!(geometries[0].description.texture_height, None);
    let b = &geometries[1].description;
    assert_eq!(b.identifier, "geometry.b");
    assert_eq!((b.texture_width, b.texture_height), (Some(64), Some(32)));
    assert_eq!(b.visible_bounds_width, Some(2.0));
    assert_eq!(b.visible_bounds_height, Some(3.0));
    assert_eq!(b.visible_bounds_offset, Some([0.0, 1.0, 0.0]));
    assert!(geometries[1].legacy.is_empty());
}

#[test]
fn u05_18_inheritance_split() {
    let geometry = first(
        r#"{"format_version": "1.8.0", "geometry.zombie:geometry.humanoid": {"texturewidth": 64}}"#,
    );
    assert_eq!(geometry.description.identifier, "geometry.zombie");
    assert_eq!(geometry.parent(), Some("geometry.humanoid"));
    assert_eq!(geometry.legacy.parent.as_deref(), Some("geometry.humanoid"));
}

#[test]
fn u06_18_legacy_bone_fields() {
    let geometry = first(
        r#"{"format_version": "1.8.0", "geometry.x": {"bones": [
            {"name": "head", "neverRender": true, "reset": true, "bind_pose_rotation": [0, 90, 0], "parent": "Body"},
            {"name": "body"}]}}"#,
    );
    let head = geometry.bone("head").unwrap();
    assert_eq!(head.legacy.never_render, Some(true));
    assert_eq!(head.legacy.reset, Some(true));
    assert_eq!(head.legacy.bind_pose_rotation, Some([0.0, 90.0, 0.0]));
    assert_eq!(head.parent.as_deref(), Some("body"));
    assert!(geometry.bone("body").unwrap().legacy.is_empty());
}

#[test]
fn u07_18_cape_dropped() {
    let document = loaded(load(
        r#"{"format_version": "1.8.0", "geometry.x": {"cape": "geometry.cape"}}"#,
    ));
    assert_eq!(document.value().geometries[0].cape, None);
    expect_at(
        document.diagnostics(),
        "DroppedField",
        Severity::Info,
        "geometry.x/cape",
        "cape is not carried over from format 1.8 geometry",
    );
    // From 1.12 the member is kept.
    let geometry = first(
        r#"{"format_version": "1.12.0", "minecraft:geometry": [{"description": {"identifier": "geometry.x"}, "cape": "geometry.cape"}]}"#,
    );
    assert_eq!(geometry.cape.as_deref(), Some("geometry.cape"));
}

fn cubes(version: &str, cubes: &str) -> String {
    format!(
        r#"{{"format_version": "{version}", "minecraft:geometry": [{{"description": {{"identifier": "geometry.x"}}, "bones": [{{"name": "body", "cubes": [{cubes}]}}]}}]}}"#
    )
}

#[test]
fn u08_112_cube_pivot_fill() {
    let geometry = first(&cubes(
        "1.12.0",
        r#"{"origin": [-4, 0, -2], "size": [8, 12, 4], "rotation": [0, 0, 45], "uv": [0, 0]},
           {"origin": [-4, 0, -2], "size": [8, 12, 4], "pivot": [1, 2, 3]},
           {"size": [8, 12, 5]}"#,
    ));
    let cubes = geometry.bones()[0].cubes();
    assert_eq!(cubes[0].pivot, Some([0.0, 6.0, 0.0]));
    assert_eq!(cubes[0].uv, Some(CubeUv::Box([0.0, 0.0])));
    assert_eq!(cubes[1].pivot, Some([1.0, 2.0, 3.0]));
    assert_eq!(cubes[2].pivot, Some([4.0, 6.0, 2.5]));
}

const BINDING: &str = r#", "binding": "q.item_slot_to_bone_name(c.item_slot)""#;

#[test]
fn u09_114_binding_dropped() {
    for version in ["1.14.0", "1.15.99", "1.12.0"] {
        let document = loaded(load(&crate::common::with_bone(version, BINDING)));
        assert!(document.value().geometries[0].bones()[0].binding.is_none());
        expect_at(
            document.diagnostics(),
            "UnrecognizedField",
            Severity::Info,
            "minecraft:geometry/0/bones/0/binding",
            "child 'binding' not valid here (removed; format_version < 1.19.30)",
        );
    }
}

#[test]
fn u10_116_binding_kept() {
    for version in ["1.16.0", "1.19.30", "1.21.0", "1.26.40"] {
        let document = loaded(load(&crate::common::with_bone(version, BINDING)));
        clean(document.diagnostics());
        let binding = document.value().geometries[0].bones()[0].binding.clone();
        assert_eq!(
            binding
                .as_ref()
                .and_then(|m| m.source())
                .map(bedrock_addon::molang::MolangSource::as_str),
            Some("q.item_slot_to_bone_name(c.item_slot)"),
            "{version}"
        );
    }
}

fn transforms(version: &str, transforms: &str) -> String {
    format!(
        r#"{{"format_version": "{version}", "minecraft:geometry": [{{"description": {{"identifier": "geometry.x"}}, "item_display_transforms": {transforms}}}]}}"#
    )
}

#[test]
fn u11_121_to_12640_shelf() {
    let geometry = first(&transforms(
        "1.21.0",
        r#"{"fixed": {"scale": [0.5, 0.5, 0.5]}}"#,
    ));
    let t = geometry.item_display_transforms.unwrap();
    assert_eq!(t.fixed.as_ref().unwrap().scale, Some([0.5, 0.5, 0.5]));
    assert_eq!(t.shelf, t.fixed);
    // Without `fixed` there is nothing to copy.
    let geometry = first(&transforms(
        "1.21.0",
        r#"{"gui": {"scale": [0.5, 0.5, 0.5]}}"#,
    ));
    assert_eq!(geometry.item_display_transforms.unwrap().shelf, None);
}

#[test]
fn u12_12640_fixed_and_shelf() {
    let document = loaded(load(&transforms(
        "1.26.40",
        r#"{"fixed": {"scale": [0.5, 0.5, 0.5]}, "shelf": {"scale": [2, 2, 2]}}"#,
    )));
    clean(document.diagnostics());
    let t = document.value().geometries[0]
        .item_display_transforms
        .clone()
        .unwrap();
    assert_eq!(t.fixed.unwrap().scale, Some([0.5, 0.5, 0.5]));
    assert_eq!(t.shelf.unwrap().scale, Some([2.0, 2.0, 2.0]));
    // A 1.26.40 file with `fixed` alone gets no copy: the step is for older files.
    let geometry = first(&transforms("1.26.40", r#"{"fixed": {"scale": [1, 1, 1]}}"#));
    assert_eq!(geometry.item_display_transforms.unwrap().shelf, None);
}

#[test]
fn u13_pre18_non_object_cubes() {
    let document = loaded(load(
        r#"{"format_version": "1.0", "geometry.x": {"bones": [{"name": "body", "cubes": [5, {}, {"size": [1, 1, 1]}, null]}]}}"#,
    ));
    let cubes = document.value().geometries[0].bones()[0].cubes().to_vec();
    assert_eq!(cubes.len(), 1);
    assert_eq!(cubes[0].size, Some([1.0, 1.0, 1.0]));
    assert_eq!(
        count(document.diagnostics(), "InvalidType", Severity::Info),
        2
    );
}

#[test]
fn u14_pivot_default_all_versions() {
    let cube = r#"{"origin": [-4, 0, -2], "size": [8, 12, 4], "rotation": [0, 0, 45]}"#;
    for version in ["1.16.0", "1.21.0", "1.26.40"] {
        let geometry = first(&cubes(version, cube));
        let cube = &geometry.bones()[0].cubes()[0];
        assert_eq!(cube.pivot, None, "{version}: not written into the file");
        assert_eq!(cube.pivot(), [0.0, 6.0, 0.0], "{version}");
    }
    // The 1.12 fill gives the same point.
    let filled = first(&cubes("1.12.0", cube));
    assert_eq!(filled.bones()[0].cubes()[0].pivot(), [0.0, 6.0, 0.0]);
}

#[test]
fn u15_121_fixed_out_of_range_copied() {
    let document = loaded(load(&transforms(
        "1.21.0",
        r#"{"fixed": {"scale": [5, 1, 1], "translation": [1, 2, 3]}}"#,
    )));
    let text = "\"item_display_transform\" - scale (5.000000, 1.000000, 1.000000) is out of bounds. Allowed range: (0, 0, 0) to (4, 4, 4).";
    for member in ["fixed", "shelf"] {
        expect_at(
            document.diagnostics(),
            "InvalidValue",
            Severity::Error,
            &format!("minecraft:geometry/0/item_display_transforms/{member}/scale"),
            text,
        );
    }
    let t = document.value().geometries[0]
        .item_display_transforms
        .clone()
        .unwrap();
    // Both were read from the copy; the vector out of range is not applied on either.
    let (fixed, shelf) = (t.fixed.unwrap(), t.shelf.unwrap());
    assert_eq!(fixed.scale, None);
    assert_eq!(shelf.scale, None);
    assert_eq!(shelf.translation, Some([1.0, 2.0, 3.0]));
    assert_eq!(fixed.translation, shelf.translation);
}

#[test]
fn an_upgraded_file_is_written_at_the_newest_version_and_a_newer_one_keeps_its_own() {
    let old = loaded(load(&cubes("1.16.0", r#"{"size": [1, 1, 1]}"#)));
    assert_eq!(old.value().format_version, FormatVersion::new(1, 16, 0));
    assert!(
        old.to_json()
            .unwrap()
            .contains(r#""format_version": "1.26.40""#)
    );
    let new = loaded(load(&cubes("1.26.50", r#"{"size": [1, 1, 1]}"#)));
    assert!(
        new.to_json()
            .unwrap()
            .contains(r#""format_version": "1.26.50""#)
    );
}
