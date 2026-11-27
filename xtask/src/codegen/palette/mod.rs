//! Generates `bedrock_vanilla_data`'s per-version block-palette tables (architecture
//! decision 12) from the pinned dumps under `xtask/dumps/bedrock-samples/`.
//!
//! Pipeline: [`manifest::read`] loads the pinning manifest, [`manifest::read_pinned_file`]
//! reads and hash-checks each vendored dump, [`input::RawDump`] parses its JSON
//! (`deny_unknown_fields`, so an unrecognized field is a hard error), [`model::build`]
//! validates and sorts it into a [`model::VersionPalette`], and [`emit`] renders that to
//! Rust source. [`generate`] drives the whole thing and writes the result under
//! `crates/vanilla_data/src/generated/`.

pub mod diff;
pub mod emit;
pub mod input;
pub mod manifest;
pub mod model;

use std::path::{Path, PathBuf};

/// Runs the full pipeline: reads the manifest and every pinned dump it names, validates
/// and renders each one, and writes `crates/vanilla_data/src/generated/` fresh --
/// clearing whatever was there first, so a version removed from the manifest does not
/// leave an orphaned file behind.
///
/// # Errors
///
/// Returns the first failure of the pipeline: a manifest, dump or `renames.json` that
/// cannot be read, does not match its pinned hash, is not UTF-8 or does not parse, a
/// dump that fails validation, two entries with the same game version, a rename entry
/// that does not apply, or an output directory or file that cannot be cleared, written
/// or formatted.
pub fn generate(project_root: &Path) -> Result<(), GeneratorError> {
    let dumps_dir = project_root.join("xtask/dumps/bedrock-samples");
    let out_dir = project_root.join("crates/vanilla_data/src/generated");

    let manifest = manifest::read(&dumps_dir)?;

    let mut palettes = Vec::with_capacity(manifest.entries.len());
    for entry in &manifest.entries {
        let text = manifest::read_pinned_file(&dumps_dir, entry)?;
        let raw: input::RawDump =
            serde_json::from_str(&text).map_err(|source| GeneratorError::ParseDump {
                tag: entry.tag.clone(),
                source,
            })?;
        let game_version = (
            entry.game_version[0],
            entry.game_version[1],
            entry.game_version[2],
        );
        let palette = model::build(&entry.tag, game_version, raw)?;
        palettes.push(palette);
    }

    // Sort oldest-to-newest by the game version, not by manifest order -- the manifest
    // is free to list entries in any order (provenance notes read better grouped by
    // release family), but the generated registry's order is a promise the generated
    // code and its tests can rely on.
    palettes.sort_by_key(|p| p.game_version);

    let duplicate = palettes
        .windows(2)
        .find(|pair| pair[0].game_version == pair[1].game_version);
    if let Some(pair) = duplicate {
        return Err(GeneratorError::DuplicateGameVersion {
            game_version: pair[0].game_version,
            first_tag: pair[0].tag.clone(),
            second_tag: pair[1].tag.clone(),
        });
    }

    // Cross-version diffing (the schema-driven state upgrade's data half): every adjacent
    // pair in the sorted, deduplicated ladder above, merged against the hand-authored
    // rename supplement. Computed here rather than lazily at upgrade time, so a bad
    // rename entry (see `diff::diff`'s doc comment) fails the generator run loudly
    // instead of surfacing as a silent no-op the first time something tries to upgrade
    // across that step.
    let rename_entries = diff::load_renames(&dumps_dir)?;
    let mut diffs = Vec::with_capacity(palettes.len().saturating_sub(1));
    for pair in palettes.windows(2) {
        diffs.push(diff::diff(&pair[0], &pair[1], &rename_entries)?);
    }

    // The biome id->name tables: a separate, smaller pipeline (see
    // `codegen::biome`'s module docs for why), folded into this same write pass so the
    // two pipelines never race over the shared output directory's lifecycle.
    let biomes = super::biome::build_all(&dumps_dir, &manifest)?;

    if out_dir.exists() {
        std::fs::remove_dir_all(&out_dir).map_err(|source| GeneratorError::ClearOutputDir {
            path: out_dir.clone(),
            source,
        })?;
    }
    std::fs::create_dir_all(&out_dir).map_err(|source| GeneratorError::ClearOutputDir {
        path: out_dir.clone(),
        source,
    })?;

    let mut palette_module_names = Vec::with_capacity(palettes.len());
    for palette in &palettes {
        let name = emit::module_name(palette.game_version);
        write_pretty(
            &out_dir.join(format!("{name}.rs")),
            &emit::render_version_file(palette),
        )?;
        palette_module_names.push(name);
    }

    let mut diff_module_names = Vec::with_capacity(diffs.len());
    for d in &diffs {
        let name = emit::diff_module_name(d.from_version, d.to_version);
        write_pretty(
            &out_dir.join(format!("{name}.rs")),
            &emit::render_diff_file(d),
        )?;
        diff_module_names.push(name);
    }

    let mut biome_module_names = Vec::with_capacity(biomes.len());
    for b in &biomes {
        let name = super::biome::emit::module_name(b.game_version);
        write_pretty(
            &out_dir.join(format!("{name}.rs")),
            &super::biome::emit::render_version_file(b),
        )?;
        biome_module_names.push(name);
    }

    write_pretty(
        &out_dir.join("mod.rs"),
        &emit::render_mod_file(
            &palette_module_names,
            &diff_module_names,
            &biome_module_names,
        ),
    )?;

    Ok(())
}

/// Renders `tokens` and writes them to `path`, then runs `rustfmt` over the result in
/// place.
///
/// `prettyplease` alone gets close but is not byte-identical to `rustfmt` -- the two
/// disagree on when a single-argument call whose argument is an array literal fits on
/// one line, which this generator's output hits constantly (`PropertyKind::Int(&[...])`
/// on every integer property). Shelling out to the real `rustfmt` after
/// `prettyplease::unparse` is what actually earns "generated files are exempt from the
/// rustfmt check but are rustfmt-clean by construction" rather than merely asserting it.
fn write_pretty(path: &Path, tokens: &proc_macro2::TokenStream) -> Result<(), GeneratorError> {
    let file = syn::parse2::<syn::File>(tokens.clone()).map_err(|source| {
        GeneratorError::RenderInvalid {
            path: path.to_path_buf(),
            tokens: tokens.to_string(),
            source,
        }
    })?;
    std::fs::write(path, prettyplease::unparse(&file)).map_err(|source| {
        GeneratorError::WriteOutput {
            path: path.to_path_buf(),
            source,
        }
    })?;

    let status = std::process::Command::new("rustfmt")
        .arg("--edition")
        .arg("2024")
        .arg(path)
        .status()
        .map_err(|source| GeneratorError::RunRustfmt {
            path: path.to_path_buf(),
            source,
        })?;
    if !status.success() {
        return Err(GeneratorError::RustfmtFailed {
            path: path.to_path_buf(),
            status,
        });
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum GeneratorError {
    #[error("reading palette manifest {path}: {source}")]
    ReadManifest {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("parsing palette manifest {path}: {source}")]
    ParseManifest {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("reading pinned dump {path}: {source}")]
    ReadDump {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error(
        "pinned dump {path} does not match its manifest sha256 -- expected {expected}, got \
         {actual}. The vendored file no longer matches what the manifest pins; re-vendor it \
         deliberately and update the manifest entry rather than editing around this."
    )]
    HashMismatch {
        path: PathBuf,
        expected: String,
        actual: String,
    },
    #[error("pinned dump {path} is not valid UTF-8: {source}")]
    NotUtf8 {
        path: PathBuf,
        source: std::string::FromUtf8Error,
    },
    #[error("parsing dump for {tag}: {source}")]
    ParseDump {
        tag: String,
        source: serde_json::Error,
    },
    #[error("{tag}: property `{property}` has unrecognized type `{ty}`")]
    UnknownPropertyType {
        tag: String,
        property: String,
        ty: String,
    },
    #[error(
        "{tag}: property `{property}` has a value that does not match its declared type `{ty}`"
    )]
    PropertyValueTypeMismatch {
        tag: String,
        property: String,
        ty: String,
    },
    #[error("{tag}: property `{property}` is declared more than once in block_properties")]
    DuplicateProperty { tag: String, property: String },
    #[error("{tag}: a data item has an empty block name")]
    EmptyBlockName { tag: String },
    #[error("{tag}: block `{block}` references undeclared property `{property}`")]
    UnknownPropertyRef {
        tag: String,
        block: String,
        property: String,
    },
    #[error("{tag}: block `{block}` lists the same property more than once")]
    DuplicatePropertyOnBlock { tag: String, block: String },
    #[error("{tag}: block `{block}` appears more than once in data_items")]
    DuplicateBlock { tag: String, block: String },
    #[error(
        "manifest entries `{first_tag}` and `{second_tag}` both resolve to game version \
         {game_version:?} -- each generated version module must be unique"
    )]
    DuplicateGameVersion {
        game_version: (u8, u8, u8),
        first_tag: String,
        second_tag: String,
    },
    #[error("clearing/creating generated output directory {path}: {source}")]
    ClearOutputDir {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("generated tokens for {path} do not parse as a Rust file: {source}\n---\n{tokens}")]
    RenderInvalid {
        path: PathBuf,
        tokens: String,
        source: syn::Error,
    },
    #[error("writing generated output {path}: {source}")]
    WriteOutput {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("running rustfmt on generated output {path}: {source}")]
    RunRustfmt {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("rustfmt exited unsuccessfully ({status}) formatting generated output {path}")]
    RustfmtFailed {
        path: PathBuf,
        status: std::process::ExitStatus,
    },
    #[error("reading rename supplement {path}: {source}")]
    ReadRenames {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("parsing rename supplement {path}: {source}")]
    ParseRenames {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error(
        "{path}: rename supplement entry `{from}` -> `{to}` carries no `provenance` -- see \
         renames.json's own `requirements` field for what an entry must cite"
    )]
    RenameMissingProvenance {
        path: PathBuf,
        from: String,
        to: String,
    },
    #[error(
        "rename supplement entry `{from}` -> `{to}` does not apply to the step it names: {reason}"
    )]
    RenameEntryNotApplicable {
        from: String,
        to: String,
        reason: String,
    },
    #[error("reading biome dump {path}: {source}")]
    ReadBiomeDump {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("parsing biome dump for {tag}: {source}")]
    ParseBiomeDump {
        tag: String,
        source: serde_json::Error,
    },
    #[error("{tag}: biome dump has an empty biome name")]
    EmptyBiomeName { tag: String },
    #[error("{tag}: biome id {id} is assigned to more than one biome")]
    DuplicateBiomeId { tag: String, id: i32 },
    #[error("{tag}: biome name `{name}` appears more than once")]
    DuplicateBiomeName { tag: String, name: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use model::PropertyKind;
    use std::collections::BTreeMap;

    fn project_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf()
    }

    /// Reads every file directly under `dir` into a name -> content map, for comparing
    /// two states of the generated-output directory without depending on OS-level
    /// directory-entry ordering.
    fn snapshot(dir: &Path) -> BTreeMap<String, String> {
        let mut out = BTreeMap::new();
        for entry in std::fs::read_dir(dir).expect("generated dir should exist") {
            let entry = entry.unwrap();
            let name = entry.file_name().to_string_lossy().into_owned();
            let content =
                std::fs::read_to_string(entry.path()).expect("generated file should be UTF-8");
            out.insert(name, content);
        }
        out
    }

    /// Runs the generator twice against the real pinned dumps and the real output
    /// location, and checks two things: that the two runs produce byte-identical
    /// output (the determinism the schema-generator task requires), and that the
    /// result matches what is already checked in (so a stale commit is caught the same
    /// way an uncommitted regeneration would be, rather than only failing in CI's own
    /// "run the generator and diff" step).
    #[test]
    fn generation_is_deterministic_and_matches_the_checked_in_output() {
        let root = project_root();
        let out_dir = root.join("crates/vanilla_data/src/generated");

        let before = snapshot(&out_dir);
        generate(&root).expect("generation against the pinned dumps should succeed");
        let first_run = snapshot(&out_dir);
        generate(&root).expect("a second generation run should also succeed");
        let second_run = snapshot(&out_dir);

        assert_eq!(
            first_run, second_run,
            "two generator runs produced different output"
        );
        assert_eq!(
            before, first_run,
            "generated output differs from what's checked in -- run `cargo xtask \
             generate-palette` and commit the result"
        );
    }

    /// Parses the real pinned `v1.20.10.1` dump (the earliest version this generator
    /// covers) and checks a handful of facts against it directly, rather than only
    /// against the generated Rust -- this is the check that the *parser* understood
    /// the real input, independent of whether emission is also correct.
    #[test]
    fn parses_known_facts_from_the_earliest_pinned_dump() {
        let root = project_root();
        let dumps_dir = root.join("xtask/dumps/bedrock-samples");
        let manifest = manifest::read(&dumps_dir).unwrap();
        let entry = manifest
            .entries
            .iter()
            .find(|e| e.tag == "v1.20.10.1")
            .expect("v1.20.10.1 should be a pinned entry");

        let text = manifest::read_pinned_file(&dumps_dir, entry).unwrap();
        let raw: input::RawDump = serde_json::from_str(&text).unwrap();
        let palette = model::build(&entry.tag, (1, 20, 10), raw).unwrap();

        // A block with no properties at all.
        let fence = palette
            .block("minecraft:acacia_fence")
            .expect("acacia_fence should be present");
        assert!(fence.properties.is_empty());

        // A block with more than one property, sorted.
        let stairs = palette
            .block("minecraft:acacia_stairs")
            .expect("acacia_stairs should be present");
        assert_eq!(
            stairs.properties,
            vec!["upside_down_bit", "weirdo_direction"]
        );

        // A string-typed property with its real value domain.
        let attachment = palette
            .property("attachment")
            .expect("attachment should be a known property");
        assert_eq!(
            attachment.kind,
            PropertyKind::Str(vec![
                "hanging".to_string(),
                "multiple".to_string(),
                "side".to_string(),
                "standing".to_string(),
            ])
        );

        assert!(
            palette
                .block("minecraft:this_block_does_not_exist")
                .is_none()
        );
    }

    /// A vendored dump whose bytes no longer match the manifest's pinned hash is a
    /// hard error, not a warning -- the whole audit trail architecture decision 12
    /// wants depends on the pin actually meaning something.
    #[test]
    fn hash_mismatch_against_the_manifest_is_a_hard_error() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("manifest.json"),
            r#"{"entries": [{"tag": "vX", "game_version": [9, 9, 9], "file": "vX.json", "sha256": "0000000000000000000000000000000000000000000000000000000000000"}]}"#,
        )
        .unwrap();
        std::fs::write(tmp.path().join("vX.json"), b"not the expected bytes").unwrap();

        let manifest = manifest::read(tmp.path()).unwrap();
        let err = manifest::read_pinned_file(tmp.path(), &manifest.entries[0]).unwrap_err();
        assert!(
            matches!(err, GeneratorError::HashMismatch { .. }),
            "got {err:?}"
        );
    }

    /// The minimal valid dump shape (empty tables): enough to pass `model::build`, so a
    /// test can focus on manifest-level rejection rather than dump content.
    const EMPTY_DUMP: &str = r#"{
        "block_properties": [],
        "data_items": [],
        "module_type": "vanilla_data",
        "name": "mojang-block"
    }"#;

    fn sha256_hex(bytes: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        use std::fmt::Write;
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        hasher.finalize().iter().fold(String::new(), |mut out, b| {
            write!(out, "{b:02x}").expect("writing to a String never fails");
            out
        })
    }

    /// Two manifest entries that resolve to the same `game_version` -- different
    /// upstream tags standing in for the same generated module -- is rejected before
    /// anything is written, rather than silently letting the second one win.
    #[test]
    fn duplicate_game_version_across_manifest_entries_is_a_hard_error() {
        let tmp = tempfile::tempdir().unwrap();
        let dumps_dir = tmp.path().join("xtask/dumps/bedrock-samples");
        std::fs::create_dir_all(&dumps_dir).unwrap();

        std::fs::write(dumps_dir.join("a.json"), EMPTY_DUMP).unwrap();
        std::fs::write(dumps_dir.join("b.json"), EMPTY_DUMP).unwrap();
        let hash = sha256_hex(EMPTY_DUMP.as_bytes());

        let manifest_json = format!(
            r#"{{"entries": [
                {{"tag": "vA", "game_version": [1, 0, 0], "file": "a.json", "sha256": "{hash}"}},
                {{"tag": "vB", "game_version": [1, 0, 0], "file": "b.json", "sha256": "{hash}"}}
            ]}}"#
        );
        std::fs::write(dumps_dir.join("manifest.json"), manifest_json).unwrap();

        let err = generate(tmp.path()).unwrap_err();
        match err {
            GeneratorError::DuplicateGameVersion { game_version, .. } => {
                assert_eq!(game_version, (1, 0, 0));
            }
            other => panic!("expected DuplicateGameVersion, got {other:?}"),
        }
    }
}
