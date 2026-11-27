//! Generates `bedrock_vanilla_data`'s per-version biome id->name tables (Phase 4's
//! biome-resolution task) from the pinned dumps under `xtask/dumps/bedrock-samples/`.
//!
//! A separate, smaller pipeline from `codegen::palette` (different dump shape, no
//! cross-version diffing -- `ContentProvider::biome_name` is a plain per-version id
//! lookup, not a schema to upgrade across versions), but driven from the same manifest
//! and the same `cargo xtask generate-palette` command, and writing into the same
//! `crates/vanilla_data/src/generated/` directory: `codegen::palette::generate` calls
//! [`build_all`] and folds the result into its own single write pass, rather than this
//! module clearing or writing the output directory itself, so the two pipelines never
//! race over who owns that directory's lifecycle.

pub mod emit;
pub mod input;
pub mod model;

use std::path::Path;

use super::palette::GeneratorError;
use super::palette::manifest::{self, Manifest};

/// Reads and validates every entry in `manifest.biome_entries`, returning the sorted
/// (oldest-first), validated [`model::VersionBiomes`] list ready for [`emit`].
///
/// Unlike `codegen::palette::generate`, does not check for duplicate game versions across
/// entries -- there are only three pinned biome entries today and they come from the same
/// manifest already checked for duplicate block-dump versions; if that stops holding once
/// the biome ladder grows independently of the block ladder, add the same check `palette`
/// does.
///
/// # Errors
///
/// Returns the error of the first entry whose dump cannot be read, does not match its
/// pinned hash, does not parse, or fails [`model::build`]'s validation.
pub fn build_all(
    dumps_dir: &Path,
    manifest: &Manifest,
) -> Result<Vec<model::VersionBiomes>, GeneratorError> {
    let mut out = Vec::with_capacity(manifest.biome_entries.len());
    for entry in &manifest.biome_entries {
        let text = manifest::read_pinned_biome_file(dumps_dir, entry)?;
        let raw: input::RawBiomeDump =
            serde_json::from_str(&text).map_err(|source| GeneratorError::ParseBiomeDump {
                tag: entry.tag.clone(),
                source,
            })?;
        let game_version = (
            entry.game_version[0],
            entry.game_version[1],
            entry.game_version[2],
        );
        out.push(model::build(&entry.tag, game_version, raw)?);
    }
    out.sort_by_key(|b| b.game_version);
    Ok(out)
}
