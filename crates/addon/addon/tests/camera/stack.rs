//! The rows that need the whole stack (`bp.cameras()`): cam07, cam09, cam10, cam17, cam18,
//! cam19 and the duplicate half of cam22, with rules 2, 4, 5 and 6.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::id::{CameraListId, CameraPresetId};

use super::common::*;

fn id(text: &str) -> CameraPresetId {
    CameraPresetId::parse(text)
}

/// A preset file `(path, text)` named `identifier` with `members` more.
fn preset_file(identifier: &str, members: &str) -> (&'static str, String) {
    let separator = if members.is_empty() { "" } else { ", " };
    let name = identifier.replace(':', "_");
    (
        leak(format!("cameras/presets/{name}.json")),
        preset_json(
            "1.21.60",
            &format!(r#""identifier": "{identifier}"{separator}{members}"#),
        ),
    )
}

/// A preset file named `identifier` inheriting from `parent`.
fn child_of(identifier: &str, parent: &str) -> (&'static str, String) {
    preset_file(identifier, &format!(r#""inherit_from": "{parent}""#))
}

/// A list file `(path, text)`.
fn list_file(identifier: &str, cameras: &str) -> (&'static str, String) {
    let name = identifier.replace(':', "_");
    (
        leak(format!("cameras/lists/{name}.json")),
        list_json("1.21.60", identifier, cameras),
    )
}

#[test]
fn cam07_invalid_inheritance_discards_the_preset() {
    let stack = stack(&[&[
        child_of("ex:unknown", "ex:missing"),
        child_of("ex:loop_a", "ex:loop_b"),
        child_of("ex:loop_b", "ex:loop_a"),
        child_of("ex:self", "ex:self"),
        child_of("ex:fp", "minecraft:first_person"),
        child_of("ex:valid", "minecraft:free"),
        child_of("ex:grandchild", "ex:valid"),
        child_of("ex:orphan", "ex:unknown"),
    ]]);
    let cameras = stack.cameras();
    let kept: Vec<&str> = cameras.presets.iter().map(|(id, _)| id.as_str()).collect();
    assert_eq!(kept, ["ex:grandchild", "ex:valid"]);
    let mut found = group(cameras, Severity::Error);
    found.sort();
    assert_eq!(
        found,
        [
            (
                "InvalidReference",
                "Illegal inheritance: Preset 'ex:fp' cannot inherit from 'minecraft:first_person'. It will be discarded.".to_owned()
            ),
            (
                "ReferenceCycle",
                "Circular reference: Preset 'ex:loop_a' inherits from 'ex:loop_a' more than once and will be discarded.".to_owned()
            ),
            (
                "ReferenceCycle",
                "Circular reference: Preset 'ex:loop_b' inherits from 'ex:loop_b' more than once and will be discarded.".to_owned()
            ),
            (
                "ReferenceCycle",
                "Circular reference: Preset 'ex:self' inherits from 'ex:self' more than once and will be discarded.".to_owned()
            ),
            (
                "UnresolvedReference",
                "Preset 'ex:orphan' inherits from unknown preset 'ex:missing' and will be discarded.".to_owned()
            ),
            (
                "UnresolvedReference",
                "Preset 'ex:unknown' inherits from unknown preset 'ex:missing' and will be discarded.".to_owned()
            ),
        ]
    );
    assert_eq!(cameras.presets.rejected().len(), 6);
    assert!(std::ptr::eq(cameras, stack.cameras()));
}

#[test]
fn cam07_a_later_preset_of_the_same_name_replaces_the_earlier_one() {
    // The top pack is read last, so its definition wins, whole.
    let stack = stack(&[
        &[preset_file(
            "ex:same",
            r#""inherit_from": "minecraft:free", "pos_y": 9"#,
        )],
        &[preset_file(
            "ex:same",
            r#""inherit_from": "minecraft:free", "pos_x": 1, "rot_y": 4"#,
        )],
    ]);
    let cameras = stack.cameras();
    assert_eq!(cameras.presets.len(), 1);
    let kept = cameras.presets.get(&id("ex:same")).unwrap();
    assert_eq!(kept.pos_y.map(|v| v.0), Some(9.0));
    assert_eq!(
        (kept.pos_x, kept.rot_y),
        (None, None),
        "not merged member by member"
    );
    assert_eq!(kept.origin().pack.name(), "bp 1");
    assert!(group(cameras, Severity::Error).is_empty());
    assert_eq!(
        group(cameras, Severity::Info),
        [(
            "ShadowedDefinition",
            "definition shadowed by a higher-priority source".to_owned()
        )]
    );
}

#[test]
fn rule_4_within_one_pack_the_later_file_replaces_the_earlier_one() {
    let stack = stack(&[&[
        (
            "cameras/presets/a.json",
            preset_json(
                "1.21.60",
                r#""identifier": "ex:same", "inherit_from": "minecraft:free", "pos_x": 1"#,
            ),
        ),
        (
            "cameras/presets/nested/b.json",
            preset_json(
                "1.21.60",
                r#""identifier": "ex:same", "inherit_from": "minecraft:free", "pos_x": 2"#,
            ),
        ),
    ]]);
    let kept = stack.cameras().presets.get(&id("ex:same")).unwrap();
    assert_eq!(kept.pos_x.map(|v| v.0), Some(2.0));
}

#[test]
fn rule_2_the_discard_pass_sees_the_merged_set_only() {
    // The higher pack's `ex:a` is the merged one: its bad parent discards the name, and the
    // lower pack's valid definition does not come back.
    let stack = stack(&[
        &[child_of("ex:a", "ex:missing")],
        &[child_of("ex:a", "minecraft:free"), child_of("ex:b", "ex:a")],
    ]);
    let cameras = stack.cameras();
    assert!(cameras.presets.is_empty(), "{:?}", cameras.presets);
    let found = group(cameras, Severity::Error);
    assert_eq!(found.len(), 2, "{found:?}");
    assert!(found.iter().all(|(code, _)| *code == "UnresolvedReference"));
    // And a replaced definition's bad parent is no finding at all.
    let stack = stack_of_replaced();
    assert!(group(stack.cameras(), Severity::Error).is_empty());
    assert_eq!(stack.cameras().presets.len(), 1);
}

fn stack_of_replaced() -> bedrock_addon::pack::PackStack<bedrock_addon::pack::Behavior> {
    stack(&[
        &[child_of("ex:a", "minecraft:free")],
        &[child_of("ex:a", "ex:missing")],
    ])
}

#[test]
fn cam17_a_vanilla_camera_may_not_set_its_radius() {
    for name in [
        "minecraft:first_person",
        "minecraft:third_person",
        "minecraft:third_person_front",
        "minecraft:death_camera",
    ] {
        let stack = stack(&[&[preset_file(name, r#""radius": 5"#)]]);
        let cameras = stack.cameras();
        assert!(cameras.presets.is_empty(), "{name}");
        assert_eq!(
            group(cameras, Severity::Error),
            [(
                "InvalidValue",
                format!(
                    "Illegal override: Preset '{name}' cannot change vanilla camera settings. It will be discarded."
                )
            )]
        );
    }
    // Another member is no override, and another preset may set its radius.
    let stack = stack(&[&[
        preset_file("minecraft:third_person", r#""view_offset": [1, 1]"#),
        preset_file("minecraft:follow_orbit", r#""radius": 5"#),
        preset_file(
            "ex:third_person",
            r#""inherit_from": "minecraft:third_person", "radius": 5"#,
        ),
    ]]);
    let cameras = stack.cameras();
    assert_eq!(cameras.presets.len(), 3);
    assert!(group(cameras, Severity::Error).is_empty());
}

#[test]
fn cam18_only_the_allowed_vanilla_cameras_may_be_inherited_from() {
    let stack = stack(&[&[
        child_of("ex:death", "minecraft:death_camera"),
        child_of("ex:first", "minecraft:first_person"),
        child_of("ex:custom", "minecraft:custom"),
        preset_file("minecraft:custom", r#""inherit_from": "minecraft:free""#),
        child_of("ex:boom", "minecraft:fixed_boom"),
        child_of("ex:orbit", "minecraft:follow_orbit"),
        child_of("ex:free", "minecraft:free"),
        child_of("ex:third", "minecraft:third_person"),
        child_of("ex:front", "minecraft:third_person_front"),
        child_of("ex:missing", "ex:nothing"),
    ]]);
    let cameras = stack.cameras();
    let mut kept: Vec<&str> = cameras.presets.iter().map(|(id, _)| id.as_str()).collect();
    kept.sort_unstable();
    assert_eq!(
        kept,
        [
            "ex:boom",
            "ex:free",
            "ex:front",
            "ex:orbit",
            "ex:third",
            "minecraft:custom"
        ]
    );
    let mut found = group(cameras, Severity::Error);
    found.sort();
    let illegal = |preset: &str, parent: &str| {
        (
            "InvalidReference",
            format!(
                "Illegal inheritance: Preset '{preset}' cannot inherit from '{parent}'. It will be discarded."
            ),
        )
    };
    assert_eq!(
        found,
        [
            illegal("ex:custom", "minecraft:custom"),
            illegal("ex:death", "minecraft:death_camera"),
            illegal("ex:first", "minecraft:first_person"),
            (
                "UnresolvedReference",
                "Preset 'ex:missing' inherits from unknown preset 'ex:nothing' and will be discarded.".to_owned()
            ),
        ]
    );
}

#[test]
fn rule_2_the_override_check_runs_before_the_inheritance_checks() {
    let stack = stack(&[&[preset_file(
        "minecraft:third_person",
        r#""radius": 5, "inherit_from": "ex:missing""#,
    )]]);
    let found = group(stack.cameras(), Severity::Error);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].0, "InvalidValue");
}

#[test]
fn cam09_a_list_must_name_known_presets_once_each() {
    let stack = stack(&[&[
        child_of("ex:a", "minecraft:free"),
        child_of("ex:b", "minecraft:free"),
        list_file("ex:empty", "[]"),
        list_file("ex:twice", r#"["ex:a", "ex:b", "ex:a"]"#),
        list_file("ex:unknown", r#"["ex:a", "ex:nothing"]"#),
        list_file("ex:good", r#"["ex:a", "ex:b", "minecraft:free"]"#),
    ]]);
    let cameras = stack.cameras();
    let kept: Vec<&str> = cameras.lists.iter().map(|(id, _)| id.as_str()).collect();
    assert_eq!(kept, ["ex:good"]);
    let good = cameras.lists.get(&CameraListId::parse("ex:good")).unwrap();
    assert_eq!(good.cameras, [id("ex:a"), id("ex:b"), id("minecraft:free")]);
    let mut found = group(cameras, Severity::Error);
    found.sort();
    assert_eq!(
        found,
        [
            (
                "DuplicateIdentifier",
                "Camera List 'ex:twice' references Camera Preset 'ex:a' more than once, it will not be loaded.".to_owned()
            ),
            (
                "InvalidValue",
                "Camera List 'ex:empty' must contain at least one Camera Preset identifier, it will not be loaded.".to_owned()
            ),
            (
                "UnresolvedReference",
                "Camera List 'ex:unknown' references unknown Camera Preset 'ex:nothing', it will not be loaded.".to_owned()
            ),
        ]
    );
    assert_eq!(cameras.lists.rejected().len(), 3);
}

#[test]
fn rule_5_the_list_checks_run_unknown_then_empty_then_duplicate() {
    // Unknown wins over duplicate.
    let stack = stack(&[&[
        child_of("ex:a", "minecraft:free"),
        list_file("ex:l", r#"["ex:a", "ex:a", "ex:nothing"]"#),
    ]]);
    let found = group(stack.cameras(), Severity::Error);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].0, "UnresolvedReference");
}

#[test]
fn rule_5_a_later_list_of_the_same_name_replaces_the_earlier_one() {
    let stack = stack(&[
        &[
            child_of("ex:a", "minecraft:free"),
            list_file("ex:l", r#"["ex:a"]"#),
        ],
        &[list_file("ex:l", r#"["minecraft:third_person"]"#)],
    ]);
    let cameras = stack.cameras();
    let kept = cameras.lists.get(&CameraListId::parse("ex:l")).unwrap();
    assert_eq!(kept.cameras, [id("ex:a")]);
    assert!(group(cameras, Severity::Error).is_empty());
}

#[test]
fn cam10_a_list_below_1_19_50_is_rejected() {
    let d = rejects(list(&list_json("1.19.40", "ex:l", r#"["minecraft:free"]"#)));
    assert_eq!(
        errors(&d),
        [
            finding(
                "UnsupportedVersion",
                "format_version",
                "unsupported format_version"
            ),
            finding("DocumentRejected", "", "Invalid Camera List file")
        ]
    );
    loads(list(&list_json("1.19.50", "ex:l", r#"["minecraft:free"]"#)));
}

#[test]
fn cam19_a_list_above_the_engine_or_without_cameras_is_invalid() {
    let invalid = finding("DocumentRejected", "", "Invalid Camera List file");
    let d = rejects(list(&list_json("1.26.61", "ex:l", r#"["minecraft:free"]"#)));
    assert_eq!(
        errors(&d),
        [
            finding(
                "UnsupportedVersion",
                "format_version",
                "unsupported format_version"
            ),
            invalid.clone()
        ]
    );
    loads(list(&list_json("1.26.60", "ex:l", r#"["minecraft:free"]"#)));
    let missing = |key: &str| {
        finding(
            "MissingRequiredField",
            &format!("minecraft:camera_list/{key}"),
            "missing required field",
        )
    };
    let d = rejects(list(
        r#"{"format_version": "1.21.60", "minecraft:camera_list": {"identifier": "ex:l"}}"#,
    ));
    assert_eq!(errors(&d), [missing("cameras"), invalid.clone()]);
    let d = rejects(list(
        r#"{"format_version": "1.21.60", "minecraft:camera_list": {"cameras": ["ex:a"]}}"#,
    ));
    assert_eq!(errors(&d), [missing("identifier"), invalid.clone()]);
    let d = rejects(list(r#"{"format_version": "1.21.60"}"#));
    assert_eq!(
        errors(&d),
        [
            finding(
                "MissingRequiredField",
                "minecraft:camera_list",
                "missing required field"
            ),
            invalid.clone()
        ]
    );
    let d = rejects(list(
        r#"{"minecraft:camera_list": {"identifier": "ex:l", "cameras": ["ex:a"]}}"#,
    ));
    assert_eq!(
        errors(&d),
        [
            finding("MissingFormatVersion", "", "missing required field"),
            invalid.clone()
        ]
    );
}

#[test]
fn cam19_a_list_naming_a_discarded_preset_is_not_loaded() {
    let stack = stack(&[&[
        child_of("ex:bad", "ex:missing"),
        preset_file("minecraft:third_person", r#""radius": 5"#),
        list_file("ex:l", r#"["ex:bad"]"#),
        list_file("ex:m", r#"["minecraft:third_person"]"#),
    ]]);
    let cameras = stack.cameras();
    assert!(cameras.lists.is_empty());
    let found: Vec<String> = cameras
        .lists
        .diagnostics()
        .filter(|f| f.severity == Severity::Error && !is_validator_line(f))
        .map(|f| f.code.to_string())
        .collect();
    // Both name a preset a file defines (discarded): no line of the game's validator, which
    // misses only a name no file defines.
    assert!(
        validator_lines(cameras)
            .iter()
            .all(|(_, path, _)| !path.starts_with("minecraft:camera_list")),
        "{:?}",
        validator_lines(cameras)
    );
    assert_eq!(
        found,
        [
            "Camera List 'ex:l' references unknown Camera Preset 'ex:bad', it will not be loaded.",
            "Camera List 'ex:m' references unknown Camera Preset 'minecraft:third_person', it will not be loaded.",
        ]
    );
}

#[test]
fn rule_5_a_list_file_is_strict_and_names_its_json_errors() {
    let d = rejects(list(&list_json("1.21.60", "nonamespace", r#"["ex:a"]"#)));
    assert_eq!(
        errors(&d)[0],
        finding(
            "InvalidValue",
            "minecraft:camera_list/identifier",
            "Identifier requires a namespace and name separated by a ':' and each must have at least one alphanumeric or underscore character."
        )
    );
    // `^\w+:\w+$`: unlike a preset's, a list's identifier may not be empty.
    let d = rejects(list(&list_json("1.21.60", "", r#"["ex:a"]"#)));
    assert_eq!(errors(&d)[0].0, "InvalidValue");
    let text = list_json("1.21.60", "ex:l", r#"["ex:a"]"#);
    let d = rejects(list(&text.replacen('{', r#"{"extra": 1, "#, 1)));
    assert_eq!(
        errors(&d),
        [
            finding(
                "UnrecognizedField",
                "extra",
                "this member was found in the input, but is not present in the Schema"
            ),
            finding("DocumentRejected", "", "Invalid Camera List file")
        ]
    );
    let d = rejects(list(&list_json("1.21.60", "ex:l", r#"{"a": 1}"#)));
    assert_eq!(
        errors(&d)[0],
        finding(
            "InvalidType",
            "minecraft:camera_list/cameras",
            "expected an array"
        )
    );
    let d = rejects(list(r#"{"format_version": "1.21.60", "#));
    let found = errors(&d);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].0, "MalformedInput");
    assert!(
        found[0]
            .2
            .starts_with("Camera List file cameras/lists/test.json is not valid JSON: "),
        "{}",
        found[0].2
    );
}

#[test]
fn cam22_two_files_defining_one_spline_name_both_stay_with_a_warning_deviation() {
    let stack = stack(&[
        &[(
            "cameras/splines/top.json",
            splines_json(&spline_json("ex:s", r#""total_time": 9.0"#)),
        )],
        &[(
            "cameras/splines/bottom.json",
            splines_json(&format!(
                "{}, {}",
                spline_json("ex:s", r#""total_time": 1.0"#),
                spline_json("ex:t", "")
            )),
        )],
    ]);
    let cameras = stack.cameras();
    let names: Vec<(&str, f32)> = cameras
        .splines
        .iter()
        .map(|s| (s.name.0.as_str(), s.total_time.0))
        .collect();
    // Load order: the lowest-priority pack first.
    assert_eq!(names, [("ex:s", 1.0), ("ex:t", 4.0), ("ex:s", 9.0)]);
    assert_eq!(cameras.splines.len(), 3);
    assert_eq!(cameras.splines.files().len(), 2);
    assert_eq!(cameras.splines.get("ex:s").unwrap().total_time.0, 1.0);
    assert!(group(cameras, Severity::Error).is_empty());
    let warnings: Vec<(&str, String, String)> = cameras
        .splines
        .diagnostics()
        .filter(|f| f.severity == Severity::Info)
        .map(|f| {
            (
                f.code.name(),
                f.location.path.as_ref().unwrap().as_str().to_owned(),
                f.location.json_path.to_string(),
            )
        })
        .collect();
    assert_eq!(
        warnings,
        [(
            "DuplicateIdentifier",
            "cameras/splines/top.json".to_owned(),
            "minecraft:camera_custom_splines/splines/0/name".to_owned()
        )]
    );
}

#[test]
fn cam24_a_camera_entity_document_in_an_add_on_pack_is_ignored_deviation() {
    // vanilla = "CameraRegistry::loadFrom reads base-game and slice packs only": no accessor
    // reads `cameras/<name>.json`, so the group is empty and reports nothing.
    let stack = stack(&[&[(
        "cameras/free.json",
        entity_json("minecraft:free", r#""minecraft:camera": {}"#),
    )]]);
    let cameras = stack.cameras();
    assert!(cameras.presets.is_empty() && cameras.lists.is_empty());
    assert!(cameras.splines.is_empty());
    assert_eq!(cameras.diagnostics().count(), 0);
}

#[test]
fn a_rejected_spline_file_is_listed_and_the_other_files_stay() {
    let stack = stack(&[&[
        (
            "cameras/splines/bad.json",
            splines_json(&spline_json("nonamespace", "")),
        ),
        (
            "cameras/splines/good.json",
            splines_json(&spline_json("ex:s", "")),
        ),
        ("cameras/splines/skipped.txt", "{}".to_owned()),
    ]]);
    let cameras = stack.cameras();
    assert_eq!(cameras.splines.len(), 1);
    assert_eq!(cameras.splines.rejected().len(), 1);
    assert_eq!(
        cameras.splines.rejected()[0].origin.path.as_str(),
        "cameras/splines/bad.json"
    );
    assert_eq!(group(cameras, Severity::Error).len(), 2);
}

/// `(preset, code, message)` of each error of the group, sorted by preset name.
fn discarded(
    stack: &bedrock_addon::pack::PackStack<bedrock_addon::pack::Behavior>,
) -> Vec<(String, &'static str, String)> {
    let mut found: Vec<_> = stack
        .cameras()
        .diagnostics()
        // The loader's own lines; the validator's are `validator_lines`'.
        .filter(|f| f.severity == Severity::Error && !is_validator_line(f))
        .map(|f| {
            let message = f.code.to_string();
            let preset = message.split('\'').nth(1).unwrap_or_default().to_owned();
            (preset, f.code.name(), message)
        })
        .collect();
    found.sort();
    found
}

fn circular(preset: &str, repeated: &str) -> (String, &'static str, String) {
    (
        preset.to_owned(),
        "ReferenceCycle",
        format!(
            "Circular reference: Preset '{preset}' inherits from '{repeated}' more than once and will be discarded."
        ),
    )
}

#[test]
fn rule_2_every_member_of_a_three_cycle_is_circular() {
    let stack = stack(&[&[
        child_of("ex:a", "ex:b"),
        child_of("ex:b", "ex:c"),
        child_of("ex:c", "ex:a"),
        child_of("ex:ok", "minecraft:free"),
    ]]);
    assert_eq!(
        discarded(&stack),
        [
            circular("ex:a", "ex:a"),
            circular("ex:b", "ex:b"),
            circular("ex:c", "ex:c"),
        ]
    );
    assert_eq!(stack.cameras().presets.len(), 1);
}

#[test]
fn rule_2_a_chain_leading_into_a_cycle_is_circular_at_the_cycles_entry() {
    // `tail2 → tail1 → a → b → a`: the walk from a tail preset passes `a`, goes round and meets
    // `a` again, so each tail preset is discarded with the cycle's entry as the repeated name; the two members name themselves.
    let stack = stack(&[&[
        child_of("ex:tail2", "ex:tail1"),
        child_of("ex:tail1", "ex:a"),
        child_of("ex:a", "ex:b"),
        child_of("ex:b", "ex:a"),
        child_of("ex:side", "ex:b"),
    ]]);
    assert_eq!(
        discarded(&stack),
        [
            circular("ex:a", "ex:a"),
            circular("ex:b", "ex:b"),
            circular("ex:side", "ex:b"),
            circular("ex:tail1", "ex:a"),
            circular("ex:tail2", "ex:a"),
        ]
    );
    assert!(stack.cameras().presets.is_empty());
}

#[test]
fn rule_2_two_disjoint_cycles_and_a_valid_chain_are_judged_apart() {
    let stack = stack(&[&[
        child_of("ex:a1", "ex:a2"),
        child_of("ex:a2", "ex:a1"),
        child_of("ex:b1", "ex:b2"),
        child_of("ex:b2", "ex:b3"),
        child_of("ex:b3", "ex:b1"),
        child_of("ex:ok1", "ex:ok2"),
        child_of("ex:ok2", "minecraft:free"),
        child_of("ex:lost1", "ex:lost2"),
        child_of("ex:lost2", "ex:nothing"),
        child_of("ex:bad1", "ex:bad2"),
        child_of("ex:bad2", "minecraft:first_person"),
    ]]);
    let unknown = |preset: &str| {
        (
            preset.to_owned(),
            "UnresolvedReference",
            format!(
                "Preset '{preset}' inherits from unknown preset 'ex:nothing' and will be discarded."
            ),
        )
    };
    let illegal = |preset: &str| {
        (
            preset.to_owned(),
            "InvalidReference",
            format!(
                "Illegal inheritance: Preset '{preset}' cannot inherit from 'minecraft:first_person'. It will be discarded."
            ),
        )
    };
    assert_eq!(
        discarded(&stack),
        [
            circular("ex:a1", "ex:a1"),
            circular("ex:a2", "ex:a2"),
            circular("ex:b1", "ex:b1"),
            circular("ex:b2", "ex:b2"),
            circular("ex:b3", "ex:b3"),
            illegal("ex:bad1"),
            illegal("ex:bad2"),
            unknown("ex:lost1"),
            unknown("ex:lost2"),
        ]
    );
    let kept: Vec<&str> = stack
        .cameras()
        .presets
        .iter()
        .map(|(id, _)| id.as_str())
        .collect();
    assert_eq!(kept, ["ex:ok1", "ex:ok2"]);
}

/// How many presets the smaller of the long inputs has; the larger has four times as many
/// (20 000). The pass over each preset's whole chain that the walk replaces took minutes at
/// 3 000.
const LONG: usize = 5_000;

/// `count` presets `ex:p0 … ex:p{count-1}`, each inheriting from the next; the last inherits
/// from `last_parent`. Built in a loop.
fn long_chain(count: usize, last_parent: &str) -> Vec<(&'static str, String)> {
    let mut files = Vec::with_capacity(count);
    for i in 0..count {
        let parent = if i + 1 == count {
            last_parent.to_owned()
        } else {
            format!("ex:p{}", i + 1)
        };
        files.push((
            leak(format!("cameras/presets/p{i}.json")),
            preset_json(
                "1.21.60",
                &format!(r#""identifier": "ex:p{i}", "inherit_from": "{parent}""#),
            ),
        ));
    }
    files
}

// The three tests below hold the load of the camera group to the crate's one protocol: the files and the stack are built before the clock starts, the group is loaded
// by its accessor, and what it holds is checked after the clock has stopped.

#[test]
fn rule_2_a_20_000_long_chain_is_judged_in_linear_time() {
    crate::cost::assert_linear_timed_at("camera chain", LONG, |count| {
        let files = long_chain(count, "minecraft:free");
        let stack = stack(&[&files]);
        let (cameras, took) = crate::cost::timed(|| stack.cameras());
        assert_eq!(cameras.presets.len(), count);
        assert_eq!(cameras.presets.rejected().len(), 0);
        assert_eq!(cameras.diagnostics().count(), 0);
        took
    });
    // The same chain ending at a missing preset: every member is discarded for it.
    crate::cost::assert_linear_timed_at("camera chain to a missing preset", LONG, |count| {
        let files = long_chain(count, "ex:nothing");
        let stack = stack_of(&files);
        let (cameras, took) = crate::cost::timed(|| stack.cameras());
        assert!(cameras.presets.is_empty());
        assert_eq!(cameras.presets.rejected().len(), count);
        assert!(cameras.diagnostics().all(|f| {
            f.code.name() == "UnresolvedReference"
                && (f.code.to_string().contains("unknown preset 'ex:nothing'")
                        // The game's validator line of the one direct reference.
                        || f.code.to_string().ends_with(":  Missing referenced asset ex:nothing"))
        }));
        took
    });
}

fn stack_of(
    files: &[(&'static str, String)],
) -> bedrock_addon::pack::PackStack<bedrock_addon::pack::Behavior> {
    stack(&[files])
}

#[test]
fn rule_2_a_20_000_long_cycle_is_judged_in_linear_time() {
    crate::cost::assert_linear_timed_at("camera cycle", LONG, |count| {
        let files = long_chain(count, "ex:p0");
        let stack = stack_of(&files);
        let (cameras, took) = crate::cost::timed(|| stack.cameras());
        assert!(cameras.presets.is_empty());
        assert_eq!(cameras.presets.rejected().len(), count);
        let mut seen = 0;
        for finding in cameras.diagnostics() {
            assert_eq!(finding.code.name(), "ReferenceCycle");
            let message = finding.code.to_string();
            let mut names = message.split('\'').skip(1).step_by(2);
            // Each member of the cycle names itself.
            assert_eq!(names.next(), names.next(), "{message}");
            seen += 1;
        }
        assert_eq!(seen, count);
        took
    });
}

#[test]
fn rule_2_a_tail_is_circular_whether_it_is_walked_before_or_after_its_cycle() {
    // Files load in path order, which is the order the pass walks the presets in. `ex:a_tail`
    // is walked first, on the same walk as the cycle `b → c → b` it leads into; `ex:z_tail` is
    // walked after the cycle has its verdict. Both are discarded, each naming the cycle's entry
    // on its own chain.
    let stack = stack(&[&[
        child_of("ex:a_tail", "ex:b"),
        child_of("ex:b", "ex:c"),
        child_of("ex:c", "ex:b"),
        child_of("ex:z_tail", "ex:c"),
        child_of("ex:a_far", "ex:a_tail"),
    ]]);
    assert_eq!(
        discarded(&stack),
        [
            circular("ex:a_far", "ex:b"),
            circular("ex:a_tail", "ex:b"),
            circular("ex:b", "ex:b"),
            circular("ex:c", "ex:c"),
            circular("ex:z_tail", "ex:c"),
        ],
        "every preset is discarded with its own chain's entry, whichever walk reached it"
    );
    let files: Vec<&str> = stack
        .cameras()
        .presets
        .rejected()
        .iter()
        .map(|rejected| rejected.origin.path.as_str())
        .collect();
    assert_eq!(
        files,
        [
            "cameras/presets/ex_a_far.json",
            "cameras/presets/ex_a_tail.json",
            "cameras/presets/ex_b.json",
            "cameras/presets/ex_c.json",
            "cameras/presets/ex_z_tail.json",
        ],
        "the walk order: both tails of `a` before the cycle, `z_tail` after it"
    );
    assert!(stack.cameras().presets.is_empty());
}

// The per-list checks are, in order: unknown preset, empty list, duplicate; a list that fails
// them is not loaded, and a later list with the same identifier replaces the earlier one.
// The checks are part of loading one file and come before the store, so
// a refused list replaces nothing and the list a lower pack stored stays. The
// presets differ: their discard pass runs once over the merged set.
#[test]
fn rule_5_two_packs_a_refused_list_leaves_the_lower_pack_s_list_in_place() {
    let stack = stack(&[
        &[list_file("ex:l", "[]")],
        &[list_file("ex:l", r#"["minecraft:free"]"#)],
    ]);
    let cameras = stack.cameras();
    let kept = cameras.lists.get(&CameraListId::parse("ex:l")).unwrap();
    assert_eq!(kept.cameras, [id("minecraft:free")]);
    assert_eq!(kept.origin().pack.index(), Some(1));
    let found = group(cameras, Severity::Error);
    assert_eq!(
        found,
        [(
            "InvalidValue",
            "Camera List 'ex:l' must contain at least one Camera Preset identifier, it will not be loaded.".to_owned()
        )]
    );
    let rejected = cameras.lists.rejected();
    assert_eq!(rejected.len(), 1);
    assert_eq!(rejected[0].origin.pack.index(), Some(0));
}

#[test]
fn rule_5_three_packs_a_refused_list_leaves_the_next_pack_s_list_in_place() {
    // Refused on top: the middle pack's list replaced the lowest pack's and stays.
    let loaded = stack(&[
        &[list_file("ex:l", r#"["ex:nothing"]"#)],
        &[list_file("ex:l", r#"["minecraft:third_person"]"#)],
        &[list_file("ex:l", r#"["minecraft:free"]"#)],
    ]);
    let cameras = loaded.cameras();
    let kept = cameras.lists.get(&CameraListId::parse("ex:l")).unwrap();
    assert_eq!(kept.cameras, [id("minecraft:third_person")]);
    assert_eq!(kept.origin().pack.index(), Some(1));
    let found = group(cameras, Severity::Error);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].0, "UnresolvedReference");
    // Refused in the middle and on top: the lowest pack's list stays.
    let loaded = stack(&[
        &[list_file("ex:l", "[]")],
        &[list_file("ex:l", r#"["minecraft:free", "minecraft:free"]"#)],
        &[list_file("ex:l", r#"["minecraft:free"]"#)],
    ]);
    let cameras = loaded.cameras();
    let kept = cameras.lists.get(&CameraListId::parse("ex:l")).unwrap();
    assert_eq!(kept.origin().pack.index(), Some(2));
    assert_eq!(cameras.lists.rejected().len(), 2);
}

// logs a name no file defines, a content-log error, beside the camera loader's own line.
#[test]
fn w3a_a_name_no_file_defines_has_the_validators_line() {
    let stack = stack(&[&[
        child_of("ex:a", "minecraft:free"),
        child_of("ex:orphan", "ex:nothing"),
        child_of("ex:grandchild", "ex:orphan"),
        list_file("ex:unknown", r#"["ex:a", "ex:missing"]"#),
    ]]);
    let cameras = stack.cameras();
    let mut lines = validator_lines(cameras);
    lines.sort();
    assert_eq!(
        lines,
        [
            (
                Severity::Error,
                "minecraft:camera_list/cameras/1".to_owned(),
                "-> minecraft:camera_list -> cameras[1]:  Missing referenced asset ex:missing"
                    .to_owned()
            ),
            // Only the direct reference: `ex:grandchild`'s parent is a file's (discarded).
            (
                Severity::Error,
                "minecraft:camera_preset/inherit_from".to_owned(),
                "-> minecraft:camera_preset -> inherit_from:  Missing referenced asset ex:nothing"
                    .to_owned()
            ),
        ]
    );
}

/// What the link pass reads of a loaded preset's places (`Diagnostics::place_of`): no step of
/// the camera chains moves a member, so a member is where the document holds it, for a legacy
/// preset the step upgraded as for a current one. The findings of the file stay in it.
#[test]
fn a_loaded_preset_places_each_member_where_the_document_holds_it() {
    use bedrock_addon::diagnostic::JsonPath;
    let legacy = preset_file("ex:legacy", r#""pos_x": 1.0"#);
    let current = (
        "cameras/presets/ex_current.json",
        preset_json("1.26.50", r#""identifier": "ex:current", "pos_x": 1.0"#),
    );
    let stack = stack(&[&[legacy, current]]);
    let cameras = stack.cameras();
    let at = JsonPath::of_keys(["minecraft:camera_preset", "pos_x"]);
    for name in ["ex:legacy", "ex:current"] {
        let document = cameras.presets.get(&id(name)).expect("the preset loads");
        assert_eq!(document.diagnostics().place_of(&at), at, "{name}");
        for finding in document.diagnostics() {
            assert_eq!(
                finding.location.path.as_ref(),
                Some(&document.origin().path),
                "{name}: {finding:?}"
            );
        }
    }
}
