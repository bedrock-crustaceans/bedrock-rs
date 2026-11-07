//! Every scalar member of a dimension, in the newest slice (1.26.50), the cereal one (1.21.60) and below it
//! (1.18.0), read from every JSON type: each line pins what the load says
//! (code, severity, path, span) and, when the document loads, the document written back. The
//! members were strict wrappers; this table is what they read, and the oracle of the change
//! that makes them plain.

use std::fmt::Write as _;

use super::common::load;

/// The values each member is read from.
const VALUES: &[&str] = &[
    "\"x\"",
    "\"5\"",
    "5",
    "0.5",
    "-1",
    "true",
    "null",
    "1e39",
    "18446744073709551616",
    "[]",
    "{}",
];

/// The documents the table reads, `@@` standing for the value.
const PLACES: &[(&str, &str)] = &[
    (
        "1.26.50 min_y",
        r#"{"format_version": "1.26.50", "minecraft:dimension": {"description": {"identifier": "test:d"}, "components": {"minecraft:dimension_height": {"min_y": @@, "height_range": 384}, "minecraft:clouds": {"height": 128}, "minecraft:generation": {"generator_type": "void"}}}}"#,
    ),
    (
        "1.26.50 height_range",
        r#"{"format_version": "1.26.50", "minecraft:dimension": {"description": {"identifier": "test:d"}, "components": {"minecraft:dimension_height": {"min_y": -64, "height_range": @@}, "minecraft:clouds": {"height": 128}, "minecraft:generation": {"generator_type": "void"}}}}"#,
    ),
    (
        "1.26.50 clouds.height",
        r#"{"format_version": "1.26.50", "minecraft:dimension": {"description": {"identifier": "test:d"}, "components": {"minecraft:dimension_height": {"min_y": -64, "height_range": 384}, "minecraft:clouds": {"height": @@}, "minecraft:generation": {"generator_type": "void"}}}}"#,
    ),
    (
        "1.26.50 generator_type",
        r#"{"format_version": "1.26.50", "minecraft:dimension": {"description": {"identifier": "test:d"}, "components": {"minecraft:dimension_height": {"min_y": -64, "height_range": 384}, "minecraft:clouds": {"height": 128}, "minecraft:generation": {"generator_type": @@}}}}"#,
    ),
    (
        "1.21.60 bounds.min",
        r#"{"format_version": "1.21.60", "minecraft:dimension": {"description": {"identifier": "test:d"}, "components": {"minecraft:dimension_bounds": {"min": @@, "max": 256}, "minecraft:generation": {"generator_type": "void"}}}}"#,
    ),
    (
        "1.21.60 bounds.max",
        r#"{"format_version": "1.21.60", "minecraft:dimension": {"description": {"identifier": "test:d"}, "components": {"minecraft:dimension_bounds": {"min": 0, "max": @@}, "minecraft:generation": {"generator_type": "void"}}}}"#,
    ),
    (
        "1.21.60 generator_type",
        r#"{"format_version": "1.21.60", "minecraft:dimension": {"description": {"identifier": "test:d"}, "components": {"minecraft:dimension_bounds": {"min": 0, "max": 256}, "minecraft:generation": {"generator_type": @@}}}}"#,
    ),
    (
        "1.18.0 bounds.min",
        r#"{"format_version": "1.18.0", "minecraft:dimension": {"description": {"identifier": "test:d"}, "components": {"minecraft:dimension_bounds": {"min": @@, "max": 256}, "minecraft:generation": {"generator_type": "void"}}}}"#,
    ),
    (
        "1.18.0 bounds.max",
        r#"{"format_version": "1.18.0", "minecraft:dimension": {"description": {"identifier": "test:d"}, "components": {"minecraft:dimension_bounds": {"min": 0, "max": @@}, "minecraft:generation": {"generator_type": "void"}}}}"#,
    ),
    (
        "1.18.0 generator_type",
        r#"{"format_version": "1.18.0", "minecraft:dimension": {"description": {"identifier": "test:d"}, "components": {"minecraft:dimension_bounds": {"min": 0, "max": 256}, "minecraft:generation": {"generator_type": @@}}}}"#,
    ),
];

/// `text` on one line: each line trimmed, the lines joined.
fn compact(text: &str) -> String {
    text.lines().map(str::trim).collect()
}

/// One line of the table: what loading `text` gave.
fn line(text: &str) -> String {
    let (loaded, diagnostics) = match load(text) {
        Ok(document) => (
            format!(
                "loaded {}",
                compact(&document.to_json().unwrap_or_default())
            ),
            document.diagnostics().clone(),
        ),
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
                d.code.id(),
                d.severity,
                d.location.json_path
            )
        })
        .collect();
    format!("{loaded} [{}]", findings.join("; "))
}

#[test]
fn every_scalar_member_reads_as_pinned() {
    let mut table = String::new();
    for (name, place) in PLACES {
        for value in VALUES {
            let _ = writeln!(
                table,
                "{name} {value} => {}",
                line(&place.replace("@@", value))
            );
        }
    }
    assert!(table == EXPECTED, "--- actual ---\n{table}--- end ---");
}

/// The table as the strict wrappers read it.
const EXPECTED: &str = r#"1.26.50 min_y "x" => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_height/min_y 141+7]
1.26.50 min_y "5" => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_height/min_y 141+7]
1.26.50 min_y 5 => rejected [dimension/validation_failed Error minecraft:dimension/components/minecraft:dimension_height -]
1.26.50 min_y 0.5 => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_height/min_y 141+7]
1.26.50 min_y -1 => rejected [dimension/validation_failed Error minecraft:dimension/components/minecraft:dimension_height -]
1.26.50 min_y true => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_height/min_y 141+7]
1.26.50 min_y null => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_height/min_y 141+7]
1.26.50 min_y 1e39 => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_height/min_y 141+7]
1.26.50 min_y 18446744073709551616 => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_height/min_y 141+7]
1.26.50 min_y [] => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_height/min_y/0 151+1]
1.26.50 min_y {} => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_height/min_y 151+1]
1.26.50 height_range "x" => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_height/height_range 155+14]
1.26.50 height_range "5" => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_height/height_range 155+14]
1.26.50 height_range 5 => rejected [core/outside_accepted_range Error minecraft:dimension/components/minecraft:dimension_height/height_range -]
1.26.50 height_range 0.5 => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_height/height_range 155+14]
1.26.50 height_range -1 => rejected [core/outside_accepted_range Error minecraft:dimension/components/minecraft:dimension_height/height_range -]
1.26.50 height_range true => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_height/height_range 155+14]
1.26.50 height_range null => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_height/height_range 155+14]
1.26.50 height_range 1e39 => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_height/height_range 155+14]
1.26.50 height_range 18446744073709551616 => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_height/height_range 155+14]
1.26.50 height_range [] => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_height/height_range/0 172+1]
1.26.50 height_range {} => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_height/height_range 172+1]
1.26.50 clouds.height "x" => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:clouds/height 198+8]
1.26.50 clouds.height "5" => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:clouds/height 198+8]
1.26.50 clouds.height 5 => loaded {"format_version": "1.26.50","minecraft:dimension": {"description": {"identifier": "test:d"},"components": {"minecraft:dimension_height": {"min_y": -64,"height_range": 384},"minecraft:clouds": {"height": 5},"minecraft:generation": {"generator_type": "void"}}}} []
1.26.50 clouds.height 0.5 => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:clouds/height 198+8]
1.26.50 clouds.height -1 => loaded {"format_version": "1.26.50","minecraft:dimension": {"description": {"identifier": "test:d"},"components": {"minecraft:dimension_height": {"min_y": -64,"height_range": 384},"minecraft:clouds": {"height": -1},"minecraft:generation": {"generator_type": "void"}}}} []
1.26.50 clouds.height true => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:clouds/height 198+8]
1.26.50 clouds.height null => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:clouds/height 198+8]
1.26.50 clouds.height 1e39 => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:clouds/height 198+8]
1.26.50 clouds.height 18446744073709551616 => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:clouds/height 198+8]
1.26.50 clouds.height [] => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:clouds/height/0 209+1]
1.26.50 clouds.height {} => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:clouds/height 209+1]
1.26.50 generator_type "x" => loaded {"format_version": "1.26.50","minecraft:dimension": {"description": {"identifier": "test:d"},"components": {"minecraft:dimension_height": {"min_y": -64,"height_range": 384},"minecraft:clouds": {"height": 128},"minecraft:generation": {"generator_type": "x"}}}} [dimension/generator_unsupported Error minecraft:dimension/components/minecraft:generation/generator_type -]
1.26.50 generator_type "5" => loaded {"format_version": "1.26.50","minecraft:dimension": {"description": {"identifier": "test:d"},"components": {"minecraft:dimension_height": {"min_y": -64,"height_range": 384},"minecraft:clouds": {"height": 128},"minecraft:generation": {"generator_type": "5"}}}} [dimension/generator_unsupported Error minecraft:dimension/components/minecraft:generation/generator_type -]
1.26.50 generator_type 5 => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:generation/generator_type 239+16]
1.26.50 generator_type 0.5 => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:generation/generator_type 239+16]
1.26.50 generator_type -1 => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:generation/generator_type 239+16]
1.26.50 generator_type true => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:generation/generator_type 239+16]
1.26.50 generator_type null => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:generation/generator_type 239+16]
1.26.50 generator_type 1e39 => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:generation/generator_type 239+16]
1.26.50 generator_type 18446744073709551616 => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:generation/generator_type 239+16]
1.26.50 generator_type [] => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:generation/generator_type/0 258+1]
1.26.50 generator_type {} => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:generation/generator_type 258+1]
1.21.60 bounds.min "x" => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_bounds/min 141+5]
1.21.60 bounds.min "5" => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_bounds/min 141+5]
1.21.60 bounds.min 5 => rejected [core/not_multiple_of Error minecraft:dimension/components/minecraft:dimension_bounds/min -]
1.21.60 bounds.min 0.5 => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_bounds/min 141+5]
1.21.60 bounds.min -1 => rejected [core/not_multiple_of Error minecraft:dimension/components/minecraft:dimension_bounds/min -]
1.21.60 bounds.min true => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_bounds/min 141+5]
1.21.60 bounds.min null => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_bounds/min 141+5]
1.21.60 bounds.min 1e39 => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_bounds/min 141+5]
1.21.60 bounds.min 18446744073709551616 => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_bounds/min 141+5]
1.21.60 bounds.min [] => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_bounds/min/0 149+1]
1.21.60 bounds.min {} => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_bounds/min 149+1]
1.21.60 bounds.max "x" => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_bounds/max 151+5]
1.21.60 bounds.max "5" => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_bounds/max 151+5]
1.21.60 bounds.max 5 => rejected [core/not_multiple_of Error minecraft:dimension/components/minecraft:dimension_bounds/max -]
1.21.60 bounds.max 0.5 => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_bounds/max 151+5]
1.21.60 bounds.max -1 => rejected [core/not_multiple_of Error minecraft:dimension/components/minecraft:dimension_bounds/max -]
1.21.60 bounds.max true => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_bounds/max 151+5]
1.21.60 bounds.max null => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_bounds/max 151+5]
1.21.60 bounds.max 1e39 => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_bounds/max 151+5]
1.21.60 bounds.max 18446744073709551616 => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_bounds/max 151+5]
1.21.60 bounds.max [] => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_bounds/max/0 159+1]
1.21.60 bounds.max {} => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_bounds/max 159+1]
1.21.60 generator_type "x" => loaded {"format_version": "1.26.50","minecraft:dimension": {"description": {"identifier": "test:d"},"components": {"minecraft:dimension_height": {"min_y": 0,"height_range": 256},"minecraft:generation": {"generator_type": "x"}}}} [dimension/generator_unsupported Error minecraft:dimension/components/minecraft:generation/generator_type -]
1.21.60 generator_type "5" => loaded {"format_version": "1.26.50","minecraft:dimension": {"description": {"identifier": "test:d"},"components": {"minecraft:dimension_height": {"min_y": 0,"height_range": 256},"minecraft:generation": {"generator_type": "5"}}}} [dimension/generator_unsupported Error minecraft:dimension/components/minecraft:generation/generator_type -]
1.21.60 generator_type 5 => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:generation/generator_type 189+16]
1.21.60 generator_type 0.5 => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:generation/generator_type 189+16]
1.21.60 generator_type -1 => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:generation/generator_type 189+16]
1.21.60 generator_type true => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:generation/generator_type 189+16]
1.21.60 generator_type null => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:generation/generator_type 189+16]
1.21.60 generator_type 1e39 => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:generation/generator_type 189+16]
1.21.60 generator_type 18446744073709551616 => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:generation/generator_type 189+16]
1.21.60 generator_type [] => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:generation/generator_type/0 208+1]
1.21.60 generator_type {} => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:generation/generator_type 208+1]
1.18.0 bounds.min "x" => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_bounds/min 140+5]
1.18.0 bounds.min "5" => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_bounds/min 140+5]
1.18.0 bounds.min 5 => rejected [core/not_multiple_of Error minecraft:dimension/components/minecraft:dimension_bounds/min -]
1.18.0 bounds.min 0.5 => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_bounds/min 140+5]
1.18.0 bounds.min -1 => rejected [core/not_multiple_of Error minecraft:dimension/components/minecraft:dimension_bounds/min -]
1.18.0 bounds.min true => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_bounds/min 140+5]
1.18.0 bounds.min null => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_bounds/min 140+5]
1.18.0 bounds.min 1e39 => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_bounds/min 140+5]
1.18.0 bounds.min 18446744073709551616 => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_bounds/min 140+5]
1.18.0 bounds.min [] => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_bounds/min/0 148+1]
1.18.0 bounds.min {} => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_bounds/min 148+1]
1.18.0 bounds.max "x" => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_bounds/max 150+5]
1.18.0 bounds.max "5" => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_bounds/max 150+5]
1.18.0 bounds.max 5 => rejected [core/not_multiple_of Error minecraft:dimension/components/minecraft:dimension_bounds/max -]
1.18.0 bounds.max 0.5 => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_bounds/max 150+5]
1.18.0 bounds.max -1 => rejected [core/not_multiple_of Error minecraft:dimension/components/minecraft:dimension_bounds/max -]
1.18.0 bounds.max true => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_bounds/max 150+5]
1.18.0 bounds.max null => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_bounds/max 150+5]
1.18.0 bounds.max 1e39 => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_bounds/max 150+5]
1.18.0 bounds.max 18446744073709551616 => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_bounds/max 150+5]
1.18.0 bounds.max [] => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_bounds/max/0 158+1]
1.18.0 bounds.max {} => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_bounds/max 158+1]
1.18.0 generator_type "x" => loaded {"format_version": "1.26.50","minecraft:dimension": {"description": {"identifier": "test:d"},"components": {"minecraft:dimension_height": {"min_y": 0,"height_range": 256},"minecraft:generation": {"generator_type": "x"}}}} [dimension/generator_unsupported Error minecraft:dimension/components/minecraft:generation/generator_type -]
1.18.0 generator_type "5" => loaded {"format_version": "1.26.50","minecraft:dimension": {"description": {"identifier": "test:d"},"components": {"minecraft:dimension_height": {"min_y": 0,"height_range": 256},"minecraft:generation": {"generator_type": "5"}}}} [dimension/generator_unsupported Error minecraft:dimension/components/minecraft:generation/generator_type -]
1.18.0 generator_type 5 => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:generation/generator_type 188+16]
1.18.0 generator_type 0.5 => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:generation/generator_type 188+16]
1.18.0 generator_type -1 => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:generation/generator_type 188+16]
1.18.0 generator_type true => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:generation/generator_type 188+16]
1.18.0 generator_type null => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:generation/generator_type 188+16]
1.18.0 generator_type 1e39 => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:generation/generator_type 188+16]
1.18.0 generator_type 18446744073709551616 => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:generation/generator_type 188+16]
1.18.0 generator_type [] => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:generation/generator_type/0 207+1]
1.18.0 generator_type {} => rejected [core/invalid_type Error minecraft:dimension/components/minecraft:generation/generator_type 207+1]
"#;

/// Documents with a struct member written `null`: what the slice's null rule makes of it,
/// told once (rejected where the input rejects nulls, read as its defaults elsewhere).
const NULL_STRUCTS: &[&str] = &[
    r#"{"format_version": "1.26.50", "minecraft:dimension": {"description": {"identifier": "test:d"}, "components": {"minecraft:dimension_height": null, "minecraft:clouds": {"height": 128}, "minecraft:generation": {"generator_type": "void"}}}}"#,
    r#"{"format_version": "1.21.60", "minecraft:dimension": {"description": {"identifier": "test:d"}, "components": {"minecraft:dimension_bounds": null, "minecraft:generation": {"generator_type": "void"}}}}"#,
    r#"{"format_version": "1.18.0", "minecraft:dimension": {"description": {"identifier": "test:d"}, "components": {"minecraft:dimension_bounds": null, "minecraft:generation": {"generator_type": "void"}}}}"#,
    r#"{"format_version": "1.18.0", "minecraft:dimension": {"description": {"identifier": "test:d"}, "components": {"minecraft:dimension_bounds": {"min": 0, "max": 256}, "minecraft:generation": null}}}"#,
];

#[test]
fn a_null_struct_member_reads_as_pinned() {
    let mut table = String::new();
    for text in NULL_STRUCTS {
        let _ = writeln!(table, "{}", line(text));
    }
    assert!(
        table == NULL_STRUCTS_EXPECTED,
        "--- actual ---\n{table}--- end ---"
    );
}

/// The null struct table.
const NULL_STRUCTS_EXPECTED: &str = r"rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_height 140+4]
rejected [core/invalid_type Error minecraft:dimension/components/minecraft:dimension_bounds 140+4]
rejected [dimension/validation_failed Error minecraft:dimension/components/minecraft:dimension_bounds -]
rejected [core/invalid_type Error minecraft:dimension/components/minecraft:generation 187+4]
";

/// The bind constraints at and past their ends, with the text of each finding: `@@` is the
/// value of the member the row names.
const CONSTRAINTS: &[(&str, &str, &[&str])] = &[
    (
        "1.26.50 min_y",
        r#"{"format_version": "1.26.50", "minecraft:dimension": {"description": {"identifier": "test:d"}, "components": {"minecraft:dimension_height": {"min_y": @@, "height_range": 384}, "minecraft:generation": {"generator_type": "void"}}}}"#,
        &["-512", "-513", "512", "513", "-8"],
    ),
    (
        "1.26.50 height_range",
        r#"{"format_version": "1.26.50", "minecraft:dimension": {"description": {"identifier": "test:d"}, "components": {"minecraft:dimension_height": {"min_y": 0, "height_range": @@}, "minecraft:generation": {"generator_type": "void"}}}}"#,
        &["16", "0", "1024", "1040", "40", "1030"],
    ),
    (
        "1.26.50 both",
        r#"{"format_version": "1.26.50", "minecraft:dimension": {"description": {"identifier": "test:d"}, "components": {"minecraft:dimension_height": {"min_y": -600, "height_range": @@}, "minecraft:clouds": {"height": -600}, "minecraft:generation": {"generator_type": "void"}}}}"#,
        &["40", "0"],
    ),
    (
        "1.26.50 clouds.height",
        r#"{"format_version": "1.26.50", "minecraft:dimension": {"description": {"identifier": "test:d"}, "components": {"minecraft:clouds": {"height": @@}, "minecraft:generation": {"generator_type": "void"}}}}"#,
        &["-512", "-513", "512", "513", "7"],
    ),
    (
        "1.21.60 bounds",
        r#"{"format_version": "1.21.60", "minecraft:dimension": {"description": {"identifier": "test:d"}, "components": {"minecraft:dimension_bounds": {"min": @@, "max": @@}, "minecraft:generation": {"generator_type": "void"}}}}"#,
        &["-512", "512", "-528", "528", "8", "-520"],
    ),
    (
        "1.18.0 bounds",
        r#"{"format_version": "1.18.0", "minecraft:dimension": {"description": {"identifier": "test:d"}, "components": {"minecraft:dimension_bounds": {"min": 0, "max": @@}, "minecraft:generation": {"generator_type": "void"}}}}"#,
        &["512", "528", "24"],
    ),
];

#[test]
fn the_bind_constraints_read_as_pinned() {
    let mut table = String::new();
    for (name, place, values) in CONSTRAINTS {
        for value in *values {
            let diagnostics = match load(&place.replace("@@", value)) {
                Ok(document) => document.diagnostics().clone(),
                Err(rejected) => rejected.diagnostics,
            };
            let findings: Vec<String> = diagnostics
                .iter()
                .map(|d| {
                    format!(
                        "{} {:?} {}",
                        bedrock_addon_core::diagnostic::testing::shown(d),
                        d.severity,
                        d.location.json_path
                    )
                })
                .collect();
            let _ = writeln!(table, "{name} {value} => [{}]", findings.join("; "));
        }
    }
    assert!(
        table == CONSTRAINTS_EXPECTED,
        "--- actual ---\n{table}--- end ---"
    );
}

/// The constraints table.
const CONSTRAINTS_EXPECTED: &str = r#"1.26.50 min_y -512 => []
1.26.50 min_y -513 => [core/outside_accepted_range ["[-512, 512]"] Error minecraft:dimension/components/minecraft:dimension_height/min_y]
1.26.50 min_y 512 => [dimension/validation_failed ["Dimension height range: min_y plus height_range must not exceed 512", "Invalid dimension height."] Error minecraft:dimension/components/minecraft:dimension_height]
1.26.50 min_y 513 => [core/outside_accepted_range ["[-512, 512]"] Error minecraft:dimension/components/minecraft:dimension_height/min_y]
1.26.50 min_y -8 => [dimension/validation_failed ["Dimension min y: min_y must be divisible by 16", "Invalid dimension height."] Error minecraft:dimension/components/minecraft:dimension_height]
1.26.50 height_range 16 => []
1.26.50 height_range 0 => [core/outside_accepted_range ["[16, 1024]"] Error minecraft:dimension/components/minecraft:dimension_height/height_range]
1.26.50 height_range 1024 => [dimension/validation_failed ["Dimension height range: min_y plus height_range must not exceed 512", "Invalid dimension height."] Error minecraft:dimension/components/minecraft:dimension_height]
1.26.50 height_range 1040 => [core/outside_accepted_range ["[16, 1024]"] Error minecraft:dimension/components/minecraft:dimension_height/height_range]
1.26.50 height_range 40 => [core/not_multiple_of ["40", "16"] Error minecraft:dimension/components/minecraft:dimension_height/height_range]
1.26.50 height_range 1030 => [core/outside_accepted_range ["[16, 1024]"] Error minecraft:dimension/components/minecraft:dimension_height/height_range]
1.26.50 both 40 => [core/outside_accepted_range ["[-512, 512]"] Error minecraft:dimension/components/minecraft:dimension_height/min_y; core/not_multiple_of ["40", "16"] Error minecraft:dimension/components/minecraft:dimension_height/height_range; core/outside_accepted_range ["[-512, 512]"] Error minecraft:dimension/components/minecraft:clouds/height]
1.26.50 both 0 => [core/outside_accepted_range ["[-512, 512]"] Error minecraft:dimension/components/minecraft:dimension_height/min_y; core/outside_accepted_range ["[16, 1024]"] Error minecraft:dimension/components/minecraft:dimension_height/height_range; core/outside_accepted_range ["[-512, 512]"] Error minecraft:dimension/components/minecraft:clouds/height]
1.26.50 clouds.height -512 => []
1.26.50 clouds.height -513 => [core/outside_accepted_range ["[-512, 512]"] Error minecraft:dimension/components/minecraft:clouds/height]
1.26.50 clouds.height 512 => []
1.26.50 clouds.height 513 => [core/outside_accepted_range ["[-512, 512]"] Error minecraft:dimension/components/minecraft:clouds/height]
1.26.50 clouds.height 7 => []
1.21.60 bounds -512 => [dimension/validation_failed ["Dimension height range: height_range must be greater than zero", "Invalid dimension height."] Error minecraft:dimension/components/minecraft:dimension_bounds]
1.21.60 bounds 512 => [dimension/validation_failed ["Dimension height range: height_range must be greater than zero", "Invalid dimension height."] Error minecraft:dimension/components/minecraft:dimension_bounds]
1.21.60 bounds -528 => [core/outside_accepted_range ["[-512, 512]"] Error minecraft:dimension/components/minecraft:dimension_bounds/min; core/outside_accepted_range ["[-512, 512]"] Error minecraft:dimension/components/minecraft:dimension_bounds/max]
1.21.60 bounds 528 => [core/outside_accepted_range ["[-512, 512]"] Error minecraft:dimension/components/minecraft:dimension_bounds/min; core/outside_accepted_range ["[-512, 512]"] Error minecraft:dimension/components/minecraft:dimension_bounds/max]
1.21.60 bounds 8 => [core/not_multiple_of ["8", "16"] Error minecraft:dimension/components/minecraft:dimension_bounds/min; core/not_multiple_of ["8", "16"] Error minecraft:dimension/components/minecraft:dimension_bounds/max]
1.21.60 bounds -520 => [core/outside_accepted_range ["[-512, 512]"] Error minecraft:dimension/components/minecraft:dimension_bounds/min; core/outside_accepted_range ["[-512, 512]"] Error minecraft:dimension/components/minecraft:dimension_bounds/max]
1.18.0 bounds 512 => []
1.18.0 bounds 528 => [core/outside_accepted_range ["[-512, 512]"] Error minecraft:dimension/components/minecraft:dimension_bounds/max]
1.18.0 bounds 24 => [core/not_multiple_of ["24", "16"] Error minecraft:dimension/components/minecraft:dimension_bounds/max]
"#;
