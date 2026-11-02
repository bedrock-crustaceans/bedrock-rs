#![allow(clippy::pedantic, missing_docs)]
//! Vanilla smoke for the worldgen root: the feature rules of the vanilla
//! behaviour packs and of the app data's `definitions/feature_rules`, and the dimension
//! documents, load through the `Driver` with zero error-level findings; MCB documents (baked
//! documents the loaders do not decode) are counted, not failed. The sample packs ship neither
//! directory in their `behavior_pack/`: they are loaded and must give nothing at all.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo test -p bedrock_addon_std --test smoke_worldgen`;
//! without the variable every test skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;
#[path = "common/pairs.rs"]
mod pairs;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{Diagnostic, DiagnosticCode, Severity};
use bedrock_addon::load::{LoadOptions, StackOptions};
use bedrock_addon::pack::{PackOrigin, StackSet};
use bedrock_addon_std::{DirSource, Driver, Options, Strategy, read_app_data};

use common::game_files;
use common::world::world_list;

/// The pack folders directly below `dir`.
fn pack_roots(dir: &Path) -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|entries| entries.filter_map(|e| e.ok().map(|e| e.path())).collect())
        .unwrap_or_default();
    roots.retain(|path| path.is_dir());
    roots.sort();
    roots
}

/// The MCB documents on disk of `directory` in the packs below `dirs` (the loose directory and
/// its brarchive).
fn mcb(dirs: &[&Path], directory: &str) -> usize {
    dirs.iter()
        .flat_map(|dir| pack_roots(dir))
        .flat_map(|root| {
            [
                root.join(directory),
                root.join("__brarchive")
                    .join(format!("{directory}.brarchive")),
            ]
        })
        .filter(|path| path.exists())
        .map(|path| game_files::mcb_on_disk(&path).total())
        .sum()
}

/// The stacks of the packs directories `dirs` (all `Package` origin) for the world's behaviour
/// list `world_behavior`, with the app data at `app_data` when given.
fn stacks(dirs: &[&Path], world_behavior: &[u8], app_data: Option<&Path>) -> StackSet {
    let mut driver = Driver::new(Options::default().with_strategy(Strategy::Sequential)).unwrap();
    let repository = driver
        .repository(
            dirs.iter()
                .map(|dir| DirSource::packs(dir.to_path_buf(), PackOrigin::Package)),
        )
        .unwrap();
    let options = LoadOptions {
        app_data: app_data.map(|dir| read_app_data(dir).expect("the app data reads")),
        ..LoadOptions::default()
    };
    driver
        .load_stacks(
            repository.stacks(&world_list(&repository, world_behavior, &options), options),
            StackOptions::default(),
        )
        .unwrap()
}

/// Prints one collection's line (loaded, rejected, MCB, warnings by code) and returns its
/// error-level findings that are not MCB documents.
fn report<'a>(
    source: &str,
    what: &str,
    loaded: usize,
    rejected: usize,
    on_disk: usize,
    findings: impl Iterator<Item = &'a Diagnostic>,
) -> Vec<String> {
    let mut mcb = 0;
    let mut errors = Vec::new();
    let mut warnings: BTreeMap<&str, usize> = BTreeMap::new();
    for finding in findings {
        if finding.severity == Severity::Warning {
            *warnings.entry(finding.code.name()).or_default() += 1;
        }
        if finding.severity < Severity::Error {
            continue;
        }
        if matches!(finding.code, DiagnosticCode::UnsupportedFileFormat) && mcb < on_disk {
            mcb += 1;
            continue;
        }
        errors.push(finding.to_string());
    }
    eprintln!(
        "{source}: {loaded} {what}, {rejected} rejected files, {mcb} MCB documents ({on_disk} on disk), {} errors, warnings {warnings:?}",
        errors.len()
    );
    for error in &errors {
        eprintln!("{source}: {error}");
    }
    // No tree has a feature rule or dimension warning: the pin is exact.
    game_files::assert_warnings(&format!("{source}: {what}"), &warnings, &[]);
    errors
}

/// Loads the feature rules and dimensions of `dirs` (and the app data at `app_data`) and
/// requires no error but MCB documents; returns how many rules loaded.
fn check(source: &str, dirs: &[&Path], world_behavior: &[u8], app_data: Option<&Path>) -> usize {
    let set = stacks(dirs, world_behavior, app_data);
    let rules = set.behavior.feature_rules();
    let rule_errors = report(
        source,
        "feature rules",
        rules.len(),
        rules.rejected().len(),
        mcb(dirs, "feature_rules"),
        rules.diagnostics(),
    );
    let dimensions = set.behavior.dimensions();
    let dimension_errors = report(
        source,
        "dimensions",
        dimensions.len(),
        dimensions.rejected().len(),
        mcb(dirs, "dimensions"),
        dimensions.diagnostics(),
    );
    assert!(rule_errors.is_empty(), "{source}: {rule_errors:#?}");
    assert!(
        dimension_errors.is_empty(),
        "{source}: {dimension_errors:#?}"
    );
    rules.len()
}

#[test]
fn sample_packs_has_no_feature_rules_or_dimensions() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    let world = br#"[{"pack_id": "ee649bcf-256c-4013-9068-6a802b89d756", "version": [0, 0, 1]}]"#;
    assert_eq!(check("sample packs", &[&samples], world, None), 0);
}

#[test]
fn server_install_feature_rules_and_dimensions_load_without_errors() {
    let server = skip!(game_files::SERVER_INSTALL);
    check(
        "server install",
        &[&server.join("behavior_packs")],
        b"[]",
        Some(&server),
    );
}

#[test]
fn client_feature_rules_with_the_app_data_load_without_errors() {
    let install_b = skip!(game_files::CLIENT_INSTALL_B);
    let loaded = check(
        "client install B",
        &[&install_b.join("behavior_packs")],
        b"[]",
        Some(&install_b),
    );
    assert!(loaded > 0, "client install B has feature rules");
    let install_a = skip!(game_files::CLIENT_INSTALL_A);
    check(
        "client install A",
        &[&install_a.join("behavior_packs")],
        b"[]",
        Some(&install_a),
    );
}

/// Every `feature_rules` occurrence of the Molang pairs file (the packs of client install B, 214) names
/// a file that loads with no error-level finding, and its Molang member reads to the pair's text
/// and effective version.
#[test]
fn molang_pairs_feature_rule_molang_loads_with_its_text_and_version() {
    use bedrock_addon::document::Content;
    use bedrock_addon::families::feature_rules::FeatureRule;

    let pairs_path = skip!(game_files::MOLANG_PAIRS);
    let install_b = skip!(game_files::CLIENT_INSTALL_B);
    let found = pairs::check(&pairs_path, &install_b, "feature_rules", |file| {
        let document = FeatureRule::parse(&file.at, file.text.as_bytes(), &file.cx)
            .map_err(|rejected| rejected.diagnostics)?;
        let diagnostics = document.diagnostics().clone();
        let mut members = pairs::Members::new();
        // The walk starts at the payload, the pointers at the root.
        pairs::members(
            &mut document.into_value(),
            Some("minecraft:feature_rules"),
            &mut members,
        );
        Ok((members, diagnostics))
    });
    eprintln!(
        "Molang pairs: {} feature rule occurrences checked in {} files",
        found.checked, found.files
    );
    assert!(
        found.failures.is_empty(),
        "{} failures: {:#?}",
        found.failures.len(),
        &found.failures[..found.failures.len().min(20)]
    );
    assert_eq!(
        found.checked, 214,
        "the feature rule occurrences of the Molang pairs file"
    );
    assert_eq!(
        found.files, 181,
        "the files of those occurrences in the Molang pairs file"
    );
}
