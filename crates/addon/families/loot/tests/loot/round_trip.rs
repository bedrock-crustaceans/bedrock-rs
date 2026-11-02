//! The Value level: `parse(write(parse(x))) == parse(x)`, unknown members kept at
//! their level, and nothing written that the document did not write.

use bedrock_addon_core::document::{Content, Extra};
use bedrock_addon_family_loot::{
    BannerPattern, BannerType, ConditionKind, Entry, EntryKind, FunctionKind, ItemColor, ItemData,
    LootCondition, LootFunction, LootTable, Pool, RandomValueBounds,
};

use super::common::{fixture, load};

/// Writes `table`, requires the text to read back to the same value, and requires a second
/// write to give the same text. Returns the text.
fn round_trip(table: &LootTable) -> String {
    let written = table.to_json().unwrap();
    let again = load(&written);
    assert_eq!(
        again.value(),
        table,
        "the written text reads back to another value:\n{written}"
    );
    assert_eq!(again.to_json().unwrap(), written, "a rewrite differs");
    written
}

#[test]
fn every_fixture_that_loads_round_trips_value_for_value() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../addon/tests/fixtures/loot/loot");
    let mut count = 0;
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "json") {
            continue;
        }
        let name = path.file_name().unwrap().to_str().unwrap().to_owned();
        let text = String::from_utf8(fixture(&name)).unwrap();
        let Ok(table) = super::common::load_in(&text, &super::common::cx()) else {
            continue;
        };
        round_trip(&table);
        count += 1;
    }
    assert!(count >= 20, "{count} fixtures round-tripped");
}

#[test]
fn vanillas_zombie_table_is_written_back_member_for_member() {
    let table = load(&String::from_utf8(fixture("pools_only.json")).unwrap());
    let written = round_trip(&table);
    // Floats are written in their shortest form, as reals (`rolls` is a `RandomValueBounds`).
    assert!(written.contains(r#""chance": 0.025"#), "{written}");
    assert!(
        written.contains(r#""looting_multiplier": 0.01"#),
        "{written}"
    );
    assert!(written.contains(r#""rolls": 1.0"#), "{written}");
    assert!(written.contains(r#""weight": 1"#), "{written}");
}

#[test]
fn members_the_document_did_not_write_are_not_written() {
    let table =
        load(r#"{"pools": [{"rolls": 1, "entries": [{"type": "item"}, {"type": "empty"}]}]}"#);
    let written = round_trip(&table);
    for absent in [
        "weight",
        "quality",
        "conditions",
        "functions",
        "name",
        "bonus_rolls",
        "tiers",
    ] {
        assert!(
            !written.contains(absent),
            "{absent} was written:\n{written}"
        );
    }
}

#[test]
fn empty_members_keep_their_presence() {
    let table = load(
        r#"{"pools": [{"rolls": 1, "conditions": [], "tiers": {}, "entries": [
            {"type": "item", "name": "minecraft:stick", "functions": [], "pools": []}]}]}"#,
    );
    let written = round_trip(&table);
    assert!(written.contains(r#""conditions": []"#), "{written}");
    assert!(written.contains(r#""functions": []"#), "{written}");
    assert!(written.contains(r#""pools": []"#), "{written}");
    assert!(written.contains(r#""tiers": {}"#), "{written}");
}

#[test]
fn unknown_members_are_kept_at_their_level() {
    let table = load(
        r#"{"type": "minecraft:chest", "format_version": "1.8.0", "pools": [{"rolls": 1, "entries": [
            {"type": "item", "name": "minecraft:wool", "aux_val": 17,
             "functions": [{"function": "minecraft:set_count", "count": 2, "add": false}]}]}]}"#,
    );
    let written = round_trip(&table);
    assert!(written.contains(r#""aux_val": 17"#), "{written}");
    assert!(written.contains(r#""add": false"#), "{written}");
    assert!(
        written.contains(r#""format_version": "1.8.0""#),
        "{written}"
    );
    assert!(
        written.contains(r#""type": "minecraft:chest""#),
        "{written}"
    );
    assert!(
        written.contains(r#""function": "minecraft:set_count""#),
        "{written}"
    );
}

#[test]
fn a_table_built_in_code_is_written_and_read_back() {
    let mut pool = Pool {
        rolls: Some(RandomValueBounds::new(1.0, 3.0)),
        ..Pool::default()
    };
    let mut item = Entry::new(EntryKind::Item {
        name: Some(bedrock_addon_core::id::ItemId::new("minecraft:white_banner").unwrap()),
        functions: Some(vec![
            LootFunction::new(FunctionKind::SetData(ItemData {
                data: Some(RandomValueBounds::Exact(15.0)),
                values: None,
            })),
            LootFunction::new(FunctionKind::SetBannerDetails {
                banner_type: Some(BannerType::IllagerCaptain),
                base_color: Some(ItemColor::LightBlue),
                patterns: None,
            }),
            LootFunction::new(FunctionKind::SetBannerDetails {
                banner_type: None,
                base_color: None,
                patterns: Some(vec![BannerPattern {
                    pattern: "bo".to_owned(),
                    color: Some(ItemColor::Black),
                    extra: Extra::default(),
                }]),
            }),
        ]),
    });
    item.weight = Some(3);
    item.conditions = Some(vec![LootCondition::new(
        ConditionKind::RandomDifficultyChance {
            default_chance: Some(0.5),
            peaceful: None,
            easy: None,
            normal: None,
            hard: Some(1.0),
        },
    )]);
    pool.entries = Some(vec![item, Entry::new(EntryKind::Empty)]);
    let table = LootTable {
        pools: Some(vec![pool]),
        ..LootTable::default()
    };
    let written = round_trip(&table);
    assert!(
        written.contains(r#""type": "illager_captain""#),
        "{written}"
    );
    assert!(
        written.contains(r#""base_color": "light_blue""#),
        "{written}"
    );
}
