//! Helpers of the trade-table tests.

#![allow(dead_code)]

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, Severity};
use bedrock_addon_core::document::{Content, Document, Rejected};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_family_trade::{ItemList, Trade, TradeItem, TradeTable};

/// A third-party behaviour pack at the engine version.
pub fn cx() -> LoadContext {
    LoadContext::detached()
}

/// A fixture file under `tests/fixtures/trade/trading/`.
pub fn fixture(name: &str) -> String {
    let path = format!(
        "{}/../../addon/tests/fixtures/trade/trading/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// Loads `json` as a trade-table file.
pub fn load(json: &str) -> Result<Document<TradeTable>, Rejected> {
    TradeTable::parse(
        &PackPath::from_static("trading/test_trades.json"),
        json.as_bytes(),
        &cx(),
    )
}

/// Loads `json`, which must load.
#[track_caller]
pub fn loads(json: &str) -> Document<TradeTable> {
    match load(json) {
        Ok(document) => document,
        Err(rejected) => panic!("rejected: {:#?}", rejected.diagnostics),
    }
}

/// Loads `json`, which must be rejected; its findings.
#[track_caller]
pub fn rejects(json: &str) -> Diagnostics {
    match load(json) {
        Ok(document) => panic!("loaded: {:#?}", document.value()),
        Err(rejected) => rejected.diagnostics,
    }
}

/// A document with `format_version` `version` (none when `None`) and `tiers` as given.
pub fn doc(version: Option<&str>, tiers: &str) -> String {
    match version {
        Some(version) => format!(r#"{{"format_version": "{version}", "tiers": {tiers}}}"#),
        None => format!(r#"{{"tiers": {tiers}}}"#),
    }
}

/// The findings whose code is `name` at `severity`.
pub fn with<'a>(d: &'a Diagnostics, name: &str, severity: Severity) -> Vec<&'a Diagnostic> {
    d.iter()
        .filter(|f| f.code.name() == name && f.severity == severity)
        .collect()
}

/// A finding of code `name` at `severity` at the path `path`.
#[track_caller]
pub fn found_at(d: &Diagnostics, name: &str, severity: Severity, path: &str) -> Diagnostic {
    with(d, name, severity)
        .into_iter()
        .find(|f| f.location.json_path.to_string() == path)
        .unwrap_or_else(|| panic!("no {name} {severity:?} at {path}: {}", summary(d)))
        .clone()
}

/// One line per finding: severity, code and path.
pub fn summary(d: &Diagnostics) -> String {
    d.iter()
        .map(|f| {
            format!(
                "\n  {:?} {} at {}",
                f.severity,
                f.code.name(),
                f.location.json_path
            )
        })
        .collect()
}

/// The findings at info (the ones of ours).
pub fn infos(d: &Diagnostics) -> Vec<&Diagnostic> {
    d.iter().filter(|f| f.severity == Severity::Info).collect()
}

/// The findings above info.
pub fn above_info(d: &Diagnostics) -> Vec<&Diagnostic> {
    d.iter().filter(|f| f.severity > Severity::Info).collect()
}

/// The trade at `tier` / `group` / `trade`.
#[track_caller]
pub fn trade(table: &TradeTable, tier: usize, group: usize, index: usize) -> &Trade {
    &table.tiers[tier].groups[group].trades[index]
}

/// The items of a `wants` / `gives` entry.
pub fn items(list: &ItemList) -> &[TradeItem] {
    list.items()
}
