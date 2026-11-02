//! The validation rules: the legacy parser's messages and what each one drops (fixture
//! t12), `num_to_select` (fixture t03), and the cereal constraints from 1.21.30 on.

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, Severity};

use super::common::{above_info, doc, found_at, infos, loads, rejects, summary, with};

/// A valid trade (JSON), with `extra` members (`"weight": 3, `).
fn a_trade(extra: &str) -> String {
    format!(
        r#"{{{extra}"wants": [{{"item": "minecraft:wheat"}}], "gives": [{{"item": "minecraft:emerald"}}]}}"#
    )
}

/// One tier (index 0, `total_exp_required` 0) with one group of `trades` (JSON members of the
/// group before `trades`, e.g. `"num_to_select": 1, `).
fn grouped(group: &str, trades: &[String]) -> String {
    format!(
        r#"[{{"total_exp_required": 0, "groups": [{{{group}"trades": [{}]}}]}}]"#,
        trades.join(",")
    )
}

/// One tier with flat `trades`.
fn flat(trades: &[String]) -> String {
    format!(r#"[{{"trades": [{}]}}]"#, trades.join(","))
}

/// The one finding of `d` above info.
#[track_caller]
fn only(d: &Diagnostics) -> &Diagnostic {
    let found = above_info(d);
    assert_eq!(found.len(), 1, "{}", summary(d));
    found[0]
}

/// The one finding of `d` at info: one of ours, nothing above it.
#[track_caller]
fn only_info(d: &Diagnostics) -> &Diagnostic {
    assert!(above_info(d).is_empty(), "{}", summary(d));
    let found = infos(d);
    assert_eq!(found.len(), 1, "{}", summary(d));
    found[0]
}

// ---- t12: the legacy parser's messages and their scope ----------------------------------

#[test]
fn t12_tiers_that_is_no_array_rejects_the_table_with_vanillas_text() {
    let d = rejects(r#"{"tiers": {}}"#);
    let finding = found_at(&d, "InvalidType", Severity::Error, "tiers");
    assert_eq!(finding.code.to_string(), "\"tiers\" must be an array");
    assert_eq!(above_info(&d).len(), 1);
}

#[test]
fn a_missing_tiers_rejects_the_table_with_the_same_text() {
    let d = rejects(r#"{"tier": []}"#);
    let finding = found_at(&d, "MissingPayload", Severity::Error, "");
    assert_eq!(finding.code.to_string(), "\"tiers\" must be an array");
}

#[test]
fn a_null_document_is_the_must_containt_tiers_error() {
    let d = rejects("null");
    let finding = found_at(&d, "MissingPayload", Severity::Error, "");
    assert_eq!(
        finding.code.to_string(),
        "Trade table file must containt \"tiers\""
    );
    assert_eq!(above_info(&d).len(), 1);
}

#[test]
fn groups_that_is_no_array_rejects_the_table_with_vanillas_text() {
    let d = rejects(r#"{"tiers": [{"groups": {}}]}"#);
    let finding = found_at(&d, "InvalidType", Severity::Error, "tiers/0/groups");
    assert_eq!(finding.code.to_string(), "\"groups\" must be an array");
    // `null` is "no groups": the flat form is read instead.
    loads(&doc(None, r#"[{"groups": null, "trades": []}]"#));
}

#[test]
fn t12_a_group_whose_trades_is_no_array_is_dropped_alone() {
    let document = loads(&doc(
        None,
        &format!(
            r#"[{{"groups": [{{"trades": {{}}}}, {{"trades": [{}]}}, {{"num_to_select": 1}}]}}]"#,
            a_trade("")
        ),
    ));
    let d = document.diagnostics();
    let wrong = found_at(d, "InvalidType", Severity::Error, "tiers/0/groups/0/trades");
    assert_eq!(wrong.code.to_string(), "\"trades\" must be an array");
    let missing = found_at(
        d,
        "MissingRequiredField",
        Severity::Error,
        "tiers/0/groups/2/trades",
    );
    assert_eq!(missing.code.to_string(), "\"trades\" must be an array");
    assert_eq!(above_info(d).len(), 2, "{}", summary(d));
    let groups = &document.value().tiers[0].groups;
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].trades.len(), 1);
}

#[test]
fn flat_trades_that_is_no_array_leaves_the_tier_without_groups() {
    let document = loads(&doc(None, r#"[{"trades": {}}]"#));
    let finding = only(document.diagnostics());
    assert_eq!(finding.code.name(), "InvalidType");
    assert_eq!(finding.location.json_path.to_string(), "tiers/0/trades");
    assert!(document.value().tiers[0].groups.is_empty());
}

#[test]
fn t12_a_trade_whose_wants_holds_only_an_empty_choice_is_dropped_alone() {
    let empty = r#"{"wants": [{"choice": []}], "gives": [{"item": "minecraft:emerald"}]}"#;
    let document = loads(&doc(None, &flat(&[empty.to_owned(), a_trade("")])));
    let finding = only(document.diagnostics());
    assert_eq!(
        (finding.code.name(), finding.severity),
        ("InvalidValue", Severity::Error)
    );
    assert_eq!(
        finding.location.json_path.to_string(),
        "tiers/0/trades/0/wants"
    );
    let text = finding.code.to_string();
    assert!(text.starts_with("Trade table {"), "{text}");
    assert!(
        text.ends_with("} doesn't contain any offers, at least one is required"),
        "{text}"
    );
    let trades = &document.value().tiers[0].groups[0].trades;
    assert_eq!(trades.len(), 1);
    assert_eq!(
        trades[0].wants[0].items()[0].item.as_str(),
        "minecraft:wheat"
    );
}

#[test]
fn a_trade_without_gives_is_dropped_alone_with_the_recieves_text() {
    for gives in [r#""gives": [], "#, r#""gives": [{"choice": []}], "#, ""] {
        let empty = format!(r#"{{{gives}"wants": [{{"item": "minecraft:wheat"}}]}}"#);
        let document = loads(&doc(None, &flat(&[a_trade(""), empty])));
        let finding = only(document.diagnostics());
        assert_eq!(
            (finding.code.name(), finding.severity),
            ("InvalidValue", Severity::Error)
        );
        assert_eq!(
            finding.location.json_path.to_string(),
            "tiers/0/trades/1/gives"
        );
        let text = finding.code.to_string();
        assert!(text.starts_with("Trade {"), "{text}");
        assert!(
            text.ends_with("} doesn't have any recieves, at least one is required"),
            "{text}"
        );
        assert_eq!(document.value().tiers[0].groups[0].trades.len(), 1);
    }
}

#[test]
fn a_trade_without_wants_reports_the_offers_only() {
    // `wants` is read first; the trade is dropped before `gives` is looked at.
    let document = loads(&doc(None, &flat(&["{}".to_owned(), a_trade("")])));
    let finding = only(document.diagnostics());
    assert_eq!(
        finding.location.json_path.to_string(),
        "tiers/0/trades/0/wants"
    );
}

#[test]
fn t12_a_legacy_weight_of_zero_or_less_drops_the_trade_with_vanillas_text() {
    for (weight, text) in [
        (0, "Invalid weight value: 0"),
        (-2, "Invalid weight value: -2"),
    ] {
        let bad = a_trade(&format!(r#""weight": {weight}, "#));
        let document = loads(&doc(
            None,
            &grouped(r#""num_to_select": 1, "#, &[bad, a_trade("")]),
        ));
        let finding = only(document.diagnostics());
        assert_eq!(
            (finding.code.name(), finding.severity),
            ("InvalidValue", Severity::Error)
        );
        assert_eq!(
            finding.location.json_path.to_string(),
            "tiers/0/groups/0/trades/0/weight"
        );
        assert_eq!(finding.code.to_string(), text);
        assert_eq!(document.value().tiers[0].groups[0].trades.len(), 1);
    }
}

/// The text of the "weight" warning.
const WEIGHT_NO_EFFECT: &str = "\"weight\" key used for a trade without using \"groups\" key or with \"num_to_select\" set to -1. Weight value will have no effect as every trade will be chosen.";

#[test]
fn t12_a_weight_in_a_flat_tier_is_warned_and_the_trade_kept() {
    let document = loads(&doc(None, &flat(&[a_trade(r#""weight": 3, "#)])));
    let finding = only(document.diagnostics());
    assert_eq!(
        (finding.code.name(), finding.severity),
        ("NoEffect", Severity::Warning)
    );
    assert_eq!(
        finding.location.json_path.to_string(),
        "tiers/0/trades/0/weight"
    );
    assert_eq!(finding.code.to_string(), WEIGHT_NO_EFFECT);
    let kept = &document.value().tiers[0].groups[0].trades;
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].weight, Some(3));
}

#[test]
fn the_weight_warning_needs_a_group_that_selects_every_trade() {
    let weighted = || a_trade(r#""weight": 1, "#);
    // An explicit -1 and a legacy group without `num_to_select` (legacy default -1) warn.
    for group in [r#""num_to_select": -1, "#, ""] {
        let document = loads(&doc(None, &grouped(group, &[weighted()])));
        let finding = only(document.diagnostics());
        assert_eq!(finding.code.name(), "NoEffect");
        assert_eq!(
            finding.location.json_path.to_string(),
            "tiers/0/groups/0/trades/0/weight"
        );
    }
    // A group that picks does not.
    for group in [r#""num_to_select": 1, "#, r#""num_to_select": 0, "#] {
        let document = loads(&doc(None, &grouped(group, &[weighted()])));
        assert!(above_info(document.diagnostics()).is_empty());
    }
    // No weight, or a `null` one, does not.
    for extra in ["", r#""weight": null, "#] {
        let document = loads(&doc(None, &flat(&[a_trade(extra)])));
        assert!(above_info(document.diagnostics()).is_empty());
    }
}

#[test]
fn the_weight_warning_is_for_minus_one_only_not_for_another_negative_count() {
    // `num_to_select == -1` exactly: -2 and 0 select otherwise.
    // The legacy parser's line is a warning, the typed document's note an information.
    for (version, severity) in [(None, Severity::Warning), (Some("1.21.30"), Severity::Info)] {
        for (selects, warned) in [(-2, false), (-1, true), (0, false)] {
            let document = loads(&doc(
                version,
                &grouped(
                    &format!(r#""num_to_select": {selects}, "#),
                    &[a_trade(r#""weight": 3, "#)],
                ),
            ));
            let found = with(document.diagnostics(), "NoEffect", severity);
            assert_eq!(found.len(), usize::from(warned), "{version:?} {selects}");
            assert_eq!(
                above_info(document.diagnostics()).len(),
                usize::from(warned && version.is_none()),
                "{version:?} {selects}"
            );
            assert_eq!(
                document.value().tiers[0].groups[0].num_to_select,
                Some(selects)
            );
        }
    }
}

#[test]
fn the_weight_note_from_1_21_30_is_an_information_deviation() {
    // Vanilla: the legacy parser only (); ours at both,
    // and where the game has no line it is an information of ours.
    let weighted = a_trade(r#""weight": 2, "#);
    let document = loads(&doc(
        Some("1.21.30"),
        &grouped(r#""num_to_select": -1, "#, std::slice::from_ref(&weighted)),
    ));
    assert!(above_info(document.diagnostics()).is_empty());
    let [finding] = with(document.diagnostics(), "NoEffect", Severity::Info)[..] else {
        panic!(
            "one NoEffect information: {}",
            summary(document.diagnostics())
        );
    };
    assert_eq!(finding.code.to_string(), WEIGHT_NO_EFFECT);
    // The stamped default is 0: a group without `num_to_select` picks, so no warning.
    let document = loads(&doc(Some("1.21.30"), &grouped("", &[weighted])));
    assert!(above_info(document.diagnostics()).is_empty());
}

// ---- t03: num_to_select ---------------------------------------------------------------------

#[test]
fn t03_num_to_select_above_the_number_of_trades_is_the_exact_error() {
    for version in [None, Some("1.21.30")] {
        let d = rejects(&doc(
            version,
            &grouped(r#""num_to_select": 3, "#, &[a_trade(""), a_trade("")]),
        ));
        let finding = found_at(
            &d,
            "InvalidValue",
            Severity::Error,
            "tiers/0/groups/0/num_to_select",
        );
        assert_eq!(
            finding.code.to_string(),
            "num_to_select[3] must not be greater than the number of trades[2]"
        );
        assert_eq!(above_info(&d).len(), 1);
        // As many as there are trades is fine.
        let document = loads(&doc(
            version,
            &grouped(r#""num_to_select": 2, "#, &[a_trade(""), a_trade("")]),
        ));
        assert!(above_info(document.diagnostics()).is_empty());
    }
}

#[test]
fn t03_num_to_select_is_an_error_from_1_21_30_too_deviation() {
    // Vanilla has no check on the cereal slice (), and
    // its legacy parser discards the whole table (): ours is one error at both, and the file is not loaded.
    let d = rejects(&doc(
        Some("1.26.10"),
        &grouped(r#""num_to_select": 2, "#, &[a_trade("")]),
    ));
    assert_eq!(with(&d, "InvalidValue", Severity::Error).len(), 1);
}

#[test]
fn legacy_num_to_select_is_compared_with_the_trades_that_were_kept() {
    // The trade with a bad weight is dropped first; two were written, one is left.
    let d = rejects(&doc(
        None,
        &grouped(
            r#""num_to_select": 2, "#,
            &[a_trade(r#""weight": 0, "#), a_trade("")],
        ),
    ));
    let finding = found_at(
        &d,
        "InvalidValue",
        Severity::Error,
        "tiers/0/groups/0/num_to_select",
    );
    assert_eq!(
        finding.code.to_string(),
        "num_to_select[2] must not be greater than the number of trades[1]"
    );
}

// ---- empty tiers ----------------------------------------------------------------------------

#[test]
fn empty_tiers_is_an_error_deviation() {
    // Vanilla loads an empty table with no message ().
    for version in [None, Some("1.21.30")] {
        let d = rejects(&doc(version, "[]"));
        found_at(&d, "InvalidValue", Severity::Error, "tiers");
        assert_eq!(above_info(&d).len(), 1);
    }
}

// ---- the cereal constraints from 1.21.30 on --------------------------------------------------

/// The one error of a stamped table of `tiers`: its code and path.
#[track_caller]
fn stamped_error(tiers: &str) -> (String, String) {
    let d = rejects(&doc(Some("1.21.30"), tiers));
    let errors = with(&d, "InvalidValue", Severity::Error)
        .into_iter()
        .chain(with(&d, "InvalidType", Severity::Error))
        .chain(with(&d, "MissingRequiredField", Severity::Error))
        .collect::<Vec<_>>();
    assert_eq!(errors.len(), 1, "{}", summary(&d));
    assert_eq!(above_info(&d).len(), 1, "{}", summary(&d));
    (
        errors[0].code.name().to_owned(),
        errors[0].location.json_path.to_string(),
    )
}

/// A code name and a path, owned.
fn at(code: &str, path: &str) -> (String, String) {
    (code.to_owned(), path.to_owned())
}

#[test]
fn stamped_trades_wants_and_gives_need_one_element() {
    assert_eq!(
        stamped_error(r#"[{"total_exp_required": 0, "groups": [{"trades": []}]}]"#),
        at("InvalidValue", "tiers/0/groups/0/trades")
    );
    let wants = r#"{"wants": [], "gives": [{"item": "minecraft:emerald"}]}"#;
    assert_eq!(
        stamped_error(&grouped("", &[wants.to_owned()])),
        at("InvalidValue", "tiers/0/groups/0/trades/0/wants")
    );
    let gives = r#"{"wants": [{"item": "minecraft:wheat"}], "gives": []}"#;
    assert_eq!(
        stamped_error(&grouped("", &[gives.to_owned()])),
        at("InvalidValue", "tiers/0/groups/0/trades/0/gives")
    );
    // `tiers[].groups` and `choice` have no size constraint.
    loads(&doc(
        Some("1.21.30"),
        r#"[{"total_exp_required": 0, "groups": []}]"#,
    ));
    let empty_choice = r#"{"wants": [{"choice": []}], "gives": [{"item": "minecraft:emerald"}]}"#;
    let document = loads(&doc(
        Some("1.21.30"),
        &grouped("", &[empty_choice.to_owned()]),
    ));
    assert!(above_info(document.diagnostics()).is_empty());
    let kept = &document.value().tiers[0].groups[0].trades[0].wants[0];
    assert!(kept.items().is_empty());
}

#[test]
fn stamped_required_members_are_required() {
    assert_eq!(
        stamped_error(r#"[{"total_exp_required": 0}]"#),
        at("MissingRequiredField", "tiers/0/groups")
    );
    assert_eq!(
        stamped_error(r#"[{"total_exp_required": 0, "groups": [{"num_to_select": 0}]}]"#),
        at("MissingRequiredField", "tiers/0/groups/0/trades")
    );
    let no_wants = r#"{"gives": [{"item": "minecraft:emerald"}]}"#;
    assert_eq!(
        stamped_error(&grouped("", &[no_wants.to_owned()])),
        at("MissingRequiredField", "tiers/0/groups/0/trades/0/wants")
    );
    let no_gives = r#"{"wants": [{"item": "minecraft:wheat"}]}"#;
    assert_eq!(
        stamped_error(&grouped("", &[no_gives.to_owned()])),
        at("MissingRequiredField", "tiers/0/groups/0/trades/0/gives")
    );
    let no_item = r#"{"wants": [{"quantity": 2}], "gives": [{"item": "minecraft:emerald"}]}"#;
    assert_eq!(
        stamped_error(&grouped("", &[no_item.to_owned()])),
        at(
            "MissingRequiredField",
            "tiers/0/groups/0/trades/0/wants/0/item"
        )
    );
    let no_choice_item =
        r#"{"wants": [{"choice": [{}]}], "gives": [{"item": "minecraft:emerald"}]}"#;
    assert_eq!(
        stamped_error(&grouped("", &[no_choice_item.to_owned()])),
        at(
            "MissingRequiredField",
            "tiers/0/groups/0/trades/0/wants/0/choice/0/item"
        )
    );
}

#[test]
fn a_stamped_weight_is_at_least_one() {
    for weight in [0, -1] {
        let trade = a_trade(&format!(r#""weight": {weight}, "#));
        assert_eq!(
            stamped_error(&grouped("", &[trade])),
            at("InvalidValue", "tiers/0/groups/0/trades/0/weight")
        );
    }
    let document = loads(&doc(
        Some("1.21.30"),
        &grouped("", &[a_trade(r#""weight": 1, "#)]),
    ));
    assert!(above_info(document.diagnostics()).is_empty());
}

#[test]
fn stamped_scalars_take_their_json_type_only() {
    // A value of another JSON type is `InvalidType`, and so is a number the member's integer
    // type does not hold (the strict wrappers' reading: the member is a `u32` / `i32`, with no
    // constraint of its own to break).
    for (member, path, code) in [
        (r#""trader_exp": "2", "#, "trader_exp", "InvalidType"),
        (r#""trader_exp": true, "#, "trader_exp", "InvalidType"),
        (r#""trader_exp": 2.5, "#, "trader_exp", "InvalidType"),
        (r#""trader_exp": -1, "#, "trader_exp", "InvalidType"),
        (r#""trader_exp": 4294967296, "#, "trader_exp", "InvalidType"),
        (r#""max_uses": 1.5, "#, "max_uses", "InvalidType"),
        (r#""max_uses": "7", "#, "max_uses", "InvalidType"),
        (r#""max_uses": 2147483648, "#, "max_uses", "InvalidType"),
        (r#""reward_exp": 1, "#, "reward_exp", "InvalidType"),
        (r#""reward_exp": "true", "#, "reward_exp", "InvalidType"),
        (r#""weight": "1", "#, "weight", "InvalidType"),
        (r#""weight": [1], "#, "weight", "InvalidType"),
    ] {
        let d = rejects(&doc(Some("1.21.30"), &grouped("", &[a_trade(member)])));
        let errors: Vec<_> = d.iter().filter(|f| f.severity == Severity::Error).collect();
        assert_eq!(errors.len(), 1, "{member}: {}", summary(&d));
        assert_eq!(errors[0].code.name(), code, "{member}");
        assert_eq!(
            errors[0].location.json_path.to_string(),
            format!("tiers/0/groups/0/trades/0/{path}")
        );
    }
    // The members of a tier, a group and an item.
    for (tiers, path, code) in [
        (
            r#"[{"total_exp_required": "0", "groups": []}]"#.to_owned(),
            "tiers/0/total_exp_required",
            "InvalidType",
        ),
        (
            r#"[{"total_exp_required": -1, "groups": []}]"#.to_owned(),
            "tiers/0/total_exp_required",
            "InvalidType",
        ),
        (
            grouped(r#""num_to_select": "1", "#, &[a_trade("")]),
            "tiers/0/groups/0/num_to_select",
            "InvalidType",
        ),
        (
            grouped(
                "",
                &[
                    r#"{"wants": [{"item": "minecraft:wheat", "price_multiplier": "0.05"}],
                    "gives": [{"item": "minecraft:emerald"}]}"#
                        .to_owned(),
                ],
            ),
            "tiers/0/groups/0/trades/0/wants/0/price_multiplier",
            "InvalidType",
        ),
        (
            grouped(
                "",
                &[r#"{"wants": [{"item": "minecraft:wheat", "functions": {}}],
                    "gives": [{"item": "minecraft:emerald"}]}"#
                    .to_owned()],
            ),
            "tiers/0/groups/0/trades/0/wants/0/functions",
            "InvalidType",
        ),
    ] {
        assert_eq!(stamped_error(&tiers), at(code, path), "{tiers}");
    }
    // `item` is a string only (`Reference<10>`); the legacy parser reads a number as its text.
    let numbered = r#"{"wants": [{"item": 5}], "gives": [{"item": "minecraft:emerald"}]}"#;
    assert_eq!(
        stamped_error(&grouped("", &[numbered.to_owned()])),
        at("InvalidType", "tiers/0/groups/0/trades/0/wants/0/item")
    );
    let document = loads(&doc(None, &flat(&[numbered.to_owned()])));
    assert_eq!(
        document.value().tiers[0].groups[0].trades[0].wants[0].items()[0]
            .item
            .as_str(),
        "5"
    );
}

#[test]
fn a_stamped_filter_is_checked_by_the_shared_filter_type() {
    let trade = r#"{"wants": [{"item": "minecraft:wheat",
        "filters": {"any_of": [{"test": "no_such_test", "value": 1}]}}],
        "gives": [{"item": "minecraft:emerald"}]}"#;
    let document = loads(&doc(Some("1.21.30"), &grouped("", &[trade.to_owned()])));
    let finding = only(document.diagnostics());
    // "Filter test not found" is a level-3 line and the trade is kept: a kept error.
    assert_eq!(finding.severity, Severity::Error);
    assert!(finding.kept);
    assert!(
        finding
            .location
            .json_path
            .to_string()
            .starts_with("tiers/0/groups/0/trades/0/wants/0/filters"),
        "{}",
        finding.location.json_path
    );
    let item = &document.value().tiers[0].groups[0].trades[0].wants[0].items()[0];
    assert!(item.filters.is_some());
}

#[test]
fn a_syntax_error_rejects_the_file() {
    let d = rejects(r#"{"tiers": [}"#);
    assert_eq!(with(&d, "MalformedInput", Severity::Error).len(), 1);
}

#[test]
fn unknown_legacy_members_are_warned_and_ignored() {
    let document = loads(
        r#"{"note": 1, "tiers": [{"tag": "x", "groups": [{"label": "y", "trades": [
            {"comment": "z", "wants": [{"item": "minecraft:wheat", "count": 2}],
             "gives": [{"choice": [{"item": "minecraft:emerald", "amount": 1}]}]}]}]}]}"#,
    );
    let d = document.diagnostics();
    for path in [
        "note",
        "tiers/0/tag",
        "tiers/0/groups/0/label",
        "tiers/0/groups/0/trades/0/comment",
        "tiers/0/groups/0/trades/0/wants/0/count",
        "tiers/0/groups/0/trades/0/gives/0/choice/0/amount",
    ] {
        found_at(d, "UnrecognizedField", Severity::Info, path);
    }
    assert!(above_info(d).is_empty(), "{}", summary(d));
}

// ---- the F.6.6 list, items 4 and 6 --------------------------------------------------------

#[test]
fn f6_6_item_4_an_unknown_key_at_1_21_30_rejects_the_file() {
    for (tiers, path) in [
        (
            r#"[{"total_exp_required": 0, "groups": [], "tag": 1}]"#.to_owned(),
            "tiers/0/tag",
        ),
        (
            grouped(r#""label": 1, "#, &[a_trade("")]),
            "tiers/0/groups/0/label",
        ),
        (
            grouped("", &[a_trade(r#""comment": 1, "#)]),
            "tiers/0/groups/0/trades/0/comment",
        ),
        (
            grouped(
                "",
                &[r#"{"wants": [{"choice": [], "item": "minecraft:wheat"}],
                    "gives": [{"item": "minecraft:emerald"}]}"#
                    .to_owned()],
            ),
            "tiers/0/groups/0/trades/0/wants/0/item",
        ),
        (
            grouped(
                "",
                &[
                    r#"{"wants": [{"item": "minecraft:wheat", "quantity": {"min": 1, "mid": 2}}],
                    "gives": [{"item": "minecraft:emerald"}]}"#
                        .to_owned(),
                ],
            ),
            "tiers/0/groups/0/trades/0/wants/0/quantity/mid",
        ),
    ] {
        let d = rejects(&doc(Some("1.21.30"), &tiers));
        found_at(&d, "UnrecognizedField", Severity::Error, path);
        assert_eq!(above_info(&d).len(), 1, "{}", summary(&d));
        // The legacy parser ignores it (an information of ours (a)).
        if !path.ends_with("/item") {
            let document = loads(&doc(None, &tiers));
            assert_eq!(
                with(document.diagnostics(), "UnrecognizedField", Severity::Info).len(),
                1
            );
        }
    }
}

#[test]
fn a_third_root_member_fails_the_envelope_from_1_21_30() {
    let text = format!(
        r#"{{"format_version": "1.21.30", "note": 1, "tiers": {}}}"#,
        grouped("", &[a_trade("")])
    );
    let d = rejects(&text);
    assert_eq!(with(&d, "MalformedInput", Severity::Error).len(), 1);
    // Below 1.21.30 the legacy parser reads the whole document and ignores the member.
    let document = loads(&text.replace("1.21.30", "1.21.20"));
    found_at(
        document.diagnostics(),
        "UnrecognizedField",
        Severity::Info,
        "note",
    );
}

#[test]
fn f6_6_item_6_num_to_select_3_of_2_is_the_error_and_a_flat_weight_the_warning() {
    let d = rejects(&doc(
        None,
        &grouped(r#""num_to_select": 3, "#, &[a_trade(""), a_trade("")]),
    ));
    assert_eq!(
        only(&d).code.to_string(),
        "num_to_select[3] must not be greater than the number of trades[2]"
    );
    let document = loads(&doc(None, &flat(&[a_trade(r#""weight": 5, "#)])));
    assert_eq!(
        only(document.diagnostics()).code.to_string(),
        WEIGHT_NO_EFFECT
    );
}

// ---- how the legacy parser reads scalars (`Json::Value::as*`) ----------------------------

#[test]
fn legacy_scalars_convert_as_jsoncpp_does_and_each_conversion_is_told() {
    let trade = r#"{"wants": [{"item": true, "price_multiplier": "x"}],
        "gives": [{"item": "minecraft:emerald", "price_multiplier": 1}],
        "trader_exp": 2.9, "max_uses": true, "reward_exp": 0, "weight": 4.2}"#;
    let document = loads(&doc(
        None,
        &grouped(r#""num_to_select": 1.0, "#, &[trade.to_owned()]),
    ));
    // Vanilla converts in silence; each value read as another one than the file writes is a
    // warning at its member. `1.0` is the number 1: nothing to tell.
    let told: Vec<(&str, Severity, String)> = infos(document.diagnostics())
        .iter()
        .map(|f| (f.code.name(), f.severity, f.location.json_path.to_string()))
        .collect();
    let at = |member: &str| format!("tiers/0/groups/0/trades/0/{member}");
    assert_eq!(
        told,
        [
            (
                "InvalidType",
                Severity::Info,
                at("wants/0/price_multiplier")
            ),
            ("InvalidType", Severity::Info, at("wants/0/item")),
            ("InvalidValue", Severity::Info, at("trader_exp")),
            ("InvalidType", Severity::Info, at("max_uses")),
            ("InvalidType", Severity::Info, at("reward_exp")),
            ("InvalidValue", Severity::Info, at("weight")),
        ]
    );
    let group = &document.value().tiers[0].groups[0];
    assert_eq!(group.num_to_select, Some(1));
    let traded = &group.trades[0];
    assert_eq!(
        (
            traded.trader_exp,
            traded.max_uses,
            traded.reward_exp,
            traded.weight
        ),
        (Some(2), Some(1), Some(false), Some(4))
    );
    let wanted = &traded.wants[0].items()[0];
    assert_eq!(wanted.item.as_str(), "true");
    // A price multiplier that is no number reads as absent (vanilla: 0.0).
    assert_eq!(wanted.price_multiplier, None);
    assert_eq!(traded.gives[0].items()[0].price_multiplier, Some(1.0));
}

#[test]
fn a_legacy_scalar_no_conversion_takes_is_warned_and_reads_as_absent() {
    for (member, key, code) in [
        (r#""trader_exp": "2", "#, "trader_exp", "InvalidType"),
        (r#""trader_exp": -1, "#, "trader_exp", "InvalidValue"),
        (r#""max_uses": [7], "#, "max_uses", "InvalidType"),
        (r#""max_uses": 3000000000, "#, "max_uses", "InvalidValue"),
        (r#""reward_exp": "yes", "#, "reward_exp", "InvalidType"),
    ] {
        let document = loads(&doc(
            None,
            &grouped(r#""num_to_select": 1, "#, &[a_trade(member)]),
        ));
        let finding = only_info(document.diagnostics());
        assert_eq!(
            (finding.code.name(), finding.severity),
            (code, Severity::Info),
            "{member}"
        );
        assert_eq!(
            finding.location.json_path.to_string(),
            format!("tiers/0/groups/0/trades/0/{key}")
        );
        let traded = &document.value().tiers[0].groups[0].trades[0];
        assert_eq!(
            (traded.trader_exp, traded.max_uses, traded.reward_exp),
            (None, None, None)
        );
    }
    // An item that is an object reads as "".
    let trade = r#"{"wants": [{"item": {}}], "gives": [{"item": "minecraft:emerald"}]}"#;
    let document = loads(&doc(None, &flat(&[trade.to_owned()])));
    let finding = only_info(document.diagnostics());
    assert_eq!(finding.code.name(), "InvalidType");
    assert_eq!(
        finding.location.json_path.to_string(),
        "tiers/0/trades/0/wants/0/item"
    );
    let item = &document.value().tiers[0].groups[0].trades[0].wants[0].items()[0];
    assert_eq!(item.item.as_str(), "");
}

#[test]
fn a_legacy_entry_that_is_no_object_is_warned_and_dropped() {
    let trade = r#"{"wants": ["minecraft:wheat", {"item": "minecraft:carrot"}],
        "gives": [{"item": "minecraft:emerald"}]}"#;
    let document = loads(&doc(None, &flat(&[trade.to_owned()])));
    let finding = only_info(document.diagnostics());
    assert_eq!(finding.code.name(), "InvalidType");
    assert_eq!(
        finding.location.json_path.to_string(),
        "tiers/0/trades/0/wants/0"
    );
    assert_eq!(document.value().tiers[0].groups[0].trades[0].wants.len(), 1);
}

#[test]
fn a_legacy_tier_that_is_no_object_rejects_the_table() {
    let d = rejects(&doc(None, "[3]"));
    found_at(&d, "InvalidType", Severity::Error, "tiers/0");
}

#[test]
fn a_blank_file_is_an_empty_document_and_a_bom_is_skipped() {
    let d = rejects("  \n");
    assert_eq!(with(&d, "EmptyDocument", Severity::Error).len(), 1);
    let text = format!("\u{feff}{}", doc(None, &flat(&[a_trade("")])));
    assert!(above_info(loads(&text).diagnostics()).is_empty());
}

// ---- pins: both ends of every range, every half of every condition ---------------------

#[test]
fn a_legacy_real_beyond_an_integer_member_is_a_value_warning() {
    let document = loads(&doc(
        None,
        &grouped(
            r#""num_to_select": 1, "#,
            &[a_trade(r#""max_uses": 1e30, "#)],
        ),
    ));
    let finding = only_info(document.diagnostics());
    assert_eq!(finding.code.name(), "InvalidValue");
    assert_eq!(
        finding.location.json_path.to_string(),
        "tiers/0/groups/0/trades/0/max_uses"
    );
    // The ends of the integer members: `u32` takes 0 and its maximum, `i32` both of its ends.
    let ends = a_trade(r#""trader_exp": 4294967295, "max_uses": -2147483648, "#);
    let document = loads(&doc(None, &grouped(r#""num_to_select": 1, "#, &[ends])));
    assert!(above_info(document.diagnostics()).is_empty());
    let traded = &document.value().tiers[0].groups[0].trades[0];
    assert_eq!(
        (traded.trader_exp, traded.max_uses),
        (Some(u32::MAX), Some(i32::MIN))
    );
    let zero = a_trade(r#""trader_exp": 0, "max_uses": 2147483647, "reward_exp": false, "#);
    let document = loads(&doc(None, &grouped(r#""num_to_select": 1, "#, &[zero])));
    assert!(above_info(document.diagnostics()).is_empty());
    let traded = &document.value().tiers[0].groups[0].trades[0];
    assert_eq!(
        (traded.trader_exp, traded.max_uses, traded.reward_exp()),
        (Some(0), Some(i32::MAX), false)
    );
    for (member, key) in [
        (r#""trader_exp": 4294967296, "#, "trader_exp"),
        (r#""max_uses": 2147483648, "#, "max_uses"),
        (r#""max_uses": -2147483649, "#, "max_uses"),
    ] {
        let document = loads(&doc(
            None,
            &grouped(r#""num_to_select": 1, "#, &[a_trade(member)]),
        ));
        let finding = only_info(document.diagnostics());
        assert_eq!(finding.code.name(), "InvalidValue", "{member}");
        assert!(finding.location.json_path.to_string().ends_with(key));
    }
}

#[test]
fn a_legacy_group_or_trade_that_is_no_object_is_dropped_by_its_message() {
    // A group that is no object has no `trades`.
    let document = loads(&doc(None, r#"[{"groups": [3]}]"#));
    let finding = only(document.diagnostics());
    assert_eq!(finding.code.name(), "MissingRequiredField");
    assert_eq!(
        finding.location.json_path.to_string(),
        "tiers/0/groups/0/trades"
    );
    assert!(document.value().tiers[0].groups.is_empty());
    // A trade that is no object has no `wants`.
    let document = loads(&doc(None, &flat(&["5".to_owned(), a_trade("")])));
    let finding = only(document.diagnostics());
    assert_eq!(finding.code.name(), "InvalidValue");
    assert_eq!(
        finding.location.json_path.to_string(),
        "tiers/0/trades/0/wants"
    );
    assert_eq!(document.value().tiers[0].groups[0].trades.len(), 1);
    // A tier with neither `groups` nor `trades` is read as the flat form: no `trades`.
    let document = loads(&doc(None, "[{}]"));
    let finding = only(document.diagnostics());
    assert_eq!(finding.code.name(), "MissingRequiredField");
    assert_eq!(finding.location.json_path.to_string(), "tiers/0/trades");
    assert_eq!(finding.code.to_string(), "\"trades\" must be an array");
}

#[test]
fn a_legacy_choice_that_is_no_array_leaves_the_entry_a_bare_item() {
    use bedrock_addon_family_trade::ItemList;
    let trade = r#"{"wants": [{"item": "minecraft:wheat", "choice": null},
        {"item": "minecraft:carrot", "choice": {}}],
        "gives": [{"item": "minecraft:emerald"}]}"#;
    let document = loads(&doc(None, &flat(&[trade.to_owned()])));
    let wants = &document.value().tiers[0].groups[0].trades[0].wants;
    assert!(matches!(wants[0], ItemList::Item(_)) && matches!(wants[1], ItemList::Item(_)));
    assert_eq!(wants[1].items()[0].item.as_str(), "minecraft:carrot");
    // `choice` is then a member the item does not have.
    let d = document.diagnostics();
    found_at(
        d,
        "UnrecognizedField",
        Severity::Info,
        "tiers/0/trades/0/wants/0/choice",
    );
    found_at(
        d,
        "UnrecognizedField",
        Severity::Info,
        "tiers/0/trades/0/wants/1/choice",
    );
    assert!(above_info(d).is_empty());
}

#[test]
fn an_unknown_member_of_a_bare_stamped_item_is_an_error_too() {
    let trade = r#"{"wants": [{"item": "minecraft:wheat", "biomes": ["plains"]}],
        "gives": [{"item": "minecraft:emerald"}]}"#;
    let d = rejects(&doc(Some("1.21.30"), &grouped("", &[trade.to_owned()])));
    found_at(
        &d,
        "UnrecognizedField",
        Severity::Error,
        "tiers/0/groups/0/trades/0/wants/0/biomes",
    );
    assert_eq!(above_info(&d).len(), 1);
}

#[test]
fn a_stamped_filter_inside_a_choice_is_checked_at_its_own_path() {
    let trade = r#"{"wants": [{"item": "minecraft:wheat"}],
        "gives": [{"choice": [{"item": "minecraft:emerald"},
            {"item": "minecraft:bread", "filters": {"test": "no_such_test"}}]}]}"#;
    let document = loads(&doc(Some("1.21.30"), &grouped("", &[trade.to_owned()])));
    let finding = only(document.diagnostics());
    assert!(
        finding
            .location
            .json_path
            .to_string()
            .starts_with("tiers/0/groups/0/trades/0/gives/0/choice/1/filters"),
        "{}",
        finding.location.json_path
    );
}

#[test]
fn a_stamped_quantity_error_names_the_bound_and_the_kind() {
    let error = |quantity: &str| {
        let trade = format!(
            r#"{{"wants": [{{"item": "minecraft:wheat", "quantity": {quantity}}}],
                "gives": [{{"item": "minecraft:emerald"}}]}}"#
        );
        stamped_error(&grouped("", &[trade]))
    };
    let base = "tiers/0/groups/0/trades/0/wants/0/quantity";
    assert_eq!(error("0"), at("InvalidValue", base));
    assert_eq!(error("-1"), at("InvalidValue", base));
    assert_eq!(error("1.5"), at("InvalidType", base));
    assert_eq!(error("true"), at("InvalidType", base));
    assert_eq!(error(r#""2""#), at("InvalidType", base));
    assert_eq!(
        error(r#"{"min": 0}"#),
        at("InvalidValue", &format!("{base}/min"))
    );
    assert_eq!(
        error(r#"{"min": 1, "max": 0}"#),
        at("InvalidValue", &format!("{base}/max"))
    );
}

#[test]
fn the_legacy_reader_rejects_a_root_that_is_no_object() {
    // Reached through the struct's own parse (the loader's envelope probe rejects such a
    // document first, except `null`, which is the "must containt" error above).
    use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    use bedrock_addon_core::json::JsonSlice;
    use bedrock_addon_core::version::FormatVersionInput;
    use bedrock_addon_core::versioning::{ParseCx, VersionStruct};
    use bedrock_addon_family_trade::history::v0;
    let load = super::common::cx();
    let cx = ParseCx {
        load: &load,
        written: FormatVersionInput::parse("0.0.0").unwrap(),
        original_version: v0::VERSION,
        at: JsonPath::new(),
    };
    for (text, code) in [
        ("[]", "InvalidType"),
        ("3", "InvalidType"),
        ("{", "MalformedInput"),
    ] {
        let mut d = Diagnostics::default();
        assert!(v0::TradeTable::parse(JsonSlice::new(text), &cx, &mut d).is_none());
        assert_eq!(d.len(), 1, "{text}");
        let finding = d.iter().next().unwrap();
        assert_eq!(
            (finding.code.name(), finding.severity),
            (code, Severity::Error),
            "{text}"
        );
    }
}

#[test]
fn a_stamped_value_of_the_wrong_json_type_is_an_invalid_type_at_its_own_path() {
    // 07: a wrong JSON type is `InvalidType`, also where the value is a slice the parser reads
    // on its own (the `tiers` payload, a `wants` / `gives` entry, a `choice` element).
    let trade_with = |wants: &str| {
        grouped(
            "",
            &[format!(
                r#"{{"wants": {wants}, "gives": [{{"item": "minecraft:emerald"}}]}}"#
            )],
        )
    };
    let base = "tiers/0/groups/0/trades/0";
    for (tiers, path) in [
        ("{}".to_owned(), "tiers".to_owned()),
        (r#""none""#.to_owned(), "tiers".to_owned()),
        ("[3]".to_owned(), "tiers/0".to_owned()),
        (
            r#"[{"total_exp_required": 0, "groups": {}}]"#.to_owned(),
            "tiers/0/groups".to_owned(),
        ),
        (
            r#"[{"total_exp_required": 0, "groups": [true]}]"#.to_owned(),
            "tiers/0/groups/0".to_owned(),
        ),
        (
            r#"[{"total_exp_required": 0, "groups": [{"trades": ["x"]}]}]"#.to_owned(),
            base.to_owned(),
        ),
        (trade_with("{}"), format!("{base}/wants")),
        (
            trade_with(r#"["minecraft:wheat"]"#),
            format!("{base}/wants/0"),
        ),
        (trade_with("[7]"), format!("{base}/wants/0")),
        (trade_with("[[]]"), format!("{base}/wants/0")),
        (
            trade_with(r#"[{"choice": {}}]"#),
            format!("{base}/wants/0/choice"),
        ),
        (
            trade_with(r#"[{"choice": [3]}]"#),
            format!("{base}/wants/0/choice/0"),
        ),
        (
            trade_with(r#"[{"choice": ["minecraft:wheat"]}]"#),
            format!("{base}/wants/0/choice/0"),
        ),
    ] {
        let d = rejects(&doc(Some("1.21.30"), &tiers));
        let errors: Vec<(String, String)> = d
            .iter()
            .filter(|f| f.severity == Severity::Error)
            .map(|f| (f.code.name().to_owned(), f.location.json_path.to_string()))
            .collect();
        assert_eq!(errors, [at("InvalidType", &path)], "{tiers}");
    }
}

#[test]
fn an_array_given_to_a_stamped_scalar_is_an_invalid_type_at_the_member() {
    // The typed parse reports such a value at its first element; the finding is the member's.
    let item_with = |member: &str| {
        grouped(
            "",
            &[format!(
                r#"{{"wants": [{{{member}}}], "gives": [{{"item": "minecraft:emerald"}}]}}"#
            )],
        )
    };
    let trade = "tiers/0/groups/0/trades/0";
    for (tiers, path) in [
        (
            r#"[{"total_exp_required": [0], "groups": []}]"#.to_owned(),
            "tiers/0/total_exp_required".to_owned(),
        ),
        (
            grouped(r#""num_to_select": [1], "#, &[a_trade("")]),
            "tiers/0/groups/0/num_to_select".to_owned(),
        ),
        (
            grouped("", &[a_trade(r#""trader_exp": [1], "#)]),
            format!("{trade}/trader_exp"),
        ),
        (
            grouped("", &[a_trade(r#""max_uses": [7], "#)]),
            format!("{trade}/max_uses"),
        ),
        (
            grouped("", &[a_trade(r#""reward_exp": [true], "#)]),
            format!("{trade}/reward_exp"),
        ),
        (
            grouped("", &[a_trade(r#""weight": [1], "#)]),
            format!("{trade}/weight"),
        ),
        (
            item_with(r#""item": ["minecraft:wheat"]"#),
            format!("{trade}/wants/0/item"),
        ),
        (
            item_with(r#""item": "minecraft:wheat", "price_multiplier": [0.5]"#),
            format!("{trade}/wants/0/price_multiplier"),
        ),
        (
            item_with(r#""item": "minecraft:wheat", "quantity": {"min": [1]}"#),
            format!("{trade}/wants/0/quantity/min"),
        ),
        (
            item_with(r#""item": "minecraft:wheat", "quantity": {"max": [2]}"#),
            format!("{trade}/wants/0/quantity/max"),
        ),
    ] {
        assert_eq!(stamped_error(&tiers), at("InvalidType", &path), "{tiers}");
    }
}

/// `__DEP_fnctns` / `__DEP_fltrs` are bound and deprecated by name (`deprecateName`):
/// written in a 1.21.30+ document each is cereal's 0x200 `member "…" is deprecated`, an
/// error in both masks, and the table is rejected.
#[test]
fn an_author_written_dep_member_rejects_the_table() {
    let trade = r#"{"wants": [{"item": "minecraft:wheat",
        "__DEP_fnctns": [{"function": "set_data", "data": 1}],
        "__DEP_fltrs": {"test": "is_daytime"}}],
        "gives": [{"item": "minecraft:emerald"}]}"#;
    let d = rejects(&doc(Some("1.21.30"), &grouped("", &[trade.to_owned()])));
    let base = "tiers/0/groups/0/trades/0/wants/0";
    for member in ["__DEP_fnctns", "__DEP_fltrs"] {
        let finding = found_at(
            &d,
            "DeprecatedMember",
            Severity::Error,
            &format!("{base}/{member}"),
        );
        assert_eq!(
            finding.code.to_string(),
            format!("member \"{member}\" is deprecated")
        );
    }
}

/// below 1.21.30 the table is read by the legacy parser,
/// which binds no `__DEP_*` member: an author-written one is an unknown member, a warning, and
/// the table loads.
#[test]
fn an_author_written_dep_member_of_a_legacy_table_is_a_warning() {
    let trade = r#"{"wants": [{"item": "minecraft:wheat",
        "__DEP_fnctns": [{"function": "set_data", "data": 1}]}],
        "gives": [{"item": "minecraft:emerald"}]}"#;
    let document = loads(&doc(Some("1.20.30"), &grouped("", &[trade.to_owned()])));
    let d = document.diagnostics();
    assert!(!d.has_errors(), "{d:#?}");
    let told: Vec<String> = infos(d)
        .iter()
        .map(|f| {
            format!(
                "{:?} {} {}",
                f.severity,
                f.code.name(),
                f.location.json_path
            )
        })
        .collect();
    assert_eq!(
        told,
        ["Info UnrecognizedField tiers/0/groups/0/trades/0/wants/0/__DEP_fnctns"],
        "{d:#?}"
    );
}
