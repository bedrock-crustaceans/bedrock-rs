//! One `deviation`-named test per deviation of the family: what we report or do where vanilla is
//! silent or does otherwise. (Three more live beside their rows: `g23_locator_any_length_deviation`,
//! `a_missing_format_version_…_deviation` and `an_unreadable_format_version_…_deviation` in
//! gates.rs, and the path order test in stack.rs.)

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::id::GeometryId;

use crate::common::{count, expect_at, load, loaded, rejected, with_bone};
use crate::stacks::stack;

/// The family notes: below 1.19.30 an unknown member is a warning and is dropped
/// (`vanilla = "silently removed during the upgrade-time lenient validation"`).
#[test]
fn an_unknown_member_below_1_19_30_is_reported_where_vanilla_removes_it_silently_deviation() {
    let document = loaded(load(&with_bone("1.19.29", r#", "glow": 1"#)));
    let d = document.diagnostics();
    assert_eq!(d.len(), 1);
    expect_at(
        d,
        "core/unrecognized_field_worded",
        Severity::Info,
        "minecraft:geometry/0/bones/0/glow",
        "child 'glow' not valid here (removed; format_version < 1.19.30)",
    );
}

/// Each `mirror` / `reset` value the 0.0.0 step removes is an info (vanilla is silent).
#[test]
fn a_mirror_or_reset_the_pre_1_8_step_removes_is_noted_deviation() {
    let document = loaded(load(
        r#"{"format_version": "1.0", "geometry.x": {"bones": [
            {"name": "a", "mirror": "maybe", "reset": false, "cubes": [{"size": [1, 1, 1], "mirror": "no"}]},
            {"name": "b", "mirror": "true", "reset": "true"}]}}"#,
    ));
    let d = document.diagnostics();
    // The three the step removes, and the two texts of bone `b` it holds as booleans
    // (`writable_legacy/candidates.rs`, `geo6_`).
    assert_eq!(d.len(), 5);
    for at in ["bones/1/mirror", "bones/1/reset"] {
        expect_at(
            d,
            "core/value_rewritten",
            Severity::Info,
            &format!("geometry.x/{at}"),
            "value held in another form than it is written in",
        );
    }
    for at in ["bones/0/mirror", "bones/0/reset", "bones/0/cubes/0/mirror"] {
        expect_at(
            d,
            "core/dropped_field",
            Severity::Info,
            &format!("geometry.x/{at}"),
            "field dropped",
        );
    }
    let bones = document.value().geometries[0].bones().to_vec();
    assert_eq!((bones[0].mirror, bones[0].legacy.reset), (None, None));
    assert_eq!(
        (bones[1].mirror, bones[1].legacy.reset),
        (Some(true), Some(true))
    );
}

/// The `cape` a 1.8 geometry loses is noted (vanilla drops it silently).
#[test]
fn the_cape_a_1_8_geometry_loses_is_noted_deviation() {
    let document = loaded(load(
        r#"{"format_version": "1.8.0", "geometry.x": {"cape": "geometry.cape"}}"#,
    ));
    assert_eq!(
        count(
            document.diagnostics(),
            "geometry/cape_dropped",
            Severity::Info
        ),
        1
    );
    assert_eq!(document.diagnostics().len(), 1);
}

/// The 1.8 geometry-level `debug` is copied to `description.debug`, which no later schema
/// declares.
#[test]
fn the_1_8_geometry_debug_is_kept_in_the_side_car_deviation() {
    let document = loaded(load(
        r#"{"format_version": "1.8.0", "geometry.x": {"debug": true}}"#,
    ));
    assert!(document.diagnostics().is_empty());
    assert_eq!(document.value().geometries[0].legacy.debug, Some(true));
}

/// Vanilla: the JSON library's member order, inferred.
#[test]
fn the_geometries_of_a_legacy_file_are_in_key_order_deviation() {
    let document = loaded(load(
        r#"{"format_version": "1.8.0", "geometry.c": {}, "geometry.a": {}, "geometry.b:geometry.a": {}}"#,
    ));
    let identifiers: Vec<&str> = document
        .value()
        .geometries
        .iter()
        .map(|g| g.description.identifier.as_str())
        .collect();
    assert_eq!(identifiers, ["geometry.a", "geometry.b", "geometry.c"]);
}

/// (`vanilla = "silent clamp"`).
#[test]
fn a_clamped_bound_is_noted_deviation() {
    let document = loaded(load(
        r#"{"format_version": "1.12.0", "minecraft:geometry": [{"description": {"identifier": "geometry.x", "visible_bounds_width": 51}}]}"#,
    ));
    expect_at(
        document.diagnostics(),
        "geometry/bounds_clamped",
        Severity::Info,
        "minecraft:geometry/0/description/visible_bounds_width",
        "visible_bounds_width 51.000000 clamped to 50.000000",
    );
    assert_eq!(document.diagnostics().len(), 1);
}

/// (`vanilla = "silent"`).
#[test]
fn an_empty_file_is_noted_deviation() {
    let d = rejected(load(""));
    assert_eq!(count(&d, "core/empty_document", Severity::Info), 1);
    assert!(!d.has_errors());
}

/// (`vanilla = "silent merge"`).
#[test]
fn two_bones_of_one_name_are_reported_deviation() {
    let document = loaded(load(
        r#"{"format_version": "1.12.0", "minecraft:geometry": [{"description": {"identifier": "geometry.x"}, "bones": [{"name": "a"}, {"name": "A"}]}]}"#,
    ));
    assert_eq!(
        count(
            document.diagnostics(),
            "core/duplicate_bone",
            Severity::Info
        ),
        1
    );
    assert_eq!(document.value().geometries[0].bones().len(), 1);
    // The finding is at the later bone and has the code's own text: `DuplicateBone` carries
    // no text of the family, so the validation rules' "bone '%s' is defined more than once in
    // geometry '%s'" is not what is printed.
    expect_at(
        document.diagnostics(),
        "core/duplicate_bone",
        Severity::Info,
        "minecraft:geometry/0/bones/1",
        "two bones collide after lower-casing",
    );
    // The bone that is replaced is noted where it is, so that what it held is known to be
    // gone (INFO, the game merges without a word).
    expect_at(
        document.diagnostics(),
        "core/dropped_field",
        Severity::Info,
        "minecraft:geometry/0/bones/0",
        "field dropped",
    );
}

/// A cube of a geometry below 1.8 that is left without a member (its `mirror` was a string
/// the reader does not read) is still a cube; in the shape of its era, which the file keeps
/// for a `reset` bone, it is written with the member it had, as the empty string, which is
/// not read either: the oldest step removes a cube without members, and the text adds no
/// member the author did not write and loads to the same cube.
#[test]
fn a_pre_1_8_cube_left_without_members_is_written_with_the_mirror_it_had_deviation() {
    let document = loaded(load(
        r#"{"format_version": "1.0", "geometry.x": {"bones": [
            {"name": "a", "cubes": [{"mirror": "no"}]}, {"name": "b", "reset": true}]}}"#,
    ));
    let text = document.to_json().unwrap();
    let again = loaded(load(&text));
    assert_eq!(again.to_json().unwrap(), text, "the written text is stable");
    assert_eq!(
        again.value(),
        document.value(),
        "the same file, value for value"
    );
    let written = text.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        written.contains(r#""cubes": [ { "mirror": "" } ]"#),
        "{written}"
    );
    let cube = &document.value().geometries[0].bones()[0].cubes()[0];
    assert_eq!((cube.origin, cube.mirror), (None, None));
}

/// (`vanilla = "silent"`).
#[test]
fn a_bone_parent_that_is_no_bone_is_reported_deviation() {
    let document = loaded(load(&with_bone("1.12.0", r#", "parent": "ghost""#)));
    assert_eq!(
        count(
            document.diagnostics(),
            "geometry/bone_parent_missing",
            Severity::Info
        ),
        1
    );
    assert_eq!(
        document.value().geometries[0].bones()[0].parent.as_deref(),
        Some("ghost"),
        "the reference is kept"
    );
}

/// (`vanilla = "silent; not loaded"`).
#[test]
fn a_legacy_parent_no_pack_defines_is_an_error_deviation() {
    let child = r#"{"format_version": "1.8.0", "geometry.child:geometry.nowhere": {}}"#;
    let set = stack(&[&[("models/entity/child.json", child)]]);
    let registry = set.resource.geometries();
    assert!(registry.get(&GeometryId::parse("geometry.child")).is_none());
    let errors: Vec<String> = registry
        .diagnostics()
        .filter(|d| d.severity == Severity::Error)
        .map(|d| format!("{} {}", d.code.id(), d.code))
        .collect();
    assert_eq!(
        errors,
        [
            "geometry/parent_not_defined geometry 'geometry.child' inherits from 'geometry.nowhere', which is not defined"
        ]
    );
}

/// pack.
#[test]
fn sub_folders_of_the_model_directories_are_read_deviation() {
    let file = r#"{"format_version": "1.12.0", "minecraft:geometry": [{"description": {"identifier": "geometry.deep"}}]}"#;
    let set = stack(&[&[("models/blocks/a/b/c/deep.json", file)]]);
    assert!(
        set.resource
            .geometries()
            .get(&GeometryId::parse("geometry.deep"))
            .is_some()
    );
}

/// Vanilla 1.26.20: schema rejection; 1.26.45: this message, file rejected: the outcome follows
/// the engine version.
#[test]
fn a_shelf_in_an_older_file_follows_the_engine_version_deviation() {
    let text = r#"{"format_version": "1.21.0", "minecraft:geometry": [{"description": {"identifier": "geometry.x"}, "item_display_transforms": {"shelf": {}}}]}"#;
    let texts = |cx| {
        rejected(crate::common::load_at("models/entity/x.json", text, &cx))
            .iter()
            .map(|d| format!("{:?} {} {}", d.severity, d.code.id(), d.code))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        texts(crate::common::engine(1, 26, 20)),
        [
            "Error core/child_not_valid child 'shelf' not valid here.",
            "Error geometry/did_not_validate geometry file didn't validate"
        ]
    );
    assert_eq!(
        texts(crate::common::engine(1, 26, 45)),
        [
            "Error geometry/shelf_too_old \"item_display_transform\" for \"geometry.x\" is invalid - \"shelf\" can only be used in a file version 1.26.40 or higher"
        ]
    );
}

/// A number beyond `f32` is read as the largest
/// finite float of its sign, with a warning, where vanilla's float conversion gives an infinity the
/// writer has no spelling for.
#[test]
#[expect(clippy::float_cmp, reason = "the reader gives exact values")]
fn a_number_beyond_f32_is_read_as_the_largest_finite_float_with_a_warning_deviation() {
    use bedrock_addon::molang::Molang;
    for version in ["1.12.0", "1.16.0", "1.26.40"] {
        let binding = if version == "1.12.0" {
            ""
        } else {
            r#", "binding": -1e39"#
        };
        let text = with_bone(
            version,
            &format!(r#", "inflate": 1e39, "pivot": [0, -1e39, 1e999]{binding}"#),
        );
        let document = loaded(load(&text));
        let d = document.diagnostics();
        let at = "minecraft:geometry/0/bones/0";
        for member in ["inflate", "pivot/1", "pivot/2"] {
            expect_at(
                d,
                "core/invalid_value",
                Severity::Info,
                &format!("{at}/{member}"),
                "invalid value",
            );
        }
        let bone = &document.value().geometries[0].bones()[0];
        assert_eq!(bone.inflate, Some(f32::MAX));
        assert_eq!(bone.pivot, Some([0.0, f32::MIN, f32::MAX]));
        // the three geometry members are informations; the Molang binding's clamp
        // is core's information too
        let mut infos = 3;
        if version != "1.12.0" {
            infos += 1;
            expect_at(
                d,
                "core/invalid_value",
                Severity::Info,
                &format!("{at}/binding"),
                "invalid value",
            );
            assert!(matches!(bone.binding, Some(Molang::Const(x)) if x == f32::MIN));
        }
        assert_eq!(
            count(d, "core/invalid_value", Severity::Info),
            infos,
            "{version}"
        );
        // What was read can be written, and reads back to the same value without a finding.
        let written = document.to_json().unwrap();
        let again = loaded(load(&written));
        assert_eq!(again.value().geometries, document.value().geometries);
        assert_eq!(
            count(again.diagnostics(), "core/invalid_value", Severity::Info),
            0
        );
    }
    // The largest float itself is no finding.
    let edge = loaded(load(&with_bone(
        "1.26.40",
        r#", "inflate": 3.4028234663852886e38"#,
    )));
    assert_eq!(
        count(edge.diagnostics(), "core/invalid_value", Severity::Info),
        0
    );
    assert_eq!(
        edge.value().geometries[0].bones()[0].inflate,
        Some(f32::MAX)
    );
}

/// Below 1.19.30 an element of `minecraft:geometry` that is no object is left out with a
/// warning ("removed silently and do not fail the file"). A list left WITHOUT an element by that still loads: it held its elements when the
/// minimum of one was checked. The file of no geometry is written at its own version with
/// one element that is no geometry, `null`, and that text loads to the same file.
#[test]
fn a_geometry_list_left_empty_by_its_dropped_elements_loads_and_is_written_deviation() {
    let document = loaded(load(
        r#"{"format_version": "1.12.0", "minecraft:geometry": [7, "x"]}"#,
    ));
    assert!(document.value().geometries.is_empty());
    assert_eq!(
        count(
            document.diagnostics(),
            "core/unknown_child_option",
            Severity::Info
        ),
        2
    );
    assert_eq!(
        count(
            document.diagnostics(),
            "core/array_too_small",
            Severity::Error
        ),
        0
    );
    let text = document.to_json().unwrap();
    let written = text.split_whitespace().collect::<Vec<_>>().join(" ");
    assert_eq!(
        written,
        r#"{ "format_version": "1.12.0", "minecraft:geometry": [ null ] }"#
    );
    let again = loaded(load(&text));
    assert_eq!(again.value(), document.value());
    assert_eq!(again.to_json().unwrap(), text, "the written text is stable");
    assert_eq!(
        count(
            again.diagnostics(),
            "core/unknown_child_option",
            Severity::Info
        ),
        1
    );
    // A list WRITTEN empty is refused, as in the game ("Array too small").
    let d = rejected(load(
        r#"{"format_version": "1.12.0", "minecraft:geometry": []}"#,
    ));
    expect_at(
        &d,
        "core/array_too_small",
        Severity::Error,
        "minecraft:geometry",
        "Array too small (0 < 1)",
    );
    // From 1.19.30 an element that is no geometry refuses the file.
    assert!(
        load(r#"{"format_version": "1.21.0", "minecraft:geometry": [7]}"#).is_err(),
        "strict from 1.19.30"
    );
    // One element left: the file loads and is written in the newest shape.
    let document = loaded(load(
        r#"{"format_version": "1.12.0", "minecraft:geometry": [7, {"description": {"identifier": "geometry.x"}}]}"#,
    ));
    assert_eq!(document.value().geometries.len(), 1);
}

/// A locator whose `offset` or `rotation`
/// is of another length is left out whole (ours, `g23`); the vector's warning alone did not
/// say that the locator's other members go with it. The locator is told too.
#[test]
fn a_locator_left_out_for_a_vector_of_another_length_is_told_at_the_locator_deviation() {
    const LOCATORS: &str = r#", "locators": {"k": {"offset": [0, 1, 0, 9], "rotation": [0, 0, 5], "ignore_inherited_scale": true}, "l": [1, 2, 3]}"#;
    for version in ["1.1.0", "1.21.0"] {
        let text = if version == "1.1.0" {
            format!(
                r#"{{"format_version": "1.1.0", "geometry.x": {{"bones": [{{"name": "body"{LOCATORS}}}]}}}}"#
            )
        } else {
            with_bone(version, LOCATORS)
        };
        let document = loaded(load(&text));
        let told: Vec<String> = document
            .diagnostics()
            .iter()
            .filter(|f| f.location.json_path.to_string().contains("/locators/k"))
            .map(|f| {
                let at = f.location.json_path.to_string();
                let at = at
                    .rsplit_once("/locators/")
                    .map_or("", |(_, at)| at)
                    .to_owned();
                format!("{} {:?} {at}", f.code.id(), f.severity)
            })
            .collect();
        assert_eq!(
            told,
            [
                "core/invalid_value Info k/offset",
                "core/dropped_field Info k"
            ],
            "{version}"
        );
        let written = document.to_json().expect("the geometry writes");
        assert!(
            !written.contains("\"k\"") && written.contains("\"l\""),
            "{written}"
        );
    }
}

/// The game lower-cases a bone's `name` and
/// `parent`, and the geometry is held and written so. A name the file has in another case is
/// noted at the member; a name in lower case says nothing.
#[test]
fn a_bone_name_or_parent_the_lower_casing_changes_is_noted_deviation() {
    let noted = |text: &str| -> Vec<String> {
        let document = loaded(load(text));
        let written = document.to_json().expect("the geometry writes");
        assert!(
            written.contains(r#""name": "head""#) && !written.contains("Head"),
            "{written}"
        );
        document
            .diagnostics()
            .iter()
            .filter(|f| f.code.to_string().starts_with("bone "))
            .map(|f| format!("{:?} {}: {}", f.severity, f.location.json_path, f.code))
            .collect()
    };
    let bones = r#""bones": [{"name": "Body", "pivot": [0, 0, 0]}, {"name": "Head", "parent": "Body", "pivot": [0, 1, 0]}, {"name": "tail", "parent": "body"}]"#;
    assert_eq!(
        noted(&format!(
            r#"{{"format_version": "1.12.0", "minecraft:geometry": [{{"description": {{"identifier": "geometry.x", "texture_width": 64, "texture_height": 64}}, {bones}}}]}}"#
        )),
        [
            "Info minecraft:geometry/0/bones/0/name: bone name 'Body' is read as 'body'",
            "Info minecraft:geometry/0/bones/1/name: bone name 'Head' is read as 'head'",
            "Info minecraft:geometry/0/bones/1/parent: bone parent 'Body' is read as 'body'",
        ],
        "a bone name in another case is not told (1.12.0)"
    );
    assert_eq!(
        noted(&format!(
            r#"{{"format_version": "1.8.0", "geometry.x": {{"texturewidth": 64, "textureheight": 32, {bones}}}}}"#
        )),
        [
            "Info geometry.x/bones/0/name: bone name 'Body' is read as 'body'",
            "Info geometry.x/bones/1/name: bone name 'Head' is read as 'head'",
            "Info geometry.x/bones/1/parent: bone parent 'Body' is read as 'body'",
        ],
        "a bone name in another case is not told (1.8.0)"
    );
    assert!(
        noted(
            r#"{"format_version": "1.12.0", "minecraft:geometry": [{"description": {"identifier": "geometry.x"}, "bones": [{"name": "body"}, {"name": "head", "parent": "body"}]}]}"#
        )
        .is_empty()
    );
}

/// An index of a `poly_mesh` polygon is read as
/// vanilla's unsigned conversion reads a float and written as that integer. A fraction that goes is noted (the
/// game's own reading); an index below zero or beyond 32 bits, bounded by this library, is a
/// warning. Both at the element. A whole index says nothing.
#[test]
fn a_poly_mesh_index_that_is_not_written_as_given_is_told_at_the_element_deviation() {
    let told = |text: &str| -> Vec<String> {
        loaded(load(text))
            .diagnostics()
            .iter()
            .filter(|f| f.location.json_path.to_string().contains("/polys/"))
            .map(|f| {
                let at = f.location.json_path.to_string();
                let at = at
                    .rsplit_once("/polys/")
                    .map_or("", |(_, at)| at)
                    .to_owned();
                format!("{} {:?} {at}", f.code.id(), f.severity)
            })
            .collect()
    };
    let mesh = |polys: &str| {
        format!(
            r#", "poly_mesh": {{"positions": [[0, 0, 0], [1, 0, 0], [0, 1, 0]], "polys": {polys}}}"#
        )
    };
    assert_eq!(
        told(&with_bone(
            "1.12.0",
            &mesh("[[[0, 0, 0], [1.5, -2, 4294967296000], [2, 0, 0]]]")
        )),
        [
            "geometry/index_not_whole Info 0/1/0",
            "geometry/index_not_whole Info 0/1/1",
            "geometry/index_not_whole Info 0/1/2",
        ],
        "an index that is not written as given is not told"
    );
    assert_eq!(
        told(&format!(
            r#"{{"format_version": "1.8.0", "geometry.x": {{"bones": [{{"name": "b"{}}}]}}}}"#,
            mesh("[[[0, 0, 0], [1.5, 1, 0], [2, 0, 0]]]")
        )),
        ["geometry/index_not_whole Info 0/1/0"]
    );
    assert!(
        told(&with_bone(
            "1.12.0",
            &mesh("[[[0, 0, 0], [1, 0, 0], [2.0, 0, 0]]]")
        ))
        .is_empty()
    );
}

/// What the findings under `polys` of a 1.12.0 geometry with this `polys` say, as
/// `code severity path` and the written indices.
fn poly_findings(polys: &str) -> (Vec<String>, String) {
    let text = with_bone(
        "1.12.0",
        &format!(
            r#", "poly_mesh": {{"positions": [[0, 0, 0], [1, 0, 0], [0, 1, 0]], "polys": {polys}}}"#
        ),
    );
    let doc = loaded(load(&text));
    let told = doc
        .diagnostics()
        .iter()
        .filter(|f| f.location.json_path.to_string().contains("/polys/"))
        .map(|f| {
            let at = f.location.json_path.to_string();
            let at = at.rsplit_once("/polys/").map_or("", |(_, at)| at);
            format!("{} {:?} {at}", f.code.id(), f.severity)
        })
        .collect();
    (told, doc.to_json().unwrap_or_default())
}

/// An integer index that fits 32 bits is carried as
/// it was given, not through an `f32`, so `16777217` is not written `16777216`; and an index
/// written as given says nothing, the largest 32-bit one included.
#[test]
fn a_poly_mesh_index_of_a_whole_number_of_32_bits_is_carried_exactly_deviation() {
    let (told, written) = poly_findings("[[[16777217, 0, 0], [4294967295, 1, 0], [2, 0, 0]]]");
    assert!(
        told.is_empty(),
        "an index written as given is told: {told:?}"
    );
    assert!(
        written.contains("16777217") && written.contains("4294967295"),
        "an index is not written as given: {written}"
    );
}

/// A number beyond `f32` as an index is told once at
/// its element, not by the float read and again by the index.
#[test]
fn a_poly_mesh_index_beyond_f32_is_told_once_deviation() {
    let (told, _) = poly_findings("[[[0, 0, 0], [1, 0, 0], [2, 0, 1e39]]]");
    assert_eq!(told, ["geometry/index_not_whole Info 0/2/2"]);
}

/// The game gives an empty name no node (inferred): ours keeps a bone whose name
/// is empty as any other bone, keyed by its name, with no finding; a second one replaces it as
/// two bones of any one name do.
#[test]
fn a_bone_with_an_empty_name_is_kept_and_keyed_deviation() {
    let document = loaded(load(
        r#"{"format_version": "1.16.0", "minecraft:geometry": [{"description": {"identifier": "geometry.x"}, "bones": [
            {"name": "", "inflate": 1}, {"name": "limb", "parent": ""}]}]}"#,
    ));
    assert!(
        document.diagnostics().is_empty(),
        "{:?}",
        crate::common::lines(document.diagnostics())
    );
    let geometry = &document.value().geometries[0];
    let names: Vec<&str> = geometry.bones().iter().map(|b| b.name.as_str()).collect();
    assert_eq!(names, ["", "limb"]);
    assert_eq!(geometry.bone("").and_then(|b| b.inflate), Some(1.0));
    assert_eq!(
        geometry.bone("limb").and_then(|b| b.parent.as_deref()),
        Some(""),
        "a parent of that name is a bone of the geometry"
    );
    crate::common::round_trip(&document);
    // Two of them are one bone, the later one's members standing.
    let document = loaded(load(
        r#"{"format_version": "1.16.0", "minecraft:geometry": [{"description": {"identifier": "geometry.x"}, "bones": [
            {"name": "", "inflate": 1}, {"name": "", "inflate": 2}]}]}"#,
    ));
    expect_at(
        document.diagnostics(),
        "core/duplicate_bone",
        Severity::Info,
        "minecraft:geometry/0/bones/1",
        "two bones collide after lower-casing",
    );
    let geometry = &document.value().geometries[0];
    assert_eq!(geometry.bones().len(), 1);
    assert_eq!(geometry.bone("").and_then(|b| b.inflate), Some(2.0));
}

/// The upgrade steps gates: vanilla turns the armor-offset
/// bones of an old mob into `armor_offset.*` locators when it parses the bones. Ours does not: in every era the bones stay bones and no locator is made.
#[test]
fn the_armor_offset_bones_of_an_old_geometry_stay_bones_deviation() {
    let bones = r#"[{"name": "body"},
        {"name": "leftBootArmorOffset", "parent": "body", "pivot": [1, 2, 3]},
        {"name": "rightbootarmoroffset", "parent": "body", "pivot": [4, 5, 6]}]"#;
    let keyed = |version: &str| {
        format!(r#"{{"format_version": "{version}", "geometry.x": {{"bones": {bones}}}}}"#)
    };
    let listed = |version: &str| {
        format!(
            r#"{{"format_version": "{version}", "minecraft:geometry": [{{"description": {{"identifier": "geometry.x"}}, "bones": {bones}}}]}}"#
        )
    };
    for text in [
        keyed("1.8.0"),
        keyed("1.10.0"),
        listed("1.12.0"),
        listed("1.16.0"),
    ] {
        let document = loaded(load(&text));
        assert_eq!(
            document.diagnostics().at_least(Severity::Warning).count(),
            0,
            "{text}"
        );
        let geometry = &document.value().geometries[0];
        let names: Vec<&str> = geometry.bones().iter().map(|b| b.name.as_str()).collect();
        assert_eq!(
            names,
            ["body", "leftbootarmoroffset", "rightbootarmoroffset"],
            "{text}"
        );
        for bone in geometry.bones() {
            assert!(bone.locators.is_none(), "{text}: {bone:?}");
        }
        assert_eq!(
            geometry.bone("leftbootarmoroffset").and_then(|b| b.pivot),
            Some([1.0, 2.0, 3.0]),
            "{text}"
        );
    }
    // Below 1.8 a bone has no `parent` (it is removed); the offset bones stay bones there too.
    let old = r#"{"geometry.x": {"bones": [{"name": "body"}, {"name": "leftbootarmoroffset", "pivot": [1, 2, 3]}]}}"#;
    let document = loaded(load(old));
    let geometry = &document.value().geometries[0];
    assert_eq!(geometry.bones().len(), 2);
    assert!(geometry.bones().iter().all(|b| b.locators.is_none()));
}
