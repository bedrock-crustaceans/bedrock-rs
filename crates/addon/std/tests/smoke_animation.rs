//! Animation smoke over the game's own files: every animation and animation controller of the
//! behaviour and resource packs of the sample packs, the server install and the two client
//! installs loads through `Driver` and `animations()` / `animation_controllers()` on its stack
//! with no error-level diagnostic with the Molang compile check on (MCB documents, the baked
//! documents the loaders do not decode, are counted apart from errors), and every
//! animation occurrence of the Molang pairs file loads with its text and version.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo test -p bedrock_addon_std --test
//! smoke_animation`; without the variable every test skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{Diagnostic, Severity};
use bedrock_addon::load::{LoadOptions, StackOptions};
use bedrock_addon::pack::{OpenOptions, PackOrigin, PackStack, StackSet};
use bedrock_addon_std::{Driver, Options};

use common::game_files;

/// Which stack a pack directory goes to.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Behavior,
    Resource,
}

/// The stack set of the base-game pack `dir` alone, on the side of `kind`, loaded by the driver
/// that opened it.
fn load(dir: &Path, kind: Kind) -> StackSet {
    let mut driver = Driver::new(Options::default().with_open(OpenOptions {
        origin: PackOrigin::Package,
        ..OpenOptions::default()
    }))
    .unwrap();
    let pack = driver
        .pack_dir(dir)
        .unwrap_or_else(|e| panic!("{}: {e:?}", dir.display()));
    // The Molang compile check on: every expression of the vanilla files compiles at its
    // version.
    let options = LoadOptions {
        molang: Some(std::sync::Arc::new(
            bedrock_addon::molang::MolangCheck::new(),
        )),
        ..LoadOptions::default()
    };
    let set = match kind {
        Kind::Behavior => StackSet {
            behavior: PackStack::of([pack], options.clone()).unwrap(),
            resource: PackStack::of([], options).unwrap(),
        },
        Kind::Resource => StackSet {
            behavior: PackStack::of([], options.clone()).unwrap(),
            resource: PackStack::of([pack], options).unwrap(),
        },
    };
    driver.load_stacks(set, StackOptions::default()).unwrap()
}

/// The animation MCB documents on disk below the pack `dir`.
fn animation_mcb(dir: &Path) -> usize {
    [
        dir.join("animations"),
        dir.join("animation_controllers"),
        dir.join("__brarchive").join("animations.brarchive"),
        dir.join("__brarchive")
            .join("animation_controllers.brarchive"),
    ]
    .iter()
    .filter(|path| path.exists())
    .map(|path| game_files::mcb_on_disk(path).total())
    .sum()
}

/// What the packs of one source reported.
#[derive(Default)]
struct Tally {
    animations: usize,
    controllers: usize,
    errors: Vec<String>,
    mcb: usize,
    warnings: BTreeMap<&'static str, usize>,
}

impl Tally {
    /// Adds the findings of one pack's two registries.
    fn add<'d>(&mut self, dir: &Path, findings: impl Iterator<Item = &'d Diagnostic>) {
        let on_disk = animation_mcb(dir);
        let mut unsupported = Vec::new();
        for diagnostic in findings {
            if diagnostic.code.name() == "UnsupportedFileFormat" {
                unsupported.push(diagnostic.to_string());
            } else if diagnostic.severity >= Severity::Error {
                self.errors.push(format!("{}: {diagnostic}", dir.display()));
            } else if diagnostic.severity == Severity::Warning {
                *self.warnings.entry(diagnostic.code.name()).or_insert(0) += 1;
            }
        }
        self.mcb += unsupported.len();
        if unsupported.len() > on_disk {
            self.errors.extend(unsupported);
        }
    }
}

/// Loads every pack in `dirs` on the side of `kind` and requires no error, and the
/// warnings by code to be `warnings`: the pin is exact, so a game file that gains a warning
/// fails the smoke.
fn assert_loads(source: &str, dirs: &[PathBuf], kind: Kind, warnings: &[(&str, usize)]) {
    let mut tally = Tally::default();
    for dir in dirs {
        if !dir.join("manifest.json").is_file() {
            continue;
        }
        let set = load(dir, kind);
        let (animations, controllers) = match kind {
            Kind::Behavior => (
                set.behavior.animations(),
                set.behavior.animation_controllers(),
            ),
            Kind::Resource => (
                set.resource.animations(),
                set.resource.animation_controllers(),
            ),
        };
        tally.animations += animations.len();
        tally.controllers += controllers.len();
        tally.add(
            dir,
            animations.diagnostics().chain(controllers.diagnostics()),
        );
    }
    eprintln!(
        "{source}: {} animations, {} controllers loaded, {} errors, {} MCB documents, warnings {:?}",
        tally.animations,
        tally.controllers,
        tally.errors.len(),
        tally.mcb,
        tally.warnings
    );
    assert!(tally.errors.is_empty(), "{source}: {:#?}", tally.errors);
    game_files::assert_warnings(source, &tally.warnings, warnings);
}

/// The `vanilla*` packs of a packs directory.
fn vanilla_packs(packs: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(packs) else {
        return Vec::new();
    };
    let mut dirs: Vec<PathBuf> = entries
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("vanilla"))
        })
        .collect();
    dirs.sort();
    dirs
}

#[test]
fn sample_packs_behavior_pack_loads() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    assert_loads(
        "sample packs behavior_pack",
        &[samples.join("behavior_pack")],
        Kind::Behavior,
        &[],
    );
}

#[test]
fn sample_packs_resource_pack_loads() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    assert_loads(
        "sample packs resource_pack",
        &[samples.join("resource_pack")],
        Kind::Resource,
        // The same-pack duplicate is an information of the crate's own (the game accepts the duplicate
        // without a word, so the crate only notes it): no warning is left.
        &[],
    );
}

#[test]
fn server_install_behavior_packs_load() {
    let server = skip!(game_files::SERVER_INSTALL);
    assert_loads(
        "server install behavior_packs",
        &vanilla_packs(&server.join("behavior_packs")),
        Kind::Behavior,
        &[],
    );
}

#[test]
fn server_install_resource_packs_load() {
    let server = skip!(game_files::SERVER_INSTALL);
    assert_loads(
        "server install resource_packs",
        &vanilla_packs(&server.join("resource_packs")),
        Kind::Resource,
        &[],
    );
}

#[test]
fn client_install_b_resource_packs_load() {
    let client = skip!(game_files::CLIENT_INSTALL_B);
    assert_loads(
        "client install B resource_packs",
        &vanilla_packs(&client.join("resource_packs")),
        Kind::Resource,
        &[],
    );
}

#[test]
fn client_install_b_behavior_packs_load() {
    let client = skip!(game_files::CLIENT_INSTALL_B);
    assert_loads(
        "client install B behavior_packs",
        &vanilla_packs(&client.join("behavior_packs")),
        Kind::Behavior,
        &[],
    );
}

#[test]
fn client_install_a_resource_packs_load() {
    let client = skip!(game_files::CLIENT_INSTALL_A);
    assert_loads(
        "client install A resource_packs",
        &vanilla_packs(&client.join("resource_packs")),
        Kind::Resource,
        &[],
    );
}

/// Every animation and controller occurrence of the Molang pairs file (the packs of client install B)
/// loads with the pair's text and its effective version: the pack's for animations, the
/// controller rule for controllers.
#[test]
fn every_animation_molang_occurrence_loads_with_its_text_and_version() {
    use bedrock_addon::document::Content;
    use bedrock_addon::families::animation::AnimationFile;
    use bedrock_addon::families::animation_controller::AnimationControllerFile;
    use bedrock_addon::json::Value;
    use bedrock_addon::load::LoadContext;
    use bedrock_addon::molang::{MolangFieldMut, MolangFields, RawVersion, visit_fields};
    use bedrock_addon::pack::PackType;
    use bedrock_addon::storage::PackPath;
    use bedrock_addon::version::{MinEngineVersion, SemVersion};

    /// Every Molang member of a document: JSON pointer → (text, version).
    type Members = BTreeMap<String, (String, Option<RawVersion>)>;

    /// Every Molang member of `document`.
    fn members<T: MolangFields>(document: &mut T) -> Members {
        let mut found = BTreeMap::new();
        visit_fields(
            document,
            &bedrock_addon::diagnostic::JsonPath::new(),
            |field| {
                let source = match field.value {
                    MolangFieldMut::Value(value) => value.source().cloned(),
                    MolangFieldMut::Source(source) => Some(source.clone()),
                    MolangFieldMut::List(_) => None,
                };
                if let Some(source) = source {
                    found.insert(
                        format!("/{}", field.path),
                        (source.as_str().to_owned(), source.raw_version()),
                    );
                }
            },
        );
        found
    }

    let pairs = skip!(game_files::MOLANG_PAIRS);
    let install_b = skip!(game_files::CLIENT_INSTALL_B);
    let text = std::fs::read_to_string(pairs).unwrap();
    let pairs_document: Value =
        bedrock_addon::json::parse_at(bedrock_addon::json::JsonSlice::new(&text)).unwrap();
    let mut cache: BTreeMap<PathBuf, Option<Members>> = BTreeMap::new();
    let (mut checked, mut archived, mut keys) = (0, 0, 0);
    let mut failures = Vec::new();
    for pair in pairs_document
        .as_object()
        .unwrap()
        .get("pairs")
        .unwrap()
        .as_array()
        .unwrap()
    {
        let pair = pair.as_object().unwrap();
        let expr = pair.get("expr").unwrap().as_string().unwrap().as_str();
        for occurrence in pair.get("occurrences").unwrap().as_array().unwrap() {
            let o = occurrence.as_object().unwrap();
            let field = |key: &str| o.get(key).unwrap().as_string().unwrap().as_str().to_owned();
            let family = field("family");
            if family != "animations" && family != "animation_controllers" {
                continue;
            }
            if o.get("is_key").and_then(Value::as_bool) == Some(true) {
                keys += 1;
                continue;
            }
            let path = install_b
                .join(field("pack_type"))
                .join(field("pack"))
                .join(field("file"));
            let members = cache.entry(path.clone()).or_insert_with(|| {
                let text = std::fs::read_to_string(&path).ok()?;
                let version = SemVersion::parse(&field("pack_min_engine_version"))?;
                let pack_type = if field("pack_type") == "resource_packs" {
                    PackType::Resource
                } else {
                    PackType::Behavior
                };
                let cx = LoadContext::for_pack(
                    &LoadOptions::default(),
                    uuid::Uuid::nil(),
                    pack_type,
                    Some(MinEngineVersion { version }),
                    true,
                );
                let at = PackPath::new(&field("file")).unwrap();
                Some(if family == "animations" {
                    members(
                        &mut AnimationFile::parse(&at, text.as_bytes(), &cx)
                            .ok()?
                            .into_value(),
                    )
                } else {
                    members(
                        &mut AnimationControllerFile::parse(&at, text.as_bytes(), &cx)
                            .ok()?
                            .into_value(),
                    )
                })
            });
            let Some(members) = members else {
                archived += 1;
                continue;
            };
            let version = o
                .get("version_effective")
                .unwrap()
                .as_number()
                .unwrap()
                .to_i64()
                .unwrap();
            let expected = (
                expr.to_owned(),
                Some(RawVersion(i16::try_from(version).unwrap())),
            );
            let pointer = field("pointer");
            match members.get(&pointer) {
                // The pairs hold the text trimmed (their "selection" rule); the source is as written.
                Some((text, version))
                    if (text.trim(), version) == (expected.0.as_str(), &expected.1) =>
                {
                    checked += 1
                }
                found => failures.push(format!(
                    "{}: {pointer}: {found:?} != {expected:?}",
                    path.display()
                )),
            }
        }
    }
    eprintln!(
        "Molang pairs: {checked} animation occurrences checked, {archived} in files not read, {keys} keys"
    );
    assert!(
        failures.is_empty(),
        "{} failures: {:#?}",
        failures.len(),
        &failures[..failures.len().min(20)]
    );
    assert!(checked > 0);
}
