//! `admire_item`, `barter`, `trade_table`, `tamemount`, `wither_target_highest_damage`.

use bedrock_addon_core::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon_core::shared::ItemDescriptor;
use bedrock_addon_core::versioning::ConvertMode;
use bedrock_addon_family_entity::component::{
    AdmireItem, Barter, EntityTypes, Tamemount, TradeTable, WitherTargetHighestDamage,
};

use super::{has, json, read_as, round_trip, strict, write};

#[test]
fn admire_item_and_barter_read_their_defaults() {
    let (admire, _) = strict::<AdmireItem>("{}");
    assert_eq!(admire, AdmireItem::default());
    assert_eq!(
        (admire.duration.0, admire.cooldown_after_being_attacked.0),
        (10, 10)
    );
    let (barter, _) = strict::<Barter>("{}");
    assert_eq!(barter, Barter::default());
    assert_eq!(barter.cooldown_after_being_attacked.0, 10);
    let barter = round_trip::<Barter>(
        r#"{"barter_table": "loot_tables/gameplay/barter.json", "cooldown_after_being_attacked": 20}"#,
    );
    assert_eq!(barter.barter_table.0, "loot_tables/gameplay/barter.json");
}

#[test]
fn trade_table_round_trips() {
    let value = round_trip::<TradeTable>(
        r#"{"display_name": "entity.villager.farmer", "table": "trading/farmer_trades.json",
            "new_screen": true, "persist_trades": true, "convert_trades_economy": true}"#,
    );
    assert!(value.new_screen.0 && value.persist_trades.0 && value.convert_trades_economy.0);
    assert_eq!(strict::<TradeTable>("{}").0, TradeTable::default());
}

#[test]
fn tamemount_reads_items_and_its_tame_event() {
    let value = round_trip::<Tamemount>(
        r#"{"min_temper": 0, "max_temper": 100, "feed_text": "action.interact.feed",
            "ride_text": "action.interact.mount",
            "feed_items": [{"item": "minecraft:wheat", "temper_mod": 3},
                           {"item": {"tags": "query.any_tag('minecraft:horse_food')"}, "temper_mod": 5}],
            "auto_reject_items": [{"item": "minecraft:horsearmorleather"}],
            "tame_event": {"event": "minecraft:on_tame", "target": "self"}}"#,
    );
    assert_eq!(value.feed_items.len(), 2);
    assert_eq!(value.feed_items[0].temper_mod.0, 3);
    assert!(matches!(
        value.feed_items[1].item,
        Some(ItemDescriptor::Tags { .. })
    ));
    assert_eq!(value.auto_reject_items.len(), 1);
    assert_eq!(
        value.tame_event.as_ref().and_then(|e| e.event.as_deref()),
        Some("minecraft:on_tame")
    );
    let (default, _) = strict::<Tamemount>("{}");
    assert_eq!(default, Tamemount::default());
    assert_eq!(default.attempt_temper_mod.0, 5);
    assert_eq!(default.max_temper.0, 100);
}

#[expect(clippy::float_cmp, reason = "the reader gives exact values")]
#[test]
fn wither_target_highest_damage_reads_one_or_many_mob_descriptors() {
    let one = round_trip::<WitherTargetHighestDamage>(
        r#"{"priority": 1, "entity_types": {"filters": {"test": "is_family", "subject": "other", "value": "monster"}, "max_dist": 32}}"#,
    );
    assert!(matches!(one.entity_types, EntityTypes::One(_)));
    let many = round_trip::<WitherTargetHighestDamage>(
        r#"{"entity_types": [{"max_dist": 24}, {"must_see": true}]}"#,
    );
    let EntityTypes::Many(list) = &many.entity_types else {
        panic!("an array")
    };
    assert_eq!(list[0].max_dist.0, 24.0);
    assert_eq!(list[1].max_dist.0, 16.0);
    assert_eq!(list[1].must_see.map(|b| b.0), Some(true));
}

#[test]
fn wither_target_highest_damage_drops_the_deprecated_target_goal_names() {
    // "every `TargetGoal` name is deprecated and `entity_types` re-bound";
    // "Deprecated inherited names": `DeprecatedMember`, the member dropped; an error
    // in both masks, which rejects the document.
    let (value, d) = super::read_as::<WitherTargetHighestDamage>(
        r#"{"must_see": true, "within_radius": 5, "entity_types": []}"#,
        ConvertMode::Strict,
    );
    let value = value.expect("the member is skipped, the value read");
    for member in ["must_see", "within_radius"] {
        assert!(
            has(
                &d,
                |c| matches!(c, DiagnosticCode::DeprecatedMember { .. }),
                Severity::Error,
                member
            ),
            "{member}: {d:?}"
        );
    }
    assert!(value.extra.get("must_see").is_none(), "dropped, not kept");
    let text = d
        .iter()
        .next()
        .map(|f| f.code.to_string())
        .unwrap_or_default();
    assert_eq!(text, "member \"must_see\" is deprecated");
    assert_eq!(
        write(&value),
        json(r#"{"priority": 0, "control_flags": [], "entity_types": []}"#)
    );
}

#[test]
fn wither_target_highest_damage_negative_priority_is_an_error() {
    let (value, d) =
        read_as::<WitherTargetHighestDamage>(r#"{"priority": -1}"#, ConvertMode::Strict);
    assert!(value.is_none());
    assert!(d.has_errors());
}
