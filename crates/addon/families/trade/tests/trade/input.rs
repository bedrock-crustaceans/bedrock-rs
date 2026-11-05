//! The input the game reads a trade table from
//! (a `MinecraftDocumentInput`, read
//! by the game's strict schema reader): `initAsJson` picks the version from the text (a number
//! and an array are versions, `null`, a bool or an object is "Malformed format version"), the
//! legacy parser then reads its own version only from a string (0.0.0 otherwise), and a member
//! written as `null` rejects the table at the 1.21.30 cereal slice.

use bedrock_addon_core::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon_core::version::FormatVersion;

use super::common::{doc, found_at, loads, rejects};

/// One flat 1.21.30 tier whose one trade has `extra` members first (each ending in a comma).
fn cereal(extra: &str) -> String {
    doc(
        Some("1.21.30"),
        &format!(
            r#"[{{"total_exp_required": 0, "groups": [{{"trades": [{{{extra} "wants": [{{"item": "minecraft:wheat"}}], "gives": [{{"item": "minecraft:emerald"}}]}}]}}]}}]"#
        ),
    )
}

/// The same table with the root `format_version` written as `version` (JSON text).
fn with_version(version: &str) -> String {
    cereal("").replace(r#""1.21.30""#, version)
}

#[test]
fn a_format_version_that_gives_no_version_is_malformed() {
    for version in ["null", "true", "{}", r#""abc""#, "[1, 21]"] {
        let d = rejects(&with_version(version));
        let errors: Vec<_> = d.iter().filter(|f| f.severity == Severity::Error).collect();
        assert_eq!(errors.len(), 1, "{version}: {d:#?}");
        assert!(
            matches!(
                errors[0].code,
                DiagnosticCode::MalformedFormatVersion { .. }
            ),
            "{version}: {:?}",
            errors[0].code
        );
    }
}

#[test]
fn a_number_or_an_array_at_1_21_30_or_above_selects_the_cereal_slice() {
    for (version, written) in [
        ("1.22", FormatVersion::new(1, 22, 0)),
        ("[1, 21, 30]", FormatVersion::new(1, 21, 30)),
    ] {
        let document = loads(&with_version(version));
        assert_eq!(document.version().written, Some(written), "{version}");
        assert_eq!(
            document.version().parsed_as,
            FormatVersion::new(1, 21, 30),
            "{version}"
        );
    }
}

#[test]
fn a_number_or_an_array_below_1_21_30_is_the_legacy_parser_s_0_0_0() {
    // The tight lower side of the cereal slice: 1.21.29 and [1, 21, 29] are legacy.
    for version in ["1.18", "[1, 18, 10]", "[1, 21, 29]"] {
        let document = loads(&with_version(version));
        assert_eq!(
            document.version().effective,
            FormatVersion::new(0, 0, 0),
            "{version}"
        );
    }
}

#[test]
fn a_null_member_of_a_cereal_table_rejects_it() {
    for (extra, path) in [
        (r#""weight": null,"#, "tiers/0/groups/0/trades/0/weight"),
        (r#""max_uses": null,"#, "tiers/0/groups/0/trades/0/max_uses"),
        (
            r#""reward_exp": null,"#,
            "tiers/0/groups/0/trades/0/reward_exp",
        ),
    ] {
        let d = rejects(&cereal(extra));
        found_at(&d, "InvalidType", Severity::Error, path);
    }
    let group = cereal("").replace(r#""groups": [{"#, r#""groups": [{"num_to_select": null, "#);
    found_at(
        &rejects(&group),
        "InvalidType",
        Severity::Error,
        "tiers/0/groups/0/num_to_select",
    );
    let quantity = cereal("").replace(
        r#""wants": [{"item": "minecraft:wheat"}]"#,
        r#""wants": [{"item": "minecraft:wheat", "quantity": null}]"#,
    );
    found_at(
        &rejects(&quantity),
        "InvalidType",
        Severity::Error,
        "tiers/0/groups/0/trades/0/wants/0/quantity",
    );
    let bound = cereal("").replace(
        r#""wants": [{"item": "minecraft:wheat"}]"#,
        r#""wants": [{"item": "minecraft:wheat", "quantity": {"min": null, "max": 2}}]"#,
    );
    found_at(
        &rejects(&bound),
        "InvalidType",
        Severity::Error,
        "tiers/0/groups/0/trades/0/wants/0/quantity/min",
    );
}

#[test]
fn a_null_member_of_a_legacy_table_is_the_legacy_parser_s() {
    // The hand-written legacy parser below 1.21.30 is not a cereal reader:
    // its `null` handling stays the family's: `"weight": null` is read as no weight.
    let table = doc(
        None,
        r#"[{"trades": [{"weight": null, "wants": [{"item": "minecraft:wheat"}], "gives": [{"item": "minecraft:emerald"}]}]}]"#,
    );
    let document = loads(&table);
    let trade = &document.value().tiers[0].groups[0].trades[0];
    assert_eq!(trade.weight, None);
    assert_eq!(trade.wants[0].items()[0].item.as_str(), "minecraft:wheat");
}

/// The game's own trade table test: its
/// template filled with the test's own arguments ("1.21.30", 2, 1, "test:offer",
/// "test:receive", 6) loads as the newest struct with the values the
/// test expects, and round-trips at the value level.
#[test]
fn the_engine_test_s_table_loads_with_its_expected_values_and_round_trips() {
    let text = super::common::fixture("engine/case-1.json");
    let document = loads(&text);
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    assert_eq!(document.version().parsed_as, FormatVersion::new(1, 21, 30));
    let table = document.value();
    assert_eq!(table.tiers.len(), 1, "Expected number of tiers to match");
    assert_eq!(
        table.tiers[0].total_exp_required, 2,
        "Expected exp to unlock to match"
    );
    assert_eq!(
        table.tiers[0].groups.len(),
        1,
        "Expected number of groups to match"
    );
    let group = &table.tiers[0].groups[0];
    assert_eq!(
        group.num_to_select,
        Some(1),
        "Expected number to select to match"
    );
    assert_eq!(group.trades.len(), 1, "Expected number of trades to match");
    let trade = &group.trades[0];
    assert_eq!(trade.max_uses, Some(6));
    assert_eq!(trade.wants[0].items()[0].item.as_str(), "test:offer");
    assert_eq!(trade.gives[0].items()[0].item.as_str(), "test:receive");
    let written = document.to_json().unwrap();
    let again = loads(&written);
    assert_eq!(again.value(), document.value());
    assert_eq!(again.to_json().unwrap(), written);
}
