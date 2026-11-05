//! Vanilla smoke for particle effects: the `particles/` documents of the sample packs'
//! `resource_pack/` and of every resource pack of the two client installs load with zero
//! error-level diagnostics; MCB documents are decoded and read like their JSON twins. The Molang
//! of the files of client install B is checked against the Molang pairs file.
//!
//! Two passes per pack folder. Through the `Driver`, as a stack of that one pack: what the
//! game's own walk gives, where an installed pack's `__brarchive/particles.brarchive` shadows
//! the loose files with MCB documents, which the loader decodes. And every loose `.json` file
//! below `particles/` on its own, with the pack's load context: the JSON the archive entries
//! were made from, so each vanilla document is read whether the install uses it or not.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo nextest run -p bedrock_addon_std --test
//! smoke_particle --no-capture`; without the variable every test skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;
#[allow(dead_code, reason = "this family uses the key-aware check")]
#[path = "common/pairs.rs"]
mod pairs;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{Diagnostic, DiagnosticCode, JsonPath, Severity};
use bedrock_addon::document::{Content, JsonDocument as _};
use bedrock_addon::families::particle::ParticleEffect;
use bedrock_addon::families::particle::history::Any;
use bedrock_addon::load::{LoadContext, LoadOptions, StackOptions};
use bedrock_addon::molang::{MolangCheck, MolangFieldMut, MolangFields, SourceForm, visit_fields};
use bedrock_addon::pack::{PackOrigin, PackRepository, PackType};
use bedrock_addon::storage::PackPath;
use bedrock_addon_std::{Driver, Options};

use common::game_files;
use common::world::base_game_world;

/// What the loads of one source found.
#[derive(Default)]
struct Outcome {
    /// Packs with a `particles/` directory, and the loose `.json` files in them.
    packs: usize,
    files: usize,
    /// The stack pass: effects registered, MCB documents on disk and refused by the loader.
    effects: usize,
    mcb: game_files::McbCount,
    mcb_seen: usize,
    /// The findings of the stack pass by severity and code name.
    stack_codes: BTreeMap<String, usize>,
    /// The pack folders whose stack is empty: a dependency of the pack is not in the
    /// repository of that one pack, which hides it (on the stack, never loaded).
    hidden: Vec<String>,
    /// The loose pass: documents by the struct that parsed them.
    legacy: usize,
    cereal: usize,
    rejected: Vec<String>,
    errors: Vec<String>,
    /// Warnings of the loose pass by code name, with one sample each.
    warnings: BTreeMap<String, (usize, String)>,
    /// Every Molang finding (`InvalidMolang`, `MolangLint`) of the loose pass.
    molang: Vec<String>,
}

/// The pack folders directly below `dir`.
fn pack_roots(dir: &Path) -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|entries| entries.filter_map(|e| e.ok().map(|e| e.path())).collect())
        .unwrap_or_default();
    roots.retain(|path| path.is_dir());
    roots.sort();
    roots
}

/// The `.json` files below `dir`, sorted.
fn json_files(dir: &Path, into: &mut Vec<PathBuf>) {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            json_files(&path, into);
        } else if path.extension().is_some_and(|e| e == "json") {
            into.push(path);
        }
    }
}

/// The load options: the Molang compile check is on.
fn options() -> LoadOptions {
    LoadOptions {
        molang: Some(std::sync::Arc::new(MolangCheck::new())),
        ..LoadOptions::default()
    }
}

/// One finding as text.
fn text(folder: &str, file: &str, finding: &Diagnostic) -> String {
    format!(
        "{folder}/{file} {} {}",
        finding.location.json_path, finding.code
    )
}

/// Loads the resource pack at `root`: as a stack of its own, then file by file.
#[expect(
    clippy::too_many_lines,
    reason = "the stack load and the file-by-file load share one outcome"
)]
fn load_pack(driver: &mut Driver, root: &Path, outcome: &mut Outcome) {
    let directory = root.join("particles");
    if !directory.is_dir() {
        return;
    }
    let folder = root.file_name().unwrap().to_string_lossy().into_owned();
    let Ok(pack) = driver.pack_dir(root) else {
        outcome
            .errors
            .push(format!("{folder}: the pack does not open"));
        return;
    };
    let min_engine_version = pack.manifest().header.min_engine_version.clone();
    let uuid = pack.id().uuid;
    let mut repository = PackRepository::new();
    repository.add_pack(pack, PackOrigin::Package);
    let options = options();
    let set = repository.stacks(&base_game_world(&repository, &options), options.clone());
    let hidden = set.selection().iter().any(|finding| {
        matches!(&finding.code, DiagnosticCode::MissingDependency { pack, .. } if pack.uuid == uuid)
    });
    let stacks = driver
        .load_stacks(set, StackOptions::default())
        .expect("the stacks load");
    assert_eq!(
        stacks.resource.packs().len(),
        usize::from(!hidden),
        "{folder}"
    );
    let archive = root.join("__brarchive").join("particles.brarchive");
    let mcb = game_files::mcb_on_disk(&directory) + game_files::mcb_on_disk(&archive);
    outcome.mcb = outcome.mcb + mcb;
    outcome.packs += 1;
    let particles = stacks.resource.particles();
    outcome.effects += particles.len();
    let mut unsupported = 0;
    // The stack's report holds the loader's own findings beside the registry's: a file it
    // could not hand to the parser is there.
    let stack_report = stacks.resource.report();
    let of_particles = stack_report.iter().filter(|finding| {
        let path = finding.location.path.as_ref();
        path.is_some_and(|path| path.as_str().starts_with("particles/"))
    });
    for finding in of_particles {
        *outcome
            .stack_codes
            .entry(format!("{:?} {}", finding.severity, finding.code.name()))
            .or_insert(0) += 1;
        if matches!(finding.code, DiagnosticCode::UnsupportedFileFormat) {
            unsupported += 1;
        } else if finding.severity >= Severity::Error {
            let file = finding.location.path.as_ref().map(ToString::to_string);
            outcome
                .errors
                .push(text(&folder, &file.unwrap_or_default(), finding));
        }
    }
    outcome.mcb_seen += unsupported;
    // The loader decodes every MCB document of a pack it loads (none is refused as an
    // unsupported format), and reads no file of a pack a missing dependency hides.
    if unsupported != 0 || (hidden && !particles.is_empty()) {
        outcome.errors.push(format!(
            "{folder}: {unsupported} files of an unsupported format and {} effects, {} MCB \
             documents on disk, hidden: {hidden}",
            particles.len(),
            mcb.total()
        ));
    }
    if hidden {
        outcome.hidden.push(folder.clone());
    }

    let cx = LoadContext::for_pack(&options, uuid, PackType::Resource, min_engine_version, true);
    let mut files = Vec::new();
    json_files(&directory, &mut files);
    outcome.files += files.len();
    for path in files {
        let relative = path.strip_prefix(root).unwrap().to_string_lossy();
        let bytes = std::fs::read(&path).unwrap();
        if game_files::is_mcb(&bytes) {
            continue;
        }
        let at = PackPath::new(&relative.replace('\\', "/")).unwrap();
        let diagnostics = match ParticleEffect::parse(&at, &bytes, &cx) {
            Ok(document) => {
                if document.version().upgraded {
                    outcome.legacy += 1;
                } else {
                    outcome.cereal += 1;
                }
                document.diagnostics().clone()
            }
            Err(rejected) => {
                outcome.rejected.push(format!("{folder}/{relative}"));
                rejected.diagnostics
            }
        };
        for finding in &diagnostics {
            let text = text(&folder, &relative, finding);
            if matches!(
                finding.code,
                DiagnosticCode::InvalidMolang { .. } | DiagnosticCode::MolangLint { .. }
            ) {
                outcome
                    .molang
                    .push(format!("{:?} {text}", finding.severity));
            }
            if finding.severity == Severity::Warning {
                let entry = outcome
                    .warnings
                    .entry(finding.code.name().to_owned())
                    .or_insert((0, text.clone()));
                entry.0 += 1;
            }
            if finding.severity >= Severity::Error {
                outcome.errors.push(text);
            }
        }
    }
}

/// Loads every pack folder below `dir`.
fn load_each(driver: &mut Driver, dir: &Path) -> Outcome {
    let mut outcome = Outcome::default();
    for root in pack_roots(dir) {
        load_pack(driver, &root, &mut outcome);
    }
    outcome
}

/// Prints the source's lines and requires no error outside MCB documents and no rejected
/// file.
fn report(source: &str, outcome: &Outcome) {
    eprintln!(
        "{source}: {} packs with particles/; as stacks: {} effects, MCB {:?} ({} refused as \
         unsupported); file by file: {} .json files, {} read by the 1.10.0 struct, {} by the \
         1.20.80 one, {} rejected; {} errors, {} warnings, {} Molang findings; hidden by a \
         missing dependency: {:?}",
        outcome.packs,
        outcome.effects,
        outcome.mcb,
        outcome.mcb_seen,
        outcome.files,
        outcome.legacy,
        outcome.cereal,
        outcome.rejected.len(),
        outcome.errors.len(),
        outcome.warnings.values().map(|w| w.0).sum::<usize>(),
        outcome.molang.len(),
        outcome.hidden,
    );
    eprintln!("  as stacks, findings: {:?}", outcome.stack_codes);
    for (code, (count, sample)) in &outcome.warnings {
        eprintln!("  {count} × warning {code}, e.g. {sample}");
    }
    for finding in outcome.molang.iter().take(60) {
        eprintln!("  molang: {finding}");
    }
    for error in &outcome.errors {
        eprintln!("  {error}");
    }
    assert!(outcome.errors.is_empty(), "{source}: {:#?}", outcome.errors);
    assert!(
        outcome.rejected.is_empty(),
        "{source}: {:#?}",
        outcome.rejected
    );
    assert_eq!(
        outcome.legacy + outcome.cereal,
        outcome.files,
        "{source}: every .json file is a loaded document"
    );
    // No vanilla particle file has a Molang finding (so there is none to sample).
    assert!(outcome.molang.is_empty(), "{source}: {:#?}", outcome.molang);
}

/// The stack pass of a source, as counted: the effects registered, the MCB documents the
/// loader refused, the findings by severity and code, and the packs a dependency hid.
type StackPass<'a> = (usize, usize, Vec<(&'a str, usize)>, Vec<&'a str>);

/// What `outcome`'s stack pass counted.
fn stack_pass(outcome: &Outcome) -> StackPass<'_> {
    (
        outcome.effects,
        outcome.mcb_seen,
        outcome
            .stack_codes
            .iter()
            .map(|(code, count)| (code.as_str(), *count))
            .collect(),
        outcome.hidden.iter().map(String::as_str).collect(),
    )
}

/// The warnings of the file-by-file pass, by code.
fn warnings(outcome: &Outcome) -> Vec<(&str, usize)> {
    let warnings = outcome.warnings.iter();
    warnings
        .map(|(code, (count, _))| (code.as_str(), *count))
        .collect()
}

/// A driver whose packs are base-game packs.
fn base_game_driver() -> Driver {
    let mut options = Options::default();
    options.open.origin = PackOrigin::Package;
    Driver::new(options).expect("a driver")
}

#[test]
fn sample_packs_particles_load_without_errors() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    let mut driver = base_game_driver();
    let mut outcome = Outcome::default();
    load_pack(&mut driver, &samples.join("resource_pack"), &mut outcome);
    report("sample packs", &outcome);
    // 192 files, all JSON, each its own identifier: the stack registers every one.
    assert_eq!(outcome.files, 192);
    assert_eq!(outcome.mcb.total(), 0);
    assert_eq!((outcome.legacy, outcome.cereal), (4, 188));
    // The stack reads the same JSON files, so it has their three warnings too
    // (`geyser_plume`, `noxious_gas_particle`, `sulfur_bubble_particle`).
    assert_eq!(
        stack_pass(&outcome),
        (192, 0, vec![("Info InvalidValue", 3)], vec![])
    );
    // the dropped `velocity` is an information: the game drops it silently
    assert_eq!(warnings(&outcome), [] as [(&str, usize); 0]);
}

#[test]
fn client_install_b_particles_load_without_errors() {
    let client = skip!(game_files::CLIENT_INSTALL_B);
    let mut driver = base_game_driver();
    let outcome = load_each(&mut driver, &client.join("resource_packs"));
    report("client install B", &outcome);
    // 213 files in 28 packs; 7 at 1.10.0, 201 at 1.26.10 and 5 at 1.26.30.
    assert_eq!(outcome.packs, 28);
    assert_eq!(outcome.files, 213);
    assert_eq!((outcome.legacy, outcome.cereal), (7, 206));
    // Each pack's `__brarchive/particles.brarchive` holds its particle files as MCB
    // documents, 200 in all, and shadows the loose JSON files of the same paths. The stacks
    // register 212 effects: the 199 decoded documents of the packs that load and the 13 loose
    // files no archive has (213 - 200). The one document never seen is `chemistry`'s
    // (`evaporation_elephant_toothpaste`). That resource pack depends on the chemistry
    // behaviour pack (UUID and version `[1, 0, 0]`, which is that pack's header), and each
    // pack here is loaded from a repository of its own: the dependency is not registered,
    // so the pack is hidden (planned, never loaded) and none of its files is read. In a
    // repository of the whole install it is found and the pack loads; the
    // `chemistry_<version>` slices stay hidden there for another reason: their `[1, 0, 0]`
    // dependency names no registered pack. No decoded document has a finding: the three
    // warnings of the loose pass are about `velocity`, which the bake does not keep, in files
    // the archives shadow.
    assert_eq!(outcome.mcb.total(), 200);
    assert_eq!(stack_pass(&outcome), (212, 0, vec![], vec!["chemistry"]));
    // the dropped `velocity` is an information: the game drops it silently
    assert_eq!(warnings(&outcome), [] as [(&str, usize); 0]);
}

#[test]
fn client_install_a_particles_load_without_errors() {
    let Some(client) = game_files::source(game_files::CLIENT_INSTALL_A) else {
        eprintln!(
            "skipped: {}",
            game_files::missing(game_files::CLIENT_INSTALL_A)
        );
        return;
    };
    let mut driver = base_game_driver();
    let outcome = load_each(&mut driver, &client.join("resource_packs"));
    report("client install A", &outcome);
    // 198 files at 1.26.10 and 9 at 1.10.0.
    assert_eq!(outcome.files, 207);
    assert_eq!((outcome.legacy, outcome.cereal), (9, 198));
    // As in client install B: 195 archived MCB documents, of which the 194 of the packs that
    // load are decoded (`chemistry` is hidden with its one document), and 12 loose files
    // outside the archives (207 - 195): 206 effects. Both warnings are in loose files of
    // `experimental_y_2026_drop_2` no archive has, so the stack of that pack has them too.
    assert_eq!(outcome.mcb.total(), 195);
    assert_eq!(
        stack_pass(&outcome),
        (206, 0, vec![("Info InvalidValue", 2)], vec!["chemistry"])
    );
    // the dropped `velocity` is an information: the game drops it silently
    assert_eq!(warnings(&outcome), [] as [(&str, usize); 0]);
}

/// The Molang members of a document as written: pointer → (text, raw version). A pointer to
/// an expression object ends at its `expression` member.
fn members<T: MolangFields>(document: &mut T, base: &JsonPath, into: &mut pairs::Members) {
    visit_fields(document, base, |field| {
        let MolangFieldMut::Value(value) = field.value else {
            return;
        };
        let Some(source) = value.source() else {
            return;
        };
        let pointer = match source.form() {
            SourceForm::Object { .. } => format!("/{}/expression", field.path),
            _ => format!("/{}", field.path),
        };
        into.insert(pointer, (source.as_str().to_owned(), source.raw_version()));
    });
}

/// Every `particles` occurrence of the Molang pairs file (the packs of client install B) names a
/// file that loads with no error-level finding, and its Molang member, read as the file
/// writes it (before the upgrade of a 1.10.0 file turns strings into expression objects),
/// has the pair's text and effective version. Four occurrences are keys: the names of
/// curves, which are the Molang variables the curves set, at the pack's version.
#[test]
fn molang_pairs_particle_molang_loads_with_its_text_and_version() {
    let pairs_path = skip!(game_files::MOLANG_PAIRS);
    let install_b = skip!(game_files::CLIENT_INSTALL_B);
    let rules = pairs::Rules::as_written(pairs::Keys::Members);
    let found = pairs::check_with_keys(&pairs_path, &install_b, "particles", &rules, |file| {
        let bytes = file.text.as_bytes();
        let loaded = ParticleEffect::parse(&file.at, bytes, &file.cx)
            .map_err(|rejected| rejected.diagnostics)?;
        let diagnostics = loaded.diagnostics().clone();
        let written = ParticleEffect::parse_as_written(&file.at, bytes, &file.cx)
            .map_err(|rejected| rejected.diagnostics)?;
        let mut members = pairs::Members::new();
        let mut payload = JsonPath::new();
        payload.push("particle_effect");
        let curves = match written.into_value() {
            // The 1.10.0 struct is the whole document: its walk starts at the root.
            Any::V1_10_0(mut effect) => {
                self::members(&mut effect, &JsonPath::new(), &mut members);
                effect.curves
            }
            Any::V1_20_80(mut effect) => {
                self::members(&mut effect, &payload, &mut members);
                effect.curves
            }
        };
        for name in curves.iter().flat_map(|curves| curves.keys()) {
            members.insert(
                format!("/particle_effect/curves/{name}"),
                (name.clone(), Some(file.cx.molang_version)),
            );
        }
        Ok((members, diagnostics))
    });
    eprintln!(
        "Molang pairs: {} particle occurrences checked in {} files",
        found.compared, found.files
    );
    assert!(
        found.failures.is_empty(),
        "{} failures: {:#?}",
        found.failures.len(),
        &found.failures[..found.failures.len().min(20)]
    );
    assert_eq!(
        (found.compared + found.keys, found.keys),
        (1730, 4),
        "the particle occurrences of the Molang pairs file, and the curve names among them"
    );
    assert_eq!(
        found.files, 211,
        "the files of those occurrences in the Molang pairs file"
    );
}
