//! The game's own files, which are not in the repository: `$BEDROCK_ADDON_CORPUS` holds one
//! entry per kind of install, each a directory or a symlink to one. A test that needs an entry
//! takes it with `skip!`, which returns from the test (printing which entry is missing) when
//! the variable is unset or the entry absent, so the suite passes on a machine without the
//! files. The MCB helpers below count baked binary documents apart from errors.
//!
//! ```ignore
//! #[macro_use]
//! mod common;
//!
//! #[test]
//! fn server_install_behavior_packs_open() {
//!     let server = skip!(common::game_files::SERVER_INSTALL);
//!     // … server.join("behavior_packs") …
//! }
//! ```

#![allow(dead_code, unused_macros)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{Diagnostic, Severity};
use bedrock_addon::pack::Pack;
use bedrock_addon::storage::PackPath;

/// The environment variable naming the folder that holds the game's files.
pub const VARIABLE: &str = "BEDROCK_ADDON_CORPUS";

/// The sample packs of the game's public repository (`behavior_pack/`, `resource_pack/`,
/// `metadata/`).
pub const SAMPLE_PACKS: &str = "bedrock-samples";
/// A Bedrock Dedicated Server install (`behavior_packs/`, `resource_packs/`, `definitions/`,
/// `config/`, `server.properties`, …).
pub const SERVER_INSTALL: &str = "bds-1.26.36.1";
/// The `data/` directory of the first client install.
pub const CLIENT_INSTALL_A: &str = "gdk-dev";
/// The `data/` directory of the second client install.
pub const CLIENT_INSTALL_B: &str = "gdk-retail";
/// The split client pack tree (`server/` and `client/`, each with `*_packs/` directories).
pub const SPLIT_TREE: &str = "packs";
/// The Molang (pack, file, JSON pointer) occurrences with their text and raw version.
pub const MOLANG_PAIRS: &str = "corpus_pairs.json";

/// Every entry of the layout.
pub const ENTRIES: [&str; 6] = [
    SAMPLE_PACKS,
    SERVER_INSTALL,
    CLIENT_INSTALL_A,
    CLIENT_INSTALL_B,
    SPLIT_TREE,
    MOLANG_PAIRS,
];

/// The folder that holds the game's files, if the variable is set (to anything non-empty).
pub fn root() -> Option<PathBuf> {
    std::env::var_os(VARIABLE)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

/// The entry `name` (one of the constants above) below the root folder, if it exists
/// (following symlinks).
pub fn source(name: &str) -> Option<PathBuf> {
    let path = root()?.join(name);
    path.exists().then_some(path)
}

/// Why `name` is not available, for the skip message.
pub fn missing(name: &str) -> String {
    match root() {
        None => format!("{VARIABLE} is not set; entry `{name}` unavailable"),
        Some(root) => format!("entry `{name}` missing under {}", root.display()),
    }
}

// MCB documents: baked binary documents that no family parses. A smoke test counts them apart
// from errors: on disk with `mcb_on_disk`, in an opened pack with
// `mcb_documents`, and per finding with `is_mcb_document` (an `UnsupportedFileFormat` on such a
// file is counted, not failed).

/// `\x7FMCB`: the magic of a baked binary document.
pub const MCB_MAGIC: &[u8; 4] = b"\x7FMCB";
/// A brarchive's magic, little-endian `0x267052A0B125277D`.
pub const BRARCHIVE_MAGIC: [u8; 8] = 0x2670_52A0_B125_277D_u64.to_le_bytes();

/// Whether `bytes` are an MCB document.
pub fn is_mcb(bytes: &[u8]) -> bool {
    bytes.starts_with(MCB_MAGIC)
}

/// Whether the file at `path` of `pack` is an MCB document. Reads it, so it must be resident
/// (archive packs, brarchive entries of a loaded stack, `Residency::All`); `false` when it
/// cannot be read.
pub fn is_mcb_document(pack: &Pack, path: &PackPath) -> bool {
    pack.read(path).is_ok_and(|bytes| is_mcb(&bytes))
}

/// How many resident files of `pack` are MCB documents (files that are only indexed are not
/// read, so a folder pack fresh from `pack_dir` counts none: use `mcb_on_disk` for it).
pub fn mcb_documents(pack: &Pack) -> usize {
    pack.paths()
        .filter(|path| pack.is_resident(path) && is_mcb_document(pack, path))
        .count()
}

/// MCB documents found on disk below a pack candidate (folder or file), per container.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct McbCount {
    /// Loose files starting with the magic.
    pub loose: usize,
    /// Brarchive entries whose data starts with the magic.
    pub archived: usize,
    /// Brarchives scanned.
    pub brarchives: usize,
}

impl McbCount {
    /// Every MCB document counted.
    pub fn total(self) -> usize {
        self.loose + self.archived
    }
}

impl std::ops::Add for McbCount {
    type Output = McbCount;

    fn add(self, other: McbCount) -> McbCount {
        McbCount {
            loose: self.loose + other.loose,
            archived: self.archived + other.archived,
            brarchives: self.brarchives + other.brarchives,
        }
    }
}

impl std::iter::Sum for McbCount {
    fn sum<I: Iterator<Item = McbCount>>(iter: I) -> McbCount {
        iter.fold(McbCount::default(), std::ops::Add::add)
    }
}

/// MCB documents below `candidate` on disk (following symlinks): loose files and brarchive
/// entries, read from the two layouts directly and independent of the loaders.
pub fn mcb_on_disk(candidate: &Path) -> McbCount {
    let mut count = McbCount::default();
    let mut pending = vec![candidate.to_path_buf()];
    while let Some(path) = pending.pop() {
        if path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&path) {
                pending.extend(entries.filter_map(|entry| entry.ok().map(|entry| entry.path())));
            }
            continue;
        }
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        if is_mcb(&bytes) {
            count.loose += 1;
        } else if bytes.starts_with(&BRARCHIVE_MAGIC) {
            count.brarchives += 1;
            count.archived += brarchive_mcb_entries(&bytes);
        }
    }
    count
}

/// Entries of a brarchive whose data starts with the MCB magic: a 16-byte header (magic, `u32`
/// entry count, `u32` version), 256-byte entries with the data offset at 0xF8 and size at
/// 0xFC, data from `0x10 + count * 256`.
pub fn brarchive_mcb_entries(bytes: &[u8]) -> usize {
    let u32_at = |at: usize| {
        bytes
            .get(at..at + 4)
            .map(|b| u32::from_le_bytes(b.try_into().unwrap()) as usize)
    };
    let Some(entries) = u32_at(8) else { return 0 };
    let data = 0x10 + entries * 256;
    (0..entries)
        .filter(|index| {
            let entry = 0x10 + index * 256;
            match (u32_at(entry + 0xF8), u32_at(entry + 0xFC)) {
                (Some(offset), Some(size)) if size >= 4 => bytes
                    .get(data + offset..data + offset + 4)
                    .is_some_and(is_mcb),
                _ => false,
            }
        })
        .count()
}

// Warnings by code: a smoke pins, for each tree of the game's files it loads, how many
// warning-level findings of each code the tree has, so a vanilla file that gains a finding
// fails the smoke of its family.

/// The warning-level findings among `findings`, counted by code name.
pub fn warnings_by_code<'a>(
    findings: impl IntoIterator<Item = &'a Diagnostic>,
) -> BTreeMap<&'static str, usize> {
    let mut warnings = BTreeMap::new();
    for finding in findings {
        if finding.severity == Severity::Warning {
            *warnings.entry(finding.code.id()).or_insert(0) += 1;
        }
    }
    warnings
}

/// How often each name occurs in `names` (the code names a smoke collected for one
/// severity).
pub fn count_names(names: &[String]) -> BTreeMap<&str, usize> {
    let mut counts = BTreeMap::new();
    for name in names {
        *counts.entry(name.as_str()).or_insert(0) += 1;
    }
    counts
}

/// Requires the warnings of `source`, counted by code name, to be exactly `expected` (an
/// empty list: no warning at all).
#[track_caller]
pub fn assert_warnings<K: AsRef<str> + Ord>(
    source: &str,
    found: &BTreeMap<K, usize>,
    expected: &[(&str, usize)],
) {
    let found: BTreeMap<&str, usize> = found
        .iter()
        .map(|(code, count)| (code.as_ref(), *count))
        .collect();
    let expected: BTreeMap<&str, usize> = expected.iter().copied().collect();
    assert_eq!(found, expected, "{source}: warnings by code");
}

/// `skip!(name)`: the entry's path, or an early `return` from the test after printing
/// which entry is missing.
macro_rules! skip {
    ($name:expr) => {
        match crate::common::game_files::source($name) {
            Some(path) => path,
            None => {
                eprintln!("skipped: {}", crate::common::game_files::missing($name));
                return;
            }
        }
    };
}
