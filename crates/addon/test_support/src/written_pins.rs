//! The characterization of a sound document's write-back (phase 3 step 8, the sound files
//! writing their typed value): every fixture of the crate the family reads, and probes that
//! reach each member, read and written with `Content::to_json`, and some of them written
//! again after an edit of the typed value. The written texts are pinned in a file beside the
//! family's tests; `BEDROCK_ADDON_SOUND_PIN=write` writes the file instead of comparing.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use bedrock_addon_core::document::{Content, Document};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::PackPath;

/// The test directory of the crate at `manifest_dir`.
fn tests(manifest_dir: &str) -> PathBuf {
    PathBuf::from(manifest_dir).join("tests")
}

/// Every fixture file whose path ends in `suffix`, by its path below
/// `<manifest_dir>/tests/fixtures`, sorted.
pub fn fixtures(manifest_dir: &str, suffix: &str) -> Vec<(String, String)> {
    let root = tests(manifest_dir).join("fixtures");
    let mut found = Vec::new();
    let mut dirs = vec![root.clone()];
    while let Some(dir) = dirs.pop() {
        for entry in std::fs::read_dir(&dir).expect("the fixtures") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                dirs.push(path);
            } else if path.to_string_lossy().replace('\\', "/").ends_with(suffix) {
                let name = path
                    .strip_prefix(&root)
                    .expect("below the fixtures")
                    .to_string_lossy()
                    .replace('\\', "/");
                let text = std::fs::read_to_string(&path).expect("a text fixture");
                found.push((name, text));
            }
        }
    }
    found.sort();
    found
}

/// `T` read from `text` at `path`.
pub fn parse<T: Content>(path: &str, text: &str) -> Result<Document<T>, String> {
    T::parse(
        &PackPath::new(path).expect("a pack path"),
        text.as_bytes(),
        &LoadContext::detached(),
    )
    .map_err(|rejected| {
        let mut codes: Vec<String> = rejected
            .diagnostics
            .iter()
            .map(|finding| format!("{} {}", finding.code.id(), finding.location.json_path))
            .collect();
        codes.sort();
        format!("rejected: {codes:?}")
    })
}

/// What a document type writes for `text`: the written text, or why there is none.
pub fn written<T: Content>(path: &str, text: &str) -> String {
    match parse::<T>(path, text) {
        Ok(document) => document
            .to_json()
            .unwrap_or_else(|error| format!("not written: {error:?}")),
        Err(rejected) => rejected,
    }
}

/// What `edit` makes of the value read from `text`, written.
pub fn edited<T: Content>(path: &str, text: &str, edit: impl FnOnce(&mut T)) -> String {
    match parse::<T>(path, text) {
        Ok(document) => {
            let mut value = document.into_value();
            edit(&mut value);
            value
                .to_json()
                .unwrap_or_else(|error| format!("not written: {error:?}"))
        }
        Err(rejected) => rejected,
    }
}

/// The pin file's text for `cases` (a title and what was written).
pub fn render(cases: &[(String, String)]) -> String {
    let mut out = String::new();
    for (title, text) in cases {
        writeln!(out, "== {title}").expect("a String takes it");
        writeln!(out, "{text}").expect("a String takes it");
    }
    out
}

/// Requires `out` to be the pin file `pins` (below `<manifest_dir>/tests/`), or writes it under
/// `BEDROCK_ADDON_SOUND_PIN=write`.
#[track_caller]
pub fn assert_as_pinned(manifest_dir: &str, pins: &str, out: &str) {
    let path: &Path = &tests(manifest_dir).join(pins);
    if std::env::var("BEDROCK_ADDON_SOUND_PIN").as_deref() == Ok("write") {
        std::fs::write(path, out).expect("the pin file is written");
        return;
    }
    let pinned = std::fs::read_to_string(path).expect("the pin file");
    for (line, (got, want)) in out.lines().zip(pinned.lines()).enumerate() {
        assert_eq!(got, want, "{pins} line {}", line + 1);
    }
    assert_eq!(out.lines().count(), pinned.lines().count(), "{pins}: lines");
}
