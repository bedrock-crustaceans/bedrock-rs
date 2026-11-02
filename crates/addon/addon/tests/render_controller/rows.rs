//! `rc01`–`rc33` (the Molang rows `rc20`,
//! `rc34`–`rc41` are in `molang.rs`, the stack rows `rc22`–`rc26` in `stack.rs`). A resource
//! pack at `min_engine_version` 1.21.100 unless the row says otherwise.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::document::{Content, Document, Rejected};
use bedrock_addon::families::render_controller::{
    BonePattern, RenderController, RenderControllerFile,
};
use bedrock_addon::load::LoadContext;
use bedrock_addon::molang::{Molang, RawVersion};
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::FormatVersion;

use super::common::{
    clean, count, expect, expect_at, expect_prefix, find, fixture, lines, loaded, rejected,
    round_trip, rp,
};

pub const FAMILY: &str = "render_controller";
pub const TYPE_ERROR: &str = "unknown child schema option type.  Allowed types: ";
pub const NO_ELEMENT: &str = "Required child  not found";

/// Loads `text` as `render_controllers/x.json`.
pub fn parse(text: &str, cx: &LoadContext) -> Result<Document<RenderControllerFile>, Rejected> {
    RenderControllerFile::parse(
        &PackPath::from_static("render_controllers/x.json"),
        text.as_bytes(),
        cx,
    )
}

/// A file at 1.8.0 holding `controllers` (the members of `render_controllers`).
pub fn file(controllers: &str) -> String {
    format!(r#"{{"format_version": "1.8.0", "render_controllers": {{{controllers}}}}}"#)
}

/// `controller.render.test` with the three members of vanilla's unit test and `more`.
pub fn test_controller(more: &str) -> String {
    let more = if more.is_empty() {
        String::new()
    } else {
        format!(", {more}")
    };
    format!(
        r#""controller.render.test": {{"geometry": "Geometry.default", "materials": [{{"*": "Material.default"}}], "textures": ["Texture.default"]{more}}}"#
    )
}

/// A valid sibling.
pub const OK: &str =
    r#""controller.render.ok": {"geometry": "Geometry.default", "textures": ["Texture.default"]}"#;

/// A file whose one controller has `more` beside the three usual members.
pub fn with(more: &str) -> String {
    file(&test_controller(more))
}

pub fn test_of(document: &Document<RenderControllerFile>) -> &RenderController {
    &document.render_controllers["controller.render.test"]
}

fn expr(value: &Molang) -> &str {
    value.source().map_or("", |s| s.as_str())
}

// ---- baseline, versions ----

#[test]
fn rc01_vanilla_1_8_0_loads_case_folded() {
    let text = fixture(FAMILY, "armor_stand.render_controllers.json");
    let document = loaded(parse(&text, &rp(1, 21, 100)));
    clean(document.diagnostics());
    let c = &document.render_controllers["controller.render.armor_stand"];
    assert_eq!(c.geometry.as_str(), "geometry.default");
    assert_eq!(
        c.materials,
        [(
            BonePattern("*".to_owned()),
            bedrock_addon::molang::MolangSource::string("material.default", RawVersion(13))
        )]
    );
    assert_eq!(c.textures[0].as_str(), "texture.default");
    assert_eq!(c.part_visibility[0].0.as_str(), "baseplate");
    assert_eq!(expr(&c.part_visibility[0].1), "!query.is_in_ui");
    let overlay = c.overlay_color.as_ref().unwrap();
    assert_eq!(expr(overlay.r.as_ref().unwrap()), "this");
    assert_eq!(expr(overlay.a.as_ref().unwrap()), "0.0");
    assert_eq!(
        document.version().written,
        Some(FormatVersion::new(1, 8, 0))
    );
    round_trip(&document, |t| parse(t, &rp(1, 21, 100)));
}

#[test]
fn rc02_vanilla_1_10_short_is_the_same_model_as_1_8_0() {
    let text = fixture(FAMILY, "bow.render_controllers.json");
    let short = loaded(parse(&text, &rp(1, 21, 100)));
    clean(short.diagnostics());
    assert_eq!(short.format_version, FormatVersion::new(1, 10, 0));
    assert_eq!(short.version().written, Some(FormatVersion::new(1, 10, 0)));
    assert_eq!(short.version().parsed_as, FormatVersion::new(1, 8, 0));
    assert!(!short.version().upgraded);
    let old = loaded(parse(
        &text.replace("\"1.10\"", "\"1.8.0\""),
        &rp(1, 21, 100),
    ));
    assert_eq!(short.render_controllers, old.render_controllers);
    let bow = &short.render_controllers["controller.render.bow"];
    let arrays = bow.arrays.as_ref().unwrap();
    assert_eq!(arrays.textures["array.bow_texture_frames"].len(), 4);
    assert_eq!(arrays.geometries["array.bow_geo_frames"].len(), 4);
    assert!(arrays.materials.is_empty());
    round_trip(&short, |t| parse(t, &rp(1, 21, 100)));
}

#[test]
fn rc03_vanilla_unit_test_input_loads() {
    let document = loaded(parse(&with(""), &rp(1, 21, 100)));
    clean(document.diagnostics());
    assert_eq!(document.render_controllers.len(), 1);
    assert_eq!(test_of(&document).geometry.as_str(), "geometry.default");
}

#[test]
fn rc04_no_textures_drops_the_controller_and_keeps_its_sibling() {
    let text = file(&format!(
        r#""controller.render.test": {{"geometry": "Geometry.default"}}, {OK}"#
    ));
    let document = loaded(parse(&text, &rp(1, 21, 100)));
    expect_at(
        document.diagnostics(),
        "MissingRequiredField",
        Severity::Error,
        "Render controller must contain textures.",
        "render_controllers/controller.render.test/textures",
    );
    assert_eq!(
        document.render_controllers.keys().collect::<Vec<_>>(),
        ["controller.render.ok"]
    );
}

#[test]
fn rc05_below_floor() {
    let d = rejected(parse(&with("").replace("1.8.0", "1.7.0"), &rp(1, 21, 100)));
    expect(
        &d,
        "UnsupportedVersion",
        Severity::Error,
        "Version 1.7.0 below min required version (1.8.0)",
    );
    // The floor itself loads.
    loaded(parse(&with(""), &rp(1, 21, 100)));
}

#[test]
fn rc06_missing_version() {
    let text = format!(r#"{{"render_controllers": {{{}}}}}"#, test_controller(""));
    let d = rejected(parse(&text, &rp(1, 21, 100)));
    expect(
        &d,
        "MissingFormatVersion",
        Severity::Error,
        "Required child format_version not found",
    );
}

#[test]
fn rc06_missing_render_controllers() {
    let d = rejected(parse(r#"{"format_version": "1.8.0"}"#, &rp(1, 21, 100)));
    expect(
        &d,
        "MissingRequiredField",
        Severity::Error,
        "Required child render_controllers not found",
    );
    // An object without a controller: the pattern child is required too.
    let d = rejected(parse(&file(""), &rp(1, 21, 100)));
    expect(
        &d,
        "MissingRequiredField",
        Severity::Error,
        r"Required child controller\.render\.[a-zA-Z0-9_.:-]+ not found",
    );
}

#[test]
fn rc07_bad_version() {
    for written in ["1.8", "[1, 8, 0]", "\"beta\"", "\"abc\"", "\"*\"", "null"] {
        let text = with("").replace("\"1.8.0\"", written);
        let d = rejected(parse(&text, &rp(1, 21, 100)));
        expect_prefix(&d, "UnsupportedVersion", Severity::Error, TYPE_ERROR);
        assert_eq!(d.len(), 1, "{written}: {:?}", lines(&d));
    }
}

#[test]
fn rc08_above_engine_loads_with_an_info_deviation() {
    let document = loaded(parse(&with("").replace("1.8.0", "99.0.0"), &rp(1, 21, 100)));
    clean(document.diagnostics());
    let info = find(
        document.diagnostics(),
        "FormatVersionAboveEngine",
        Severity::Info,
    )
    .unwrap();
    assert_eq!(info.location.json_path.to_string(), "format_version");
    assert_eq!(document.version().parsed_as, FormatVersion::new(1, 8, 0));
    // At the engine version itself nothing is said.
    let engine = bedrock_addon::version::ENGINE_VERSION.to_string();
    let document = loaded(parse(&with("").replace("1.8.0", &engine), &rp(1, 21, 100)));
    assert_eq!(
        count(
            document.diagnostics(),
            "FormatVersionAboveEngine",
            Severity::Info
        ),
        0
    );
}

#[test]
fn rc09_above_the_games_maximum_version() {
    let d = rejected(parse(
        &with("").replace("1.8.0", "65535.0.1"),
        &rp(1, 21, 100),
    ));
    expect(
        &d,
        "UnsupportedVersion",
        Severity::Error,
        "Version 65535.0.1 greater than max allowed version (65535.0.0)",
    );
    // The maximum itself loads.
    loaded(parse(
        &with("").replace("1.8.0", "65535.0.0"),
        &rp(1, 21, 100),
    ));
}

#[test]
fn rc08_prerelease_is_an_ordinary_version() {
    let document = loaded(parse(
        &with("").replace("1.8.0", "1.10.0-beta.1"),
        &rp(1, 21, 100),
    ));
    assert!(
        find(
            document.diagnostics(),
            "PrereleaseFormatVersion",
            Severity::Info
        )
        .is_some()
    );
    // A pre-release of the floor sorts below it.
    let d = rejected(parse(
        &with("").replace("1.8.0", "1.8.0-beta.1"),
        &rp(1, 21, 100),
    ));
    expect_prefix(
        &d,
        "UnsupportedVersion",
        Severity::Error,
        "Version 1.8.0-beta.1 below",
    );
}

// ---- the strict schema ----

#[test]
fn rc10_unknown_member_skips_the_whole_file_deviation() {
    let text = file(&format!(
        r#"{OK}, {}"#,
        test_controller(r#""is_invisible_color": {}"#)
    ));
    let d = rejected(parse(&text, &rp(1, 21, 100)));
    expect_at(
        &d,
        "UnrecognizedField",
        Severity::Error,
        "child 'is_invisible_color' not valid here.",
        "render_controllers/controller.render.test/is_invisible_color",
    );
    // An unknown root member fails the file as well.
    let text = with("").replace("{\"format_version\"", "{\"extra\": 1, \"format_version\"");
    let d = rejected(parse(&text, &rp(1, 21, 100)));
    expect(
        &d,
        "UnrecognizedField",
        Severity::Error,
        "child 'extra' not valid here.",
    );
}

#[test]
fn rc10_a_key_that_is_no_controller_name_is_unknown() {
    for key in [
        "controller.render.",
        "controller.animation.x",
        "cow",
        "controller.render.a b",
    ] {
        let text = file(&format!(
            r#""{key}": {{"geometry": "geometry.default", "textures": ["texture.default"]}}"#
        ));
        let d = rejected(parse(&text, &rp(1, 21, 100)));
        expect(
            &d,
            "UnrecognizedField",
            Severity::Error,
            &format!("child '{key}' not valid here."),
        );
    }
    // `_`, `.`, `:` and `-` are name characters.
    loaded(parse(
        &file(
            r#""controller.render.a_b.c:d-e9": {"geometry": "geometry.default", "textures": ["texture.default"]}"#,
        ),
        &rp(1, 21, 100),
    ));
}

#[test]
fn rc10_a_member_that_failed_is_not_reported_missing_too() {
    // The required check of a node runs only when all of its members validated.
    let text = file(r#""controller.render.test": {"geometry": 1, "textures": ["t"]}"#);
    let d = rejected(parse(&text, &rp(1, 21, 100)));
    expect_prefix(&d, "InvalidType", Severity::Error, TYPE_ERROR);
    assert_eq!(d.len(), 1, "{:?}", lines(&d));
    // Nor is a required member next to an unknown one.
    let text = file(r#""controller.render.test": {"nope": 1, "textures": ["t"]}"#);
    let d = rejected(parse(&text, &rp(1, 21, 100)));
    assert_eq!(d.len(), 1, "{:?}", lines(&d));
}

#[test]
fn rc10_missing_geometry_skips_the_file() {
    let text = file(&format!(
        r#"{OK}, "controller.render.test": {{"textures": ["Texture.default"]}}"#
    ));
    let d = rejected(parse(&text, &rp(1, 21, 100)));
    expect_at(
        &d,
        "MissingRequiredField",
        Severity::Error,
        "Required child geometry not found",
        "render_controllers/controller.render.test",
    );
}

#[test]
fn rc10_wrong_types_are_type_errors() {
    for member in [
        r#""geometry": 1"#,
        r#""textures": "texture.default""#,
        r#""textures": [1]"#,
        r#""materials": {}"#,
        r#""materials": ["x"]"#,
        r#""part_visibility": [[]]"#,
        r#""arrays": []"#,
        r#""arrays": {"textures": []}"#,
        r#""arrays": {"textures": {"array.x": "texture.default"}}"#,
        r#""arrays": {"textures": {"array.x": [true]}}"#,
        r#""color": []"#,
        r#""color": {"r": null}"#,
        r#""color": {"r": []}"#,
        r#""uv_anim": []"#,
        r#""uv_anim": {"offset": 1, "scale": [1, 1]}"#,
        r#""uv_anim": {"offset": [{}, 1], "scale": [1, 1]}"#,
        r#""light_color_multiplier": {}"#,
        r#""ignore_lighting": 1"#,
        r#""filter_lighting": "true""#,
        r#""rebuild_animation_matrices": "yes""#,
    ] {
        let text = file(&format!(
            r#""controller.render.test": {{"geometry": "geometry.default", "textures": ["texture.default"], {member}}}"#
        ));
        let d = rejected(parse(&text, &rp(1, 21, 100)));
        assert!(
            d.iter().any(|f| f.code.name() == "InvalidType"
                && f.severity == Severity::Error
                && f.code.to_string().starts_with(TYPE_ERROR)),
            "{member}: {:?}",
            lines(&d)
        );
    }
    // The root and the two envelope members.
    for text in [
        "[]".to_owned(),
        r#"{"format_version": "1.8.0", "render_controllers": []}"#.to_owned(),
        file(r#""controller.render.test": []"#),
    ] {
        let d = rejected(parse(&text, &rp(1, 21, 100)));
        expect_prefix(&d, "InvalidType", Severity::Error, TYPE_ERROR);
    }
}

#[test]
fn rc11_filter_lighting_false() {
    let d = rejected(parse(&with(r#""filter_lighting": false"#), &rp(1, 21, 100)));
    expect_at(
        &d,
        "InvalidValue",
        Severity::Error,
        "Value 'false' not allowed",
        "render_controllers/controller.render.test/filter_lighting",
    );
    let document = loaded(parse(&with(r#""filter_lighting": true"#), &rp(1, 21, 100)));
    assert!(test_of(&document).filter_lighting());
    round_trip(&document, |t| parse(t, &rp(1, 21, 100)));
}

#[test]
fn rc12_uv_anim_needs_both_members() {
    let d = rejected(parse(
        &with(r#""uv_anim": {"offset": [0, 0]}"#),
        &rp(1, 21, 100),
    ));
    expect(
        &d,
        "MissingRequiredField",
        Severity::Error,
        "Required child scale not found",
    );
    let d = rejected(parse(
        &with(r#""uv_anim": {"scale": [1, 1]}"#),
        &rp(1, 21, 100),
    ));
    expect(
        &d,
        "MissingRequiredField",
        Severity::Error,
        "Required child offset not found",
    );
}

#[test]
fn rc13_uv_anim_arrays_have_exactly_two_elements() {
    let d = rejected(parse(
        &with(r#""uv_anim": {"offset": [0, 0, 0], "scale": [1, 1]}"#),
        &rp(1, 21, 100),
    ));
    expect_at(
        &d,
        "InvalidValue",
        Severity::Error,
        "Array too large (3 > 2)",
        "render_controllers/controller.render.test/uv_anim/offset",
    );
    let d = rejected(parse(
        &with(r#""uv_anim": {"offset": [0, 0], "scale": [1]}"#),
        &rp(1, 21, 100),
    ));
    expect_at(
        &d,
        "InvalidValue",
        Severity::Error,
        "Array too small (1 < 2)",
        "render_controllers/controller.render.test/uv_anim/scale",
    );
    let document = loaded(parse(
        &with(r#""uv_anim": {"offset": [0, 0.5], "scale": [1, 2]}"#),
        &rp(1, 21, 100),
    ));
    let uv = test_of(&document).uv_anim();
    assert_eq!(uv.offset, [Molang::Const(0.0), Molang::Const(0.5)]);
    assert_eq!(uv.scale, [Molang::Const(1.0), Molang::Const(2.0)]);
    round_trip(&document, |t| parse(t, &rp(1, 21, 100)));
}

#[test]
fn rc14_bone_pattern_hyphen() {
    let text = file(
        r#""controller.render.test": {"geometry": "geometry.default", "textures": ["texture.default"], "materials": [{"left-arm": "Material.default"}]}"#,
    );
    let d = rejected(parse(&text, &rp(1, 21, 100)));
    expect_at(
        &d,
        "UnrecognizedField",
        Severity::Error,
        "child 'left-arm' not valid here.",
        "render_controllers/controller.render.test/materials/0/left-arm",
    );
    // The same charset holds for `part_visibility`.
    let d = rejected(parse(
        &with(r#""part_visibility": [{"left arm": 1}]"#),
        &rp(1, 21, 100),
    ));
    expect(
        &d,
        "UnrecognizedField",
        Severity::Error,
        "child 'left arm' not valid here.",
    );
}

// ---- case folding ----

#[test]
fn rc15_case_folding() {
    let text = fixture(FAMILY, "rabbit.render_controllers.json");
    let document = loaded(parse(&text, &rp(1, 21, 100)));
    clean(document.diagnostics());
    let rabbit = &document.render_controllers["controller.render.rabbit"];
    assert_eq!(
        rabbit.textures[0].as_str(),
        "query.get_name == 'Toast' ? texture.toast : array.skins[query.variant]"
    );
    assert_eq!(
        rabbit.arrays.as_ref().unwrap().textures["array.skins"][0].as_str(),
        "texture.brown"
    );
    round_trip(&document, |t| parse(t, &rp(1, 21, 100)));

    let custom = r#"{"Format_Version": "1.8.0", "Render_Controllers": {"Controller.Render.Foo": {
        "Arrays": {"Textures": {"Array.Skins": ["Texture.A"]}},
        "Geometry": "Geometry.Default", "Textures": ["Array.Skins[Q.Variant]"]}}}"#;
    let document = loaded(parse(custom, &rp(1, 21, 100)));
    clean(document.diagnostics());
    let foo = document.get("Controller.Render.FOO").unwrap();
    assert_eq!(foo.geometry.as_str(), "geometry.default");
    assert_eq!(foo.textures[0].as_str(), "array.skins[q.variant]");
    assert!(
        foo.arrays
            .as_ref()
            .unwrap()
            .textures
            .contains_key("array.skins")
    );
}

#[test]
fn rc16_apostrophe_quirk() {
    let text = "{\"format_version\": \"1.8.0\", \"render_controllers\": {\n\"controller.render.a\": {\n\"textures\": [\"textures/entity/bob's_hat\"],\n\"Geometry\": \"geometry.default\"}}}";
    let d = rejected(parse(text, &rp(1, 21, 100)));
    expect_at(
        &d,
        "UnrecognizedField",
        Severity::Error,
        "child 'Geometry' not valid here.",
        "render_controllers/controller.render.a/Geometry",
    );
    let hint = find(&d, "CaseFoldingDisabled", Severity::Info).unwrap();
    assert_eq!(
        hint.code.to_string(),
        "an apostrophe at line 3 disabled case folding"
    );
    assert_eq!(
        hint.location.json_path.to_string(),
        "render_controllers/controller.render.a/Geometry"
    );
    // Without the apostrophe the key is folded and the file loads; an error before any
    // apostrophe has no hint.
    loaded(parse(&text.replace("bob's", "bobs"), &rp(1, 21, 100)));
    let d = rejected(parse(&with(r#""nope": 1"#), &rp(1, 21, 100)));
    assert!(find(&d, "CaseFoldingDisabled", Severity::Info).is_none());
}

#[test]
fn rc28_comments_and_the_vanilla_apostrophe() {
    let text = fixture(FAMILY, "creaking.render_controllers.json");
    let document = loaded(parse(&text, &rp(1, 21, 100)));
    clean(document.diagnostics());
    assert!(
        find(
            document.diagnostics(),
            "CaseFoldingDisabled",
            Severity::Info
        )
        .is_none()
    );
    let eyes = &document.render_controllers["controller.render.creaking_eyes"];
    // Folding stops at the apostrophe of the comment's "there's".
    assert_eq!(
        eyes.materials[0].1.as_str(),
        "Array.materials[query.is_invisible ? 0 : 1]"
    );
    assert_eq!(eyes.textures[0].as_str(), "Texture.eyes");
    assert!(
        eyes.arrays
            .as_ref()
            .unwrap()
            .materials
            .contains_key("array.materials")
    );
    assert_eq!(eyes.geometry.as_str(), "geometry.default");
    assert_eq!(eyes.ignore_lighting, Some(true));
    assert_eq!(eyes.part_visibility[0].1, Molang::Bool(false));
    // The capitals the apostrophe kept are written as escapes the fold does not reach, so the
    // written file (which has no comment) reads back as the same value.
    let text = document.to_json().unwrap();
    assert!(text.contains(&esc("\"Texture.eyes\"")), "{text}");
    assert_eq!(
        bedrock_addon::families::render_controller::fold_case(&text),
        text
    );
    round_trip(&document, |t| parse(t, &rp(1, 21, 100)));
}

// ---- values and defaults ----

#[test]
fn rc17_color_partial() {
    let document = loaded(parse(&with(r#""color": {"r": 0.5}"#), &rp(1, 21, 100)));
    let c = test_of(&document);
    let written = c.color.as_ref().unwrap();
    assert_eq!(written.r, Some(Molang::Const(0.5)));
    assert_eq!((&written.g, &written.b, &written.a), (&None, &None, &None));
    // The channels the object leaves out are `this`, not the member's default.
    let [r, g, b, a] = c.color();
    assert_eq!(r, Molang::Const(0.5));
    for channel in [g, b, a] {
        let source = channel.source().unwrap();
        assert_eq!(source.as_str(), "this");
        assert_eq!(source.raw_version(), Some(RawVersion(13)));
    }
    // Write-back keeps the one channel the author wrote.
    assert!(
        document.to_json().unwrap().contains(
            r#""color": {
        "r": 0.5
      }"#
        ),
        "{}",
        document.to_json().unwrap()
    );
    round_trip(&document, |t| parse(t, &rp(1, 21, 100)));
    // A written `on_fire_color` loses its built-in channels the same way.
    let document = loaded(parse(&with(r#""on_fire_color": {}"#), &rp(1, 16, 0)));
    for channel in test_of(&document).on_fire_color() {
        let source = channel.source().unwrap();
        assert_eq!(source.as_str(), "this");
        assert_eq!(source.raw_version(), Some(RawVersion(0)));
    }
    round_trip(&document, |t| parse(t, &rp(1, 16, 0)));
}

#[test]
fn rc18_defaults() {
    let document = loaded(parse(&file(OK), &rp(1, 21, 100)));
    let c = &document.render_controllers["controller.render.ok"];
    assert!(c.materials.is_empty() && c.part_visibility.is_empty());
    assert_eq!(
        (
            &c.arrays,
            &c.color,
            &c.overlay_color,
            &c.on_fire_color,
            &c.is_hurt_color
        ),
        (&None, &None, &None, &None, &None)
    );
    assert_eq!(
        (&c.uv_anim, &c.light_color_multiplier, c.ignore_lighting),
        (&None, &None, None)
    );
    assert_eq!(
        (c.filter_lighting, c.rebuild_animation_matrices),
        (None, None)
    );

    for channel in c.color().into_iter().chain(c.overlay_color()) {
        let source = channel.source().unwrap();
        assert_eq!(source.as_str(), "this");
        assert_eq!(source.raw_version(), Some(RawVersion(13)));
    }
    let [r, g, b, a] = c.on_fire_color();
    assert_eq!((r, b), (Molang::Const(0.8), Molang::Const(0.0)));
    assert_eq!(
        expr(&g),
        "math.lerp(0.3f, 0.15f, math.pow(math.sin((query.on_fire_time / 20.0f) * 360.0f), 2.0f))"
    );
    assert_eq!(
        expr(&a),
        "math.hermite_blend(math.clamp((query.is_on_fire ? query.on_fire_time : (((query.on_fire_time < 5.0f) ? (5.0f - query.on_fire_time) : 0.0f))) / 5.0f, 0.0f, 1.0f)) * 0.7"
    );
    assert_eq!(g.source().unwrap().raw_version(), Some(RawVersion(13)));
    assert_eq!(c.is_hurt_color(), [1.0, 0.0, 0.0, 0.25].map(Molang::Const));
    assert_eq!(c.light_color_multiplier(), Molang::Const(1.0));
    assert!(!c.ignore_lighting() && !c.filter_lighting() && !c.rebuild_animation_matrices());
    let uv = c.uv_anim();
    assert_eq!(uv.offset, [0.0, 0.0].map(Molang::Const));
    assert_eq!(uv.scale, [1.0, 1.0].map(Molang::Const));
    // Nothing of that is written back.
    let text = document.to_json().unwrap();
    for member in [
        "color",
        "uv_anim",
        "lighting",
        "arrays",
        "materials",
        "part_visibility",
    ] {
        assert!(!text.contains(member), "{text}");
    }
}

#[test]
fn rc18_written_scalars_are_kept() {
    let document = loaded(parse(
        &with(
            r#""ignore_lighting": false, "rebuild_animation_matrices": true, "light_color_multiplier": 0.5, "arrays": {}"#,
        ),
        &rp(1, 21, 100),
    ));
    let c = test_of(&document);
    assert_eq!(c.ignore_lighting, Some(false));
    assert!(!c.ignore_lighting());
    assert_eq!(c.rebuild_animation_matrices, Some(true));
    assert!(c.rebuild_animation_matrices());
    assert_eq!(c.light_color_multiplier(), Molang::Const(0.5));
    assert_eq!(c.arrays, Some(Default::default()));
    let text = document.to_json().unwrap();
    assert!(text.contains(r#""ignore_lighting": false"#), "{text}");
    assert!(text.contains(r#""arrays": {}"#), "{text}");
    round_trip(&document, |t| parse(t, &rp(1, 21, 100)));
}

#[test]
fn rc19_part_visibility_types() {
    let document = loaded(parse(
        &with(r#""part_visibility": [{"a": 1}, {"b": false}, {"c": "query.is_baby"}]"#),
        &rp(1, 21, 100),
    ));
    clean(document.diagnostics());
    let list = &test_of(&document).part_visibility;
    assert_eq!(list[0], (BonePattern("a".to_owned()), Molang::Const(1.0)));
    assert_eq!(list[1], (BonePattern("b".to_owned()), Molang::Bool(false)));
    assert_eq!(expr(&list[2].1), "query.is_baby");
    round_trip(&document, |t| parse(t, &rp(1, 21, 100)));

    let d = rejected(parse(
        &with(r#""part_visibility": [{"a": {}}]"#),
        &rp(1, 21, 100),
    ));
    expect_prefix(&d, "InvalidType", Severity::Error, TYPE_ERROR);
}

#[test]
fn rc21_molang_version_from_pack() {
    for (cx, version) in [(rp(1, 16, 0), 0), (rp(1, 21, 100), 13), (rp(1, 20, 0), 8)] {
        // A file version far from the pack's never enters.
        let text = with(r#""light_color_multiplier": "1.0""#).replace("1.8.0", "1.21.0");
        let document = loaded(parse(&text, &cx));
        let c = test_of(&document);
        let version = Some(RawVersion(version));
        assert_eq!(c.geometry.raw_version(), version);
        assert_eq!(c.textures[0].raw_version(), version);
        assert_eq!(c.materials[0].1.raw_version(), version);
        assert_eq!(
            c.light_color_multiplier
                .as_ref()
                .unwrap()
                .source()
                .unwrap()
                .raw_version(),
            version
        );
    }
}

#[test]
fn rc25_duplicate_name_case() {
    let text = file(
        r#""controller.render.a": {"geometry": "geometry.first", "textures": ["texture.default"]},
           "controller.render.A": {"geometry": "geometry.second", "textures": ["texture.default"]}"#,
    );
    let document = loaded(parse(&text, &rp(1, 21, 100)));
    assert_eq!(document.render_controllers.len(), 1);
    assert_eq!(
        document.render_controllers["controller.render.a"]
            .geometry
            .as_str(),
        "geometry.second"
    );
}

#[test]
fn rc27_invalid_json() {
    let d = rejected(parse(
        r#"{"format_version": "1.8.0", "render_controllers": {"#,
        &rp(1, 21, 100),
    ));
    expect_prefix(
        &d,
        "MalformedInput",
        Severity::Error,
        "error parsing render_controllers/x.json:\n",
    );
    assert_eq!(d.len(), 1);
}

#[test]
fn rc33_empty_file() {
    let d = rejected(parse("", &rp(1, 21, 100)));
    expect_prefix(
        &d,
        "MalformedInput",
        Severity::Error,
        "error parsing render_controllers/x.json:\n",
    );
}

#[test]
fn rc29_undeclared_array_deviation() {
    let text = file(
        r#""controller.render.test": {"geometry": "Geometry.default", "textures": ["Array.skins[0]"]}"#,
    );
    let document = loaded(parse(&text, &rp(1, 21, 100)));
    let warning = find(
        document.diagnostics(),
        "UnresolvedReference",
        Severity::Info,
    )
    .unwrap();
    assert_eq!(
        warning.location.json_path.to_string(),
        "render_controllers/controller.render.test/textures"
    );
    assert!(
        warning.to_string().contains("array 'array.skins'"),
        "{warning}"
    );
    assert_eq!(
        count(
            document.diagnostics(),
            "UnresolvedReference",
            Severity::Info
        ),
        1
    );
    assert_eq!(document.render_controllers.len(), 1);

    // An array declared in another kind does not count; one declared in the member's own does.
    let wrong_kind = file(
        r#""controller.render.test": {"arrays": {"geometries": {"Array.skins": ["Geometry.a"]}},
            "geometry": "Array.skins[0]", "textures": ["Array.skins[0]"],
            "materials": [{"*": "Array.mats[0]"}]}"#,
    );
    let document = loaded(parse(&wrong_kind, &rp(1, 21, 100)));
    let at: Vec<String> = document
        .diagnostics()
        .iter()
        .filter(|f| f.code.name() == "UnresolvedReference")
        .map(|f| f.location.json_path.to_string())
        .collect();
    assert_eq!(
        at,
        [
            "render_controllers/controller.render.test/textures",
            "render_controllers/controller.render.test/materials"
        ]
    );
}

#[test]
fn rc31_empty_arrays_skip_the_whole_file() {
    for (member, text, at) in [
        (r#""textures": []"#, NO_ELEMENT, "textures"),
        (
            r#""textures": ["t"], "materials": []"#,
            NO_ELEMENT,
            "materials",
        ),
        (
            r#""textures": ["t"], "part_visibility": []"#,
            NO_ELEMENT,
            "part_visibility",
        ),
        (
            r#""textures": ["t"], "arrays": {"textures": {"Array.skins": []}}"#,
            NO_ELEMENT,
            // The key as the file spells it, not as the fold reads it.
            "arrays/textures/Array.skins",
        ),
    ] {
        let file = file(&format!(
            r#"{OK}, "controller.render.test": {{"geometry": "geometry.default", {member}}}"#
        ));
        let d = rejected(parse(&file, &rp(1, 21, 100)));
        expect_at(
            &d,
            "InvalidValue",
            Severity::Error,
            text,
            &format!("render_controllers/controller.render.test/{at}"),
        );
    }
    for (member, pattern) in [
        (r#""part_visibility": [{}]"#, "[a-zA-Z0-9_.:*]+"),
        (r#""materials": [{}]"#, "[a-zA-Z0-9_.:*]+"),
        (r#""arrays": {"textures": {}}"#, r"array\.[a-zA-Z0-9_.:-]+"),
    ] {
        let d = rejected(parse(&with(member), &rp(1, 21, 100)));
        expect(
            &d,
            "MissingRequiredField",
            Severity::Error,
            &format!("Required child {pattern} not found"),
        );
    }
}

#[test]
fn rc32_material_key_order() {
    let pattern = |c: &RenderController| -> Vec<(String, String)> {
        c.materials
            .iter()
            .map(|(p, s)| (p.as_str().to_owned(), s.as_str().to_owned()))
            .collect()
    };
    let one = |materials: &str| {
        file(&format!(
            r#""controller.render.test": {{"geometry": "geometry.default", "textures": ["texture.default"], "materials": {materials}}}"#
        ))
    };
    let together = loaded(parse(
        &one(r#"[{"saddle": "Material.a", "*": "Material.b"}]"#),
        &rp(1, 21, 100),
    ));
    // Sorted keys inside one object: `*` first, so `saddle` is applied last and wins.
    assert_eq!(
        pattern(test_of(&together)),
        [
            ("*".to_owned(), "material.b".to_owned()),
            ("saddle".to_owned(), "material.a".to_owned())
        ]
    );
    let apart = loaded(parse(
        &one(r#"[{"saddle": "Material.a"}, {"*": "Material.b"}]"#),
        &rp(1, 21, 100),
    ));
    assert_eq!(
        pattern(test_of(&apart)),
        [
            ("saddle".to_owned(), "material.a".to_owned()),
            ("*".to_owned(), "material.b".to_owned())
        ]
    );
    // Written one pattern per element, the order survives a round trip.
    round_trip(&together, |t| parse(t, &rp(1, 21, 100)));
    round_trip(&apart, |t| parse(t, &rp(1, 21, 100)));
}

#[test]
fn bone_patterns_become_the_game_s_regex() {
    assert_eq!(
        BonePattern("arm*.l".to_owned()).to_regex(),
        r"arm[a-zA-Z0-9_\.]*.l"
    );
    assert_eq!(BonePattern("head".to_owned()).to_regex(), "head");
}

#[test]
fn materials_bool_and_number_are_read_as_the_game_reads_them() {
    let document = loaded(parse(
        &file(
            r#""controller.render.test": {"geometry": "geometry.default", "textures": ["texture.default"], "materials": [{"a": true}, {"b": 2}]}"#,
        ),
        &super::common::unchecked(),
    ));
    let materials = &test_of(&document).materials;
    assert_eq!(materials[0].1.as_str(), "true");
    assert_eq!(materials[1].1.as_str(), "");
}

// ---- capitals the fold leaves, and what a write means to the game ----

/// The reviewer's probe: after a comment's apostrophe nothing is folded, so the controller key,
/// a bone pattern, an array name and the Molang identifiers keep their capitals.
const UNFOLDED: &str = "{\"format_version\": \"1.8.0\", \"render_controllers\": {\n// it's\n\"controller.render.Foo\": {\n\"arrays\": {\"textures\": {\"array.Skins\": [\"Texture.A\"]}},\n\"geometry\": \"Geometry.Default\",\n\"materials\": [{\"Head*\": \"Material.Default\"}],\n\"textures\": [\"array.Skins[Q.Variant]\", \"Q.Get_Name == 'Toast' ? Texture.T : Texture.A\"]}}}";

/// `text` with each ASCII capital as the JSON escape the game's fold does not reach.
fn esc(text: &str) -> String {
    let backslash = char::from(92_u8);
    text.chars()
        .map(|c| {
            if c.is_ascii_uppercase() {
                format!("{backslash}u{:04x}", u32::from(c))
            } else {
                c.to_string()
            }
        })
        .collect()
}

#[test]
fn capitals_the_fold_left_are_written_so_that_the_game_reads_them_again() {
    let document = loaded(parse(UNFOLDED, &rp(1, 21, 100)));
    let c = &document.render_controllers["controller.render.Foo"];
    assert_eq!(c.geometry.as_str(), "Geometry.Default");
    assert_eq!(c.materials[0].0.as_str(), "Head*");
    assert_eq!(c.materials[0].1.as_str(), "Material.Default");
    assert!(
        c.arrays
            .as_ref()
            .unwrap()
            .textures
            .contains_key("array.Skins")
    );
    assert_eq!(
        c.textures[1].as_str(),
        "Q.Get_Name == 'toast' ? Texture.T : Texture.A"
    );

    let text = document.to_json().unwrap();
    // No capital is left where the game's fold would lower it: the fold changes nothing.
    assert_eq!(
        bedrock_addon::families::render_controller::fold_case(&text),
        text,
        "{text}"
    );
    for written in [
        esc(r#""controller.render.Foo""#),
        esc(r#""Head*""#),
        esc(r#""array.Skins""#),
        esc(r#""Geometry.Default""#),
        // The literal's first apostrophe ends the comment's span, so the game folds `Toast`;
        // written, the literal is a span of its own again.
        format!(
            "{}'toast'{}",
            esc("Q.Get_Name == "),
            esc(" ? Texture.T : Texture.A")
        ),
    ] {
        assert!(text.contains(&written), "{written} not in {text}");
    }
    // Keys, bone patterns, array names and Molang identifiers read back as they were held.
    round_trip(&document, |t| parse(t, &rp(1, 21, 100)));
}

#[test]
fn text_the_fold_lowered_is_held_and_written_lower_case() {
    let folded = UNFOLDED.replace("// it's\n", "");
    let document = loaded(parse(&folded, &rp(1, 21, 100)));
    clean(document.diagnostics());
    let c = &document.render_controllers["controller.render.foo"];
    assert_eq!(c.materials[0].0.as_str(), "head*");
    assert!(
        c.arrays
            .as_ref()
            .unwrap()
            .textures
            .contains_key("array.skins")
    );
    assert_eq!(
        c.textures[1].as_str(),
        "q.get_name == 'Toast' ? texture.t : texture.a"
    );
    let text = document.to_json().unwrap();
    assert!(!text.contains("\\u"), "{text}");
    assert!(text.contains("'Toast'"), "{text}");
    round_trip(&document, |t| parse(t, &rp(1, 21, 100)));
}

#[test]
fn a_capital_written_as_an_escape_is_kept_by_the_fold_and_written_as_one() {
    let text = file(&esc(
        r#""controller.render.Foo": {"geometry": "Geometry.default", "textures": ["texture.default"], "part_visibility": [{"Arm": true}]}"#,
    ));
    let document = loaded(parse(&text, &rp(1, 21, 100)));
    let c = &document.render_controllers["controller.render.Foo"];
    assert_eq!(c.geometry.as_str(), "Geometry.default");
    assert_eq!(c.part_visibility[0].0.as_str(), "Arm");
    let written = document.to_json().unwrap();
    assert!(
        written.contains(&esc(r#""controller.render.Foo""#)),
        "{written}"
    );
    round_trip(&document, |t| parse(t, &rp(1, 21, 100)));
}

#[test]
fn an_odd_apostrophe_in_a_written_string_protects_what_follows_it() {
    // The texture path's apostrophe switches the game's fold off for the rest of the written
    // file too, so the capitals after it are written as they are and still read back.
    let text = "{\"format_version\": \"1.8.0\", \"render_controllers\": {\"controller.render.a\": {\"geometry\": \"geometry.default\", \"textures\": [\"textures/bob's_Hat\", \"Texture.B\"]}}}";
    let document = loaded(parse(text, &super::common::unchecked()));
    let c = &document.render_controllers["controller.render.a"];
    assert_eq!(c.textures[0].as_str(), "textures/bob's_Hat");
    assert_eq!(c.textures[1].as_str(), "Texture.B");
    let written = document.to_json().unwrap();
    assert!(
        written.contains("bob's_Hat") && written.contains("\"Texture.B\""),
        "{written}"
    );
    assert_eq!(
        bedrock_addon::families::render_controller::fold_case(&written),
        written
    );
    round_trip(&document, |t| parse(t, &super::common::unchecked()));
}

/// A controller key or an array name that kept capitals is registered by the game under
/// a name no lower-cased reference reaches; we say so every time, without an error beside it.
#[test]
fn a_name_that_kept_capitals_is_reported_as_unresolvable() {
    let document = loaded(parse(UNFOLDED, &rp(1, 21, 100)));
    let warnings: Vec<(String, String)> = document
        .diagnostics()
        .iter()
        .filter(|f| f.code.name() == "CaseFoldingDisabled" && f.severity == Severity::Info)
        .map(|f| (f.location.json_path.to_string(), f.code.to_string()))
        .collect();
    let text = "an apostrophe at line 2 disabled case folding; the game resolves no reference to this name";
    assert_eq!(
        warnings,
        [
            (
                "render_controllers/controller.render.Foo".to_owned(),
                text.to_owned()
            ),
            (
                "render_controllers/controller.render.Foo/arrays/textures/array.Skins".to_owned(),
                text.to_owned()
            ),
        ]
    );
    // A capital from an escape has no apostrophe to name.
    let escaped = file(&esc(
        r#""controller.render.Foo": {"geometry": "geometry.default", "textures": ["texture.default"]}"#,
    ));
    let document = loaded(parse(&escaped, &rp(1, 21, 100)));
    let warning = find(
        document.diagnostics(),
        "CaseFoldingDisabled",
        Severity::Info,
    )
    .unwrap();
    assert_eq!(
        warning.code.to_string(),
        "a capital letter the game's case folding does not reach; the game resolves no reference to this name"
    );
    // A bone pattern or an expression with capitals is no name others refer to: no warning.
    let pattern = file(&esc(
        r#""controller.render.a": {"geometry": "Geometry.default", "textures": ["texture.default"], "materials": [{"Head": "material.default"}]}"#,
    ));
    let document = loaded(parse(&pattern, &rp(1, 21, 100)));
    assert!(
        find(
            document.diagnostics(),
            "CaseFoldingDisabled",
            Severity::Info
        )
        .is_none()
    );
}

/// Two spellings the game normalises when it reads them are written normalised. A
/// partial `"1.10"` is 1.10.0 to the schema's version node, and a `materials`
/// bool is the text `true` / `false` to the controller's parser.
/// The value read back is the value written.
#[test]
fn a_partial_version_and_a_materials_bool_are_written_as_the_game_reads_them_deviation() {
    let text = file(
        r#""controller.render.test": {"geometry": "geometry.default", "textures": ["texture.default"], "materials": [{"*": true}, {"arm": false}]}"#,
    )
    .replace("\"1.8.0\"", "\"1.10\"");
    let document = loaded(parse(&text, &rp(1, 21, 100)));
    clean(document.diagnostics());
    assert_eq!(document.format_version, FormatVersion::new(1, 10, 0));
    let written = document.to_json().unwrap();
    assert!(
        written.contains(r#""format_version": "1.10.0""#),
        "{written}"
    );
    assert!(written.contains(r#""*": "true""#), "{written}");
    assert!(written.contains(r#""arm": "false""#), "{written}");
    round_trip(&document, |t| parse(t, &rp(1, 21, 100)));
    // A `part_visibility` bool, which the game keeps a constant, stays a bool.
    let visible = loaded(parse(
        &with(r#""part_visibility": [{"*": true}]"#),
        &rp(1, 21, 100),
    ));
    assert!(visible.to_json().unwrap().contains(r#""*": true"#));
}

/// An array reference is compared with the declared names ignoring case, as the game's Molang
/// reads identifiers: after an apostrophe both sides can keep capitals of their own.
#[test]
fn an_array_reference_matches_its_declaration_ignoring_case() {
    let text = "{\"format_version\": \"1.8.0\", \"render_controllers\": {\n// it's\n\"controller.render.a\": {\n\"arrays\": {\"textures\": {\"array.Skins\": [\"texture.a\"]}},\n\"geometry\": \"geometry.default\",\n\"textures\": [\"Array.SKINS[0]\", \"array.Other[0]\"]}}}";
    let document = loaded(parse(text, &rp(1, 21, 100)));
    let unresolved: Vec<String> = document
        .diagnostics()
        .iter()
        .filter(|f| f.code.name() == "UnresolvedReference")
        .map(ToString::to_string)
        .collect();
    assert_eq!(unresolved.len(), 1, "{unresolved:?}");
    assert!(
        unresolved[0].contains("array 'array.other'"),
        "{unresolved:?}"
    );
}

/// Rule 26 is our check on a controller that loaded: a controller the game drops raises no
/// array finding, whatever its expressions index.
#[test]
fn rc29_a_dropped_controller_raises_no_array_finding() {
    let text = file(&format!(
        r#""controller.render.test": {{"geometry": "array.missing[0]", "materials": [{{"*": "array.none[0]"}}]}}, {OK}"#
    ));
    let document = loaded(parse(&text, &rp(1, 21, 100)));
    assert_eq!(
        count(
            document.diagnostics(),
            "UnresolvedReference",
            Severity::Info
        ),
        0,
        "{:?}",
        lines(document.diagnostics())
    );
    assert_eq!(
        count(
            document.diagnostics(),
            "MissingRequiredField",
            Severity::Error
        ),
        1
    );
}

/// A controller the game drops is kept apart in the file and written back as its author
/// wrote it, so a file whose controllers were all dropped still writes text this loader reads,
/// to the same value and the same findings.
#[test]
fn a_file_whose_controllers_were_all_dropped_is_written_back_whole() {
    let text = file(
        r#""controller.render.b": {"geometry": "geometry.b", "color": {"r": 1}},
           "controller.render.a": {"geometry": "geometry.a", "materials": [{"*": "material.default"}]}"#,
    );
    let document = loaded(parse(&text, &rp(1, 21, 100)));
    assert!(document.render_controllers.is_empty());
    assert_eq!(
        document.dropped.keys().collect::<Vec<_>>(),
        ["controller.render.a", "controller.render.b"]
    );
    assert_eq!(
        document.dropped["controller.render.a"].geometry.as_str(),
        "geometry.a"
    );
    assert!(document.get("controller.render.a").is_none());
    let errors = |d: &bedrock_addon::diagnostic::Diagnostics| -> Vec<String> {
        d.iter()
            .filter(|f| f.severity == Severity::Error)
            .map(|f| f.location.json_path.to_string())
            .collect()
    };
    let written = document.to_json().unwrap();
    let again = loaded(parse(&written, &rp(1, 21, 100)));
    assert_eq!(again.value(), document.value(), "{written}");
    assert_eq!(errors(again.diagnostics()), errors(document.diagnostics()));
    assert_eq!(errors(again.diagnostics()).len(), 2);
    assert_eq!(again.to_json().unwrap(), written);
}

#[test]
fn a_dropped_controller_beside_a_loaded_one_is_written_back_in_place() {
    let text = file(&format!(
        r#""controller.render.zz": {{"geometry": "geometry.z"}}, {OK}, "controller.render.aa": {{"geometry": "geometry.a"}}"#
    ));
    let document = loaded(parse(&text, &rp(1, 21, 100)));
    assert_eq!(document.render_controllers.len(), 1);
    assert_eq!(document.dropped.len(), 2);
    let written = document.to_json().unwrap();
    // Sorted keys, loaded and dropped together.
    let at = |key: &str| written.find(key).unwrap();
    assert!(at("controller.render.aa") < at("controller.render.ok"));
    assert!(at("controller.render.ok") < at("controller.render.zz"));
    let again = loaded(parse(&written, &rp(1, 21, 100)));
    assert_eq!(again.value(), document.value());
}

/// A `materials` number is the empty text to the game; written as a string it would fail the
/// whole file, so it is written as a number again.
#[test]
fn a_numeric_material_is_written_as_a_number() {
    let text = file(&format!(
        r#""controller.render.test": {{"geometry": "geometry.default", "textures": ["texture.default"], "materials": [{{"*": 7}}]}}, {OK}"#
    ));
    for cx in [rp(1, 21, 100), rp(1, 16, 0), super::common::unchecked()] {
        let document = loaded(parse(&text, &cx));
        let written = document.to_json().unwrap();
        assert!(written.contains(r#""*": 0"#), "{written}");
        let again = loaded(parse(&written, &cx));
        assert_eq!(again.value(), document.value());
        assert_eq!(again.to_json().unwrap(), written);
    }
}

#[test]
fn a_file_without_any_controller_is_refused_with_its_path() {
    let error = RenderControllerFile::new().to_json().unwrap_err();
    let bedrock_addon::json::EncodeError::NotRepresentable { at, .. } = error else {
        panic!("{error:?}");
    };
    assert_eq!(at.to_string(), "render_controllers");
}

/// A file built by hand may hold one key as a loaded and as a dropped controller; a JSON object
/// has the key once, so one of the two would be lost.
#[test]
fn a_key_both_loaded_and_dropped_is_refused_with_its_path() {
    let controller = |geometry: &str| {
        RenderController::new(
            bedrock_addon::molang::MolangSource::string(geometry, 13),
            vec![bedrock_addon::molang::MolangSource::string(
                "texture.default",
                13,
            )],
        )
    };
    let mut file = RenderControllerFile::new();
    for key in ["controller.render.a", "controller.render.both"] {
        file.render_controllers
            .insert(key.to_owned(), controller("geometry.kept"));
    }
    file.dropped.insert(
        "controller.render.z".to_owned(),
        controller("geometry.dropped"),
    );
    // Different keys: written.
    file.to_json().unwrap();
    file.dropped.insert(
        "controller.render.both".to_owned(),
        controller("geometry.dropped"),
    );
    let error = file.to_json().unwrap_err();
    let bedrock_addon::json::EncodeError::NotRepresentable { at, .. } = error else {
        panic!("{error:?}");
    };
    assert_eq!(at.to_string(), "render_controllers/controller.render.both");
}

/// A Molang member that is a number beyond `f32` is the
/// largest finite float of its sign, with a warning at the number.
#[test]
fn a_molang_number_beyond_f32_is_read_as_the_largest_finite_float_deviation() {
    let text = with(r#""light_color_multiplier": 1e39, "color": {"r": -1e999, "g": 1}"#);
    let document = loaded(parse(&text, &rp(1, 21, 100)));
    let c = test_of(&document);
    assert_eq!(c.light_color_multiplier, Some(Molang::Const(f32::MAX)));
    assert_eq!(c.color.as_ref().unwrap().r, Some(Molang::Const(f32::MIN)));
    // In the order the reader takes the members.
    let warnings: Vec<String> = document
        .diagnostics()
        .iter()
        .filter(|f| f.code.name() == "InvalidValue" && f.severity == Severity::Info)
        .map(|f| {
            let span = f.location.span.expect("the number's token");
            let (from, to) = (span.offset as usize, (span.offset + span.len) as usize);
            format!("{} {}", f.location.json_path, &text[from..to])
        })
        .collect();
    assert_eq!(
        warnings,
        [
            "render_controllers/controller.render.test/color/r -1e999",
            "render_controllers/controller.render.test/light_color_multiplier 1e39",
        ],
        "{}",
        lines(document.diagnostics()).join("\n")
    );
    round_trip(&document, |text| parse(text, &rp(1, 21, 100)));
    let written = document.to_json().unwrap();
    let again = loaded(parse(&written, &rp(1, 21, 100)));
    assert!(find(again.diagnostics(), "InvalidValue", Severity::Info).is_none());
}

/// The schema takes a number as a `materials` value and the controller parser reads it
/// as the empty text (RC-S.4), which the file is written with as `0`: told at the member, as
/// the file spells its key.
#[test]
fn a_materials_number_is_told_at_the_member_as_the_file_spells_it() {
    use bedrock_addon::load::LoadContext;
    let text = file(
        r#""controller.render.test": {"geometry": "Geometry.default", "materials": [{"*": "Material.default"}, {"bodyLayer_*": 7}], "textures": ["Texture.default"]}"#,
    );
    // Without the Molang check the controller loads (with it, the empty text fails: rc41).
    let parsed = parse(&text, &LoadContext::detached());
    let document = parsed.expect("the file loads");
    let told: Vec<(String, Severity, String)> = document
        .diagnostics()
        .iter()
        .filter(|f| f.code.name() == "InvalidType")
        .map(|f| {
            (
                f.code.name().to_owned(),
                f.severity,
                f.location.json_path.to_string(),
            )
        })
        .collect();
    assert_eq!(
        told,
        [(
            "InvalidType".to_owned(),
            Severity::Info,
            "render_controllers/controller.render.test/materials/1/bodyLayer_*".to_owned()
        )]
    );
}

/// the members of an object are validated in sorted
/// key order and validation goes on after a failing member, so one file tells several errors,
/// in key order whatever order the file wrote them in.
#[test]
fn rc_v1_two_schema_errors_are_told_in_sorted_key_order() {
    // Written: an unknown member `zzz`, a `rebuild_animation_matrices` of the wrong type, an
    // unknown member `aaa`; and the failing controller behind a valid one of a later name.
    let text = file(
        r#""controller.render.zebra": {"geometry": "Geometry.default", "textures": ["Texture.default"]},
           "controller.render.ant": {"zzz": 1, "geometry": "Geometry.default", "rebuild_animation_matrices": 3, "textures": ["Texture.default"], "aaa": 1}"#,
    );
    let d = rejected(parse(&text, &rp(1, 21, 100)));
    let errors: Vec<(&str, String)> = d
        .iter()
        .filter(|f| f.severity >= Severity::Error && f.code.name() != "DocumentRejected")
        .map(|f| (f.code.name(), f.location.json_path.to_string()))
        .collect();
    assert_eq!(
        errors,
        [
            (
                "UnrecognizedField",
                "render_controllers/controller.render.ant/aaa".to_owned()
            ),
            (
                "InvalidType",
                "render_controllers/controller.render.ant/rebuild_animation_matrices".to_owned()
            ),
            (
                "UnrecognizedField",
                "render_controllers/controller.render.ant/zzz".to_owned()
            ),
        ],
        "{:?}",
        lines(&d)
    );
    expect_at(
        &d,
        "UnrecognizedField",
        Severity::Error,
        "child 'aaa' not valid here.",
        "render_controllers/controller.render.ant/aaa",
    );
}
