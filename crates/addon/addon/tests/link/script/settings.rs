//! 20 S11 to S13: the saved values of pack settings against the packs of the world.

use bedrock_addon::diagnostic::{DiagnosticDetail, Phase, Severity};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::settings::{GLOBAL_FILE, SavedSettings, WORLD_FILE, unmatched_entries};
use bedrock_addon::pack::{PackSide, StackSet};
use uuid::Uuid;

use super::super::common::{behavior_pack, resource_pack, stacks_of};

/// A world with the behaviour packs 1 and 3 and the resource pack 2.
fn world() -> StackSet {
    stacks_of(
        vec![behavior_pack(1, &[]), behavior_pack(3, &[])],
        vec![resource_pack(2, &[])],
        LoadOptions::default(),
    )
}

/// A saved-settings file called `name` with one entry per id of `packs`.
fn saved(name: &str, packs: &[&str]) -> bedrock_addon::document::Document<SavedSettings> {
    let entries: Vec<String> = packs
        .iter()
        .map(|id| format!(r#"{{"pack_id": "{id}", "values": {{}}}}"#))
        .collect();
    let text = format!(
        r#"{{"format_version": "1.21.100", "minecraft:pack_settings": {{"settings": [{}]}}}}"#,
        entries.join(", ")
    );
    SavedSettings::parse_named(name, text.as_bytes()).expect("a settings file")
}

fn id(n: u128) -> String {
    Uuid::from_u128(n).to_string()
}

/// (file, JSON path, the pack named) of each finding, each required to be the row's.
fn told(found: &bedrock_addon::diagnostic::Diagnostics) -> Vec<(String, String, String)> {
    found
        .iter()
        .map(|f| {
            assert_eq!(f.code.id(), "core/unresolved_reference", "{f}");
            assert_eq!((f.severity, f.phase), (Severity::Info, Phase::Link), "{f}");
            assert!(
                f.location
                    .pack
                    .as_ref()
                    .is_some_and(bedrock_addon::pack::PackRef::is_app_data)
            );
            let Some(DiagnosticDetail::Reference { kind, id, hint }) = f.detail.as_deref() else {
                panic!("{f}");
            };
            assert_eq!((*kind, hint), ("pack", &None));
            (
                f.location
                    .path
                    .as_ref()
                    .expect("the file")
                    .as_str()
                    .to_owned(),
                f.location.json_path.to_string(),
                id.clone(),
            )
        })
        .collect()
}

#[test]
fn s11_a_saved_entry_for_a_pack_of_the_stack_is_no_finding() {
    let set = world();
    assert!(unmatched_entries(&saved(WORLD_FILE, &[&id(1), &id(3)]), &set).is_empty());
    assert!(unmatched_entries(&saved(GLOBAL_FILE, &[&id(2)]), &set).is_empty());
    assert!(unmatched_entries(&saved(WORLD_FILE, &[]), &set).is_empty());
}

#[test]
fn s11_a_saved_entry_for_a_pack_the_world_does_not_have_is_a_note() {
    let set = world();
    let world_file = saved(WORLD_FILE, &[&id(1), &id(9), &id(3), &id(8)]);
    assert_eq!(world_file.origin().side, PackSide::Behavior);
    assert_eq!(
        told(&unmatched_entries(&world_file, &set)),
        [
            (
                WORLD_FILE.to_owned(),
                "minecraft:pack_settings/settings/1/pack_id".to_owned(),
                id(9)
            ),
            (
                WORLD_FILE.to_owned(),
                "minecraft:pack_settings/settings/3/pack_id".to_owned(),
                id(8)
            ),
        ]
    );
}

#[test]
fn s11_each_file_is_matched_against_the_stack_of_its_own_packs() {
    // The world file holds the behaviour packs' values and the global file the resource
    // packs': an entry that names a pack of the other stack applies to nothing either.
    let set = world();
    assert_eq!(
        told(&unmatched_entries(&saved(WORLD_FILE, &[&id(2)]), &set)),
        [(
            WORLD_FILE.to_owned(),
            "minecraft:pack_settings/settings/0/pack_id".to_owned(),
            id(2)
        )]
    );
    assert_eq!(
        told(&unmatched_entries(
            &saved(GLOBAL_FILE, &[&id(1), &id(2)]),
            &set
        )),
        [(
            GLOBAL_FILE.to_owned(),
            "minecraft:pack_settings/settings/0/pack_id".to_owned(),
            id(1)
        )]
    );
}

#[test]
fn s11_a_malformed_pack_id_is_the_load_s_information_and_is_not_told_again() {
    // The audit half of the row: a `pack_id` that is no UUID reads as the nil UUID, with an
    // `InvalidValue` information of the load (ours); the entry applies to no pack,
    // which the load said.
    let file = saved(WORLD_FILE, &["not-a-uuid", &id(9)]);
    let load: Vec<(&str, Severity, String)> = file
        .diagnostics()
        .iter()
        .map(|f| (f.code.id(), f.severity, f.location.json_path.to_string()))
        .collect();
    assert_eq!(
        load,
        [(
            "core/invalid_value",
            Severity::Info,
            "minecraft:pack_settings/settings/0/pack_id".to_owned()
        )]
    );
    assert!(file.settings[0].pack_id.is_nil());
    assert_eq!(
        told(&unmatched_entries(&file, &world())),
        [(
            WORLD_FILE.to_owned(),
            "minecraft:pack_settings/settings/1/pack_id".to_owned(),
            id(9)
        )]
    );
}

#[test]
fn s11_the_link_pass_itself_holds_no_saved_settings() {
    // `StackSet::link()` sees the stacks alone: the row is told where the saved file meets
    // them, by the function above.
    let set = world();
    assert!(set.link().iter().all(|finding| {
        finding
            .location
            .path
            .as_ref()
            .is_none_or(|path| path.as_str() != WORLD_FILE && path.as_str() != GLOBAL_FILE)
    }));
}

#[test]
fn s12_a_saved_value_for_no_setting_or_no_option_is_a_warning_of_the_load() {
    use bedrock_addon::diagnostic::Diagnostics;
    use bedrock_addon::manifest::Manifest;
    use bedrock_addon::pack::settings::PackSettings;
    use bedrock_addon::storage::PackPath;
    let manifest = format!(
        r#"{{
    "format_version": 3,
    "header": {{ "name": "s", "description": "", "uuid": "{}", "version": "1.0.0",
        "min_engine_version": "1.26.40" }},
    "modules": [{{ "type": "data", "uuid": "{}", "version": "1.0.0" }}],
    "settings": [
        {{ "type": "dropdown", "name": "demo:mode", "text": "Mode", "default": "a",
           "options": [{{ "name": "a", "text": "A" }}, {{ "name": "b", "text": "B" }}] }}
    ]
}}"#,
        Uuid::from_u128(1),
        Uuid::from_u128(11),
    );
    let manifest = Manifest::parse(
        &PackPath::from_static("manifest.json"),
        manifest.as_bytes(),
        &bedrock_addon::manifest::CapabilityRegistry::default(),
    )
    .unwrap_or_else(|rejected| panic!("{rejected:?}"));
    let text = format!(
        r#"{{"format_version": "1.21.100", "minecraft:pack_settings": {{"settings": [
            {{"pack_id": "{}", "values": {{"demo:mode": "c", "demo:gone": true}}}}]}}}}"#,
        Uuid::from_u128(1)
    );
    let world_file = SavedSettings::parse(text.as_bytes()).expect("a settings file");
    let mut found = Diagnostics::default();
    let settings = PackSettings::for_pack(
        &manifest,
        Some(&world_file),
        &SavedSettings::default(),
        &mut found,
    )
    .expect("the pack has settings");
    let told: Vec<(&str, Severity, String)> = found
        .iter()
        .map(|f| (f.code.id(), f.severity, f.location.json_path.to_string()))
        .collect();
    assert_eq!(
        told,
        [
            (
                "core/setting_unknown_option",
                Severity::Info,
                "values/demo:mode".to_owned()
            ),
            (
                "core/setting_unknown_name",
                Severity::Info,
                "values/demo:gone".to_owned()
            ),
        ]
    );
    // The default stands in.
    assert_eq!(
        settings.value("demo:mode"),
        Some(&bedrock_addon::pack::settings::SettingValue::String(
            "a".to_owned()
        ))
    );
}

/// 20 S13: the pack-setting queries in behaviour-pack Molang are a lint of the load (a
/// warning per expression, core's compile check); what their arguments name is Molang-carried
/// and stays the game's to resolve when the expression runs.
#[cfg(all(feature = "entity", feature = "molang"))]
#[test]
fn s13_the_setting_a_pack_setting_query_names_stays_run() {
    use std::sync::Arc;

    use bedrock_addon::molang::MolangCheck;
    // A behaviour-pack entity whose animate condition asks for a setting no manifest of the
    // world defines.
    let entity = r#"{"format_version": "1.26.50", "minecraft:entity": {
        "description": {"identifier": "demo:e", "is_spawnable": false, "is_summonable": true,
            "animations": {"x": "controller.animation.demo"},
            "scripts": {"animate": [{"x": "query.get_pack_setting('demo:nothing_defines_me')"}]}},
        "components": {}}}"#;
    let options = LoadOptions {
        molang: Some(Arc::new(MolangCheck::new())),
        ..LoadOptions::default()
    };
    // The queries exist from 1.21.130: the pack is one for that engine.
    let manifest = format!(
        r#"{{"format_version": 2,
    "header": {{ "name": "bp 1", "description": "", "uuid": "{}", "version": [1, 0, 0],
        "min_engine_version": [1, 21, 130] }},
    "modules": [{{ "type": "data", "uuid": "{}", "version": [1, 0, 0] }}]}}"#,
        Uuid::from_u128(1),
        Uuid::from_u128(11),
    );
    // The controller the entity animates is defined: that reference is another row's
    // (20 E8).
    let controller = r#"{"format_version": "1.10.0", "animation_controllers": {"controller.animation.demo": {"initial_state": "default", "states": {"default": {}}}}}"#;
    let files = [
        ("manifest.json", manifest.as_str()),
        ("entities/e.json", entity),
        ("animation_controllers/demo.json", controller),
    ]
    .map(|(path, text)| {
        (
            bedrock_addon::storage::PackPath::from_static(path),
            text.as_bytes().to_vec(),
        )
    });
    let pack =
        bedrock_addon::pack::Pack::from_files(files, &bedrock_addon::pack::OpenOptions::default())
            .expect("a pack");
    let set = stacks_of(vec![pack], Vec::new(), options);
    let report = set.load_all();

    let in_file: Vec<_> = report
        .iter()
        .filter(|f| {
            f.location
                .path
                .as_ref()
                .is_some_and(|path| path.as_str() == "entities/e.json")
        })
        .collect();
    // The load's lint: a warning, once for the expression.
    let lints: Vec<Severity> = in_file
        .iter()
        .filter(|f| matches!(f.code.id(), "core/molang_lint"))
        .map(|f| f.severity)
        .collect();
    assert_eq!(lints, [Severity::Info], "{in_file:?}");
    // Neither the load nor the link pass looks at the argument: no reference finding.
    let is_reference = |f: &&bedrock_addon::diagnostic::Diagnostic| {
        matches!(
            f.detail.as_deref(),
            Some(DiagnosticDetail::Reference { .. })
        ) || format!("{f}").contains("nothing_defines_me")
            && !matches!(f.code.id(), "core/molang_lint")
    };
    assert_eq!(in_file.iter().copied().filter(is_reference).count(), 0);
    let linked = set.link();
    assert_eq!(
        linked
            .iter()
            .filter(|f| {
                f.location
                    .path
                    .as_ref()
                    .is_some_and(|path| path.as_str() == "entities/e.json")
            })
            .count(),
        0,
        "{linked:?}"
    );
}
