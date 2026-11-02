//! Write-back: what was read is written, nothing else; the
//! history path; and values built in code.

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::document::Content;
use bedrock_addon_core::id::ItemId;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_family_trade::history::{self, Any, v0};
use bedrock_addon_family_trade::{Group, ItemList, Quantity, Tier, Trade, TradeItem, TradeTable};

use super::common::{above_info, cx, doc, fixture, loads, with};

/// JSON(C) text as a `Value`.
fn json(text: &str) -> Value {
    bedrock_addon_core::json::from_str_jsonc(text).unwrap()
}

/// The member `key` of the object `value`.
fn member<'a>(value: &'a Value, key: &str) -> &'a Value {
    value.as_object().unwrap().get(key).unwrap()
}

/// Writes `table`, reads the text back strictly (it is a 1.21.30 document) and requires the
/// same value and an identical rewrite; the written text.
#[track_caller]
fn rewritten(table: &TradeTable) -> String {
    let written = table.to_json().unwrap();
    let again = loads(&written);
    assert_eq!(again.version().parsed_as, FormatVersion::new(1, 21, 30));
    assert_eq!(again.value(), table);
    assert_eq!(again.value().to_json().unwrap(), written);
    written
}

#[test]
fn vanillas_1_26_10_files_are_written_back_member_for_member() {
    for name in [
        "t07_butcher_trades_1_26_10.json",
        "t07_cleric_trades_1_26_10.json",
        "t09_farmer_trades_1_26_10.json",
    ] {
        let text = fixture(name);
        let document = loads(&text);
        let written = rewritten(document.value());
        // Nothing added, nothing lost, in the file's own member order: the `tiers` of the
        // written document are the file's. The stamp is written as 1.21.30, the version of
        // the only struct from it on.
        let (original, written) = (json(&text), json(&written));
        assert_eq!(
            member(&written, "tiers"),
            member(&original, "tiers"),
            "{name}"
        );
        assert_eq!(member(&written, "format_version"), &json(r#""1.21.30""#));
        assert_eq!(written.as_object().unwrap().len(), 2);
        // `Value` equality does not look at member order; the texts of the two `tiers` do.
        let text_of =
            |value: &Value| bedrock_addon_core::json::to_json(member(value, "tiers")).unwrap();
        assert_eq!(text_of(&written), text_of(&original), "{name}");
    }
}

#[test]
fn an_upgraded_legacy_table_is_written_as_a_document_that_reads_back_equal() {
    // A table without a legacy blob is written as a 1.21.30 document; one with a blob in the
    // legacy shape at 1.20.30, never with a `__DEP_` member.
    for name in [
        "t01_flat_to_groups.json",
        "t04_quantity_forms.json",
        "t09_farmer_trades_unstamped.json",
    ] {
        let document = loads(&fixture(name));
        assert_eq!(document.version().parsed_as, FormatVersion::new(0, 0, 0));
        if holds_a_legacy_blob(document.value()) {
            legacy_rewritten(document.value());
        } else {
            rewritten(document.value());
        }
    }
}

/// Whether an item of `table` holds `__DEP_fnctns` or `__DEP_fltrs`.
fn holds_a_legacy_blob(table: &TradeTable) -> bool {
    table
        .tiers
        .iter()
        .flat_map(|tier| &tier.groups)
        .flat_map(|group| &group.trades)
        .flat_map(|trade| trade.wants.iter().chain(&trade.gives))
        .flat_map(ItemList::items)
        .any(|item| item.legacy_functions.is_some() || item.legacy_filters.is_some())
}

/// Writes `table`, which holds a legacy blob: a 1.20.30 legacy document without a `__DEP_`
/// member, which reads back to the same value with no finding above information and writes
/// again to the same text; the written text.
#[track_caller]
fn legacy_rewritten(table: &TradeTable) -> String {
    let written = table.to_json().unwrap();
    assert!(!written.contains("__DEP_"), "{written}");
    assert_eq!(
        member(&json(&written), "format_version"),
        &json(r#""1.20.30""#)
    );
    let again = loads(&written);
    assert_eq!(again.version().parsed_as, FormatVersion::new(0, 0, 0));
    assert_eq!(again.value(), table, "{written}");
    assert!(
        above_info(again.diagnostics()).is_empty(),
        "{:#?}",
        again.diagnostics()
    );
    assert_eq!(again.value().to_json().unwrap(), written);
    written
}

#[test]
fn the_side_cars_are_written_back_in_the_legacy_shape_and_read_equal() {
    let document = loads(&doc(
        Some("1.18.20"),
        r#"[{"trades": [{"wants": [{"item": "minecraft:wheat",
            "functions": [{"function": "set_data", "data": 3}],
            "biomes": ["desert"]}], "gives": [{"item": "minecraft:emerald"}]}]}]"#,
    ));
    let written = legacy_rewritten(document.value());
    let again = loads(&written);
    assert!(with(again.diagnostics(), "DeprecatedMember", Severity::Warning).is_empty());
}

/// What the written table says that the file did not is said when the file is loaded: a
/// legacy `functions` or `filters` (or the filter the gates make of `biomes` or of a
/// `trader_material_type` function) keeps its legacy evaluation, and the newest format has it
/// under a deprecated name the game warns of. The first load announces it as an information
/// of the upgrade, a finding of this crate with no text of the game's, at the member the
/// FILE has, saying which member the written table carries it as.
// K107; review R-TG6-WRITE, point 5.
#[test]
fn k107_a_blob_written_under_its_deprecated_name_is_announced_at_the_member_of_the_file() {
    let announced = |d: &bedrock_addon_core::diagnostic::Diagnostics| -> Vec<String> {
        d.iter()
            .filter(|f| f.code.name() == "ValueRewritten")
            .map(|f| {
                assert!(
                    matches!(
                        f.code,
                        bedrock_addon_core::diagnostic::DiagnosticCode::ValueRewritten {
                            text: None
                        }
                    ),
                    "a finding of ours has no text of the game's: {:?}",
                    f.code
                );
                format!(
                    "{:?} {:?} at {}: {f}",
                    f.severity, f.phase, f.location.json_path
                )
            })
            .collect()
    };
    let table = |version: &str, wants: &str| {
        loads(&doc(
            Some(version),
            &format!(
                r#"[{{"trades": [{{"wants": [{wants}], "gives": [{{"item": "minecraft:emerald"}}]}}]}}]"#
            ),
        ))
    };
    let at = "tiers/0/trades/0/wants/0";
    // Where the finding is, and what its message ends with.
    let line = |member: &str, written_as: &str| {
        (
            format!("Info Upgrade at {at}/{member}: "),
            format!(
                "| value held in another form than it is written in: written back as \
                 `{written_as}` of a 1.20.30 legacy document, which keeps its legacy evaluation"
            ),
        )
    };
    for (version, wants, expected, warnings) in [
        // `functions` and the filter made of `biomes`.
        (
            "1.18.20",
            r#"{"item": "minecraft:wheat", "functions": [{"function": "set_data", "data": 3}], "biomes": ["desert"]}"#,
            vec![line("functions", "functions"), line("biomes", "filters")],
            2,
        ),
        // The file's own `filters`.
        (
            "1.20.30",
            r#"{"item": "minecraft:wheat", "filters": {"test": "is_daytime"}}"#,
            vec![line("filters", "filters")],
            1,
        ),
        // The filters of the five items a `trader_material_type` function expands into.
        (
            "1.18.10",
            r#"{"item": "minecraft:boat", "functions": [{"function": "trader_material_type"}]}"#,
            vec![line("functions", "filters")],
            5,
        ),
        // Neither: nothing is announced.
        (
            "1.18.20",
            r#"{"item": "minecraft:wheat", "quantity": 2}"#,
            vec![],
            0,
        ),
    ] {
        let document = table(version, wants);
        let told = announced(document.diagnostics());
        assert!(
            told.iter()
                .zip(&expected)
                .all(|(told, (at, says))| told.starts_with(at.as_str())
                    && told.ends_with(says.as_str()))
                && told.len() == expected.len(),
            "{wants}: {told:#?}\nexpected {expected:#?}"
        );
        // No finding of the first load has the game's deprecation text.
        assert_eq!(
            with(document.diagnostics(), "DeprecatedMember", Severity::Info).len(),
            0
        );
        assert_eq!(
            with(
                document.diagnostics(),
                "DeprecatedMember",
                Severity::Warning
            )
            .len(),
            0
        );
        // The written table is a legacy document when a blob was announced (no `__DEP_`
        // member, which the game rejects), reads back to the same value with
        // no finding above information, and its upgrade announces the same members again.
        let written = document.value().to_json().unwrap();
        assert!(!written.contains("__DEP_"), "{written}");
        let again = loads(&written);
        assert_eq!(again.value(), document.value(), "{written}");
        assert!(above_info(again.diagnostics()).is_empty(), "{written}");
        assert_eq!(warnings > 0, !told.is_empty(), "{written}");
        assert_eq!(
            announced(again.diagnostics()).is_empty(),
            told.is_empty(),
            "{written}"
        );
    }
}

#[test]
fn the_forms_and_the_presence_of_members_are_kept() {
    let tiers = r#"[{"groups": [{"trades": [{
        "wants": [
            {"item": "minecraft:wheat"},
            {"choice": []},
            {"choice": [{"quantity": 3, "item": "minecraft:carrot", "functions": []}]},
            {"quantity": {"min": 3, "max": 3}, "item": "minecraft:potato"},
            {"quantity": {"max": 2}, "item": "minecraft:melon"},
            {"quantity": {}, "item": "minecraft:pumpkin", "price_multiplier": 0.0}
        ],
        "gives": [{"item": "minecraft:emerald", "filters": {"test": "is_daytime", "value": true}}],
        "trader_exp": 1, "max_uses": 7, "reward_exp": true, "weight": 1
    }]}, {"num_to_select": 0, "trades": [
        {"wants": [{"item": "minecraft:wheat"}], "gives": [{"item": "minecraft:emerald"}]}
    ]}], "total_exp_required": 0}]"#;
    let document = loads(&doc(Some("1.21.30"), tiers));
    assert!(above_info(document.diagnostics()).is_empty());
    let written = rewritten(document.value());
    assert_eq!(member(&json(&written), "tiers"), &json(tiers));
    // A written default is a value; an absent member is not written as its default.
    let traded = &document.value().tiers[0].groups[0].trades[0];
    assert_eq!(
        (
            traded.trader_exp,
            traded.max_uses,
            traded.reward_exp,
            traded.weight
        ),
        (Some(1), Some(7), Some(true), Some(1))
    );
    assert!(matches!(traded.wants[1], ItemList::Choice { ref choice } if choice.is_empty()));
    assert_eq!(traded.wants[5].items()[0].price_multiplier, Some(0.0));
    assert_eq!(traded.wants[0].items()[0].price_multiplier(), 0.0);
}

#[test]
fn a_table_built_by_hand_is_written_and_read_back() {
    let emerald = TradeItem::new(ItemId::new("minecraft:emerald").unwrap());
    let wheat = TradeItem {
        quantity: Some(Quantity::Range {
            min: Some(18),
            max: Some(22),
        }),
        price_multiplier: Some(0.05),
        ..TradeItem::new(ItemId::new("minecraft:wheat").unwrap())
    };
    let table = TradeTable {
        tiers: vec![Tier {
            groups: vec![Group {
                num_to_select: Some(1),
                trades: vec![Trade {
                    wants: vec![ItemList::Choice {
                        choice: vec![wheat],
                    }],
                    gives: vec![ItemList::Item(emerald)],
                    trader_exp: Some(2),
                    ..Trade::default()
                }],
            }],
            total_exp_required: 0,
        }],
    };
    let written = rewritten(&table);
    assert_eq!(
        json(&written),
        json(
            r#"{"format_version": "1.21.30", "tiers": [{"groups": [{"num_to_select": 1, "trades": [{
                "wants": [{"choice": [{"quantity": {"min": 18, "max": 22},
                    "item": "minecraft:wheat", "price_multiplier": 0.05}]}],
                "gives": [{"item": "minecraft:emerald"}], "trader_exp": 2}]}],
                "total_exp_required": 0}]}"#
        )
    );
    // The accessors apply the defaults of the members left out.
    let traded = &table.tiers[0].groups[0].trades[0];
    assert_eq!(
        (
            traded.trader_exp(),
            traded.max_uses(),
            traded.reward_exp(),
            traded.weight()
        ),
        (2, 7, true, 1)
    );
    assert_eq!(traded.wants[0].items()[0].quantity_range(), (18, 22));
    assert_eq!(traded.gives[0].items()[0].quantity_range(), (1, 1));
    assert_eq!(Quantity::Exact(4).bounds(), (4, 4));
    assert_eq!(
        Quantity::Range {
            min: Some(2),
            max: None
        }
        .bounds(),
        (2, 1)
    );
    assert_eq!(Group::default().num_to_select(), 0);
    assert_eq!(table.tiers[0].groups[0].num_to_select(), 1);
}

// ---- history -----------------------------------------------------------------

/// The path the history tests parse at.
fn path() -> PackPath {
    PackPath::from_static("trading/test_trades.json")
}

#[test]
fn a_legacy_file_parses_as_written_and_upgrades_to_what_parse_gives() {
    let text = fixture("t01_flat_to_groups.json");
    let as_written = TradeTable::parse_as_written(&path(), text.as_bytes(), &cx()).unwrap();
    assert_eq!(as_written.version().parsed_as, FormatVersion::new(0, 0, 0));
    assert_eq!(as_written.value().parsed_version(), v0::VERSION);
    let Any::V0(legacy) = as_written.value() else {
        panic!("not the legacy struct: {:?}", as_written.value());
    };
    // The struct is the document as written: flat trades, no total_exp_required.
    assert_eq!(legacy.format_version, None);
    assert_eq!(legacy.tiers.len(), 2);
    assert!(legacy.tiers[0].groups.is_none());
    assert_eq!(legacy.tiers[0].trades.as_ref().map(Vec::len), Some(2));
    assert_eq!(legacy.tiers[1].total_exp_required, None);
    // Written in its own version, it is the same document.
    let rewritten = as_written.value().to_json().unwrap();
    assert_eq!(json(&rewritten), json(&text));
    let again = TradeTable::parse_as_written(&path(), rewritten.as_bytes(), &cx()).unwrap();
    assert_eq!(again.value(), as_written.value());
    // The upgrade is the rest of `parse`.
    let upgraded = history::upgrade(as_written, &cx()).unwrap();
    let parsed = loads(&text);
    assert_eq!(upgraded.value(), parsed.value());
    assert_eq!(upgraded.version(), parsed.version());
    assert!(upgraded.version().upgraded);
}

#[test]
fn a_stamped_legacy_file_keeps_its_version_when_written_in_its_own_shape() {
    let text = doc(
        Some("1.18.10"),
        r#"[{"trades": [{"wants": [{"item": "minecraft:boat", "biomes": ["desert"],
            "functions": [{"function": "trader_material_type"}]}],
            "gives": [{"item": "minecraft:emerald"}]}]}]"#,
    );
    let as_written = TradeTable::parse_as_written(&path(), text.as_bytes(), &cx()).unwrap();
    let rewritten = as_written.value().to_json().unwrap();
    assert_eq!(json(&rewritten), json(&text));
    // The gates still see 1.18.10 after the rewrite: the same upgrade.
    assert_eq!(loads(&rewritten).value(), loads(&text).value());
    let upgraded = history::upgrade(as_written, &cx()).unwrap();
    assert_eq!(upgraded.value(), loads(&text).value());
    assert_eq!(
        upgraded.value().tiers[0].groups[0].trades[0].wants[0]
            .items()
            .len(),
        5
    );
}

#[test]
fn a_1_21_30_file_parses_as_the_newest_struct_and_upgrade_runs_no_step() {
    let text = fixture("t07_cleric_trades_1_26_10.json");
    let as_written = TradeTable::parse_as_written(&path(), text.as_bytes(), &cx()).unwrap();
    assert!(matches!(as_written.value(), Any::V1_21_30(_)));
    assert_eq!(
        as_written.value().parsed_version(),
        FormatVersion::new(1, 21, 30)
    );
    let rewritten = as_written.value().to_json().unwrap();
    assert_eq!(
        member(&json(&rewritten), "tiers"),
        member(&json(&text), "tiers")
    );
    let upgraded = history::upgrade(as_written, &cx()).unwrap();
    assert_eq!(upgraded.value(), loads(&text).value());
    assert!(!upgraded.version().upgraded);
}

#[test]
fn a_rejected_file_is_rejected_as_written_too() {
    for text in [
        "null",
        r#"{"tiers": 3}"#,
        "{",
        r#"{"format_version": "9.0.0", "tiers": []}"#,
    ] {
        let rejected = TradeTable::parse_as_written(&path(), text.as_bytes(), &cx()).unwrap_err();
        assert!(rejected.diagnostics.has_errors(), "{text}");
        assert_eq!(rejected.origin.path, path());
    }
    // A rule of the loaded table rejects in the upgrade.
    let empty = TradeTable::parse_as_written(&path(), br#"{"tiers": []}"#, &cx()).unwrap();
    assert!(history::upgrade(empty, &cx()).is_err());
}

// ---- unknown members of a legacy document ----------------------------------------

/// A legacy document with a member the parser does not read at every level.
const WITH_UNKNOWN: &str = r#"{"note": 1, "tiers": [{"tag": "x", "groups": [{"label": "y", "trades": [
    {"comment": "z",
     "wants": [{"item": "minecraft:wheat", "count": 2, "quanitity": 3}],
     "gives": [{"choice": [{"item": "minecraft:emerald", "amount": 1}], "hint": true}]}]}]}]}"#;

/// The unknown members of [`WITH_UNKNOWN`], by path.
const UNKNOWN: [&str; 8] = [
    "note",
    "tiers/0/tag",
    "tiers/0/groups/0/label",
    "tiers/0/groups/0/trades/0/comment",
    "tiers/0/groups/0/trades/0/wants/0/count",
    "tiers/0/groups/0/trades/0/wants/0/quanitity",
    "tiers/0/groups/0/trades/0/gives/0/hint",
    "tiers/0/groups/0/trades/0/gives/0/choice/0/amount",
];

#[test]
fn a_legacy_documents_unknown_members_are_written_back_in_its_own_shape() {
    let as_written = TradeTable::parse_as_written(&path(), WITH_UNKNOWN.as_bytes(), &cx()).unwrap();
    // Each is reported once (the legacy parser does not read it), and kept.
    let d = as_written.diagnostics();
    for at in UNKNOWN {
        super::common::found_at(d, "UnrecognizedField", Severity::Info, at);
    }
    assert!(above_info(d).is_empty());
    let Any::V0(legacy) = as_written.value() else {
        panic!("not the legacy struct");
    };
    assert_eq!(legacy.extra.get("note"), Some(&json("1")));
    let tier = &legacy.tiers[0];
    assert_eq!(tier.extra.get("tag"), Some(&json(r#""x""#)));
    let group = &tier.groups.as_ref().unwrap()[0];
    assert_eq!(group.extra.get("label"), Some(&json(r#""y""#)));
    let traded = &group.trades[0];
    assert_eq!(traded.extra.get("comment"), Some(&json(r#""z""#)));
    let v0::ItemList::Item(wheat) = &traded.wants[0] else {
        panic!("not a bare item");
    };
    assert_eq!(wheat.extra.get("count"), Some(&json("2")));
    assert_eq!(wheat.extra.get("quanitity"), Some(&json("3")));
    let v0::ItemList::Choice { choice, extra } = &traded.gives[0] else {
        panic!("not a choice");
    };
    assert_eq!(extra.get("hint"), Some(&json("true")));
    assert_eq!(choice[0].extra.get("amount"), Some(&json("1")));
    let rewritten = as_written.value().to_json().unwrap();
    assert_eq!(json(&rewritten), json(WITH_UNKNOWN));
    for key in [
        "note",
        "tag",
        "label",
        "comment",
        "count",
        "quanitity",
        "hint",
        "amount",
    ] {
        assert!(rewritten.contains(&format!("\"{key}\"")), "{key}");
    }
    // Reading the rewrite gives the same struct, unknown members included.
    let again = TradeTable::parse_as_written(&path(), rewritten.as_bytes(), &cx()).unwrap();
    assert_eq!(again.value(), as_written.value());
    assert_eq!(again.value().to_json().unwrap(), rewritten);
}

#[test]
fn the_upgraded_table_drops_a_legacy_documents_unknown_members_deviation() {
    // keeps unknown members at their level; a 1.21.30 document cannot carry
    // them (an unknown member is an error there), so the upgraded write leaves them out and
    // the warning of the legacy read has said so.
    let document = loads(WITH_UNKNOWN);
    assert_eq!(
        with(document.diagnostics(), "UnrecognizedField", Severity::Info).len(),
        UNKNOWN.len()
    );
    let written = rewritten(document.value());
    for key in [
        "note",
        "tag",
        "label",
        "comment",
        "count",
        "quanitity",
        "hint",
        "amount",
    ] {
        assert!(!written.contains(&format!("\"{key}\"")), "{key}");
    }
    let again = loads(&written);
    assert!(again.diagnostics().is_empty(), "{:#?}", again.diagnostics());
}

#[test]
fn a_member_named_extra_is_an_unknown_member_like_any_other() {
    // Legacy: warned, kept in the document's own shape.
    let legacy = r#"{"extra": 1, "tiers": [{"extra": 2, "trades": [{"extra": 3,
        "wants": [{"item": "minecraft:wheat", "extra": 4}],
        "gives": [{"choice": [{"item": "minecraft:emerald"}], "extra": 5}]}]}]}"#;
    let as_written = TradeTable::parse_as_written(&path(), legacy.as_bytes(), &cx()).unwrap();
    assert_eq!(
        with(
            as_written.diagnostics(),
            "UnrecognizedField",
            Severity::Info
        )
        .len(),
        5
    );
    assert!(above_info(as_written.diagnostics()).is_empty());
    assert_eq!(json(&as_written.value().to_json().unwrap()), json(legacy));
    // From 1.21.30: an unknown member, so an `UnrecognizedField` error at its own path.
    let stamped = r#"{"format_version": "1.21.30", "tiers": [{"total_exp_required": 0, "extra": 1,
        "groups": [{"trades": [{"wants": [{"item": "minecraft:wheat", "extra": 2}],
        "gives": [{"item": "minecraft:emerald"}]}]}]}]}"#;
    let rejected = TradeTable::parse_as_written(&path(), stamped.as_bytes(), &cx()).unwrap_err();
    let errors: Vec<String> = rejected
        .diagnostics
        .iter()
        .filter(|f| f.severity == Severity::Error)
        .map(|f| format!("{} {}", f.code.name(), f.location.json_path))
        .collect();
    assert_eq!(
        errors,
        [
            "UnrecognizedField tiers/0/extra",
            "UnrecognizedField tiers/0/groups/0/trades/0/wants/0/extra"
        ]
    );
}

/// a legacy blob beside a typed `functions` or `filters` has no form a reader reads
/// back (the legacy shape has no typed members, the newest format no blob the game takes).
#[test]
fn a_legacy_blob_beside_a_typed_member_is_not_representable() {
    let mut item = TradeItem::new(ItemId::new("minecraft:wheat").unwrap());
    item.legacy_functions = Some(json(r#"[{"function": "set_data", "data": 1}]"#));
    item.functions = Some(vec![json(r#"{"function": "set_count", "count": 2}"#)]);
    let table = TradeTable {
        tiers: vec![Tier {
            groups: vec![Group {
                num_to_select: None,
                trades: vec![Trade {
                    wants: vec![ItemList::Item(item)],
                    gives: vec![ItemList::Item(TradeItem::new(
                        ItemId::new("minecraft:emerald").unwrap(),
                    ))],
                    ..Trade::default()
                }],
            }],
            total_exp_required: 0,
        }],
    };
    assert!(table.to_json().is_err());
}
