//! Characterization of the write-back of unknown members (phase 3 step 1, the `Unknown` side
//! table): every JSON fixture of this crate that a family of the list reads is read as written
//! and with an unknown member put into its objects (every object, or those at one depth), and
//! written back (`Document::to_json`, and as it was read where the family has a chain). The
//! texts, or the codes of a rejection, are pinned by hash in `characterize_unknown.txt`, made
//! before the table: an unedited document writes what it wrote when its unknown members lived
//! in its structs.
//!
//! `BEDROCK_ADDON_CHARACTERIZE=write` writes the pin file instead of comparing.

#![cfg(feature = "full")]

#[path = "characterize_unknown/fixtures.rs"]
mod fixtures;

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use fixtures::{each_kind, files, hash, variants};

use ba::document::Content;
use ba::load::LoadContext;
use ba::storage::PackPath;
use bedrock_addon as ba;

/// The pin file.
fn pins() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/characterize_unknown.txt")
}

/// What a document type made of a text.
fn outcome<T: Content>(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> String {
    match T::parse(path, bytes, cx) {
        Ok(document) => match document.to_json() {
            Ok(text) => text,
            Err(error) => format!("not written: {error:?}"),
        },
        Err(rejected) => {
            let mut codes: Vec<String> = rejected
                .diagnostics
                .iter()
                .map(|finding| {
                    format!(
                        "{} {:?} {}",
                        finding.code.id(),
                        finding.severity,
                        finding.location.json_path
                    )
                })
                .collect();
            codes.sort();
            format!("rejected: {codes:?}")
        }
    }
}

/// A document type: its name, the pack paths it reads and what it makes of a text.
struct Kind {
    name: &'static str,
    claims: fn(&str) -> bool,
    run: fn(&PackPath, &[u8], &LoadContext) -> String,
}

macro_rules! kind {
    ($name:literal, $ty:ty, $claims:expr) => {
        Kind {
            name: $name,
            claims: $claims,
            run: |path, bytes, cx| outcome::<$ty>(path, bytes, cx),
        }
    };
    ($name:literal, $ty:ty, $claims:expr, chain) => {
        Kind {
            name: $name,
            claims: $claims,
            run: |path, bytes, cx| {
                #[allow(
unused_imports,
    reason = "a JsonDocument type takes parse_as_written from the trait and the others have it inherent, so the import is unused for them"
)]
                use ba::document::JsonDocument as _;
                let mut out = outcome::<$ty>(path, bytes, cx);
                out.push_str("\n-- as written --\n");
                match <$ty>::parse_as_written(path, bytes, cx) {
                    Ok(document) => match document.to_json() {
                        Ok(text) => out.push_str(&text),
                        Err(error) => {
                            let _ = write!(out, "not written: {error:?}");
                        }
                    },
                    Err(_) => out.push_str("rejected"),
                }
                out
            },
        }
    };
}

fn kinds() -> Vec<Kind> {
    each_kind!(kind)
}

/// Every record: `kind file variant`, with the hash of its outcome.
fn records() -> BTreeMap<String, u64> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut all = Vec::new();
    files(&root, &root, &mut all);
    // The container fixtures moved to the storage crate with it; their records keep the
    // `storage/` prefix they had below this crate's fixtures.
    let storage = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../storage/tests/fixtures");
    let mut moved = Vec::new();
    files(&storage, &storage, &mut moved);
    all.extend(
        moved
            .into_iter()
            .map(|(relative, path)| (format!("storage/{relative}"), path)),
    );
    let kinds = kinds();
    let cx = LoadContext::detached();
    let mut out = BTreeMap::new();
    for (relative, path) in &all {
        let segments: Vec<&str> = relative.split('/').collect();
        for kind in &kinds {
            // The pack path: the first tail of the file's path the kind reads.
            let Some(pack_path) = (0..segments.len())
                .map(|from| segments[from..].join("/"))
                .find(|tail| (kind.claims)(tail))
            else {
                continue;
            };
            let Ok(bytes) = std::fs::read(path) else {
                continue;
            };
            let pack_path = PackPath::new(&pack_path).unwrap();
            for (variant, text) in variants(&bytes) {
                let result = (kind.run)(&pack_path, &text, &cx);
                let record = format!("{} {relative} {variant}", kind.name);
                // `BEDROCK_ADDON_CHARACTERIZE_DUMP=<dir>`: each outcome as text, to compare.
                if let Ok(dir) = std::env::var("BEDROCK_ADDON_CHARACTERIZE_DUMP") {
                    let name = format!("{:016x}.txt", hash(&record));
                    let _ = std::fs::write(
                        Path::new(&dir).join(name),
                        format!(
                            "{record}\n{result}\n-- input --\n{}",
                            String::from_utf8_lossy(&text)
                        ),
                    );
                }
                out.insert(record, hash(&result));
            }
        }
    }
    out
}

#[test]
fn every_fixture_writes_its_unknown_members_as_before_the_table() {
    let records = records();
    assert!(records.len() > 1000, "{} records", records.len());
    let mut text = String::new();
    for (record, hash) in &records {
        let _ = writeln!(text, "{hash:016x} {record}");
    }
    if std::env::var("BEDROCK_ADDON_CHARACTERIZE").as_deref() == Ok("write") {
        std::fs::write(pins(), text).unwrap();
        return;
    }
    let pinned = std::fs::read_to_string(pins()).unwrap();
    let pinned: BTreeMap<&str, &str> = pinned
        .lines()
        .filter_map(|line| line.split_once(' ').map(|(hash, record)| (record, hash)))
        .collect();
    let mut differ = Vec::new();
    for (record, hash) in &records {
        let now = format!("{hash:016x}");
        match pinned.get(record.as_str()) {
            Some(was) if *was == now => {}
            Some(_) => differ.push(format!("changed: {record}")),
            None => differ.push(format!("new: {record}")),
        }
    }
    for record in pinned.keys() {
        if !records.contains_key(*record) {
            differ.push(format!("gone: {record}"));
        }
    }
    assert!(
        differ.is_empty(),
        "{} of {} records differ:\n{}",
        differ.len(),
        records.len(),
        differ.join("\n")
    );
}
