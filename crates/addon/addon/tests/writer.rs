//! `PackWriter`: a pack built in memory, its files in order, its zip
//! bytes, and the pack it opens as; the round trips run over the fixture packs.

#![allow(dead_code, reason = "each feature set uses part of the helpers")]

#[cfg(all(feature = "block", feature = "block_culling"))]
#[path = "writer/add.rs"]
mod add;
#[cfg(feature = "zip")]
#[path = "writer/archive.rs"]
mod archive;
#[cfg(feature = "block")]
#[path = "writer/example.rs"]
mod example;
#[path = "writer/files.rs"]
mod files;
#[path = "writer/from_pack.rs"]
mod from_pack;
#[path = "writer/into_pack.rs"]
mod into_pack;
#[path = "writer/packs.rs"]
mod packs;
#[path = "common/responder.rs"]
mod responder;

use std::path::PathBuf;

use bedrock_addon::manifest::{CapabilityRegistry, Manifest};
use bedrock_addon::pack::{PackType, PackWriter};
use bedrock_addon::storage::PackPath;
use uuid::Uuid;

/// The fixture directory.
fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// A fixture's bytes.
fn fixture(relative: &str) -> Vec<u8> {
    std::fs::read(fixtures().join(relative)).unwrap_or_else(|e| panic!("{relative}: {e}"))
}

fn path(text: &str) -> PackPath {
    PackPath::new(text).unwrap()
}

/// A new behaviour-pack manifest with fixed UUIDs.
fn ruby_manifest() -> Manifest {
    Manifest::with_ids(
        "ruby",
        PackType::Behavior,
        Uuid::from_u128(1),
        Uuid::from_u128(2),
    )
}

/// A manifest text, parsed.
fn parsed(bytes: &[u8]) -> Manifest {
    Manifest::parse(
        &path("manifest.json"),
        bytes,
        &CapabilityRegistry::default(),
    )
    .unwrap_or_else(|rejected| panic!("{rejected:?}"))
    .into_value()
}

/// The writer's files as `(path, bytes)`, in the order `files()` gives them.
fn listed(writer: &PackWriter) -> Vec<(String, Vec<u8>)> {
    writer
        .files()
        .map(|(path, bytes)| (path.as_str().to_owned(), bytes.to_vec()))
        .collect()
}

/// The writer's paths, in the order `files()` gives them.
fn paths(writer: &PackWriter) -> Vec<String> {
    writer
        .files()
        .map(|(path, _)| path.as_str().to_owned())
        .collect()
}

/// The bytes the writer holds at `at`.
fn bytes_at(writer: &PackWriter, at: &str) -> Option<Vec<u8>> {
    writer
        .files()
        .find(|(path, _)| path.as_str() == at)
        .map(|(_, bytes)| bytes.to_vec())
}
