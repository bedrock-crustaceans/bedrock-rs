//! The pinning manifest for the vendored dumps (`xtask/dumps/bedrock-samples/manifest.json`):
//! which upstream tag each vendored file came from, the game version it stands for, and
//! the hash it must still match.

use std::path::Path;

use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::GeneratorError;

// Deliberately no `#[serde(deny_unknown_fields)]` here, unlike every struct in
// `input.rs`: this file is hand-authored provenance prose (`_comment`, `note`,
// `coverage_note`, and similar fields on individual entries or the document as a
// whole), not a transcription of a format this generator needs to police. A new prose
// field being ignored is not the "unrecognized input" failure mode `input.rs` exists to
// catch -- there is no game data hiding behind it to silently drop.
#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub entries: Vec<ManifestEntry>,
}

#[derive(Debug, Deserialize)]
pub struct ManifestEntry {
    /// The upstream release tag the file was fetched from, e.g. `"v1.20.10.1"`.
    pub tag: String,
    /// `(major, minor, patch)`, the three bytes a stored palette entry's version stamp
    /// carries in the same positions (see `bedrock_level::subchunk::BlockDef::version`).
    /// Taken from the tag, not from the dump's own `minecraft_version` field, which later
    /// dumps stop populating.
    pub game_version: [u8; 3],
    /// Path to the vendored file, relative to `manifest.json`'s own directory.
    pub file: String,
    /// The vendored file's expected SHA-256, lower-case hex. Checked on every generator
    /// run: a mismatch means the working tree's copy no longer matches what this manifest
    /// claims to pin, which is exactly the loss of auditability architecture decision 12
    /// warns about, so it is a hard error rather than a warning.
    pub sha256: String,
}

/// Reads and parses `manifest.json` from `dumps_dir`.
pub fn read(dumps_dir: &Path) -> Result<Manifest, GeneratorError> {
    let path = dumps_dir.join("manifest.json");
    let text = std::fs::read_to_string(&path).map_err(|source| GeneratorError::ReadManifest {
        path: path.clone(),
        source,
    })?;
    serde_json::from_str(&text).map_err(|source| GeneratorError::ParseManifest { path, source })
}

/// Reads the file `entry` names and confirms it hashes to `entry.sha256`.
pub fn read_pinned_file(dumps_dir: &Path, entry: &ManifestEntry) -> Result<String, GeneratorError> {
    let path = dumps_dir.join(&entry.file);
    let bytes = std::fs::read(&path).map_err(|source| GeneratorError::ReadDump {
        path: path.clone(),
        source,
    })?;

    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let actual = hex_lower(&hasher.finalize());
    if actual != entry.sha256 {
        return Err(GeneratorError::HashMismatch {
            path,
            expected: entry.sha256.clone(),
            actual,
        });
    }

    String::from_utf8(bytes).map_err(|source| GeneratorError::NotUtf8 { path, source })
}

fn hex_lower(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        write!(out, "{b:02x}").expect("writing to a String never fails");
    }
    out
}
