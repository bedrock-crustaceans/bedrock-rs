#![allow(clippy::pedantic, missing_docs)]
//! Vanilla smoke for trade tables: the `trading/` documents of the sample packs'
//! `behavior_pack/`, of the server install's behaviour packs and of the packs of both client
//! installs load through the `Driver` with zero error-level diagnostics. Vanilla bakes its
//! tables into brarchives as MCB documents (baked documents), which shadow the loose files: they
//! are decoded and read like a JSON file, counted from the stack report, and an install's JSON
//! is checked a second time file by file, through `TradeTable::parse`, so every vanilla table is
//! read whatever shadows it.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo test -p bedrock_addon_std --test
//! smoke_trading`; without the variable every test skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon::document::Content;
use bedrock_addon::families::trade::TradeTable;
use bedrock_addon::load::{LoadContext, LoadOptions, StackOptions};
use bedrock_addon::pack::PackOrigin;
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::FormatVersion;
use bedrock_addon_std::{DirSource, Driver, Options, Strategy};

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

/// The trade-table MCB documents on disk in the packs below `dirs` (`trading/` and the
/// `trading` brarchives).
fn trading_mcb(dirs: &[&Path]) -> usize {
    dirs.iter()
        .flat_map(|dir| pack_roots(dir))
        .flat_map(|root| {
            [
                root.join("trading"),
                root.join("__brarchive").join("trading"),
                root.join("__brarchive").join("trading.brarchive"),
            ]
        })
        .filter(|path| path.exists())
        .map(|path| game_files::mcb_on_disk(&path).total())
        .sum()
}

/// Every `.json` file below `dir`, sorted.
fn json_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|entries| entries.filter_map(|e| e.ok().map(|e| e.path())).collect())
        .unwrap_or_default();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            json_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "json") {
            out.push(path);
        }
    }
}

/// Counts by name, for the printed line.
fn count(counts: &mut BTreeMap<String, usize>, name: &str) {
    *counts.entry(name.to_owned()).or_default() += 1;
}

/// Loads the base-game behaviour stack composed from the packs directories `dirs` and checks
/// its trade tables: no error, and no MCB document refused as an unsupported format (each is
/// decoded and read like a JSON file); returns how many tables are effective and how
/// many MCB documents are on disk.
fn check_stack(source: &str, dirs: &[&Path], world_behavior: &[u8]) -> (usize, usize) {
    let on_disk = trading_mcb(dirs);
    let mut driver = Driver::new(Options::default().with_strategy(Strategy::Sequential)).unwrap();
    let repository = driver
        .repository(
            dirs.iter()
                .map(|dir| DirSource::packs(dir.to_path_buf(), PackOrigin::Package)),
        )
        .unwrap();
    let stacks = driver
        .load_stacks(
            repository.stacks(
                &world_list(&repository, world_behavior, &LoadOptions::default()),
                LoadOptions::default(),
            ),
            StackOptions::default(),
        )
        .unwrap();
    let tables = stacks.behavior.trade_tables();
    // The stack report has the collection's findings and the loader's own about its files
    // (an MCB document the loader could not decode would be `UnsupportedFileFormat` there).
    let report = stacks.behavior.report();
    let mut mcb = 0;
    let mut errors = Vec::new();
    let mut warnings = BTreeMap::new();
    for finding in report.iter() {
        let in_trading = finding
            .location
            .path
            .as_ref()
            .is_some_and(|path| path.as_str().starts_with("trading/"));
        if !in_trading {
            continue;
        }
        if finding.severity == Severity::Warning {
            count(&mut warnings, finding.code.name());
        }
        if matches!(finding.code, DiagnosticCode::UnsupportedFileFormat) {
            mcb += 1;
            continue;
        }
        if finding.severity >= Severity::Error {
            errors.push(finding.to_string());
        }
    }
    let effective = tables.iter().count();
    let legacy = tables
        .iter()
        .filter(|(_, document)| document.version().upgraded)
        .count();
    eprintln!(
        "{source}: {effective} trade tables ({legacy} upgraded from the legacy form) over {} packs, {} rejected files, {mcb} MCB documents refused ({on_disk} on disk), {} errors, warnings {warnings:?}",
        stacks.behavior.packs().len(),
        tables.rejected().len(),
        errors.len(),
    );
    for error in &errors {
        eprintln!("{source}: {error}");
    }
    assert!(errors.is_empty(), "{source}: {} errors", errors.len());
    // No tree has a trade table warning in its stack: the pin is exact.
    game_files::assert_warnings(source, &warnings, &[]);
    assert!(
        tables.rejected().is_empty(),
        "{source}: a table was rejected"
    );
    assert_eq!(mcb, 0, "{source}: every MCB document is decoded");
    (effective, on_disk)
}

/// Parses every loose `trading/**/*.json` of the packs below `dir` as a base-game file, one
/// by one (the stack keeps one copy per path, and an MCB brarchive entry shadows the JSON):
/// zero errors; returns `(files, legacy, stamped 1.26.10)`.
fn check_files(source: &str, dir: &Path) -> (usize, usize, usize) {
    let mut cx = LoadContext::detached();
    cx.is_base_game_pack = true;
    let stamp = FormatVersion::new(1, 26, 10);
    let (mut files, mut legacy, mut stamped) = (0, 0, 0);
    let mut warnings = BTreeMap::new();
    let mut errors = Vec::new();
    for root in pack_roots(dir) {
        let mut found = Vec::new();
        json_files(&root.join("trading"), &mut found);
        for file in found {
            let relative = file.strip_prefix(&root).unwrap().to_string_lossy();
            let path = PackPath::new(&relative.replace('\\', "/")).unwrap();
            let bytes = std::fs::read(&file).unwrap();
            files += 1;
            match TradeTable::parse(&path, &bytes, &cx) {
                Ok(document) => {
                    legacy += usize::from(document.version().upgraded);
                    stamped += usize::from(document.version().written == Some(stamp));
                    for finding in document.diagnostics().iter() {
                        if finding.severity == Severity::Warning {
                            count(&mut warnings, finding.code.name());
                        }
                    }
                    // What was read is written back and reads the same.
                    let written = document.value().to_json().unwrap();
                    let again = TradeTable::parse(&path, written.as_bytes(), &cx)
                        .unwrap_or_else(|r| panic!("{}: {:#?}", file.display(), r.diagnostics));
                    assert_eq!(again.value(), document.value(), "{}", file.display());
                }
                Err(rejected) => {
                    for finding in rejected.diagnostics.iter() {
                        errors.push(format!("{}: {finding}", file.display()));
                    }
                }
            }
        }
    }
    eprintln!(
        "{source}: {files} loose trade-table files ({legacy} legacy, {stamped} stamped 1.26.10), {} errors, warnings {warnings:?}",
        errors.len()
    );
    for error in &errors {
        eprintln!("{source}: {error}");
    }
    assert!(errors.is_empty(), "{source}: {} errors", errors.len());
    // No loose file of a tree has a warning: the pin is exact.
    game_files::assert_warnings(source, &warnings, &[]);
    (files, legacy, stamped)
}

#[test]
fn sample_packs_trade_tables_load_without_errors() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    // the sample pack, selected by the world from the vanilla (`Package`) location, loads as a
    // base-game pack
    let world = br#"[{"pack_id": "ee649bcf-256c-4013-9068-6a802b89d756", "version": [0, 0, 1]}]"#;
    let (effective, mcb) = check_stack("sample packs", &[&samples], world);
    // 13 tables in `trading/` and 14 in `trading/economy_trades/`, no brarchive.
    assert_eq!(effective, 27, "The sample packs have 27 trade tables");
    assert_eq!(mcb, 0, "the sample packs bake nothing");
    let (files, legacy, stamped) = check_files("sample packs", &samples);
    assert_eq!(files, 27);
    assert!(legacy > 0 && stamped > 0, "both forms are in the samples");
}

#[test]
fn server_install_trade_tables_load_without_errors() {
    let server = skip!(game_files::SERVER_INSTALL);
    let dir = server.join("behavior_packs");
    let (effective, mcb) = check_stack("server install", &[&dir], b"[]");
    let (files, legacy, stamped) = check_files("server install", &dir);
    assert!(files > 27 && legacy > 0 && stamped > 0);
    // The baked tables are the stamped ones: each shadows its loose JSON.
    assert_eq!(mcb, stamped, "server: one MCB document per stamped table");
    // 18 legacy tables and the 9 paths the 22 baked documents of the slices are at.
    assert_eq!(effective, 27);
}

#[test]
fn client_install_b_trade_tables_load_without_errors() {
    let client = skip!(game_files::CLIENT_INSTALL_B);
    let dir = client.join("behavior_packs");
    let (effective, mcb) = check_stack("client install B", &[&dir], b"[]");
    assert_eq!(effective, 27);
    let (files, legacy, stamped) = check_files("client install B", &dir);
    // 75 files in client install B: 51 without `format_version`, 2 at 1.18.10, 22 at 1.26.10.
    assert_eq!((files, legacy, stamped), (75, 53, 22));
    assert_eq!(
        mcb, 22,
        "client install B: one MCB document per stamped table"
    );
}

#[test]
fn client_install_a_trade_tables_load_without_errors() {
    let client = skip!(game_files::CLIENT_INSTALL_A);
    let dir = client.join("behavior_packs");
    let (effective, mcb) = check_stack("client install A", &[&dir], b"[]");
    assert_eq!(effective, 27);
    let (files, legacy, stamped) = check_files("client install A", &dir);
    assert!(files > 27 && legacy > 0 && stamped > 0);
    assert_eq!(
        mcb, stamped,
        "client install A: one MCB document per stamped table"
    );
}
