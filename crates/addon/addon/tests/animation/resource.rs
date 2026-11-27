//! The resource-pack rows: version gate `v00`–`v07`, strictness `s00`–`s17`, Molang `m00`–`m08`,
//! shapes `c00`–`c16`, stack `k00`–`k04`. Resource pack at `min_engine_version` 1.21.0 unless the
//! row says otherwise.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::document::Content;
use bedrock_addon::families::animation::{
    AnimationFile, AxisElement, Channel, Keyframe, LerpMode, LoopMode, OneOrMany,
};
use bedrock_addon::id::AnimationRef;
use bedrock_addon::molang::Molang;
use bedrock_addon_test_support::corpus;

use super::common::{
    animation, clean, expect, expect_prefix, find, fixture, lines, loaded, rejected, round_trip,
    rp, stacks,
};

const FAMILY: &str = "client_animation";

fn file(version: &str, body: &str) -> String {
    format!(r#"{{"format_version": "{version}", "animations": {{"animation.x": {body}}}}}"#)
}

fn anim(body: &str) -> String {
    file("1.8.0", body)
}

const TYPE_ERROR: &str = "unknown child schema option type.  Allowed types: ";

/// The Molang version of a pack at `min_engine_version` `a.b.c`.
fn pack_molang(a: u16, b: u16, c: u16) -> bedrock_addon::molang::RawVersion {
    bedrock_addon::molang::RawVersion::from(
        bedrock_addon::version::MinEngineVersion::new(a, b, c).molang_version(),
    )
}

fn the(file: &AnimationFile) -> &bedrock_addon::families::animation::Animation {
    &file.animations["animation.x"]
}

// ---- version gate ----

#[test]
fn v00_missing_version() {
    let d = rejected(animation(
        r#"{"animations": {"animation.x": {}}}"#,
        &rp(1, 21, 0),
    ));
    expect(
        &d,
        "core/required_child",
        Severity::Error,
        "Required child format_version not found",
    );
}

#[test]
fn v01_number_version() {
    let text = r#"{"format_version": 1.8, "animations": {"animation.x": {}}}"#;
    expect_prefix(
        &rejected(animation(text, &rp(1, 21, 0))),
        "core/unknown_child_option",
        Severity::Error,
        TYPE_ERROR,
    );
}

#[test]
fn v02_below_floor() {
    let d = rejected(animation(&file("1.7.0", "{}"), &rp(1, 21, 0)));
    expect(
        &d,
        "core/version_below_min",
        Severity::Error,
        "Version 1.7.0 below min required version (1.8.0)",
    );
}

#[test]
fn v03_prerelease_floor() {
    let d = rejected(animation(&file("1.8.0-beta.1", "{}"), &rp(1, 21, 0)));
    expect(
        &d,
        "core/version_below_min",
        Severity::Error,
        "Version 1.8.0-beta.1 below min required version (1.8.0)",
    );
}

#[test]
fn v04_partial() {
    loaded(animation(&file("1.10", "{}"), &rp(1, 21, 0)));
}

#[test]
fn v05_1_8_and_1_10() {
    let body = r#"{"loop": true, "bones": {"a": {"position": [0, 1, 2]}}}"#;
    let old = loaded(animation(&file("1.8.0", body), &rp(1, 21, 0)));
    let new = loaded(animation(&file("1.10.0", body), &rp(1, 21, 0)));
    assert_eq!(old.animations, new.animations);
}

#[test]
fn v06_above_engine() {
    // ENGINE + 0.0.1
    let engine = bedrock_addon::version::ENGINE_VERSION;
    let above = format!("{}.{}.{}", engine.major, engine.minor, engine.patch + 1);
    let document = loaded(animation(&file(&above, "{}"), &rp(1, 21, 0)));
    assert!(
        find(
            document.diagnostics(),
            "core/format_version_above_engine",
            Severity::Info
        )
        .is_some(),
        "{:?}",
        lines(document.diagnostics())
    );
}

#[test]
fn v07_beta_literal() {
    expect_prefix(
        &rejected(animation(&file("beta", "{}"), &rp(1, 21, 0))),
        "core/unknown_child_option",
        Severity::Error,
        TYPE_ERROR,
    );
}

// ---- strictness ----

#[test]
fn s00_invalid_json_and_the_trailing_comma_deviation() {
    let d = rejected(animation(r#"{"format_version" "1.8.0"}"#, &rp(1, 21, 0)));
    expect_prefix(&d, "core/parse_error", Severity::Error, "parse error:\n");
    // Ours: facet-json accepts a trailing comma that jsoncpp rejects.
    loaded(animation(
        &file("1.8.0", r#"{"loop": true,}"#),
        &rp(1, 21, 0),
    ));
}

#[test]
fn s01_unknown_member_drops_every_animation_of_the_file() {
    let text = r#"{"format_version": "1.8.0", "animations": {"animation.ok": {}, "animation.x": {"loops": true}}}"#;
    let d = rejected(animation(text, &rp(1, 21, 0)));
    expect(
        &d,
        "core/child_not_valid",
        Severity::Error,
        "child 'loops' not valid here.",
    );
}

#[test]
fn s02_bad_identifier() {
    let text = r#"{"format_version": "1.8.0", "animations": {"animation:foo": {}}}"#;
    let d = rejected(animation(text, &rp(1, 21, 0)));
    expect(
        &d,
        "core/child_not_valid",
        Severity::Error,
        "child 'animation:foo' not valid here.",
    );
}

#[test]
fn s03_bad_bone_name() {
    let d = rejected(animation(
        &anim(r#"{"bones": {"left arm": {}}}"#),
        &rp(1, 21, 0),
    ));
    expect(
        &d,
        "core/child_not_valid",
        Severity::Error,
        "child 'left arm' not valid here.",
    );
}

#[test]
fn s04_bad_time_key() {
    let body = r#"{"bones": {"a": {"position": {".5": [0, 0, 0]}}}, "timeline": {"1e1": "/say"}}"#;
    let d = rejected(animation(&anim(body), &rp(1, 21, 0)));
    let unknown: Vec<_> = d
        .iter()
        .filter(|f| f.code.id() == "core/child_not_valid")
        .collect();
    assert_eq!(unknown.len(), 2, "{:?}", lines(&d));
    expect(
        &d,
        "core/child_not_valid",
        Severity::Error,
        "child '.5' not valid here.",
    );
    expect(
        &d,
        "core/child_not_valid",
        Severity::Error,
        "child '1e1' not valid here.",
    );
}

#[test]
fn s05_scalar_keyframe() {
    let d = rejected(animation(
        &anim(r#"{"bones": {"a": {"scale": {"0.0": 1.0}}}}"#),
        &rp(1, 21, 0),
    ));
    expect_prefix(&d, "core/unknown_child_option", Severity::Error, TYPE_ERROR);
}

#[test]
fn s06_vec2_keyframe() {
    let d = rejected(animation(
        &anim(r#"{"bones": {"a": {"scale": {"0.0": [0, 1]}}}}"#),
        &rp(1, 21, 0),
    ));
    expect(
        &d,
        "core/array_too_small",
        Severity::Error,
        "Array too small (2 < 3)",
    );
}

#[test]
fn s07_loop_string() {
    let d = rejected(animation(&anim(r#"{"loop": "hold"}"#), &rp(1, 21, 0)));
    expect(
        &d,
        "core/not_allowed_regex",
        Severity::Error,
        "Value 'hold' not allowed.  Must match at least one of the following regex patterns: hold_on_last_frame",
    );
}

#[test]
fn s08_relative_to() {
    let d = rejected(animation(
        &anim(r#"{"bones": {"a": {"relative_to": {"rotation": "parent"}}}}"#),
        &rp(1, 21, 0),
    ));
    expect_prefix(
        &d,
        "core/not_allowed_regex",
        Severity::Error,
        "Value 'parent' not allowed.",
    );
    let d = rejected(animation(
        &anim(r#"{"bones": {"a": {"relative_to": {"position": "entity"}}}}"#),
        &rp(1, 21, 0),
    ));
    expect(
        &d,
        "core/child_not_valid",
        Severity::Error,
        "child 'position' not valid here.",
    );
}

#[cfg(feature = "molang")]
#[test]
fn s09_bad_molang_drops_the_file() {
    let d = rejected(animation(
        &anim(r#"{"blend_weight": "query.("}"#),
        &rp(1, 21, 0),
    ));
    assert!(
        find(&d, "core/invalid_molang", Severity::Error).is_some(),
        "{:?}",
        lines(&d)
    );
}

#[test]
fn s10_extension_must_be_json_as_written() {
    let set = stacks(&[], &[&[("animations/a.JSON", &anim("{}"))]]);
    assert!(set.resource.animations().is_empty());
    assert!(set.resource.animations().diagnostics().next().is_none());
}

#[test]
fn s11_channel_vec2_fails_the_predicate() {
    let empty = rejected(animation(
        &anim(r#"{"bones": {"a": {"position": []}}}"#),
        &rp(1, 21, 0),
    ));
    expect(
        &empty,
        "core/child_not_valid",
        Severity::Error,
        "child 'position' not valid here.",
    );
    let d = rejected(animation(
        &anim(r#"{"bones": {"a": {"position": [0, 1]}}}"#),
        &rp(1, 21, 0),
    ));
    expect(
        &d,
        "core/child_not_valid",
        Severity::Error,
        "child 'position' not valid here.",
    );
    assert!(
        d.iter()
            .all(|f| !f.code.to_string().starts_with("Array too small"))
    );
}

#[test]
fn s12_object_in_position() {
    let d = rejected(animation(
        &anim(r#"{"bones": {"a": {"position": [{"x": 1}, {"y": 2}]}}}"#),
        &rp(1, 21, 0),
    ));
    let found = d
        .iter()
        .find(|f| f.code.id() == "core/unknown_child_option")
        .unwrap_or_else(|| panic!("{:?}", lines(&d)));
    assert!(
        found.location.json_path.to_string().ends_with("position/0"),
        "{}",
        found.location.json_path
    );
}

#[test]
fn s13_empty_required_groups() {
    let cx = rp(1, 21, 0);
    let d = rejected(animation(&anim(r#"{"bones": {}}"#), &cx));
    expect(
        &d,
        "core/required_child",
        Severity::Error,
        "Required child [a-zA-Z0-9_.-]+ not found",
    );
    let d = rejected(animation(
        r#"{"format_version": "1.8.0", "animations": {}}"#,
        &cx,
    ));
    expect(
        &d,
        "core/required_child",
        Severity::Error,
        r"Required child animation\.[a-zA-Z0-9_.-]+ not found",
    );
    let d = rejected(animation(
        &anim(r#"{"bones": {"a": {"rotation": {}}}}"#),
        &cx,
    ));
    expect(
        &d,
        "core/required_child",
        Severity::Error,
        r"Required child [0-9]+(\.[0-9]+)? not found",
    );
}

#[test]
fn s14_lerp_case() {
    let cx = rp(1, 21, 0);
    let document = loaded(animation(
        &anim(
            r#"{"bones": {"a": {"position": {"0": {"lerp_mode": "Linear", "post": [0, 0, 0]}}}}}"#,
        ),
        &cx,
    ));
    let Some(Channel::Keyframes(frames)) = &the(&document).bone("a").unwrap().position else {
        panic!()
    };
    assert!(matches!(&frames["0"], Keyframe::Split(p) if p.lerp_mode == Some(LerpMode::Linear)));
    let d = rejected(animation(
        &anim(r#"{"bones": {"a": {"position": {"0": {"lerp_mode": "step"}}}}}"#),
        &cx,
    ));
    expect_prefix(
        &d,
        "core/not_allowed_keys",
        Severity::Error,
        "Value 'step' not allowed.  Must match at least one of the following keys: ",
    );
}

#[test]
fn s15_key_case() {
    let text = r#"{"FORMAT_VERSION": "1.8.0", "animations": {"Animation.foo": {"Loop": true, "bones": {"b": {"rotation": [{"X": 1}, {"y": 2}]}}}}}"#;
    let document = loaded(animation(text, &rp(1, 21, 0)));
    clean(document.diagnostics());
    let foo = document.get("animation.foo").unwrap();
    assert_eq!(foo.loop_mode(), LoopMode::Loop);
    let Some(Channel::AxisSequence(axes)) = &foo.bone("b").unwrap().rotation else {
        panic!()
    };
    assert!(matches!(&axes[0], AxisElement::Axes(a) if a.contains_key("X")));
    let set = stacks(&[], &[&[("animations/a.json", text)]]);
    assert!(
        set.resource
            .animations()
            .get(&AnimationRef::parse("animation.foo"))
            .is_some()
    );
}

#[test]
fn s17_bind_to_actor_true() {
    let cx = rp(1, 21, 0);
    let d = rejected(animation(
        &anim(r#"{"particle_effects": {"0": {"effect": "a", "bind_to_actor": true}}}"#),
        &cx,
    ));
    expect(
        &d,
        "core/value_not_allowed",
        Severity::Error,
        "Value 'true' not allowed",
    );
    loaded(animation(
        &anim(r#"{"particle_effects": {"0": {"effect": "a", "bind_to_actor": false}}}"#),
        &cx,
    ));
}

#[test]
fn s16_timeline_bad_molang_loads() {
    let document = loaded(animation(
        &anim(r#"{"timeline": {"0.0": "v.x = ("}}"#),
        &rp(1, 21, 0),
    ));
    assert!(the(&document).timeline.is_some());
}

// ---- Molang ----

#[test]
fn m00_number_bool() {
    let document = loaded(animation(
        &anim(r#"{"blend_weight": 0.5, "start_delay": true}"#),
        &rp(1, 21, 0),
    ));
    let a = the(&document);
    assert!(matches!(a.blend_weight, Some(Molang::Const(_))));
    assert!(matches!(a.start_delay, Some(Molang::Bool(true))));
    clean(document.diagnostics());
}

#[test]
fn m01_string_is_an_expression_at_the_pack_version() {
    let document = loaded(animation(
        &anim(r#"{"anim_time_update": "query.anim_time + query.delta_time * 2"}"#),
        &rp(1, 21, 0),
    ));
    let Some(Molang::Expr(source)) = &the(&document).anim_time_update else {
        panic!()
    };
    // The folder says 1.21.0 maps to 13; the engine-version mapping gives 12.
    assert_eq!(source.raw_version(), Some(pack_molang(1, 21, 0)));
    clean(document.diagnostics());
}

#[test]
fn m02_client_only_query_is_clean() {
    clean(
        loaded(animation(
            &anim(r#"{"blend_weight": "query.distance_from_camera < 16"}"#),
            &rp(1, 21, 0),
        ))
        .diagnostics(),
    );
}

#[test]
fn m03_is_on_screen_is_clean() {
    clean(
        loaded(animation(
            &anim(r#"{"blend_weight": "query.is_on_screen ? 1 : 0"}"#),
            &rp(1, 21, 0),
        ))
        .diagnostics(),
    );
}

#[cfg(feature = "molang")]
#[test]
fn m04_unknown_query_drops_the_file() {
    let d = rejected(animation(
        &anim(r#"{"blend_weight": "query.does_not_exist"}"#),
        &rp(1, 21, 0),
    ));
    let invalid = d
        .iter()
        .filter(|f| f.code.id() == "core/invalid_molang" && f.severity == Severity::Error)
        .count();
    assert_eq!(invalid, 2, "{:?}", lines(&d));
}

#[cfg(feature = "molang")]
#[test]
fn m05_timeline_rejected_is_a_kept_error() {
    let document = loaded(animation(
        &anim(r#"{"timeline": {"0.0": "v.x = ("}}"#),
        &rp(1, 21, 0),
    ));
    assert!(
        find(
            document.diagnostics(),
            "core/invalid_molang",
            Severity::Error
        )
        .is_some(),
        "{:?}",
        lines(document.diagnostics())
    );
    // the parser's content-log error, kept: nothing rejects the file
    assert!(document.diagnostics().iter().all(|f| !f.rejects()));
}

#[cfg(feature = "molang")]
#[test]
fn m06_timeline_split_compiles_only_molang() {
    let body = r#"{"timeline": {"0.0": ["", "/say hi", "@s minecraft:foo", "@sfoo", "v.x = 1;"]}}"#;
    let document = loaded(animation(&anim(body), &rp(1, 21, 0)));
    assert!(
        document
            .diagnostics()
            .iter()
            .all(|f| f.code.id() != "core/invalid_molang" && f.code.id() != "core/molang_lint"),
        "{:?}",
        lines(document.diagnostics())
    );
    let OneOrMany::Many(events) = &the(&document).timeline.as_ref().unwrap()["0.0"] else {
        panic!()
    };
    let molang: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            bedrock_addon::families::animation::EventString::Molang(source) => {
                Some(source.as_str())
            }
            _ => None,
        })
        .collect();
    assert_eq!(molang, ["v.x = 1;"]);
}

#[cfg(feature = "molang")]
#[test]
fn m07_version_from_pack() {
    let body = anim(r#"{"start_delay": "'a' + 1"}"#);
    clean(loaded(animation(&body, &rp(1, 16, 0))).diagnostics());
    let d = rejected(animation(&body, &rp(1, 21, 0)));
    assert!(
        find(&d, "core/invalid_molang", Severity::Error).is_some(),
        "{:?}",
        lines(&d)
    );
}

#[test]
fn m08_null_member_is_a_type_error() {
    let d = rejected(animation(&anim(r#"{"loop_delay": null}"#), &rp(1, 21, 0)));
    expect_prefix(&d, "core/unknown_child_option", Severity::Error, TYPE_ERROR);
    assert!(d.iter().all(|f| f.code.id() != "core/invalid_molang"));
}

// ---- shapes and semantics ----

#[test]
fn c00_uniform_scale() {
    let document = loaded(animation(
        &anim(r#"{"bones": {"a": {"scale": 0.7}}}"#),
        &rp(1, 21, 0),
    ));
    assert!(matches!(
        the(&document).bone("a").unwrap().scale,
        Some(Channel::Uniform(_))
    ));
}

#[test]
fn c01_vec3() {
    let document = loaded(animation(
        &anim(r#"{"bones": {"a": {"rotation": [0, "math.sin(q.anim_time*90)", 0]}}}"#),
        &rp(1, 21, 0),
    ));
    assert!(matches!(
        the(&document).bone("a").unwrap().rotation,
        Some(Channel::Vec3(_))
    ));
}

#[test]
fn c02_axis_sequence_of_the_vanilla_squid() {
    // resource_pack/animations/squid.animation.json
    let document = loaded(animation(
        &fixture(FAMILY, "squid.animation.json"),
        &rp(1, 21, 0),
    ));
    let rotate = document.get("animation.squid.rotate").unwrap();
    let Some(Channel::AxisSequence(elements)) = &rotate.bone("body").unwrap().rotation else {
        panic!()
    };
    let axes: Vec<&str> = elements
        .iter()
        .map(|e| match e {
            AxisElement::Axes(a) => a.keys().next().unwrap().as_str(),
            AxisElement::Positional(_) => "positional",
        })
        .collect();
    assert_eq!(axes, ["y", "x", "y"]);
}

#[test]
fn c03_axis_sequence_on_position() {
    let body = r#"{"bones": {"body": {"position": [{"y": 1}, {"x": 2}, {"y": 3}]}}}"#;
    expect_prefix(
        &rejected(animation(&anim(body), &rp(1, 21, 0))),
        "core/unknown_child_option",
        Severity::Error,
        TYPE_ERROR,
    );
}

#[test]
fn c04_keyframes_mixed() {
    let body = r#"{"bones": {"a": {"rotation": {"0": [0,0,0], "0.5": {"lerp_mode": "catmullrom", "post": [1,0,0]}, "1.0": {"pre": [2,0,0], "post": [3,0,0]}}}}}"#;
    let document = loaded(animation(&anim(body), &rp(1, 21, 0)));
    let channel = the(&document).bone("a").unwrap().rotation.as_ref().unwrap();
    let modes: Vec<LerpMode> = channel
        .keyframes_by_time()
        .iter()
        .map(|(_, k)| match k {
            Keyframe::Split(p) => p.lerp_mode.unwrap_or_default(),
            _ => LerpMode::Linear,
        })
        .collect();
    assert_eq!(
        modes,
        [LerpMode::Linear, LerpMode::CatmullRom, LerpMode::Linear]
    );
}

#[test]
fn c05_keyframes_unsorted_are_sorted_by_time() {
    let body = r#"{"bones": {"a": {"position": {"1.0": [1,0,0], "0.0": [0,0,0]}}}}"#;
    let document = loaded(animation(&anim(body), &rp(1, 21, 0)));
    let times: Vec<f32> = the(&document)
        .bone("a")
        .unwrap()
        .position
        .as_ref()
        .unwrap()
        .keyframes_by_time()
        .iter()
        .map(|(t, _)| *t)
        .collect();
    assert_eq!(times, [0.0, 1.0]);
}

#[test]
fn c06_length_absent_is_computed() {
    let body = r#"{"bones": {"a": {"position": {"1.25": [0,0,0]}}}, "timeline": {"2.0": "/say"}, "sound_effects": {"3.0": {"effect": "s"}}}"#;
    let document = loaded(animation(&anim(body), &rp(1, 21, 0)));
    assert!((the(&document).length() - 2.0).abs() < f32::EPSILON);
}

#[test]
fn c07_length_present() {
    let body = r#"{"animation_length": 0.5, "bones": {"a": {"position": {"1.0": [0,0,0]}}}}"#;
    let document = loaded(animation(&anim(body), &rp(1, 21, 0)));
    assert!((the(&document).length() - 0.5).abs() < f32::EPSILON);
}

#[test]
fn c08_events_array() {
    let body = r#"{"particle_effects": {"0": [{"effect": "a"}, {"effect": "b"}]}, "sound_effects": {"0": [{"effect": "s"}]}, "timeline": {"0": ["/a", "/b"]}}"#;
    let document = loaded(animation(&anim(body), &rp(1, 21, 0)));
    let a = the(&document);
    assert!(
        matches!(&a.particle_effects.as_ref().unwrap()["0"], OneOrMany::Many(v) if v.len() == 2)
    );
    assert!(matches!(&a.sound_effects.as_ref().unwrap()["0"], OneOrMany::Many(v) if v.len() == 1));
    assert!(matches!(&a.timeline.as_ref().unwrap()["0"], OneOrMany::Many(v) if v.len() == 2));
}

#[test]
fn c09_loop_modes() {
    let cx = rp(1, 21, 0);
    for (json, mode) in [
        (r#"{"loop": true}"#, LoopMode::Loop),
        (r#"{"loop": false}"#, LoopMode::Stop),
        (
            r#"{"loop": "hold_on_last_frame"}"#,
            LoopMode::HoldOnLastFrame,
        ),
        ("{}", LoopMode::Stop),
    ] {
        assert_eq!(
            the(&loaded(animation(&anim(json), &cx))).loop_mode(),
            mode,
            "{json}"
        );
    }
}

#[test]
fn c10_case_folding_and_duplicates_in_one_pack() {
    let first = r#"{"format_version": "1.8.0", "animations": {"animation.Foo.Bar": {"bones": {"Head": {"scale": 1}}}}}"#;
    let second = r#"{"format_version": "1.8.0", "animations": {"animation.foo.bar": {}}}"#;
    let set = stacks(
        &[],
        &[&[("animations/a.json", first), ("animations/b.json", second)]],
    );
    let registry = set.resource.animations();
    assert!(
        registry
            .get(&AnimationRef::parse("animation.foo.bar"))
            .is_some()
    );
    assert!(
        registry
            .diagnostics()
            .any(|d| d.code.id() == "core/duplicate_identifier" && d.severity == Severity::Info),
        "{registry:?}"
    );
    let alone = stacks(&[], &[&[("animations/a.json", first)]]);
    let foo = alone
        .resource
        .animations()
        .get(&AnimationRef::parse("animation.foo.bar"))
        .unwrap();
    assert!(foo.bone("head").is_some());
}

#[test]
fn c11_vanilla_corpus_loads_without_errors() {
    let Some(corpus) = std::env::var_os("BEDROCK_ADDON_CORPUS") else {
        corpus::skipped(
            "the vanilla corpus row c11",
            corpus::not_set(corpus::CORPUS),
        );
        return;
    };
    let dir = std::path::PathBuf::from(corpus).join("bedrock-samples/resource_pack/animations");
    let cx = rp(1, 21, 0);
    let mut count = 0;
    let mut errors = Vec::new();
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        let text = std::fs::read_to_string(&path).unwrap();
        count += 1;
        match animation(&text, &cx) {
            Ok(document) => errors.extend(
                document
                    .diagnostics()
                    .at_least(Severity::Error)
                    .map(|d| format!("{}: {d}", path.display())),
            ),
            Err(rejected) => errors.push(format!(
                "{}: {}",
                path.display(),
                lines(&rejected.diagnostics).join("; ")
            )),
        }
    }
    assert_eq!(count, 143);
    assert!(errors.is_empty(), "{errors:#?}");
}

#[test]
fn c12_mixed_rotation_array() {
    let document = loaded(animation(
        &anim(r#"{"bones": {"a": {"rotation": [0, {"y": "90"}, 0]}}}"#),
        &rp(1, 21, 0),
    ));
    let Some(Channel::AxisSequence(e)) = &the(&document).bone("a").unwrap().rotation else {
        panic!()
    };
    assert!(matches!(&e[0], AxisElement::Positional(_)));
    assert!(matches!(&e[1], AxisElement::Axes(a) if a.contains_key("y")));
    assert!(matches!(&e[2], AxisElement::Positional(_)));
}

#[test]
fn c13_comments_of_the_vanilla_bee() {
    // resource_pack/animations/bee.animation.json (`//` comments)
    let document = loaded(animation(
        &fixture(FAMILY, "bee.animation.json"),
        &rp(1, 21, 0),
    ));
    assert!(document.get("animation.bee.fly.bobbing").is_some());
    let block = "{ /* before */ \"format_version\": \"1.8.0\", // after\n \"animations\": {\"animation.x\": {}} }";
    loaded(animation(block, &rp(1, 21, 0)));
}

#[test]
fn c14_relative_to_case() {
    let document = loaded(animation(
        &anim(r#"{"bones": {"a": {"relative_to": {"rotation": "Entity"}}}}"#),
        &rp(1, 21, 0),
    ));
    assert!(
        the(&document)
            .bone("a")
            .unwrap()
            .rotation_relative_to_entity()
    );
}

#[test]
fn c15_equal_times_keep_member_order() {
    let document = loaded(animation(
        &anim(r#"{"bones": {"a": {"position": {"0": [1,0,0], "0.0": [2,0,0]}}}}"#),
        &rp(1, 21, 0),
    ));
    let channel = the(&document).bone("a").unwrap().position.as_ref().unwrap();
    let Channel::Keyframes(frames) = channel else {
        panic!()
    };
    let sorted = channel.keyframes_by_time();
    assert_eq!(sorted.len(), 2);
    assert!(std::ptr::eq(sorted[0].1, &raw const frames["0"]));
}

#[test]
fn c16_particle_defaults() {
    let document = loaded(animation(
        &anim(r#"{"particle_effects": {"0": {"effect": "Smoke", "locator": "Head"}}}"#),
        &rp(1, 21, 0),
    ));
    let OneOrMany::One(effect) = &the(&document).particle_effects.as_ref().unwrap()["0"] else {
        panic!()
    };
    assert!(effect.binds_to_actor());
    assert!(effect.pre_effect_script.is_none());
    assert_eq!(
        (
            effect.to_effect_name().as_str(),
            effect.to_locator_name().as_str()
        ),
        ("smoke", "head")
    );
    assert_eq!(effect.effect, "Smoke");
    let bare = loaded(animation(
        &anim(r#"{"particle_effects": {"0": {"effect": "a"}}}"#),
        &rp(1, 21, 0),
    ));
    let OneOrMany::One(bare) = &the(&bare).particle_effects.as_ref().unwrap()["0"] else {
        panic!()
    };
    assert_eq!(bare.to_locator_name(), "");
}

// ---- stack and versions ----

#[test]
fn k00_override_replaces_whole() {
    let high = r#"{"format_version": "1.8.0", "animations": {"animation.x": {"bones": {"high": {"scale": 1}}}}}"#;
    let low = r#"{"format_version": "1.8.0", "animations": {"animation.x": {"bones": {"low": {"scale": 1}}}}}"#;
    let set = stacks(
        &[],
        &[
            &[("animations/a.json", high)],
            &[("animations/a.json", low)],
        ],
    );
    let registry = set.resource.animations();
    let x = registry.get(&AnimationRef::parse("animation.x")).unwrap();
    assert!(x.bone("high").is_some() && x.bone("low").is_none());
    assert!(
        registry
            .diagnostics()
            .any(|d| d.code.id() == "core/shadowed_definition" && d.severity == Severity::Info)
    );
}

#[test]
fn k01_subfolder_is_read() {
    let set = stacks(&[], &[&[("animations/mobs/a.json", &anim("{}"))]]);
    assert!(
        set.resource
            .animations()
            .get(&AnimationRef::parse("animation.x"))
            .is_some()
    );
}

#[test]
fn k04_subpack_and_base() {
    let manifest = format!(
        r#"{{"format_version": 2, "header": {{"name": "p", "description": "", "uuid": "{}", "version": [1, 0, 0], "min_engine_version": [1, 21, 0]}},
           "modules": [{{"type": "resources", "uuid": "{}", "version": [1, 0, 0]}}],
           "subpacks": [{{"folder_name": "hi", "name": "hi", "memory_tier": 0}}]}}"#,
        uuid::Uuid::from_u128(1),
        uuid::Uuid::from_u128(1 | 0xffff_0000_0000_0000_0000),
    );
    let base = r#"{"format_version": "1.8.0", "animations": {"animation.x": {"bones": {"base": {"scale": 1}}}}}"#;
    let sub = r#"{"format_version": "1.8.0", "animations": {"animation.x": {"bones": {"sub": {"scale": 1}}}}}"#;
    let set = stacks(
        &[],
        &[&[
            ("manifest.json", manifest.as_str()),
            ("@subpack", "hi"),
            ("animations/base.json", base),
            ("subpacks/hi/animations/sub.json", sub),
        ]],
    );
    let x = set
        .resource
        .animations()
        .get(&AnimationRef::parse("animation.x"))
        .unwrap();
    assert!(x.bone("base").is_some(), "{x:?}");
}

#[test]
fn k02_molang_version_follows_the_pack() {
    let body = anim(r#"{"blend_weight": "q.anim_time"}"#);
    let version = |cx| match &the(&loaded(animation(&body, &cx))).blend_weight {
        Some(Molang::Expr(s)) => s.raw_version(),
        _ => None,
    };
    let old = version(rp(1, 16, 0));
    let new = version(rp(1, 21, 0));
    assert_ne!(old, new);
    assert_eq!(new, Some(pack_molang(1, 21, 0)));
    assert_eq!(old, Some(pack_molang(1, 16, 0)));
}

#[test]
fn k03_timeline_command_version_follows_the_pack() {
    let body = anim(r#"{"timeline": {"0.0": "/say hi"}}"#);
    let version = |cx| match &the(&loaded(animation(&body, &cx)))
        .timeline
        .as_ref()
        .unwrap()["0.0"]
    {
        OneOrMany::One(bedrock_addon::families::animation::EventString::Command {
            command_version,
            ..
        }) => *command_version,
        other => panic!("{other:?}"),
    };
    assert_eq!(version(rp(1, 16, 0)), 12);
    assert_eq!(version(rp(1, 21, 0)), 39);
}

// ---- ----

#[test]
fn bones_colliding_after_lower_casing_are_both_kept_with_a_warning() {
    let document = loaded(animation(
        &anim(r#"{"bones": {"Head": {"scale": 1}, "head": {"scale": 2}}}"#),
        &rp(1, 21, 0),
    ));
    assert_eq!(the(&document).bones.as_ref().unwrap().len(), 2);
    assert!(
        find(
            document.diagnostics(),
            "core/duplicate_bone",
            Severity::Info
        )
        .is_some(),
        "{:?}",
        lines(document.diagnostics())
    );
}

// ---- link rows (When = link) ----

/// The two files that give the pack the material the particle effect is drawn with.
const MATERIAL_LIST: (&str, &str) = (
    "materials/common.json",
    r#"[{"path": "materials/particles.material"}]"#,
);
const SMOKE_MATERIAL: (&str, &str) = (
    "materials/particles.material",
    r#"{"materials": {"version": "1.0.0", "particles_alpha": {}}}"#,
);

/// A particle effect that loads.
const SMOKE: &str = r#"{"format_version": "1.10.0", "particle_effect": {"description": {"identifier": "demo:smoke", "basic_render_parameters": {"material": "particles_alpha", "texture": "textures/particle/particles"}}, "components": {"minecraft:emitter_rate_instant": {"num_particles": 1}, "minecraft:emitter_lifetime_once": {"active_time": 1}}}}"#;

/// A particle or sound `effect` that is no key of the using client entity's
/// `particle_effects` / `sound_effects` is an `UnresolvedReference` info of the link pass.
// The rule is compiled with its target's feature.
#[cfg(feature = "entity")]
#[test]
fn r15_effect_keys_resolve_against_the_using_client_entity() {
    let entity = r#"{"format_version": "1.10.0", "minecraft:client_entity": {"description": {"identifier": "minecraft:pig",
        "particle_effects": {"smoke": "demo:smoke"}, "sound_effects": {"step": "random.click"},
        "animations": {"a": "animation.x"}}}}"#;
    let text = anim(
        r#"{"particle_effects": {"0": {"effect": "Smoke"}}, "sound_effects": {"0": [{"effect": "step"}, {"effect": "nowhere"}]}}"#,
    );
    let set = stacks(
        &[],
        &[&[
            ("entity/x.entity.json", entity),
            ("animations/a.json", &text),
            // The entity is one of the game's and its particle is defined: the references
            // of the client entity itself are other rows'.
            ("particles/smoke.particle.json", SMOKE),
            // The particle's material is defined too: its reference is another row's.
            MATERIAL_LIST,
            SMOKE_MATERIAL,
        ]],
    );
    let found: Vec<_> = set
        .link()
        .iter()
        .filter(|d| d.code.id() == "core/unresolved_reference")
        .map(|d| (d.severity, d.location.json_path.to_string()))
        .collect();
    // `Smoke` matches `smoke`; only `nowhere` is unresolved.
    assert_eq!(
        found,
        [(
            Severity::Info,
            "animations/animation.x/sound_effects/0/1/effect".to_owned()
        )]
    );
}

#[test]
fn a_keyframe_array_of_four_is_too_large() {
    let d = rejected(animation(
        &anim(r#"{"bones": {"a": {"scale": {"0.0": [0, 1, 2, 3]}}}}"#),
        &rp(1, 21, 0),
    ));
    expect(
        &d,
        "core/array_too_large",
        Severity::Error,
        "Array too large (4 > 3)",
    );
    let d = rejected(animation(
        &anim(r#"{"bones": {"a": {"scale": {"0.0": {"pre": [0, 1, 2, 3]}}}}}"#),
        &rp(1, 21, 0),
    ));
    expect(
        &d,
        "core/array_too_large",
        Severity::Error,
        "Array too large (4 > 3)",
    );
}

#[test]
fn an_unreadable_file_is_a_parse_error() {
    // Vanilla ignores `getResource`'s result, so an unreadable file is the empty string.
    for bytes in [&b""[..], &b"\xff\xfe\x00"[..]] {
        let d = rejected(AnimationFile::parse(
            &bedrock_addon::storage::PackPath::from_static("animations/x.json"),
            bytes,
            &rp(1, 21, 0),
        ));
        expect_prefix(&d, "core/parse_error", Severity::Error, "parse error:\n");
    }
}

/// The copied vanilla files (`resource_pack/animations/`) round-trip
/// at the Value level: written, re-read strictly, equal, and written identically again.
#[test]
fn the_vanilla_squid_and_bee_round_trip() {
    let cx = rp(1, 21, 0);
    for name in ["squid.animation.json", "bee.animation.json"] {
        let document = loaded(animation(&fixture(FAMILY, name), &cx));
        clean(document.diagnostics());
        round_trip(&document, |text| animation(text, &cx));
    }
}

/// a declared member spelled in another case is written back in its declared case;
/// pattern keys (identifiers, bones, time keys, axes) keep their spelling.
#[test]
fn declared_member_names_are_written_canonically() {
    let text = r#"{"FORMAT_VERSION": "1.8.0", "Animations": {"Animation.Foo": {"Loop": true, "Bones": {"Head": {"Rotation": [{"X": 1}, 0, 0]}}}}}"#;
    let document = loaded(animation(text, &rp(1, 21, 0)));
    let written = document.to_json().unwrap();
    for canonical in [
        "\"format_version\"",
        "\"animations\"",
        "\"loop\"",
        "\"bones\"",
        "\"rotation\"",
    ] {
        assert!(written.contains(canonical), "{canonical} in {written}");
    }
    for kept in ["\"Animation.Foo\"", "\"Head\"", "\"X\""] {
        assert!(written.contains(kept), "{kept} in {written}");
    }
}
