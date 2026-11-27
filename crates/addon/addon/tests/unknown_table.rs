//! The `Unknown` side table of a loaded document: a document written as it was read writes
//! every unknown member it kept (none is left out: each row is at an object the value
//! holds), over every fixture the characterization reads, as written and with unknown members
//! put in; and an edited document that no longer holds an object writes without that object's
//! unknown members and tells each (`DroppedField`, a warning of the write).

#![cfg(feature = "full")]

#[path = "characterize_unknown/fixtures.rs"]
mod fixtures;

use std::path::PathBuf;

use ba::diagnostic::{Phase, Severity};
use ba::document::{Content, Document, WriteWith};
use ba::json::WriteStyle;
use ba::load::LoadContext;
use ba::storage::PackPath;
use bedrock_addon as ba;
use fixtures::{each_kind, files, variants};

/// What one document type made of the texts it read.
#[derive(Default)]
struct Kept {
    /// Documents loaded, and of them those with an unknown member.
    loaded: usize,
    with_unknown: usize,
    /// Unknown members a write left out: `file: path`.
    left_out: Vec<String>,
}

/// The write of `document` and what it left out.
fn written<T: WriteWith>(document: &Document<T>, kept: &mut Kept, record: &str) {
    kept.loaded += 1;
    kept.with_unknown += usize::from(!document.unknown().is_empty());
    let mut out = Vec::new();
    if let Ok(told) = document.write_json_reporting(&mut out, WriteStyle::default()) {
        for finding in told {
            kept.left_out
                .push(format!("{record}: {}", finding.location.json_path));
        }
    }
}

/// A document type and how it reads and writes a text.
struct Kind {
    name: &'static str,
    claims: fn(&str) -> bool,
    run: fn(&PackPath, &[u8], &LoadContext, &mut Kept, &str),
}

macro_rules! kind {
    ($name:literal, $ty:ty, $claims:expr $(, chain)?) => {
        Kind {
            name: $name,
            claims: $claims,
            run: |path, bytes, cx, kept, record| {
                if let Ok(document) = <$ty as Content>::parse(path, bytes, cx) {
                    written(&document, kept, record);
                }
            },
        }
    };
}

fn kinds() -> Vec<Kind> {
    each_kind!(kind)
}

#[test]
fn an_unedited_document_writes_every_unknown_member_it_kept() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut all = Vec::new();
    files(&root, &root, &mut all);
    let cx = LoadContext::detached();
    let mut kept = Kept::default();
    for (relative, path) in &all {
        let segments: Vec<&str> = relative.split('/').collect();
        for kind in kinds() {
            let Some(pack_path) = (0..segments.len())
                .map(|from| segments[from..].join("/"))
                .find(|tail| (kind.claims)(tail))
            else {
                continue;
            };
            let bytes = std::fs::read(path).unwrap();
            let pack_path = PackPath::new(&pack_path).unwrap();
            for (variant, text) in variants(&bytes) {
                let record = format!("{} {relative} {variant}", kind.name);
                (kind.run)(&pack_path, &text, &cx, &mut kept, &record);
            }
        }
    }
    assert!(
        kept.left_out.is_empty(),
        "{} unknown members left out:\n{}",
        kept.left_out.len(),
        kept.left_out.join("\n")
    );
    // The texts with unknown members put in say something only when documents keep them.
    assert!(
        kept.with_unknown > 100,
        "{} of {} documents kept an unknown member",
        kept.with_unknown,
        kept.loaded
    );
}

#[cfg(feature = "block")]
#[test]
fn an_edited_document_without_the_object_leaves_its_unknown_members_out_and_tells_them() {
    use ba::families::block::Block;
    use ba::families::block::component::DestructibleByMining;
    let text = r#"{
  "format_version": "1.21.40",
  "minecraft:block": {
    "description": { "identifier": "demo:orphan" },
    "components": {
      "minecraft:destructible_by_mining": { "seconds_to_destroy": 1.0, "demo_note": 7 }
    }
  }
}"#;
    let cx = LoadContext::detached();
    let document = Block::parse(
        &PackPath::from_static("blocks/orphan.json"),
        text.as_bytes(),
        &cx,
    )
    .unwrap();
    let component = ba::diagnostic::JsonPath::of_keys([
        "minecraft:block",
        "components",
        "minecraft:destructible_by_mining",
    ]);
    assert!(document.unknown().contains(&component, "demo_note"));
    // Unedited, the member is written back.
    assert!(document.to_json().unwrap().contains("\"demo_note\": 7"));
    // Edited: the component is gone, and with it the object its member was kept at.
    let edited = document.map(|mut block| {
        block.components.remove::<DestructibleByMining>();
        block
    });
    let mut out = Vec::new();
    let told = edited
        .write_json_reporting(&mut out, WriteStyle::default())
        .unwrap();
    let written = String::from_utf8(out).unwrap();
    assert!(!written.contains("demo_note"), "{written}");
    assert_eq!(told.len(), 1, "{told:?}");
    let finding = &told[0];
    assert!(
        matches!(finding.code.id(), "core/dropped_field_worded"),
        "{finding:?}"
    );
    assert_eq!(finding.severity, Severity::Warning);
    assert_eq!(finding.phase, Phase::Write);
    assert_eq!(finding.location.json_path, component.child("demo_note"));
    // `to_json` writes the same text, without the finding.
    assert_eq!(edited.to_json().unwrap(), written);
}
