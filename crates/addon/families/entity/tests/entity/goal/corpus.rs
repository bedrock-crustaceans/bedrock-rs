//! The vanilla corpus: every goal of every vanilla entity file reads (strictly at or
//! above the goal's typed version, leniently below it) with no error and no warning, and reads
//! back equal after writing. Needs `BEDROCK_ADDON_CORPUS`; skipped without it.

use std::path::{Path, PathBuf};

use bedrock_addon_core::json::Value;
use bedrock_addon_core::versioning::ConvertMode;

use super::types::GOALS;

/// The format version `text` names, as three numbers (missing parts 0).
fn version(text: &str) -> (u16, u16, u16) {
    let mut parts = text.split('.').map(|p| p.parse::<u16>().unwrap_or(0));
    (
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
    )
}

/// Every `.json` file below `dir`.
fn json_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            json_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "json") {
            out.push(path);
        }
    }
}

/// Each `(key, value)` of `components` and of every component group of an entity payload.
fn goal_entries(payload: &Value) -> Vec<(String, Value)> {
    let mut out = Vec::new();
    let Some(payload) = payload.as_object() else {
        return out;
    };
    let mut sections = Vec::new();
    if let Some(components) = payload.get("components") {
        sections.push(components.clone());
    }
    if let Some(groups) = payload.get("component_groups").and_then(Value::as_object) {
        sections.extend(groups.iter().map(|(_, group)| group.clone()));
    }
    for section in sections {
        if let Some(section) = section.as_object() {
            for (key, value) in section {
                out.push((key.as_str().to_owned(), value.clone()));
            }
        }
    }
    out
}

#[test]
fn every_vanilla_goal_reads_cleanly_and_round_trips() {
    let Some(corpus) = std::env::var_os("BEDROCK_ADDON_CORPUS").map(PathBuf::from) else {
        eprintln!("skipped: BEDROCK_ADDON_CORPUS is not set");
        return;
    };
    let mut files = Vec::new();
    json_files(
        &corpus.join("bedrock-samples/behavior_pack/entities"),
        &mut files,
    );
    files.sort();
    assert!(!files.is_empty(), "the corpus has entity files");
    let mut read = 0;
    let mut problems = Vec::new();
    let mut unknown = Vec::new();
    for file in &files {
        let text = std::fs::read_to_string(file).expect("the file reads");
        let text = text.trim_start_matches('\u{feff}');
        let Ok(document) = facet_json::from_str_jsonc::<Value>(text) else {
            continue;
        };
        let Some(document) = document.as_object() else {
            continue;
        };
        let doc_version = document
            .get("format_version")
            .and_then(Value::as_string)
            .map_or((1, 8, 0), |v| version(v.as_str()));
        let Some(payload) = document.get("minecraft:entity") else {
            continue;
        };
        for (key, value) in goal_entries(payload) {
            let Some((_, typed, round_trip)) = GOALS.iter().find(|(k, _, _)| *k == key) else {
                continue;
            };
            let mode = if doc_version >= *typed {
                ConvertMode::Strict
            } else {
                ConvertMode::Lenient
            };
            let json = facet_json::to_string(&value).expect("the value writes");
            read += 1;
            let name = file
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();
            match round_trip(&key, &json, mode) {
                None => problems.push(format!("{name}: {key} dropped")),
                Some(d) => {
                    for finding in d.iter() {
                        let lenient_unknown = matches!(mode, ConvertMode::Lenient)
                            && finding.code.name() == "UnrecognizedField";
                        let line = format!("{name} {key} {}", finding.location.json_path);
                        if lenient_unknown {
                            unknown.push(line);
                        } else {
                            problems.push(format!("{line}: {finding:?}"));
                        }
                    }
                }
            }
        }
    }
    eprintln!("{read} vanilla goals read");
    assert!(read > 500, "the corpus holds vanilla goals ({read})");
    assert!(
        problems.is_empty(),
        "{} problems:\n{}",
        problems.len(),
        problems.join("\n")
    );
    // Members of older vanilla files that the goal's typed struct does not have: the lenient
    // conversion keeps and reports them.
    assert_eq!(
        unknown,
        [
            "ocelot.json minecraft:behavior.leap_at_target target_dist",
            "parrot.json minecraft:behavior.random_fly avoid_damage_blocks",
            "parrot.json minecraft:behavior.random_fly y_offset",
            "witch.json minecraft:behavior.random_look_around look_distance",
        ]
    );
}
