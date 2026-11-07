//! The 221 manifests of the 1.26.20.04 client: every one loads without an error, and the shapes
//! they exercise (CRLF, comments, format 1 defaults and caps, script modules, subpacks,
//! metadata) read as vanilla reads them.

use bedrock_addon_core::diagnostic::codes as core_codes;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon::manifest::{
    Capability, Dependency, ManifestFormat, MemoryTier, ModuleType, NativeModuleVersion,
};
use bedrock_addon::pack::PackType;
use bedrock_addon::version::{MinEngineVersion, SemVersion};

use super::{fixture, fixtures, parse};

/// Every `manifest.json` under `vanilla/`, relative to the fixture directory, sorted.
fn vanilla_manifests() -> Vec<String> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.file_name().is_some_and(|name| name == "manifest.json") {
                out.push(path);
            }
        }
    }
    let root = fixtures();
    let mut paths = Vec::new();
    walk(&root.join("vanilla"), &mut paths);
    let mut relative: Vec<String> = paths
        .iter()
        .map(|path| {
            path.strip_prefix(&root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect();
    relative.sort();
    relative
}

#[test]
fn the_corpus_holds_221_manifests() {
    assert_eq!(vanilla_manifests().len(), 221);
}

#[test]
fn every_vanilla_manifest_loads_without_an_error() {
    let mut failures = Vec::new();
    for relative in vanilla_manifests() {
        match parse(&fixture(&relative)) {
            Ok(document) => {
                if let Some(error) = document.diagnostics().first_error() {
                    failures.push(format!("{relative}: {error}"));
                }
            }
            Err(rejected) => {
                failures.push(format!("{relative}: {}", rejected.diagnostics.summary()));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn vanilla_warnings_are_only_the_refused_chemistry_capability_and_the_format_1_cap() {
    let mut warnings = std::collections::BTreeMap::<&str, usize>::new();
    for relative in vanilla_manifests() {
        for diagnostic in parse(&fixture(&relative)).unwrap().diagnostics() {
            *warnings.entry(diagnostic.code.name()).or_default() += 1;
        }
    }
    let expected = std::collections::BTreeMap::from([
        ("InvalidCapability", 14),
        ("MinEngineVersionCapFormatVersion1", 5),
    ]);
    assert_eq!(warnings, expected);
}

#[test]
fn crlf_manifests_load() {
    let crlf: Vec<String> = vanilla_manifests()
        .into_iter()
        .filter(|relative| fixture(relative).windows(2).any(|w| w == b"\r\n"))
        .collect();
    assert_eq!(crlf.len(), 212, "the corpus's CRLF manifests");
    for relative in crlf {
        assert!(parse(&fixture(&relative)).is_ok(), "{relative}");
    }
}

#[test]
fn chemistry_manifests_with_comments_load_with_the_chemistry_capability() {
    let commented: Vec<String> = vanilla_manifests()
        .into_iter()
        .filter(|relative| {
            String::from_utf8(fixture(relative))
                .unwrap()
                .lines()
                .any(|line| line.trim_start().starts_with("//"))
        })
        .collect();
    assert_eq!(
        commented.len(),
        14,
        "the 14 comment-bearing chemistry manifests"
    );
    for relative in commented {
        assert!(relative.contains("chemistry"), "{relative}");
        let document = parse(&fixture(&relative)).unwrap();
        assert_eq!(document.capabilities, [Capability::Chemistry], "{relative}");
        // The server default registry has the education toggle off: chemistry is refused,
        // which is an information of ours, not a rejection.
        assert!(
            document
                .diagnostics()
                .iter()
                .any(|d| d.severity == Severity::Info
                    && d.code
                        .args_of(&core_codes::INVALID_CAPABILITY)
                        .is_some_and(|args| args == ["chemistry"])),
            "{relative}"
        );
    }
}

#[test]
fn the_vanilla_behavior_pack_reads_as_format_2_data() {
    let document = parse(&fixture("vanilla/behavior_packs/vanilla/manifest.json")).unwrap();
    assert_eq!(document.format, ManifestFormat::V2);
    assert_eq!(document.pack_type(), PackType::Behavior);
    assert_eq!(document.modules[0].module_type, ModuleType::Data);
    assert!(document.header.min_engine_version.is_some());
}

#[test]
fn the_experimental_spawn_rules_pack_has_a_script_module_and_an_internal_dependency() {
    let document = parse(&fixture(
        "vanilla/behavior_packs/experimental_spawn_rules/manifest.json",
    ))
    .unwrap();
    let types: Vec<ModuleType> = document.modules.iter().map(|m| m.module_type).collect();
    assert_eq!(types, [ModuleType::Data, ModuleType::Script]);
    assert_eq!(
        document.modules[1].entry.as_deref(),
        Some("scripts/ExperimentalSpawnRules.js")
    );
    let internal = SemVersion::parse("2.2.0-internal").unwrap();
    assert!(document.dependencies.iter().any(|d| matches!(
        d,
        Dependency::NativeModule { module_name, version: NativeModuleVersion::Version(v), optional: None }
            if module_name == "@minecraft/server" && *v == internal
    )));
}

#[test]
fn a_format_1_behavior_pack_without_min_engine_version_gets_1_2_5() {
    let document = parse(&fixture(
        "vanilla/test/assets/CraftingTableComponentFunctionalTests/CraftingTableComponentBP/manifest.json",
    ))
    .unwrap();
    assert_eq!(document.format, ManifestFormat::V1);
    assert_eq!(
        document.header.min_engine_version,
        Some(MinEngineVersion::FORMAT_1_DEFAULT)
    );
}

#[test]
fn a_format_1_min_engine_version_above_1_12_is_capped_with_a_warning() {
    let document = parse(&fixture(
        "vanilla/test/assets/ActiveResourcePackChangedFunctionalTests/MinimalRP/manifest.json",
    ))
    .unwrap();
    assert_eq!(
        document.header.min_engine_version,
        Some(MinEngineVersion::FORMAT_1_CAP)
    );
    assert!(
        document
            .diagnostics()
            .iter()
            .any(|d| d.severity == Severity::Warning
                && d.code
                    == DiagnosticCode::from(&core_codes::MIN_ENGINE_VERSION_CAP_FORMAT_VERSION1))
    );
}

#[test]
fn subpacks_and_metadata_of_the_test_packs_read() {
    let subpacks = parse(&fixture(
        "vanilla/test/assets/ArchivedPackWithSubpacksLoadingFunctionalTests/manifest.json",
    ))
    .unwrap();
    let folders: Vec<&str> = subpacks
        .subpacks
        .iter()
        .map(|s| s.folder_name.as_str())
        .collect();
    assert_eq!(folders, ["rp1", "rp2"]);
    assert!(
        subpacks
            .subpacks
            .iter()
            .all(|s| s.memory_tier == MemoryTier::Tier0)
    );

    let metadata = parse(&fixture(
        "vanilla/test/assets/ReplaceBiomesComponentFunctionalTests/ReplaceBiomesBehaviorPack/manifest.json",
    ))
    .unwrap();
    let tools = metadata
        .metadata
        .as_ref()
        .unwrap()
        .generated_with
        .as_ref()
        .unwrap();
    assert_eq!(tools["blockbench_block_wizard"].len(), 2);
    let item_wizard = &tools["blockbench_item_wizard"];
    assert_eq!(item_wizard.len(), 1);
    assert_eq!(item_wizard[0].version, SemVersion::new(1, 1, 0));
    assert_eq!(item_wizard[0].text.as_deref(), Some("1.1.0"));
}

#[test]
fn skin_packs_have_no_min_engine_version_and_no_finding() {
    let document = parse(&fixture("vanilla/skin_packs/vanilla/manifest.json")).unwrap();
    assert_eq!(document.pack_type(), PackType::Skins);
    assert_eq!(document.header.min_engine_version, None);
    assert!(document.diagnostics().is_empty());
}
