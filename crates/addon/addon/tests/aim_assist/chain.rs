//! The aim-assist chains, the step from 1.21.50, the history path and write-back (the Value
//! level).

use bedrock_addon::document::Content;
use bedrock_addon::families::aim_assist::AimAssistPreset;
use bedrock_addon::families::aim_assist::history::{
    self, Any, CategoriesAny, CategoriesFamily, PresetFamily,
};
use bedrock_addon::json::Value;
use bedrock_addon::load::LoadContext;
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::{ENGINE_VERSION, FormatVersion, FormatVersionInput};
use bedrock_addon::versioning::testing::{Checks, Run, step_output};
use bedrock_addon::versioning::{
    EnvelopeCheck, Family, MissingVersion, RangeModel, Selection, VersionChain, check_chain, select,
};

use super::common::*;

/// The JSON at `path` (`/`-separated) of `value`.
fn at<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('/')
        .try_fold(value, |v, key| v.as_object().and_then(|o| o.get(key)))
}

/// The strings of the array at `path`.
fn list(value: &Value, path: &str) -> Option<Vec<String>> {
    Some(
        at(value, path)?
            .as_array()?
            .iter()
            .filter_map(|v| Some(v.as_string()?.as_str().to_owned()))
            .collect(),
    )
}

/// The whole document the step reads: a 1.21.50 preset with `exclusion_list` set to
/// `entries`, when there are any.
fn old_preset(entries: Option<&[&str]>) -> Value {
    let list = entries.map_or(String::new(), |entries| {
        let list: Vec<String> = entries.iter().map(|e| format!("\"{e}\"")).collect();
        format!(r#""exclusion_list": [{}], "#, list.join(","))
    });
    facet_json::from_str(&format!(
        r#"{{"format_version": "1.21.50", "minecraft:aim_assist_preset": {{{list}"identifier": "test:p", "liquid_targeting_list": ["minecraft:bucket"], "item_settings": {{"minecraft:iron_sword": "test:melee"}}, "default_item_settings": "test:default", "hand_settings": "test:empty_hand"}}}}"#
    ))
    .unwrap()
}

/// The payload and findings of the step from 1.21.50 on `input`.
fn step(input: &Value) -> bedrock_addon::versioning::testing::StepOutput {
    let checks = Checks {
        identifier: "test:p",
        ..Checks::default()
    };
    step_output::<history::v1_21_50::AimAssistPreset>(input, Run::default(), &checks)
        .expect("the step runs")
}

fn selects<F: Family>(version: &str) -> Selection {
    select::<F>(&FormatVersionInput::parse(version).expect("a version"))
}

#[test]
fn the_preset_chain_is_1_21_50_then_1_21_120() {
    let versions = <Any as VersionChain<PresetFamily>>::VERSIONS;
    assert_eq!(
        versions,
        [
            FormatVersion::new(1, 21, 50),
            FormatVersion::new(1, 21, 120)
        ]
    );
}

#[test]
fn the_step_moves_the_exclusion_list_into_blocks_and_copies_it_to_entities() {
    const SETTINGS: &str = "minecraft:aim_assist_preset/exclusion_settings";
    let entries = ["minecraft:stone", "minecraft:PIG<e>", "minecraft:stone"];
    let out = step(&old_preset(Some(&entries)));
    // The same source list in both, in order, duplicates kept, the text as written.
    let expected = Some(entries.map(str::to_owned).to_vec());
    assert_eq!(list(&out.payload, &format!("{SETTINGS}/blocks")), expected);
    assert_eq!(
        list(&out.payload, &format!("{SETTINGS}/entities")),
        expected
    );
    assert_eq!(
        at(&out.payload, "minecraft:aim_assist_preset/exclusion_list"),
        None
    );
    // The two lists the old format had no spelling for arrive empty.
    for member in ["block_tags", "entity_type_families"] {
        assert_eq!(
            list(&out.payload, &format!("{SETTINGS}/{member}")),
            Some(Vec::new()),
            "{member}"
        );
    }
    assert!(out.diagnostics.is_empty(), "{:#?}", out.diagnostics);
    // Without the list: an empty entity list, not an absent one.
    let out = step(&old_preset(None));
    assert_eq!(
        list(&out.payload, &format!("{SETTINGS}/entities")),
        Some(Vec::new())
    );
}

#[test]
fn the_chains_are_the_plans() {
    const HAS_BETA: bool = <Any as VersionChain<PresetFamily>>::HAS_BETA;

    check_chain::<PresetFamily>().expect("presets");
    check_chain::<CategoriesFamily>().expect("categories");
    assert_eq!(
        <CategoriesAny as VersionChain<CategoriesFamily>>::VERSIONS,
        [FormatVersion::new(1, 21, 50)]
    );
    assert_eq!(PresetFamily::PAYLOAD_KEY, "minecraft:aim_assist_preset");
    assert_eq!(
        CategoriesFamily::PAYLOAD_KEY,
        "minecraft:aim_assist_categories"
    );
    assert_eq!(PresetFamily::DIRECTORIES, ["aim_assist/presets"]);
    assert_eq!(CategoriesFamily::DIRECTORIES, ["aim_assist/categories"]);
    for (floor, model, envelope, missing) in [
        (
            PresetFamily::FLOOR,
            PresetFamily::RANGE_MODEL,
            PresetFamily::ENVELOPE,
            PresetFamily::MISSING_VERSION,
        ),
        (
            CategoriesFamily::FLOOR,
            CategoriesFamily::RANGE_MODEL,
            CategoriesFamily::ENVELOPE,
            CategoriesFamily::MISSING_VERSION,
        ),
    ] {
        assert_eq!(floor, FormatVersion::new(1, 21, 50));
        assert_eq!(model, RangeModel::ClosedToEngine);
        assert_eq!(envelope, EnvelopeCheck::Never);
        assert_eq!(missing, MissingVersion::Reject);
    }
    let has_beta = HAS_BETA;
    assert!(!has_beta, "Beta = Infallible");
}

#[test]
fn each_version_selects_the_struct_whose_range_holds_it() {
    assert_eq!(selects::<PresetFamily>("1.21.40"), Selection::BelowFloor);
    assert_eq!(selects::<PresetFamily>("1.21.50"), Selection::Struct(0));
    assert_eq!(selects::<PresetFamily>("1.21.110"), Selection::Struct(0));
    assert_eq!(selects::<PresetFamily>("1.21.120"), Selection::Struct(1));
    assert_eq!(
        selects::<PresetFamily>(&ENGINE_VERSION.to_string()),
        Selection::Struct(1)
    );
    assert_eq!(selects::<PresetFamily>("1.26.61"), Selection::Unsupported);
    assert_eq!(
        selects::<CategoriesFamily>("1.21.40"),
        Selection::BelowFloor
    );
    assert_eq!(selects::<CategoriesFamily>("1.21.50"), Selection::Struct(0));
    assert_eq!(
        selects::<CategoriesFamily>("1.21.120"),
        Selection::Struct(0)
    );
    assert_eq!(
        selects::<CategoriesFamily>(&ENGINE_VERSION.to_string()),
        Selection::Struct(0)
    );
    assert_eq!(
        selects::<CategoriesFamily>("1.26.61"),
        Selection::Unsupported
    );
}

#[test]
fn the_history_path_writes_as_read_and_upgrades_to_the_same_preset() {
    let text = preset_json(
        "1.21.50",
        r#""identifier": "test:p", "exclusion_list": ["minecraft:stone"]"#,
    );
    let path = PackPath::from_static("aim_assist/presets/p.json");
    let cx = LoadContext::detached();
    let written = AimAssistPreset::parse_as_written(&path, text.as_bytes(), &cx).unwrap();
    assert_eq!(written.version().parsed_as, FormatVersion::new(1, 21, 50));
    let Any::V1_21_50(old) = written.value() else {
        panic!("v1_21_50")
    };
    assert_eq!(old.exclusion_list.as_ref().unwrap()[0].0, "minecraft:stone");
    // A history value is written at the version it was read at, and reads back the same.
    let later = preset_json(
        "1.21.100",
        r#""identifier": "test:p", "exclusion_list": ["minecraft:stone"]"#,
    );
    let read = AimAssistPreset::parse_as_written(&path, later.as_bytes(), &cx).unwrap();
    let rewritten = read.to_json().unwrap();
    assert!(
        rewritten.contains(r#""format_version": "1.21.100""#),
        "{rewritten}"
    );
    assert!(rewritten.contains("exclusion_list"), "{rewritten}");
    let again = AimAssistPreset::parse_as_written(&path, rewritten.as_bytes(), &cx).unwrap();
    assert_eq!(again.value(), read.value());
    assert_eq!(
        again.version().written,
        Some(FormatVersion::new(1, 21, 100))
    );
    assert_eq!(again.to_json().unwrap(), rewritten);
    let upgraded = history::upgrade(written, &cx).unwrap();
    let direct = <AimAssistPreset as Content>::parse(&path, text.as_bytes(), &cx).unwrap();
    assert_eq!(upgraded.value(), direct.value());
    // The namespace checks run on the history path too.
    let bad = preset_json("1.21.50", r#""identifier": "p""#);
    let written = AimAssistPreset::parse_as_written(&path, bad.as_bytes(), &cx).unwrap();
    let rejected = history::upgrade(written, &cx).unwrap_err();
    assert_eq!(
        errors(&rejected.diagnostics)[0].2,
        "Namespace required for preset Id: p"
    );
}

#[test]
fn vanillas_aim_assist_files_are_written_back_member_for_member() {
    // Copied unchanged from the BDS 1.26.36.1 `behavior_packs/vanilla/aim_assist/`.
    let text = fixture("vanilla/aim_assist/presets/aim_assist_preset.json");
    let document = loads(preset(&text));
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    assert_eq!(document.identifier.as_str(), "minecraft:aim_assist_default");
    assert_eq!(document.liquid_targeting_list.as_ref().unwrap().len(), 19);
    let written = document.to_json().unwrap();
    let again = loads(preset(&written));
    assert_eq!(again.value(), document.value());
    assert_eq!(again.to_json().unwrap(), written);
    let member = |value: &Value, key: &str| value.as_object().unwrap().get(key).unwrap().clone();
    let (a, b): (Value, Value) = (
        facet_json::from_str(&written).unwrap(),
        facet_json::from_str_jsonc(&text).unwrap(),
    );
    // The same members with the same values (the struct's member order, not the file's).
    let (a, b) = (
        member(&a, "minecraft:aim_assist_preset"),
        member(&b, "minecraft:aim_assist_preset"),
    );
    let (a, b) = (a.as_object().unwrap(), b.as_object().unwrap());
    assert_eq!(a.len(), b.len());
    for (key, value) in b {
        assert_eq!(a.get(key.as_str()), Some(value), "{key:?}");
    }

    let text = fixture("vanilla/aim_assist/categories/categories.json");
    let document = loads(categories(&text));
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    let written = document.to_json().unwrap();
    let again = loads(categories(&written));
    assert_eq!(again.value(), document.value());
    assert_eq!(again.to_json().unwrap(), written);
    let (a, b): (Value, Value) = (
        facet_json::from_str(&written).unwrap(),
        facet_json::from_str_jsonc(&text).unwrap(),
    );
    let names = |value: &Value| -> Vec<String> {
        at(value, "minecraft:aim_assist_categories/categories")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|c| {
                c.as_object()
                    .unwrap()
                    .get("name")
                    .unwrap()
                    .as_string()
                    .unwrap()
                    .as_str()
                    .to_owned()
            })
            .collect()
    };
    assert_eq!(names(&a), names(&b));
    assert_eq!(names(&a).len(), document.categories.len());
}

#[test]
fn an_upgraded_preset_is_written_at_1_21_120() {
    let document = loads(preset(&preset_json(
        "1.21.50",
        r#""identifier": "test:p", "exclusion_list": ["minecraft:stone"]"#,
    )));
    let written = document.to_json().unwrap();
    assert!(
        written.contains(r#""format_version": "1.21.120""#),
        "{written}"
    );
    assert!(!written.contains("exclusion_list"), "{written}");
    let again = loads(preset(&written));
    assert!(!again.version().upgraded);
    assert_eq!(again.value(), document.value());
}

/// The 1.26.60 vanilla aim-assist files (`vanilla_unpacked/behavior_packs/vanilla/
/// aim_assist/`, copied verbatim) are the BDS 1.26.36.1 ones, and the two preset literals of
/// `AimAssistCommandServerTests` (`PRESET_JSON_1`, `VANILLA_PRESET_JSON_1`)
/// load, as the command tests need them registered,
/// without a finding; each is written back and read again to the same value.
#[test]
fn the_1_26_60_vanilla_files_and_the_engine_test_presets_load_and_round_trip() {
    let current = loads(preset(&fixture(
        "vanilla_current/aim_assist/presets/aim_assist_preset.json",
    )));
    let older = loads(preset(&fixture(
        "vanilla/aim_assist/presets/aim_assist_preset.json",
    )));
    assert_eq!(current.value(), older.value());
    let current = loads(categories(&fixture(
        "vanilla_current/aim_assist/categories/categories.json",
    )));
    let older = loads(categories(&fixture(
        "vanilla/aim_assist/categories/categories.json",
    )));
    assert_eq!(current.value(), older.value());
    for (file, identifier) in [
        ("preset.json", "test:preset1"),
        ("vanilla_preset.json", "minecraft:preset1"),
    ] {
        let text = fixture(&format!("engine/{file}"));
        let document = loads(preset(&text));
        assert!(
            document.diagnostics().is_empty(),
            "{file}: {:#?}",
            document.diagnostics()
        );
        assert_eq!(document.identifier.as_str(), identifier);
        assert!(!document.version().upgraded, "{file}: read at 1.21.120");
        let written = document.to_json().unwrap();
        let again = loads(preset(&written));
        assert_eq!(again.value(), document.value(), "{file}");
        assert_eq!(again.to_json().unwrap(), written, "{file}");
    }
}
