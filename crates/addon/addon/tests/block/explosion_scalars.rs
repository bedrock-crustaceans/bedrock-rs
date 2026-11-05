//! `minecraft:destructible_by_explosion`, read in a cereal document (the strict reader) and a
//! legacy one (the non-strict reader) from every JSON type at each of its scalar places: the
//! shorthand and `explosion_resistance`. Each line pins what the load says (code, severity,
//! path, span) and, when the block loads, the component written back. The component's members
//! are plain and read by the door's scalar pre-pass under the slice's `Reading`; they were
//! strict wrappers, and this table is what they read.

use super::common::{context, document, load, newest_shape};

/// The values of the component the table reads.
const VALUES: &[&str] = &[
    "true",
    "false",
    "5",
    "0.5",
    "\"true\"",
    "null",
    "[]",
    r#"{"explosion_resistance": 3}"#,
    r#"{"explosion_resistance": 3.5}"#,
    r#"{"explosion_resistance": 0.1}"#,
    r#"{"explosion_resistance": "3"}"#,
    r#"{"explosion_resistance": true}"#,
    r#"{"explosion_resistance": false}"#,
    r#"{"explosion_resistance": 1e39}"#,
    r#"{"explosion_resistance": null}"#,
    r#"{"explosion_resistance": -1}"#,
    r#"{"explosion_resistance": 18446744073709551616}"#,
    r#"{"explosion_resistance": [1]}"#,
    r#"{"explosion_resistance": 2, "zz": 1}"#,
    r#"{"explosion_resistance": 2, "explosion_resistance": "x"}"#,
    "{}",
];

/// One line of the table: the version, the value, and what the load gave.
fn line(version: &str, value: &str) -> String {
    let text = document(
        version,
        &format!(
            r#"{{"description": {{"identifier": "demo:x"}}, "components": {{"minecraft:destructible_by_explosion": {value}}}}}"#
        ),
    );
    let (loaded, diagnostics) = match load(&text, &context()) {
        Ok(document) => {
            let shape = newest_shape(&document);
            let written = shape
                .as_object()
                .and_then(|o| o.get("minecraft:block"))
                .and_then(|b| b.as_object())
                .and_then(|b| b.get("components"))
                .and_then(|c| c.as_object())
                .and_then(|c| c.get("minecraft:destructible_by_explosion"))
                .map(|v| bedrock_addon::json::to_facet_json(v).unwrap_or_default());
            (
                format!("loaded {}", written.unwrap_or_else(|| "-".to_owned())),
                document.diagnostics().clone(),
            )
        }
        Err(rejected) => ("rejected".to_owned(), rejected.diagnostics),
    };
    let findings: Vec<String> = diagnostics
        .iter()
        .map(|d| {
            let span = d
                .location
                .span
                .map_or("-".to_owned(), |s| format!("{}+{}", s.offset, s.len));
            format!(
                "{} {:?} {} {span}",
                d.code.name(),
                d.severity,
                d.location.json_path
            )
        })
        .collect();
    format!("{version} {value} => {loaded} [{}]", findings.join("; "))
}

#[test]
fn destructible_by_explosion_reads_every_scalar_as_pinned() {
    let mut table = String::new();
    for version in ["1.21.60", "1.26.20"] {
        for value in VALUES {
            table.push_str(&line(version, value));
            table.push('\n');
        }
    }
    let expected = EXPECTED;
    assert!(table == expected, "--- actual ---\n{table}--- end ---");
}

/// The table as the strict wrappers read it (`1.21.60`: the legacy slice, non-strict; `1.26.20`:
/// cereal, strict). A legacy load's findings carry no span.
const EXPECTED: &str = r#"1.21.60 true => loaded true []
1.21.60 false => loaded false []
1.21.60 5 => rejected [InvalidType Error minecraft:block/components/minecraft:destructible_by_explosion -]
1.21.60 0.5 => rejected [InvalidType Error minecraft:block/components/minecraft:destructible_by_explosion -]
1.21.60 "true" => rejected [InvalidType Error minecraft:block/components/minecraft:destructible_by_explosion -]
1.21.60 null => rejected [InvalidType Error minecraft:block/components/minecraft:destructible_by_explosion -]
1.21.60 [] => rejected [InvalidType Error minecraft:block/components/minecraft:destructible_by_explosion/0 -]
1.21.60 {"explosion_resistance": 3} => loaded {"explosion_resistance":3} []
1.21.60 {"explosion_resistance": 3.5} => loaded {"explosion_resistance":3.5} []
1.21.60 {"explosion_resistance": 0.1} => loaded {"explosion_resistance":0.1} []
1.21.60 {"explosion_resistance": "3"} => rejected [InvalidType Error minecraft:block/components/minecraft:destructible_by_explosion/explosion_resistance -]
1.21.60 {"explosion_resistance": true} => loaded {"explosion_resistance":1} []
1.21.60 {"explosion_resistance": false} => loaded {"explosion_resistance":0} []
1.21.60 {"explosion_resistance": 1e39} => rejected [InvalidType Error minecraft:block/components/minecraft:destructible_by_explosion/explosion_resistance -]
1.21.60 {"explosion_resistance": null} => rejected [InvalidType Error minecraft:block/components/minecraft:destructible_by_explosion/explosion_resistance -]
1.21.60 {"explosion_resistance": -1} => loaded {"explosion_resistance":-1} [InvalidValue Info minecraft:block/components/minecraft:destructible_by_explosion/explosion_resistance -]
1.21.60 {"explosion_resistance": 18446744073709551616} => loaded {"explosion_resistance":18446744000000000000} []
1.21.60 {"explosion_resistance": [1]} => rejected [InvalidType Error minecraft:block/components/minecraft:destructible_by_explosion/explosion_resistance/0 -]
1.21.60 {"explosion_resistance": 2, "zz": 1} => loaded {"explosion_resistance":2,"zz":1} [UnrecognizedField Warning minecraft:block/components/minecraft:destructible_by_explosion/zz -]
1.21.60 {"explosion_resistance": 2, "explosion_resistance": "x"} => rejected [InvalidType Error minecraft:block/components/minecraft:destructible_by_explosion/explosion_resistance -]
1.21.60 {} => rejected [MissingRequiredField Error minecraft:block/components/minecraft:destructible_by_explosion/explosion_resistance -]
1.26.20 true => loaded true []
1.26.20 false => loaded false []
1.26.20 5 => rejected [InvalidType Error minecraft:block/components/minecraft:destructible_by_explosion -]
1.26.20 0.5 => rejected [InvalidType Error minecraft:block/components/minecraft:destructible_by_explosion -]
1.26.20 "true" => rejected [InvalidType Error minecraft:block/components/minecraft:destructible_by_explosion -]
1.26.20 null => rejected [InvalidType Error minecraft:block/components/minecraft:destructible_by_explosion -]
1.26.20 [] => rejected [InvalidType Error minecraft:block/components/minecraft:destructible_by_explosion/0 146+1]
1.26.20 {"explosion_resistance": 3} => loaded {"explosion_resistance":3} []
1.26.20 {"explosion_resistance": 3.5} => loaded {"explosion_resistance":3.5} []
1.26.20 {"explosion_resistance": 0.1} => loaded {"explosion_resistance":0.1} []
1.26.20 {"explosion_resistance": "3"} => rejected [InvalidType Error minecraft:block/components/minecraft:destructible_by_explosion/explosion_resistance 146+22]
1.26.20 {"explosion_resistance": true} => rejected [InvalidType Error minecraft:block/components/minecraft:destructible_by_explosion/explosion_resistance 146+22]
1.26.20 {"explosion_resistance": false} => rejected [InvalidType Error minecraft:block/components/minecraft:destructible_by_explosion/explosion_resistance 146+22]
1.26.20 {"explosion_resistance": 1e39} => rejected [InvalidType Error minecraft:block/components/minecraft:destructible_by_explosion/explosion_resistance 146+22]
1.26.20 {"explosion_resistance": null} => rejected [InvalidType Error minecraft:block/components/minecraft:destructible_by_explosion/explosion_resistance 146+22]
1.26.20 {"explosion_resistance": -1} => rejected [InvalidValue Error minecraft:block/components/minecraft:destructible_by_explosion/explosion_resistance -]
1.26.20 {"explosion_resistance": 18446744073709551616} => loaded {"explosion_resistance":18446744000000000000} []
1.26.20 {"explosion_resistance": [1]} => rejected [InvalidType Error minecraft:block/components/minecraft:destructible_by_explosion/explosion_resistance/0 172+1]
1.26.20 {"explosion_resistance": 2, "zz": 1} => rejected [UnrecognizedField Error minecraft:block/components/minecraft:destructible_by_explosion/zz -]
1.26.20 {"explosion_resistance": 2, "explosion_resistance": "x"} => rejected [InvalidType Error minecraft:block/components/minecraft:destructible_by_explosion/explosion_resistance 173+22]
1.26.20 {} => rejected [MissingRequiredField Error minecraft:block/components/minecraft:destructible_by_explosion/explosion_resistance 146+1]
"#;
