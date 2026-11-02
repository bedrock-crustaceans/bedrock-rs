//! The loot texts the game writes with
//! `ContentLogHelper::_contentLog(true, LogLevel 3, …)` while it keeps the table are kept
//! errors. Each row: the table, the path, the text; the table loads and the finding is
//! an error marked `kept`.

use bedrock_addon_core::diagnostic::Severity;

use super::common::{cx_at, findings, load_in, path, text};

/// A table whose one pool has `conditions` and no entries.
fn with_conditions(conditions: &str) -> String {
    format!(r#"{{"pools": [{{"rolls": 1, "conditions": [{conditions}], "entries": []}}]}}"#)
}

/// A table whose one pool has the entries `entries`.
fn with_entries(entries: &str) -> String {
    format!(r#"{{"pools": [{{"rolls": 1, "entries": [{entries}]}}]}}"#)
}

/// A table whose one entry has the functions `functions`.
fn with_functions(functions: &str) -> String {
    with_entries(&format!(
        r#"{{"type": "item", "name": "minecraft:stick", "functions": [{functions}]}}"#
    ))
}

#[test]
fn the_game_s_content_log_errors_of_a_kept_table_are_kept_errors() {
    const F0: &str = "pools/0/entries/0/functions/0";
    let pattern = r#"{"pattern": "bri", "color": "red"}"#;
    let seven = [pattern; 7].join(", ");
    let rows: Vec<(String, bool, String, String)> = vec![
        (
            with_conditions("7"),
            false,
            "pools/0/conditions/0".into(),
            "Error loading loot table \"conditions\" as it is not a valid Json object. \"conditions\" must be key: value pairs".into(),
        ),
        (
            with_conditions(r#"{"condition": "random_chance", "chance": 0.5, "operator": "<"}"#),
            false,
            "pools/0/conditions/0".into(),
            "Invalid loot condition operator \"<\". Expected \"==\", \"equals\", \"!=\", or \"not\".".into(),
        ),
        (
            with_functions("7"),
            false,
            F0.into(),
            "Error loading loot table \"functions\" as it is not a valid Json object. \"functions\" must be key: value pairs".into(),
        ),
        (
            with_functions(r#"{"function": "nope"}"#),
            false,
            F0.into(),
            "Unsupported looting function: nope".into(),
        ),
        (
            with_functions(r#"{"function": "carry_over_block_entity_data"}"#),
            false,
            F0.into(),
            "Unsupported looting function outside of Upcoming Creator Features Experiment: carry_over_block_entity_data".into(),
        ),
        (
            with_entries("7"),
            false,
            "pools/0/entries/0".into(),
            "Every entry of \"entries\" must be a JSON object - it has to start with an \"{\" and end with a \"}\"".into(),
        ),
        (
            with_entries(r#"{"type": "nope"}"#),
            false,
            "pools/0/entries/0".into(),
            "Every entry of \"entries\" must specify a type of either \"item\", \"loot_table\", or \"empty\". Given: nope".into(),
        ),
        (
            with_functions(r#"{"function": "carry_over_block_entity_data", "dynamic_properties": "yes"}"#),
            true,
            format!("{F0}/dynamic_properties"),
            "Error loading loot table \"carry_over_block_entity_data\" dynamic_properties as it is not a bool".into(),
        ),
        (
            with_functions(r#"{"function": "enchant_book_for_trading", "enchantments": 7}"#),
            false,
            format!("{F0}/enchantments"),
            "Error loading \"enchant_book_for_trading\" function, \"enchantments\": should be type array".into(),
        ),
        (
            with_functions(r#"{"function": "set_banner_details", "type": "x"}"#),
            false,
            format!("{F0}/type"),
            "Invalid banner type input: \"x\"\n".into(),
        ),
        (
            with_functions(r#"{"function": "set_banner_details", "base_color": "x"}"#),
            false,
            format!("{F0}/base_color"),
            "Invalid banner base_color input: \"x\"\n".into(),
        ),
        (
            with_functions(&format!(
                r#"{{"function": "set_banner_details", "type": "illager_captain", "patterns": [{pattern}]}}"#
            )),
            false,
            format!("{F0}/patterns"),
            "Non-default banner types cannot have custom patterns".into(),
        ),
        (
            with_functions(&format!(
                r#"{{"function": "set_banner_details", "patterns": [{seven}]}}"#
            )),
            false,
            format!("{F0}/patterns"),
            "Banner exceeds maximum number of 6 patterns! Patterns given: 7".into(),
        ),
        (
            with_functions(r#"{"function": "set_banner_details", "patterns": [{"color": "red"}]}"#),
            false,
            format!("{F0}/patterns/0"),
            "Invalid banner pattern name: ".into(),
        ),
        (
            with_functions(r#"{"function": "set_banner_details", "patterns": [{"pattern": "bri", "color": "x"}]}"#),
            false,
            format!("{F0}/patterns/0/color"),
            "Invalid color input in banner pattern \"x\"\n".into(),
        ),
    ];
    let mut wrong = Vec::new();
    for (table, ucf, at, expected) in rows {
        let document = match load_in(&table, &cx_at(1, 26, 60, ucf)) {
            Ok(document) => document,
            Err(rejected) => {
                wrong.push(format!("{expected}: refused {rejected}"));
                continue;
            }
        };
        let found = findings(&document);
        let hit = found.iter().find(|f| text(&f.code) == expected);
        match hit {
            Some(f) if f.severity == Severity::Error && f.kept && path(f) == at => {}
            other => wrong.push(format!("{expected}: {other:?} in {found:?}")),
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
