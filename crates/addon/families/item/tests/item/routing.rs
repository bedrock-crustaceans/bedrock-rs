//! Format routing, the legacy format and versions as written.

use bedrock_addon_core::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::molang::{MolangVersion, RawVersion, format_molang_version};
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput};
use bedrock_addon_family_item::history::Any;
use bedrock_addon_family_item::{Item, ItemFormat};

use super::common::*;

/// The struct a document is parsed by, as written.
fn written_as(json: &str, cx: &bedrock_addon_core::load::LoadContext) -> Any {
    match Item::parse_as_written(&PackPath::from_static("items/x.json"), json.as_bytes(), cx) {
        Ok(document) => document.into_value(),
        Err(rejected) => panic!("rejected: {:#?}", rejected.diagnostics),
    }
}

/// The sample pack's `behavior_pack/items/<name>.json`, from the corpus (never committed).
fn sample(name: &str) -> Option<String> {
    let root = std::env::var_os("BEDROCK_ADDON_CORPUS")?;
    let path = std::path::Path::new(&root)
        .join("bedrock-samples/behavior_pack/items")
        .join(format!("{name}.json"));
    std::fs::read_to_string(path).ok()
}

#[test]
fn vanilla_legacy_items_reach_latest_unconverted() {
    let names = [
        "camera",
        "suspicious_stew",
        "beetroot_seeds",
        "chorus_fruit",
        "glow_berries",
    ];
    let Some(texts) = names.iter().map(|n| sample(n)).collect::<Option<Vec<_>>>() else {
        eprintln!("BEDROCK_ADDON_CORPUS unset: i00 skipped");
        return;
    };
    for (name, text) in names.iter().zip(&texts) {
        let item = loads(text, &allowlisted());
        assert!(item.components.is_empty(), "{name}: components");
        assert_eq!(item.legacy.format, ItemFormat::Legacy, "{name}");
        assert!(item.legacy.format_item.is_some(), "{name}: format_item");
        assert!(
            above_info(item.diagnostics()).is_empty(),
            "{name}: {:?}",
            item.diagnostics()
        );
        // a third-party pack loads it the same way: no gate on the legacy format
        let third = loads(text, &third_party());
        assert_eq!(*third.value(), *item.value(), "{name}: third-party");
    }
    let camera = loads(&texts[0], &allowlisted());
    let format = camera.legacy.format_item.as_ref().unwrap();
    assert_eq!(format.use_duration, 100_000);
    assert_eq!(format.block.as_deref(), Some("minecraft:camera"));
    let lens = format.camera.as_ref().unwrap();
    assert!((lens.black_bars_screen_ratio - 0.08).abs() < 1e-6);
    assert!((lens.picture_duration - 1.0).abs() < 1e-6);
    let stew = loads(&texts[1], &allowlisted());
    let food = stew
        .legacy
        .format_item
        .as_ref()
        .unwrap()
        .food
        .as_ref()
        .unwrap();
    assert_eq!(food.nutrition, 6);
    assert!((food.saturation() - 0.6).abs() < 1e-6);
    assert_eq!(
        food.on_use_action.as_deref(),
        Some("suspicious_stew_effect")
    );
    assert!(food.can_always_eat);
    let chorus = loads(&texts[3], &allowlisted());
    let food = chorus
        .legacy
        .format_item
        .as_ref()
        .unwrap()
        .food
        .as_ref()
        .unwrap();
    assert_eq!(food.cooldown_time, 20);
    assert_eq!(food.cooldown_type.as_deref(), Some("chorusfruit"));
    let berries = loads(&texts[4], &allowlisted());
    let format = berries.legacy.format_item.as_ref().unwrap();
    // `is_meat` is no legacy key: kept for write-back, no finding
    assert!(format.food.as_ref().unwrap().extra.get("is_meat").is_some());
    let seed = format.seed.as_ref().unwrap();
    assert_eq!(seed.crop_result.as_deref(), Some("cave_vines"));
    assert_eq!(seed.plant_at_face.as_deref(), Some("DOWN"));
    assert!(seed.plant_at_any_solid_surface);
}

#[test]
fn versions_after_1_16_100_and_before_1_17_0_are_the_legacy_format() {
    for version in ["1.16.99", "1.16.101", "1.16.999"] {
        let json = item(version, "minecraft:x", "");
        assert!(
            matches!(written_as(&json, &allowlisted()), Any::V1_10_0(_)),
            "{version}"
        );
    }
    // 1.16.101 is not fixed by the 1.18 fixers: the name stays as written
    let json = item(
        "1.16.101",
        "minecraft:x",
        r#""minecraft:food": {"nutrition": 1, "saturation_modifier": "good"}"#,
    );
    let Any::V1_10_0(legacy) = written_as(&json, &allowlisted()) else {
        panic!("not the legacy format");
    };
    let food = legacy.components.food.unwrap();
    assert_eq!(food.saturation_modifier, Some(Value::from("good")));
}

#[test]
fn component_documents_route_by_version_and_components() {
    let stack = r#""minecraft:max_stack_size": 16"#;
    for version in ["1.16.100", "1.17.0"] {
        let json = item(version, "minecraft:x", stack);
        assert!(
            matches!(written_as(&json, &allowlisted()), Any::V1_16_100(_)),
            "{version}"
        );
    }
    for components in ["", r#""minecraft:foil": true"#] {
        let json = item("1.21.0", "demo:x", components);
        assert!(
            matches!(written_as(&json, &third_party()), Any::V1_10_0(_)),
            "1.21.0 with {components:?}"
        );
    }
    let json = item("1.21.0", "demo:x", r#""tag:demo:x": {}"#);
    assert!(matches!(
        written_as(&json, &third_party()),
        Any::V1_20_80(_)
    ));
}

/// A JSON item without `format_version` is refused.
/// `parseDocument` finds no version (`fromString("")` gives none), logs "document's version not
/// found or not readable", and `upgradeJson` adds "Item is an invalid json object."; the item
/// is not loaded. (Was the legacy item format at 0.0.0 with a warning.)
#[test]
fn a_json_item_without_a_version_is_refused() {
    let json = r#"{"minecraft:item": {"description": {"identifier": "demo:x"}, "components": {"minecraft:max_stack_size": 16}}}"#;
    let d = rejects(json, &third_party());
    let errors: Vec<String> = d
        .iter()
        .filter(|f| f.severity == Severity::Error)
        .map(|f| f.code.to_string())
        .collect();
    assert_eq!(
        errors,
        [
            "document's version not found or not readable",
            "Item is an invalid json object."
        ],
        "{d:#?}"
    );
    assert!(has(&d, |c| matches!(
        c,
        DiagnosticCode::MissingFormatVersion { .. }
    )));
}

#[test]
fn two_part_versions_are_read_as_x_y_0() {
    for (written, expected) in [
        ("1.10", FormatVersion::new(1, 10, 0)),
        ("1.16", FormatVersion::new(1, 16, 0)),
    ] {
        let document = loads(&item(written, "demo:x", ""), &third_party());
        assert_eq!(document.version().written, Some(expected), "{written}");
    }
    // an unparsable string is rejected; the core envelope reads it as a "Malformed
    // format version", where the fixture names UnsupportedVersion
    let d = rejects(&item("one.two", "demo:x", ""), &third_party());
    assert!(
        has(&d, |c| matches!(
            c,
            DiagnosticCode::MalformedFormatVersion { .. }
        )),
        "{d:#?}"
    );
}

/// The seed warning (`minecraft:seed has been deprecated…`) in `d`.
fn seed_warned(d: &bedrock_addon_core::diagnostic::Diagnostics) -> bool {
    has_text(d, "minecraft:seed has been deprecated")
}

#[test]
fn the_seed_warning_is_above_1_16_100() {
    let seed = r#""minecraft:seed": {"crop_result": "wheat"}"#;
    let at_100 = loads(&item("1.16.100", "demo:x", seed), &third_party());
    assert!(!seed_warned(at_100.diagnostics()));
    let at_101 = loads(&item("1.16.101", "demo:x", seed), &third_party());
    assert!(seed_warned(at_101.diagnostics()));
    let format = at_101.legacy.format_item.as_ref().unwrap();
    assert_eq!(
        format.seed.as_ref().unwrap().crop_result.as_deref(),
        Some("wheat")
    );
    let components = format!(r#"{seed}, "minecraft:max_stack_size": 16"#);
    let modern = loads(&item("1.21.0", "demo:x", &components), &third_party());
    assert!(seed_warned(modern.diagnostics()));
    assert_eq!(modern.legacy.format, ItemFormat::DataDriven);
    assert!(
        modern.legacy.format_item.is_none(),
        "nothing reads the seed"
    );
}

#[test]
fn the_molang_version_is_the_format_version_window() {
    let window = |version: Option<FormatVersion>| {
        format_molang_version(&FormatVersionInput {
            version,
            beta_literal: false,
            prerelease: false,
        })
    };
    // "1.10" is 1.10.0, a missing version 0.0.0 (the loader's effective version)
    let ten = loads(&item("1.10", "demo:x", ""), &third_party());
    assert_eq!(
        window(Some(ten.version().effective)),
        RawVersion::from(MolangVersion::V0)
    );
    assert_eq!(
        window(Some(FormatVersion::new(1, 20, 50))),
        RawVersion::from(MolangVersion::V11)
    );
    assert_eq!(
        window(Some(FormatVersion::new(0, 0, 0))),
        RawVersion::from(MolangVersion::V0)
    );
    assert_ne!(
        window(Some(FormatVersion::new(1, 10, 0))),
        RawVersion::from(MolangVersion::Invalid)
    );
}

/// The error texts of a refused load, in order.
fn error_texts(d: &bedrock_addon_core::diagnostic::Diagnostics) -> Vec<String> {
    d.iter()
        .filter(|f| f.severity == Severity::Error)
        .map(|f| f.code.to_string())
        .collect()
}

/// The JSON item path (`upgradeJson`, `parseDocument`) has its own texts. A syntax error is
/// reported alone (the generic parse line is not logged on this path); a root that is no
/// object is reported, then the invalid-object line; an unreadable version likewise; a
/// missing or non-object `minecraft:item` has its own line.
#[test]
fn the_json_item_path_says_what_upgrade_json_says() {
    let invalid = "Item is an invalid json object.";
    let cases: [(&str, &[&str]); 5] = [
        (
            r#"{"format_version": "1.26.50", "minecraft:item": {"#,
            &[invalid],
        ),
        (
            "[1, 2]",
            &[
                "failed to parse file into a json document, root value needs to be a json object",
                invalid,
            ],
        ),
        (
            r#"{"format_version": "abc", "minecraft:item": {"description": {"identifier": "demo:x"}, "components": {"minecraft:icon": "x"}}}"#,
            &["document's version not found or not readable", invalid],
        ),
        (
            r#"{"format_version": "1.26.50"}"#,
            &[
                r#"'minecraft:item' should be a json object - it has to start with an "{" and end with a "}""#,
            ],
        ),
        (
            r#"{"format_version": "1.26.50", "minecraft:item": 5}"#,
            &[
                r#"'minecraft:item' should be a json object - it has to start with an "{" and end with a "}""#,
            ],
        ),
    ];
    for (text, expected) in cases {
        assert_eq!(
            error_texts(&rejects(text, &third_party())),
            expected,
            "{text}"
        );
    }
    // where each is told: the envelope's texts at the root, an unreadable version at
    // `format_version`, a payload that is no object at `minecraft:item`
    let places: [(&str, &[&str]); 5] = [
        (
            r#"{"format_version": "1.26.50", "minecraft:item": {"#,
            &[""],
        ),
        ("[1, 2]", &["", ""]),
        (
            r#"{"format_version": "abc", "minecraft:item": {"description": {"identifier": "demo:x"}, "components": {"minecraft:icon": "x"}}}"#,
            &["format_version", ""],
        ),
        (r#"{"format_version": "1.26.50"}"#, &[""]),
        (
            r#"{"format_version": "1.26.50", "minecraft:item": 5}"#,
            &["minecraft:item"],
        ),
    ];
    for (text, expected) in places {
        assert_eq!(
            error_paths(&rejects(text, &third_party())),
            expected,
            "{text}"
        );
    }
}

/// The paths of the errors of a refused load, in order.
fn error_paths(d: &bedrock_addon_core::diagnostic::Diagnostics) -> Vec<String> {
    d.iter()
        .filter(|f| f.severity == Severity::Error)
        .map(|f| f.location.json_path.to_string())
        .collect()
}

/// A JSON item above `CurrentGameSemVersion` (1.26.60) is refused by
/// `validateParsedVersion` with its version text; an MCB item by slice selection with another.
#[test]
fn an_item_above_the_engine_is_refused_with_the_text_of_its_path() {
    let text = item("1.26.61", "demo:x", r#""minecraft:icon": "x""#);
    let refused = rejects(&text, &third_party());
    assert_eq!(
        error_texts(&refused),
        ["document's version is out of handled range"]
    );
    assert_eq!(error_paths(&refused), ["format_version"]);
    let mut mcb = third_party();
    mcb.source = bedrock_addon_core::load::DocumentSource::Mcb;
    assert_eq!(
        error_texts(&rejects(&text, &mcb)),
        ["unsupported version: no parser available for version 1.26.61"]
    );
    // the engine's own version loads
    loads(
        &item("1.26.60", "demo:x", r#""minecraft:icon": "x""#),
        &third_party(),
    );
}
