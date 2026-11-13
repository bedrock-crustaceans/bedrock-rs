//! The version a Molang string compiles at. `$BEDROCK_ADDON_CORPUS/corpus_pairs.json`, when
//! present, records for every Molang string of a set of the game's own packs the pack's
//! `min_engine_version`, the file's `format_version`, the version the game compiles that string
//! at and the rule that chose it. Each occurrence must get that version from this crate's
//! mapping and version sources. The pairs hold the game's content, so they are never committed,
//! and the test is skipped when the variable is unset.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::collections::BTreeMap;

use bedrock_addon_core::json::Value;
use bedrock_addon_core::molang::{
    ExperimentMask, MolangContext, MolangVersion, MolangVersionSource, RawVersion,
};
use bedrock_addon_core::version::{FormatVersionInput, MinEngineVersion, SemVersion};
use bedrock_addon_test_support::corpus;

fn reference_pairs() -> Option<Value> {
    let Some(dir) = std::env::var_os("BEDROCK_ADDON_CORPUS") else {
        corpus::skipped(
            "the Molang reference pairs",
            corpus::not_set(corpus::CORPUS),
        );
        return None;
    };
    let path = std::path::PathBuf::from(dir).join("corpus_pairs.json");
    let Ok(text) = std::fs::read_to_string(&path) else {
        corpus::skipped(
            "the Molang reference pairs",
            format_args!("{} is missing", path.display()),
        );
        return None;
    };
    Some(facet_json::from_str::<Value>(&text).expect("corpus_pairs.json is JSON"))
}

fn text<'a>(object: &'a facet_value::VObject, key: &str) -> Option<&'a str> {
    object
        .get(key)?
        .as_string()
        .map(facet_value::VString::as_str)
}

#[test]
fn every_molang_string_gets_vanillas_version() {
    let Some(reference) = reference_pairs() else {
        return;
    };
    let reference = reference.as_object().unwrap();
    let mut counts = BTreeMap::<String, u64>::new();
    let mut failures = Vec::new();
    let mut without_format_version = 0_u32;
    for pair in reference.get("pairs").unwrap().as_array().unwrap() {
        for occurrence in pair
            .as_object()
            .unwrap()
            .get("occurrences")
            .unwrap()
            .as_array()
            .unwrap()
        {
            let occurrence = occurrence.as_object().unwrap();
            let rule = text(occurrence, "version_rule").unwrap();
            let pack = text(occurrence, "pack_min_engine_version").map(|v| MinEngineVersion {
                version: SemVersion::parse(v).expect("a manifest version"),
            });
            let source = match rule {
                "pack_min_engine_version" => MolangVersionSource::Pack,
                "item_file_format_version" => MolangVersionSource::FormatVersion,
                "controller_format_version_backcompat" => MolangVersionSource::ControllerBackCompat,
                "block_modern_loader_constant_13" => {
                    MolangVersionSource::Constant(MolangVersion::V13)
                }
                "biome_height_range_constant_1" => MolangVersionSource::Constant(MolangVersion::V1),
                other => panic!("an unknown rule: {other}"),
            };
            let document = if let Some(version) = text(occurrence, "file_format_version") {
                FormatVersionInput::parse(version).expect("a format version")
            } else {
                // A document without `format_version` has no document version; only a rule
                // that does not read one may meet it.
                without_format_version += 1;
                assert!(
                    matches!(
                        source,
                        MolangVersionSource::Pack | MolangVersionSource::Constant(_)
                    ),
                    "{rule} needs the file's format_version"
                );
                FormatVersionInput::parse("0.0.0").unwrap()
            };
            let cx = MolangContext::new(pack.as_ref(), ExperimentMask::empty(), &document, None);
            *counts.entry(rule.to_owned()).or_default() += 1;
            let effective = occurrence
                .get("version_effective")
                .unwrap()
                .as_number()
                .unwrap();
            let effective = RawVersion(i16::try_from(effective.to_i64().unwrap()).unwrap());
            let resolved = source.resolve(&cx);
            if resolved != effective {
                // The text stays out of the log: pack, file and pointer identify the occurrence.
                failures.push(format!(
                    "{} {} {}: {rule} gives {resolved}, vanilla {effective}",
                    text(occurrence, "pack").unwrap(),
                    text(occurrence, "file").unwrap(),
                    text(occurrence, "pointer").unwrap(),
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} occurrences differ:\n{}",
        failures.len(),
        failures.join("\n")
    );
    let recorded: BTreeMap<String, u64> = reference
        .get("counts")
        .unwrap()
        .as_object()
        .unwrap()
        .get("rules")
        .unwrap()
        .as_object()
        .unwrap()
        .iter()
        .map(|(rule, count)| {
            (
                rule.as_str().to_owned(),
                count.as_number().unwrap().to_u64().unwrap(),
            )
        })
        .collect();
    assert_eq!(counts, recorded, "the per-rule counts of the file");
    // A handful of occurrences have no `format_version`, and all of them are under the pack rule.
    assert_eq!(
        without_format_version, 30,
        "occurrences without a format_version"
    );
}
