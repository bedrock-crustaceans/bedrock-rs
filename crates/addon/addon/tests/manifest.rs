#![allow(clippy::pedantic, missing_docs)]
//! `manifest`: the legacy and comprehensive readers against vanilla's manifests, the malformed
//! corpus, the quirk table of 12, the pack-settings definitions and the writer.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[path = "manifest/malformed.rs"]
mod malformed;
#[path = "manifest/quirks.rs"]
mod quirks;
#[path = "manifest/settings.rs"]
mod settings;
#[path = "manifest/vanilla.rs"]
mod vanilla;
#[path = "manifest/write.rs"]
mod write;

use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, Severity};
use bedrock_addon::document::{Document, Rejected};
use bedrock_addon::manifest::{CapabilityRegistry, Manifest};
use bedrock_addon::storage::PackPath;

/// The manifest fixture directory.
fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/manifest")
}

/// The bytes of `manifest/<relative>`.
fn fixture(relative: &str) -> Vec<u8> {
    let path = fixtures().join(relative);
    std::fs::read(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// Parses `bytes` as `manifest.json` with the server-default capability registry.
fn parse(bytes: &[u8]) -> Result<Document<Manifest>, Rejected> {
    Manifest::parse(
        &PackPath::from_static("manifest.json"),
        bytes,
        &CapabilityRegistry::default(),
    )
}

/// Parses `text`.
fn parse_str(text: &str) -> Result<Document<Manifest>, Rejected> {
    parse(text.as_bytes())
}

/// The manifest `text`, which must load.
fn loads(text: &str) -> Document<Manifest> {
    match parse_str(text) {
        Ok(document) => document,
        Err(rejected) => panic!("rejected: {:#?}", rejected.diagnostics),
    }
}

/// The findings of `text`, which must be rejected.
fn rejected(text: &str) -> Diagnostics {
    match parse_str(text) {
        Ok(document) => panic!("loaded: {:#?}", document.value()),
        Err(rejected) => rejected.diagnostics,
    }
}

/// The findings of a parse, loaded or not.
fn findings(result: &Result<Document<Manifest>, Rejected>) -> &Diagnostics {
    match result {
        Ok(document) => document.diagnostics(),
        Err(rejected) => &rejected.diagnostics,
    }
}

/// Whether `diagnostics` has a finding of `severity` at `path` whose code satisfies `code`.
fn has(
    diagnostics: &Diagnostics,
    severity: Severity,
    path: &str,
    code: impl Fn(&DiagnosticCode) -> bool,
) -> bool {
    diagnostics.iter().any(|d: &Diagnostic| {
        d.severity == severity && d.location.json_path.to_string() == path && code(&d.code)
    })
}

/// A format-2 behaviour pack manifest around `header_extra` (members added to the header) and
/// `root_extra` (members added to the root), both written as JSON members with a leading comma.
fn format_2(header_extra: &str, root_extra: &str) -> String {
    format!(
        r#"{{
  "format_version": 2,
  "header": {{
    "name": "Ruby",
    "uuid": "11111111-1111-1111-1111-111111111111",
    "version": [1, 0, 0],
    "min_engine_version": [1, 26, 20]{header_extra}
  }},
  "modules": [
    {{ "type": "data", "uuid": "22222222-2222-2222-2222-222222222222", "version": [1, 0, 0] }}
  ]{root_extra}
}}"#
    )
}

/// A format-3 resource pack manifest around `header_extra` and `root_extra` (`format_version`
/// the integer 3, the `u8` the game binds).
fn format_3(header_extra: &str, root_extra: &str) -> String {
    format!(
        r#"{{
  "format_version": 3,
  "header": {{
    "name": "Ruby",
    "uuid": "11111111-1111-1111-1111-111111111111",
    "version": "1.0.0",
    "min_engine_version": "1.26.20"{header_extra}
  }},
  "modules": [
    {{ "type": "resources", "uuid": "22222222-2222-2222-2222-222222222222", "version": "1.0.0" }}
  ]{root_extra}
}}"#
    )
}
