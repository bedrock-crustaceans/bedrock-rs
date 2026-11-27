//! Fixture access and finding helpers shared by the pack-settings tests.

#![allow(
    dead_code,
    reason = "a helper module shared by the test files of this folder; none of them uses every helper"
)]

use std::path::PathBuf;

use bedrock_addon::diagnostic::{Diagnostic, Diagnostics, Severity};
use bedrock_addon::document::{Document, Rejected};
use bedrock_addon::pack::settings::SavedSettings;

/// `tests/fixtures/pack/pack_settings/`.
pub fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pack/pack_settings")
}

/// The bytes of the fixture `name` (relative to the family's fixture folder).
pub fn fixture(name: &str) -> Vec<u8> {
    let path = fixtures().join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// The fixture `name` parsed as the file `name`.
pub fn parse(name: &str) -> Result<Document<SavedSettings>, Rejected> {
    SavedSettings::parse_named(name, &fixture(name))
}

/// `text` parsed as the file `name`.
pub fn parse_text(name: &str, text: &str) -> Result<Document<SavedSettings>, Rejected> {
    SavedSettings::parse_named(name, text.as_bytes())
}

/// The fixture `name`, which must load.
pub fn loads(name: &str) -> Document<SavedSettings> {
    parse(name).unwrap_or_else(|rejected| panic!("{name}: {:#?}", rejected.diagnostics))
}

/// `text`, which must load.
pub fn loads_text(text: &str) -> Document<SavedSettings> {
    parse_text(WORLD, text).unwrap_or_else(|rejected| panic!("{:#?}", rejected.diagnostics))
}

/// The findings of the fixture `name`, which must be rejected.
pub fn refused(name: &str) -> Diagnostics {
    match parse(name) {
        Ok(document) => panic!("{name} loaded: {:#?}", document.value()),
        Err(rejected) => rejected.diagnostics,
    }
}

/// The findings of `text`, which must be rejected.
pub fn refused_text(name: &str, text: &str) -> Diagnostics {
    match parse_text(name, text) {
        Ok(document) => panic!("loaded: {:#?}", document.value()),
        Err(rejected) => rejected.diagnostics,
    }
}

/// The name used for inline documents.
pub const WORLD: &str = "world_behavior_pack_settings.json";

/// The code names of the findings at `severity`, in order.
pub fn names(diagnostics: &Diagnostics, severity: Severity) -> Vec<&'static str> {
    diagnostics
        .iter()
        .filter(|d| d.severity == severity)
        .map(|d| d.code.id())
        .collect()
}

/// The rendered texts of the findings at `severity`, in order.
pub fn texts(diagnostics: &Diagnostics, severity: Severity) -> Vec<String> {
    diagnostics
        .iter()
        .filter(|d| d.severity == severity)
        .map(|d| d.code.to_string())
        .collect()
}

/// The finding at `path` (its JSON path as `Display` prints it).
pub fn at<'d>(diagnostics: &'d Diagnostics, path: &str) -> Vec<&'d Diagnostic> {
    diagnostics
        .iter()
        .filter(|d| d.location.json_path.to_string() == path)
        .collect()
}

/// A document of `version` (a JSON value's text) with the payload `payload`.
pub fn document(version: &str, payload: &str) -> String {
    format!(r#"{{"format_version": {version}, "minecraft:pack_settings": {payload}}}"#)
}

/// A payload with one entry for the vanilla tests' pack, holding `values` (an object's text).
pub fn one_entry(values: &str) -> String {
    format!(
        r#"{{"settings": [{{"pack_id": "d3349771-6782-4420-875a-fcd6179a8915", "values": {values}}}]}}"#
    )
}
