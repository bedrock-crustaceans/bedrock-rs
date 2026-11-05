//! The camera chains, the history path, write-back (the Value level) and the vanilla name
//! lists against core's generated identifiers.

use bedrock_addon::document::{Content, JsonDocument};
use bedrock_addon::families::camera::history::{
    self, Any, EntityAny, EntityFamily, ListAny, ListFamily, PresetFamily, SplineAny, SplineFamily,
};
use bedrock_addon::families::camera::{
    ALLOWED_MC_INHERIT, CameraList, CameraPreset, CameraSpline, VANILLA_CAMERAS,
};
use bedrock_addon::id::{CameraListId, CameraPresetId};
use bedrock_addon::json::Value;
use bedrock_addon::load::LoadContext;
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::{ENGINE_VERSION, FormatVersion, FormatVersionInput};
use bedrock_addon::versioning::{
    EnvelopeCheck, Family, MissingVersion, RangeModel, Selection, VersionChain, check_chain, select,
};

use super::common::*;

fn selects<F: Family>(version: &str) -> Selection {
    select::<F>(&FormatVersionInput::parse(version).expect("a version"))
}

#[test]
fn the_preset_chain_is_1_19_50_then_1_26_50() {
    let versions = <Any as VersionChain<PresetFamily>>::VERSIONS;
    assert_eq!(
        versions,
        [FormatVersion::new(1, 19, 50), FormatVersion::new(1, 26, 50)]
    );
}

#[test]
fn the_preset_step_copies_every_member_and_sets_the_apply_inherited_flag() {
    // Every member of a full 1.19.50 preset arrives unchanged, and nothing else is written.
    let members = r#""identifier": "ex:all", "inherit_from": "minecraft:follow_orbit",
        "pos_x": 1, "pos_y": 2.5, "pos_z": -3, "rot_x": -45, "rot_y": 720,
        "rotation_speed": 180, "snap_to_target": true,
        "horizontal_rotation_limit": [10, 20], "vertical_rotation_limit": [30, 60],
        "continue_targeting": false, "tracking_radius": 25,
        "view_offset": [1, -1], "entity_offset": [0.5, 1.5, -2], "radius": 12,
        "yaw_limits_min": -90, "yaw_limits_max": 90, "listener": "player",
        "player_effects": true,
        "aim_assist": {"preset": "ex:aa", "target_mode": "distance", "angle": [40, 30], "distance": 8},
        "control_scheme": "camera_relative_strafe""#;
    let old = loads(preset(&preset_json("1.19.50", members)));
    let new = loads(preset(&preset_json("1.26.50", members)));
    assert!(old.version().upgraded && !new.version().upgraded);
    // The step stores 1 in the inherited-starting-rotation flag and touches
    // nothing else; a document read at 1.26.50 keeps the constructor's 0.
    assert!(old.applies_inherited_starting_rotation());
    assert!(!new.applies_inherited_starting_rotation());
    let mut copied = old.value().clone();
    copied.apply_inherited_starting_rotation = false;
    assert_eq!(&copied, new.value());
    let written: Value = facet_json::from_str(&old.to_json().unwrap()).unwrap();
    let payload = written
        .as_object()
        .unwrap()
        .get("minecraft:camera_preset")
        .unwrap();
    assert_eq!(
        payload.as_object().unwrap().len(),
        22,
        "the 22 members of the 1.19.50 struct"
    );
}

#[test]
fn the_chains_are_the_plans() {
    const HAS_BETA: bool = <Any as VersionChain<PresetFamily>>::HAS_BETA;

    check_chain::<PresetFamily>().expect("presets");
    check_chain::<ListFamily>().expect("lists");
    check_chain::<SplineFamily>().expect("splines");
    check_chain::<EntityFamily>().expect("camera entities");
    assert_eq!(
        <ListAny as VersionChain<ListFamily>>::VERSIONS,
        [FormatVersion::new(1, 19, 50)]
    );
    assert_eq!(
        <SplineAny as VersionChain<SplineFamily>>::VERSIONS,
        [FormatVersion::new(1, 26, 0)]
    );
    assert_eq!(
        <EntityAny as VersionChain<EntityFamily>>::VERSIONS,
        [FormatVersion::new(0, 0, 0)]
    );
    assert_eq!(PresetFamily::PAYLOAD_KEY, "minecraft:camera_preset");
    assert_eq!(ListFamily::PAYLOAD_KEY, "minecraft:camera_list");
    assert_eq!(SplineFamily::PAYLOAD_KEY, "minecraft:camera_custom_splines");
    assert_eq!(EntityFamily::PAYLOAD_KEY, "minecraft:camera_entity");
    assert_eq!(
        bedrock_addon::families::camera::LOADER.walk.directories,
        ["cameras/presets", "cameras/lists", "cameras/splines"]
    );
    assert_eq!(PresetFamily::FLOOR, FormatVersion::new(1, 19, 50));
    assert_eq!(ListFamily::FLOOR, FormatVersion::new(1, 19, 50));
    assert_eq!(SplineFamily::FLOOR, FormatVersion::new(1, 26, 0));
    assert_eq!(EntityFamily::FLOOR, FormatVersion::new(0, 0, 0));
    assert_eq!(PresetFamily::RANGE_MODEL, RangeModel::ClosedToEngine);
    assert_eq!(PresetFamily::ENVELOPE.check, EnvelopeCheck::Never);
    assert_eq!(ListFamily::ENVELOPE.check, EnvelopeCheck::Never);
    assert_eq!(
        SplineFamily::ENVELOPE.check,
        EnvelopeCheck::From(FormatVersion::new(1, 26, 0))
    );
    assert_eq!(EntityFamily::ENVELOPE.check, EnvelopeCheck::Always);
    assert!(matches!(
        PresetFamily::ENVELOPE.missing,
        MissingVersion::Reject { .. }
    ));
    assert!(matches!(
        ListFamily::ENVELOPE.missing,
        MissingVersion::Reject { .. }
    ));
    assert!(matches!(
        SplineFamily::ENVELOPE.missing,
        MissingVersion::Reject { .. }
    ));
    assert!(matches!(
        EntityFamily::ENVELOPE.missing,
        MissingVersion::Reject { .. }
    ));
    let has_beta = HAS_BETA;
    assert!(!has_beta, "Beta = Infallible");
}

#[test]
fn each_version_selects_the_struct_whose_range_holds_it() {
    assert_eq!(selects::<PresetFamily>("1.19.40"), Selection::BelowFloor);
    assert_eq!(selects::<PresetFamily>("1.19.50"), Selection::Struct(0));
    assert_eq!(selects::<PresetFamily>("1.21.60"), Selection::Struct(0));
    assert_eq!(selects::<PresetFamily>("1.26.40"), Selection::Struct(0));
    assert_eq!(selects::<PresetFamily>("1.26.50"), Selection::Struct(1));
    assert_eq!(
        selects::<PresetFamily>(&ENGINE_VERSION.to_string()),
        Selection::Struct(1)
    );
    assert_eq!(selects::<PresetFamily>("1.26.61"), Selection::Unsupported);
    assert_eq!(selects::<ListFamily>("1.19.40"), Selection::BelowFloor);
    assert_eq!(selects::<ListFamily>("1.19.50"), Selection::Struct(0));
    assert_eq!(
        selects::<ListFamily>(&ENGINE_VERSION.to_string()),
        Selection::Struct(0)
    );
    assert_eq!(selects::<ListFamily>("1.26.61"), Selection::Unsupported);
    assert_eq!(selects::<SplineFamily>("1.25.90"), Selection::BelowFloor);
    assert_eq!(selects::<SplineFamily>("1.26.0"), Selection::Struct(0));
    assert_eq!(selects::<SplineFamily>("1.26.61"), Selection::Unsupported);
    assert_eq!(selects::<EntityFamily>("0.0.0"), Selection::Struct(0));
    assert_eq!(selects::<EntityFamily>("1.26.61"), Selection::Unsupported);
}

#[test]
fn the_beta_literal_selects_nothing() {
    for outcome in [
        rejects(preset(&preset_json("beta", r#""identifier": "ex:a""#))),
        rejects(list(&list_json("beta", "ex:l", r#"["ex:a"]"#))),
    ] {
        let found = errors(&outcome);
        assert!(
            matches!(found[0].0, "BetaFeaturesNotEnabled" | "UnsupportedVersion"),
            "{found:?}"
        );
    }
}

#[test]
fn the_history_path_writes_as_read_and_upgrades_to_the_same_preset() {
    let text = preset_with("1.21.60", r#""radius": 0.1, "listener": "player""#);
    let path = PackPath::from_static("cameras/presets/a.json");
    let cx = LoadContext::detached();
    let written = CameraPreset::parse_as_written(&path, text.as_bytes(), &cx).unwrap();
    assert_eq!(written.version().parsed_as, FormatVersion::new(1, 19, 50));
    assert!(matches!(written.value(), Any::V1_19_50(_)));
    // The history value is written at the version it was read at, and reads back the same.
    let rewritten = written.to_json().unwrap();
    assert!(
        rewritten.contains(r#""format_version": "1.21.60""#),
        "{rewritten}"
    );
    let again = CameraPreset::parse_as_written(&path, rewritten.as_bytes(), &cx).unwrap();
    assert_eq!(again.value(), written.value());
    assert_eq!(again.version().written, Some(FormatVersion::new(1, 21, 60)));
    assert_eq!(again.to_json().unwrap(), rewritten);
    let upgraded = history::upgrade(written, &cx).unwrap();
    let direct = <CameraPreset as Content>::parse(&path, text.as_bytes(), &cx).unwrap();
    assert_eq!(upgraded.value(), direct.value());
    assert_eq!(upgraded.diagnostics(), direct.diagnostics());
    // A rejected file is rejected on the history path too, with the same findings.
    let bad = preset_with("1.21.60", r#""radius": 100"#);
    let a = CameraPreset::parse_as_written(&path, bad.as_bytes(), &cx).unwrap_err();
    let b = <CameraPreset as Content>::parse(&path, bad.as_bytes(), &cx).unwrap_err();
    assert_eq!(errors(&a.diagnostics), errors(&b.diagnostics));
}

/// Writes `value`, reads the text back and requires the same value and an identical rewrite;
/// the written text.
#[track_caller]
fn rewritten<T: Content + PartialEq + std::fmt::Debug>(path: &'static str, value: &T) -> String {
    let written = value.to_json().unwrap();
    let again = loads(load::<T>(path, &written));
    assert_eq!(again.value(), value);
    assert_eq!(again.to_json().unwrap(), written);
    written
}

#[test]
fn vanillas_presets_are_written_back_member_for_member() {
    // Copied unchanged from the BDS 1.26.36.1 `behavior_packs/vanilla/cameras/presets/` and
    // `behavior_packs/experimental_creator_cameras/cameras/presets/`.
    for name in [
        "control_scheme_camera",
        "first_person",
        "fixed_boom",
        "follow_orbit",
        "free",
        "third_person",
        "third_person_front",
    ] {
        let text = fixture(&format!("vanilla/cameras/presets/{name}.json"));
        let document = loads(preset(&text));
        assert!(
            document.diagnostics().is_empty(),
            "{name}: {:#?}",
            document.diagnostics()
        );
        let written = rewritten("cameras/presets/test.json", document.value());
        let original: Value = facet_json::from_str(&text).unwrap();
        let written: Value = facet_json::from_str(&written).unwrap();
        let payload = |value: &Value| {
            value
                .as_object()
                .unwrap()
                .get("minecraft:camera_preset")
                .unwrap()
                .clone()
        };
        // Nothing added, nothing lost: the payload is the file's, member for member.
        assert_eq!(payload(&written), payload(&original), "{name}");
        assert_eq!(
            written
                .as_object()
                .unwrap()
                .get("format_version")
                .unwrap()
                .as_string()
                .unwrap()
                .as_str(),
            "1.19.50",
            "{name}: an upgraded preset is written where the step's flag reads back"
        );
        assert!(document.applies_inherited_starting_rotation(), "{name}");
    }
}

/// The 1.26.60 vanilla presets (`vanilla_unpacked/behavior_packs/{vanilla,
/// experimental_creator_cameras}/cameras/presets/`, copied verbatim): every one a legacy-slice
/// document (1.19.50, 1.21.0, 1.21.80, 1.21.90), each loads without a finding,
/// upgraded with the apply-inherited flag set, is the preset the BDS 1.26.36.1 copy gives, and
/// is written back and read again to the same preset.
#[test]
fn the_1_26_60_vanilla_presets_load_upgraded_and_round_trip() {
    for name in [
        "control_scheme_camera",
        "first_person",
        "fixed_boom",
        "follow_orbit",
        "free",
        "third_person",
        "third_person_front",
    ] {
        let text = fixture(&format!("vanilla_current/cameras/presets/{name}.json"));
        let document = loads(preset(&text));
        assert!(
            document.diagnostics().is_empty(),
            "{name}: {:#?}",
            document.diagnostics()
        );
        assert!(document.version().upgraded, "{name}");
        assert!(document.applies_inherited_starting_rotation(), "{name}");
        let older = loads(preset(&fixture(&format!(
            "vanilla/cameras/presets/{name}.json"
        ))));
        assert_eq!(document.value(), older.value(), "{name}");
        rewritten("cameras/presets/test.json", document.value());
    }
}

#[test]
fn hand_built_documents_write_what_they_hold_and_nothing_else() {
    let mut preset = CameraPreset::new(CameraPresetId::parse("ex:built"));
    preset.inherit_from = Some(CameraPresetId::parse("minecraft:free"));
    preset.radius = Some(0.1);
    preset.player_effects = Some(false);
    preset.starting_rot = Some([1.5, -2.0]);
    let written = rewritten("cameras/presets/test.json", &preset);
    assert_eq!(
        written,
        "{\n  \"format_version\": \"1.26.50\",\n  \"minecraft:camera_preset\": {\n    \"identifier\": \"ex:built\",\n    \"inherit_from\": \"minecraft:free\",\n    \"radius\": 0.1,\n    \"player_effects\": false,\n    \"starting_rot\": [\n      1.5,\n      -2.0\n    ]\n  }\n}\n"
    );

    let list = CameraList::new(
        CameraListId::parse("ex:l"),
        vec![CameraPresetId::parse("minecraft:free")],
    );
    let written = list.to_json().unwrap();
    assert_eq!(
        written,
        "{\n  \"format_version\": \"1.19.50\",\n  \"minecraft:camera_list\": {\n    \"identifier\": \"ex:l\",\n    \"cameras\": [\n      \"minecraft:free\"\n    ]\n  }\n}\n"
    );
    assert_eq!(loads(super::common::list(&written)).value(), &list);

    let text = splines_json(&spline_json(
        "ex:s",
        r#""progress_key_frames": [{"progress": 0.25, "time": 1.5, "easing": "out_expo"}]"#,
    ));
    let document = loads(spline(&text));
    let written = rewritten::<CameraSpline>("cameras/splines/test.json", document.value());
    assert!(
        written.contains(r#""format_version": "1.26.0""#),
        "{written}"
    );
    assert!(
        !written.contains("spline_type"),
        "an absent member is not invented: {written}"
    );
    let original: Value = facet_json::from_str(&text).unwrap();
    let written: Value = facet_json::from_str(&written).unwrap();
    assert_eq!(written, original);
}

#[test]
fn a_preset_named_extra_or_holding_json_null_is_read_as_written() {
    // facet rc.7 binds a skipped field by name: no struct here has one, so a member
    // called `extra` is an unknown member like any other.
    let d = rejects(preset(&preset_with("1.21.60", r#""extra": {"a": 1}"#)));
    assert_eq!(errors(&d)[0].1, "minecraft:camera_preset/extra");
}

#[test]
fn the_vanilla_name_lists_are_vanilla_preset_names() {
    // The names are checked against core's generated table, never a copy.
    let generated = bedrock_addon::vanilla::ids().camera_presets;
    for name in ALLOWED_MC_INHERIT {
        assert!(
            generated.contains(name),
            "{name} is not in mojang-camera-presets.json"
        );
    }
    // `minecraft:death_camera` is a client camera without a preset document.
    for name in VANILLA_CAMERAS
        .iter()
        .filter(|name| **name != "minecraft:death_camera")
    {
        assert!(
            generated.contains(name),
            "{name} is not in mojang-camera-presets.json"
        );
    }
    assert_eq!((ALLOWED_MC_INHERIT.len(), VANILLA_CAMERAS.len()), (5, 4));
    // Every generated name but the experimental pack's own preset is one of the two lists.
    for name in generated {
        assert!(
            ALLOWED_MC_INHERIT.contains(name)
                || VANILLA_CAMERAS.contains(name)
                || *name == "minecraft:control_scheme_camera",
            "{name}"
        );
    }
}

#[test]
fn a_list_naming_a_vanilla_preset_loads_without_the_vanilla_pack_but_not_an_unknown_one() {
    use bedrock_addon::diagnostic::Severity;
    let file = |name: &'static str, cameras: &str| (name, list_json("1.21.60", "ex:l", cameras));
    let stack = stack(&[&[file(
        "cameras/lists/l.json",
        r#"["minecraft:first_person", "minecraft:control_scheme_camera", "minecraft:free"]"#,
    )]]);
    assert_eq!(stack.cameras().lists.len(), 1);
    assert!(group(stack.cameras(), Severity::Error).is_empty());
    let stack = super::common::stack(&[&[file(
        "cameras/lists/l.json",
        r#"["minecraft:death_camera"]"#,
    )]]);
    assert!(stack.cameras().lists.is_empty());
}

/// A 1.26.50 document cannot spell the flag the step sets, so an upgraded
/// preset that keeps it (no `starting_rot`) is written at 1.19.50, the struct's first version,
/// which the game reads with the legacy slice and upgrades the same way; one with a
/// `starting_rot` is written at 1.26.50, where the flag decides nothing.
#[test]
fn an_upgraded_preset_is_written_where_its_flag_reads_back() {
    let document = loads(preset(&preset_with("1.21.60", r#""radius": 5"#)));
    assert!(document.version().upgraded);
    assert!(document.applies_inherited_starting_rotation());
    assert_eq!(
        document.version().written,
        Some(FormatVersion::new(1, 21, 60))
    );
    let written = document.to_json().unwrap();
    assert!(
        written.contains(r#""format_version": "1.19.50""#),
        "{written}"
    );
    let again = loads(preset(&written));
    assert!(again.version().upgraded);
    assert_eq!(again.value(), document.value());
    assert_eq!(again.to_json().unwrap(), written);
    // With a starting rotation the 1.26.50 form is written; the flag reads back `false`, which
    // the client treats the same when `starting_rot` is set.
    let mut rotated = document.value().clone();
    rotated.starting_rot = Some([10.0, 20.0]);
    let written = rotated.to_json().unwrap();
    assert!(
        written.contains(r#""format_version": "1.26.50""#),
        "{written}"
    );
    let again = loads(preset(&written));
    assert!(!again.version().upgraded);
    assert!(!again.applies_inherited_starting_rotation());
    assert_eq!(again.starting_rot, rotated.starting_rot);
    // A preset read at 1.26.50 is written at 1.26.50.
    let native = loads(preset(&preset_with("1.26.50", r#""radius": 5"#)));
    let written = native.to_json().unwrap();
    assert!(
        written.contains(r#""format_version": "1.26.50""#),
        "{written}"
    );
    assert_eq!(loads(preset(&written)).value(), native.value());
}
