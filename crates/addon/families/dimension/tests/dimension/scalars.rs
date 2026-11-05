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
                d.code.name(),
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
const EXPECTED: &str = r#"1.26.50 min_y "x" => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_height/min_y 141+7]
1.26.50 min_y "5" => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_height/min_y 141+7]
1.26.50 min_y 5 => rejected [InvalidValue Error minecraft:dimension/components/minecraft:dimension_height -]
1.26.50 min_y 0.5 => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_height/min_y 141+7]
1.26.50 min_y -1 => rejected [InvalidValue Error minecraft:dimension/components/minecraft:dimension_height -]
1.26.50 min_y true => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_height/min_y 141+7]
1.26.50 min_y null => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_height/min_y 141+7]
1.26.50 min_y 1e39 => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_height/min_y 141+7]
1.26.50 min_y 18446744073709551616 => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_height/min_y 141+7]
1.26.50 min_y [] => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_height/min_y/0 151+1]
1.26.50 min_y {} => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_height/min_y 151+1]
1.26.50 height_range "x" => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_height/height_range 155+14]
1.26.50 height_range "5" => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_height/height_range 155+14]
1.26.50 height_range 5 => rejected [InvalidValue Error minecraft:dimension/components/minecraft:dimension_height/height_range -]
1.26.50 height_range 0.5 => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_height/height_range 155+14]
1.26.50 height_range -1 => rejected [InvalidValue Error minecraft:dimension/components/minecraft:dimension_height/height_range -]
1.26.50 height_range true => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_height/height_range 155+14]
1.26.50 height_range null => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_height/height_range 155+14]
1.26.50 height_range 1e39 => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_height/height_range 155+14]
1.26.50 height_range 18446744073709551616 => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_height/height_range 155+14]
1.26.50 height_range [] => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_height/height_range/0 172+1]
1.26.50 height_range {} => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_height/height_range 172+1]
1.26.50 clouds.height "x" => rejected [InvalidType Error minecraft:dimension/components/minecraft:clouds/height 198+8]
1.26.50 clouds.height "5" => rejected [InvalidType Error minecraft:dimension/components/minecraft:clouds/height 198+8]
1.26.50 clouds.height 5 => loaded {"format_version": "1.26.50","minecraft:dimension": {"description": {"identifier": "test:d"},"components": {"minecraft:dimension_height": {"min_y": -64,"height_range": 384},"minecraft:clouds": {"height": 5},"minecraft:generation": {"generator_type": "void"}}}} []
1.26.50 clouds.height 0.5 => rejected [InvalidType Error minecraft:dimension/components/minecraft:clouds/height 198+8]
1.26.50 clouds.height -1 => loaded {"format_version": "1.26.50","minecraft:dimension": {"description": {"identifier": "test:d"},"components": {"minecraft:dimension_height": {"min_y": -64,"height_range": 384},"minecraft:clouds": {"height": -1},"minecraft:generation": {"generator_type": "void"}}}} []
1.26.50 clouds.height true => rejected [InvalidType Error minecraft:dimension/components/minecraft:clouds/height 198+8]
1.26.50 clouds.height null => rejected [InvalidType Error minecraft:dimension/components/minecraft:clouds/height 198+8]
1.26.50 clouds.height 1e39 => rejected [InvalidType Error minecraft:dimension/components/minecraft:clouds/height 198+8]
1.26.50 clouds.height 18446744073709551616 => rejected [InvalidType Error minecraft:dimension/components/minecraft:clouds/height 198+8]
1.26.50 clouds.height [] => rejected [InvalidType Error minecraft:dimension/components/minecraft:clouds/height/0 209+1]
1.26.50 clouds.height {} => rejected [InvalidType Error minecraft:dimension/components/minecraft:clouds/height 209+1]
1.26.50 generator_type "x" => loaded {"format_version": "1.26.50","minecraft:dimension": {"description": {"identifier": "test:d"},"components": {"minecraft:dimension_height": {"min_y": -64,"height_range": 384},"minecraft:clouds": {"height": 128},"minecraft:generation": {"generator_type": "x"}}}} [InvalidValue Error minecraft:dimension/components/minecraft:generation/generator_type -]
1.26.50 generator_type "5" => loaded {"format_version": "1.26.50","minecraft:dimension": {"description": {"identifier": "test:d"},"components": {"minecraft:dimension_height": {"min_y": -64,"height_range": 384},"minecraft:clouds": {"height": 128},"minecraft:generation": {"generator_type": "5"}}}} [InvalidValue Error minecraft:dimension/components/minecraft:generation/generator_type -]
1.26.50 generator_type 5 => rejected [InvalidType Error minecraft:dimension/components/minecraft:generation/generator_type 239+16]
1.26.50 generator_type 0.5 => rejected [InvalidType Error minecraft:dimension/components/minecraft:generation/generator_type 239+16]
1.26.50 generator_type -1 => rejected [InvalidType Error minecraft:dimension/components/minecraft:generation/generator_type 239+16]
1.26.50 generator_type true => rejected [InvalidType Error minecraft:dimension/components/minecraft:generation/generator_type 239+16]
1.26.50 generator_type null => rejected [InvalidType Error minecraft:dimension/components/minecraft:generation/generator_type 239+16]
1.26.50 generator_type 1e39 => rejected [InvalidType Error minecraft:dimension/components/minecraft:generation/generator_type 239+16]
1.26.50 generator_type 18446744073709551616 => rejected [InvalidType Error minecraft:dimension/components/minecraft:generation/generator_type 239+16]
1.26.50 generator_type [] => rejected [InvalidType Error minecraft:dimension/components/minecraft:generation/generator_type/0 258+1]
1.26.50 generator_type {} => rejected [InvalidType Error minecraft:dimension/components/minecraft:generation/generator_type 258+1]
1.21.60 bounds.min "x" => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_bounds/min 141+5]
1.21.60 bounds.min "5" => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_bounds/min 141+5]
1.21.60 bounds.min 5 => rejected [InvalidValue Error minecraft:dimension/components/minecraft:dimension_bounds/min -]
1.21.60 bounds.min 0.5 => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_bounds/min 141+5]
1.21.60 bounds.min -1 => rejected [InvalidValue Error minecraft:dimension/components/minecraft:dimension_bounds/min -]
1.21.60 bounds.min true => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_bounds/min 141+5]
1.21.60 bounds.min null => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_bounds/min 141+5]
1.21.60 bounds.min 1e39 => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_bounds/min 141+5]
1.21.60 bounds.min 18446744073709551616 => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_bounds/min 141+5]
1.21.60 bounds.min [] => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_bounds/min/0 149+1]
1.21.60 bounds.min {} => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_bounds/min 149+1]
1.21.60 bounds.max "x" => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_bounds/max 151+5]
1.21.60 bounds.max "5" => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_bounds/max 151+5]
1.21.60 bounds.max 5 => rejected [InvalidValue Error minecraft:dimension/components/minecraft:dimension_bounds/max -]
1.21.60 bounds.max 0.5 => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_bounds/max 151+5]
1.21.60 bounds.max -1 => rejected [InvalidValue Error minecraft:dimension/components/minecraft:dimension_bounds/max -]
1.21.60 bounds.max true => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_bounds/max 151+5]
1.21.60 bounds.max null => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_bounds/max 151+5]
1.21.60 bounds.max 1e39 => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_bounds/max 151+5]
1.21.60 bounds.max 18446744073709551616 => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_bounds/max 151+5]
1.21.60 bounds.max [] => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_bounds/max/0 159+1]
1.21.60 bounds.max {} => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_bounds/max 159+1]
1.21.60 generator_type "x" => loaded {"format_version": "1.26.50","minecraft:dimension": {"description": {"identifier": "test:d"},"components": {"minecraft:dimension_height": {"min_y": 0,"height_range": 256},"minecraft:generation": {"generator_type": "x"}}}} [InvalidValue Error minecraft:dimension/components/minecraft:generation/generator_type -]
1.21.60 generator_type "5" => loaded {"format_version": "1.26.50","minecraft:dimension": {"description": {"identifier": "test:d"},"components": {"minecraft:dimension_height": {"min_y": 0,"height_range": 256},"minecraft:generation": {"generator_type": "5"}}}} [InvalidValue Error minecraft:dimension/components/minecraft:generation/generator_type -]
1.21.60 generator_type 5 => rejected [InvalidType Error minecraft:dimension/components/minecraft:generation/generator_type 189+16]
1.21.60 generator_type 0.5 => rejected [InvalidType Error minecraft:dimension/components/minecraft:generation/generator_type 189+16]
1.21.60 generator_type -1 => rejected [InvalidType Error minecraft:dimension/components/minecraft:generation/generator_type 189+16]
1.21.60 generator_type true => rejected [InvalidType Error minecraft:dimension/components/minecraft:generation/generator_type 189+16]
1.21.60 generator_type null => rejected [InvalidType Error minecraft:dimension/components/minecraft:generation/generator_type 189+16]
1.21.60 generator_type 1e39 => rejected [InvalidType Error minecraft:dimension/components/minecraft:generation/generator_type 189+16]
1.21.60 generator_type 18446744073709551616 => rejected [InvalidType Error minecraft:dimension/components/minecraft:generation/generator_type 189+16]
1.21.60 generator_type [] => rejected [InvalidType Error minecraft:dimension/components/minecraft:generation/generator_type/0 208+1]
1.21.60 generator_type {} => rejected [InvalidType Error minecraft:dimension/components/minecraft:generation/generator_type 208+1]
1.18.0 bounds.min "x" => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_bounds/min 140+5]
1.18.0 bounds.min "5" => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_bounds/min 140+5]
1.18.0 bounds.min 5 => rejected [InvalidValue Error minecraft:dimension/components/minecraft:dimension_bounds/min -]
1.18.0 bounds.min 0.5 => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_bounds/min 140+5]
1.18.0 bounds.min -1 => rejected [InvalidValue Error minecraft:dimension/components/minecraft:dimension_bounds/min -]
1.18.0 bounds.min true => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_bounds/min 140+5]
1.18.0 bounds.min null => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_bounds/min 140+5]
1.18.0 bounds.min 1e39 => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_bounds/min 140+5]
1.18.0 bounds.min 18446744073709551616 => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_bounds/min 140+5]
1.18.0 bounds.min [] => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_bounds/min/0 148+1]
1.18.0 bounds.min {} => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_bounds/min 148+1]
1.18.0 bounds.max "x" => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_bounds/max 150+5]
1.18.0 bounds.max "5" => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_bounds/max 150+5]
1.18.0 bounds.max 5 => rejected [InvalidValue Error minecraft:dimension/components/minecraft:dimension_bounds/max -]
1.18.0 bounds.max 0.5 => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_bounds/max 150+5]
1.18.0 bounds.max -1 => rejected [InvalidValue Error minecraft:dimension/components/minecraft:dimension_bounds/max -]
1.18.0 bounds.max true => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_bounds/max 150+5]
1.18.0 bounds.max null => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_bounds/max 150+5]
1.18.0 bounds.max 1e39 => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_bounds/max 150+5]
1.18.0 bounds.max 18446744073709551616 => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_bounds/max 150+5]
1.18.0 bounds.max [] => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_bounds/max/0 158+1]
1.18.0 bounds.max {} => rejected [InvalidType Error minecraft:dimension/components/minecraft:dimension_bounds/max 158+1]
1.18.0 generator_type "x" => loaded {"format_version": "1.26.50","minecraft:dimension": {"description": {"identifier": "test:d"},"components": {"minecraft:dimension_height": {"min_y": 0,"height_range": 256},"minecraft:generation": {"generator_type": "x"}}}} [InvalidValue Error minecraft:dimension/components/minecraft:generation/generator_type -]
1.18.0 generator_type "5" => loaded {"format_version": "1.26.50","minecraft:dimension": {"description": {"identifier": "test:d"},"components": {"minecraft:dimension_height": {"min_y": 0,"height_range": 256},"minecraft:generation": {"generator_type": "5"}}}} [InvalidValue Error minecraft:dimension/components/minecraft:generation/generator_type -]
1.18.0 generator_type 5 => rejected [InvalidType Error minecraft:dimension/components/minecraft:generation/generator_type 188+16]
1.18.0 generator_type 0.5 => rejected [InvalidType Error minecraft:dimension/components/minecraft:generation/generator_type 188+16]
1.18.0 generator_type -1 => rejected [InvalidType Error minecraft:dimension/components/minecraft:generation/generator_type 188+16]
1.18.0 generator_type true => rejected [InvalidType Error minecraft:dimension/components/minecraft:generation/generator_type 188+16]
1.18.0 generator_type null => rejected [InvalidType Error minecraft:dimension/components/minecraft:generation/generator_type 188+16]
1.18.0 generator_type 1e39 => rejected [InvalidType Error minecraft:dimension/components/minecraft:generation/generator_type 188+16]
1.18.0 generator_type 18446744073709551616 => rejected [InvalidType Error minecraft:dimension/components/minecraft:generation/generator_type 188+16]
1.18.0 generator_type [] => rejected [InvalidType Error minecraft:dimension/components/minecraft:generation/generator_type/0 207+1]
1.18.0 generator_type {} => rejected [InvalidType Error minecraft:dimension/components/minecraft:generation/generator_type 207+1]
"#;
