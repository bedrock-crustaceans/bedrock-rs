//! The 66 meta files of the client's built-in persona pack (`resource_packs/persona/pieces/`,
//! 1.26.60 as shipped, under `tests/fixtures/persona_piece/vanilla/`): the files the game's
//! 1.26.40+ reader loads, so the crate's reader must load every one of them without an error
//! or a warning and write each back as it was read.

use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::document::Content;
use bedrock_addon::families::persona::{ArmSize, Named, PieceMetadata, Written};
use bedrock_addon::load::LoadContext;
use bedrock_addon::storage::PackPath;

use crate::common::{code_name, same_json};

/// The folder the 66 files are copied to.
fn vanilla_pieces() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/persona_piece/vanilla")
}

/// Every `*.meta.json` below `dir`, by path.
fn meta_files(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for entry in std::fs::read_dir(dir).expect("the fixture folder") {
        let path = entry.expect("an entry").path();
        if path.is_dir() {
            found.extend(meta_files(&path));
        } else if path.to_string_lossy().ends_with(".meta.json") {
            found.push(path);
        }
    }
    found.sort();
    found
}

/// Each vanilla meta file as the reader takes it, with its text.
fn vanilla_metas() -> Vec<(String, String, PieceMetadata)> {
    let root = vanilla_pieces();
    meta_files(&root)
        .into_iter()
        .map(|path| {
            let name = path
                .strip_prefix(&root)
                .expect("below the root")
                .to_string_lossy()
                .replace('\\', "/");
            let text = std::fs::read_to_string(&path).expect("the file");
            let document = PieceMetadata::parse(
                &PackPath::new(&format!("pieces/{name}")).expect("a pack path"),
                text.as_bytes(),
                &LoadContext::detached(),
            )
            .unwrap_or_else(|rejected| panic!("{name}: {:?}", rejected.diagnostics));
            let told: Vec<String> = document
                .diagnostics()
                .iter()
                .filter(|found| found.severity >= Severity::Warning)
                .map(|found| format!("{} {}", code_name(found), found.location.json_path))
                .collect();
            assert!(told.is_empty(), "{name}: {told:?}");
            (name, text, document.value().clone())
        })
        .collect()
}

#[test]
fn every_vanilla_meta_file_loads_and_is_written_back_as_read() {
    let metas = vanilla_metas();
    assert_eq!(metas.len(), 66);
    for (name, text, meta) in metas {
        let written = meta.to_json().unwrap_or_else(|e| panic!("{name}: {e}"));
        assert!(same_json(&written, &text), "{name}:\n{written}\n{text}");
    }
}

#[test]
fn the_empty_arm_size_of_vanilla_is_the_bound_constant() {
    // 116 `arm_size` members, 60 of them `""`.
    let mut sizes = std::collections::BTreeMap::new();
    for (_, _, meta) in vanilla_metas() {
        for source in meta.geometry_sources.iter().flatten() {
            let Some(written) = &source.arm_size else {
                continue;
            };
            assert!(matches!(written, Written::Known(_)), "{written:?}");
            *sizes.entry(written.as_str().to_owned()).or_insert(0) += 1;
        }
    }
    let expected: std::collections::BTreeMap<String, i32> = [("", 60), ("slim", 28), ("wide", 28)]
        .into_iter()
        .map(|(name, count)| (name.to_owned(), count))
        .collect();
    assert_eq!(sizes, expected);
    assert_eq!(ArmSize::from_name("").map(ArmSize::value), Some(3));
}
