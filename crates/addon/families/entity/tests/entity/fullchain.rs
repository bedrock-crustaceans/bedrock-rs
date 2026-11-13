//! Whole documents through every step: the newest struct each
//! loads to is compared with a snapshot (`<name>.latest.json` beside it; set `ENTITY_BLESS=1` to
//! rewrite the snapshots after a reviewed change).

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::document::Content;
use bedrock_addon_core::json::Value;

use super::common::{codes, cx, fixture, fixtures, json, stable};

/// Loads `chain/<name>.json` to the newest struct and compares its JSON with the snapshot.
fn snapshot(name: &str) -> bedrock_addon_core::diagnostic::Diagnostics {
    let text = fixture(&format!("chain/{name}.json"));
    let (entity, found) = stable(&text, &cx());
    // With the unknown members the load kept, as the document writes them.
    let mut written = Vec::new();
    entity
        .write_json_with(
            found.unknown(),
            &mut written,
            bedrock_addon_core::json::WriteStyle::default(),
        )
        .expect("the entity writes");
    let written = String::from_utf8(written).unwrap();
    let path = fixtures().join(format!("chain/{name}.latest.json"));
    if std::env::var_os("ENTITY_BLESS").is_some() {
        std::fs::write(&path, &written).unwrap();
    }
    let expected = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: {e} (run with ENTITY_BLESS=1)", path.display()));
    assert_eq!(
        json(&written),
        json(&expected),
        "{name}: the snapshot differs"
    );
    // The snapshot is the newest struct: reading it gives the same value.
    let (again, _) = stable(&expected, &cx());
    assert_eq!(again, entity, "{name}: the snapshot does not read back");
    found
}

#[test]
fn full_chain_1_16_0() {
    let found = snapshot("v09_zombieish_1_16_0");
    // `move_towards_restriction` without `dweller` and `controlling_seat` are
    // dropped with the crate's texts; the root filter of `demo:killed` is ignored below 1.19.20.
    // The two removals are informations (the game drops them without a word),
    // and so is the ignored root filter.
    assert!(codes(&found, Severity::Warning).is_empty(), "{found:#?}");
    assert_eq!(
        super::steps::removed(&found),
        [
            "entity/component_removed_by_upgrade",
            "entity/member_removed_by_upgrade"
        ],
        "{found:#?}"
    );
}

#[test]
fn full_chain_1_8_0() {
    for name in [
        "v10_cloud_1_8_0",
        "v10_pearl_1_12_0",
        "v10_cave_spider_1_13_0",
        "v10_enderman_1_13_0",
    ] {
        let found = snapshot(name);
        assert!(
            codes(&found, Severity::Error).is_empty(),
            "{name}: {found:#?}"
        );
    }
}

#[test]
fn a_floor_document_without_components_gets_no_op() {
    // at or below 1.8.0 without `components`, vanilla runs neither fixer section.
    let text = r#"{"format_version": "1.8.0", "minecraft:entity": {"description": {"identifier": "minecraft:wither"}}}"#;
    let (entity, found) = stable(text, &cx());
    // The 1.26.50 step's cereal upgrader is no fixer op: it sets `uses_legacy_ambient_sounds`
    // on every document it upgrades, and the wither is one of the freezing-immune mobs.
    let written = json(&entity.to_json().unwrap());
    assert_eq!(
        written
            .as_object()
            .and_then(|d| d.get("minecraft:entity"))
            .and_then(Value::as_object)
            .and_then(|p| p.get("components")),
        Some(&json(
            r#"{"minecraft:uses_legacy_ambient_sounds": {}, "minecraft:freezing_immune": {}}"#
        ))
    );
    assert!(found.is_empty(), "{found:#?}");
    // With `components`, the same document gets the 1.10.0 boss members and the later ops.
    let text = r#"{"format_version": "1.8.0", "minecraft:entity": {"description": {"identifier": "minecraft:wither"}, "components": {}}}"#;
    let (entity, _) = stable(text, &cx());
    assert!(!entity.components.is_empty());
}

#[test]
fn do_not_upgrade() {
    // A 1.20.0 vanilla cow with `do_not_upgrade`: section A is skipped (no `spawn_category`,
    // no `uses_legacy_friction`), sections B and C still run (`panic.damage_sources`).
    let text = r#"{"format_version": "1.20.0", "minecraft:entity": {"description": {"identifier": "minecraft:cow"},
        "do_not_upgrade": true,
        "components": {"minecraft:behavior.panic": {"speed_multiplier": 1.25}, "minecraft:pushable": {"is_pushable": true, "is_pushable_by_piston": false}}}}"#;
    let (entity, found) = stable(text, &cx());
    assert!(codes(&found, Severity::Warning).is_empty(), "{found:#?}");
    assert_eq!(entity.description.spawn_category, None);
    let written = json(&entity.to_json().unwrap());
    let components = written
        .as_object()
        .and_then(|d| d.get("minecraft:entity"))
        .and_then(Value::as_object)
        .and_then(|p| p.get("components"))
        .cloned()
        .unwrap();
    assert_eq!(
        components,
        json(
            r#"{"minecraft:behavior.panic": {"speed_multiplier": 1.25, "damage_sources": ["all"]}, "minecraft:pushable_by_entity": {"presets": [], "use_default_with_overlap_check": false}, "minecraft:uses_legacy_ambient_sounds": {}}"#
        )
    );
    // The key is a flag of the chain, not written back.
    let payload = bedrock_addon_core::diagnostic::JsonPath::of_keys(["minecraft:entity"]);
    assert!(found.unknown().get(&payload, "do_not_upgrade").is_none());
    // Without the key the same document gets the section-A ops.
    let text = text.replace(r#""do_not_upgrade": true,"#, "");
    let (entity, _) = stable(&text, &cx());
    assert!(entity.description.spawn_category.is_some());
}
