#![allow(clippy::pedantic, missing_docs)]
//! The vanilla gate: the game's own packs load and link with no error and no warning, every
//! experiment on, apart from the lines the game itself logs for its own files.
//!
//! `BEDROCK_ADDON_VANILLA_1_26_60` names the install root: the folder holding
//! `behavior_packs/`, `resource_packs/`, `definitions/` and `test/`, every pack with its
//! `manifest.json` and its `__brarchive` folders as shipped. Without the variable every test
//! prints `skipped:` and passes, which is no pass of the gate. A root whose packs carry no
//! manifest (a tree of expanded brarchives) is no install: the test fails and says so.
//!
//! - [`every_vanilla_pack_loads_and_links_with_no_error_and_no_warning`]: every pack of both
//!   folders, the stacks composed as the game composes them (`common::world::base_game_world`:
//!   the base-game layer in vanilla's order, with the experimental packs whose experiment is on
//!   at its top; every other pack of the install, the chemistry packs among them, above it; the
//!   resource packs in the order of the client's base-game resource stack), the install's app
//!   data, every experiment a world can turn on with the beta APIs and the upcoming creator
//!   features, every Molang field compiled. The open (manifest) findings of every pack, the
//!   findings of `load_all()` and those of `link()` are printed, one line each with pack, file,
//!   code, severity and text, then counted per pack in a table; the test fails on any error or
//!   warning but those [`EXPECTED`] pins (the game's own content-log lines for its files,
//!   printed apart), and on a pin that does not occur exactly as often as pinned.
//!   Informations are counted and printed in one summary line.
//! - [`the_install_s_test_packs_are_read_and_reported`]: every pack folder below `test/` (the
//!   game's own test assets, many of them built to fail), each alone on its stack with the same
//!   options but no app data; reported, never asserted.
//!
//! With `BEDROCK_ADDON_VANILLA_1_26_60_DUMP` naming a folder, every finding is also written
//! there as tab-separated lines (`vanilla.tsv`, `vanilla_test_packs.tsv`).
//!
//! Run: `BEDROCK_ADDON_VANILLA_1_26_60=<root> cargo test -p bedrock_addon_std --test
//! vanilla_gate -- --nocapture`.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;
#[macro_use]
#[path = "full/support.rs"]
mod support;

use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use bedrock_addon::diagnostic::{Diagnostic, Severity};
use bedrock_addon::load::{LoadOptions, RECOMMENDED_JOB_STACK, Residency, StackOptions};
use bedrock_addon::manifest::CapabilityRegistry;
use bedrock_addon::pack::{PackIdVersion, PackOrigin, PackStack, PackType, StackSet};
use bedrock_addon_std::{DirSource, Driver, Options, read_app_data};

use common::world::base_game_world;

/// The environment variable naming the install root.
const VARIABLE: &str = "BEDROCK_ADDON_VANILLA_1_26_60";

/// The environment variable naming a folder the findings are written into.
const DUMP: &str = "BEDROCK_ADDON_VANILLA_1_26_60_DUMP";

/// The install root, or `None` after printing why the test is skipped.
fn install_root() -> Option<PathBuf> {
    let Some(root) = std::env::var_os(VARIABLE).filter(|value| !value.is_empty()) else {
        eprintln!("skipped: {VARIABLE} is not set; the vanilla gate did not run");
        return None;
    };
    let root = PathBuf::from(root);
    assert!(
        root.join("behavior_packs").is_dir() && root.join("resource_packs").is_dir(),
        "{VARIABLE}={}: no behavior_packs/ and resource_packs/ below it",
        root.display()
    );
    let vanilla = root.join("behavior_packs").join("vanilla");
    assert!(
        vanilla.join("manifest.json").is_file(),
        "{VARIABLE}={}: behavior_packs/vanilla has no manifest.json; the root is no install \
         (a tree of expanded brarchives carries no manifest); name the shipped assets folder, \
         name the folder the game ships its assets in",
        root.display()
    );
    Some(root)
}

/// The load options of the gate: every experiment a world can turn on, the beta APIs and the
/// upcoming creator features, every Molang field compiled, the install's app data, and an
/// education world, to which the game adds the chemistry behaviour packs itself.
fn gate_options(root: &Path) -> LoadOptions {
    let mut options = LoadOptions::default();
    if root.join("definitions").is_dir() {
        options.app_data = Some(read_app_data(root).expect("the app data"));
    }
    (
        options.experiments,
        options.beta_apis,
        options.upcoming_creator_features,
    ) = support::everything_on();
    options.molang = Some(Arc::new(bedrock_addon::molang::MolangCheck::new()));
    options.education = true;
    options
}

/// A driver that opens packs as the app's own, with the education toggle on: the chemistry
/// packs' `chemistry` capability succeeds (`CapabilityRegistry::server(true)`), as in an
/// education world, the only one the game adds those packs to.
fn gate_driver() -> Driver {
    let mut options = Options::default();
    options.open.origin = PackOrigin::Package;
    options.open.capabilities = CapabilityRegistry::server(true);
    Driver::new(options).expect("a driver")
}

/// When a finding was made.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Pass {
    /// Opening the pack: its manifest and container.
    Open,
    /// `load_all()`.
    Load,
    /// `link()`.
    Link,
}

impl Pass {
    fn word(self) -> &'static str {
        match self {
            Pass::Open => "open",
            Pass::Load => "load",
            Pass::Link => "link",
        }
    }
}

/// One pack of the install: where it is, what it holds, how long it took to open.
#[derive(Clone, Debug)]
struct PackRow {
    folder: String,
    kind: PackType,
    files: usize,
    open: Duration,
    counts: [usize; 3],
}

/// A finding, flattened for printing and counting.
#[derive(Clone, Debug)]
struct Found {
    pass: Pass,
    severity: Severity,
    code: &'static str,
    folder: String,
    path: String,
    json_path: String,
    text: String,
    /// The code's own text, without the location: what an expected entry's text is
    /// compared with.
    message: String,
}

impl Found {
    fn of(pass: Pass, finding: &Diagnostic, folders: &HashMap<PackIdVersion, String>) -> Found {
        let folder = finding.location.pack.as_ref().map_or_else(
            || String::from("(no pack)"),
            |pack| {
                folders
                    .get(pack.id())
                    .cloned()
                    .unwrap_or_else(|| format!("{} {}", pack.name(), pack.id().version))
            },
        );
        Found {
            pass,
            severity: finding.severity,
            code: finding.code.name(),
            folder,
            path: finding
                .location
                .path
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default(),
            json_path: finding.location.json_path.to_string(),
            text: finding.to_string(),
            message: finding.code.to_string(),
        }
    }

    fn letter(&self) -> char {
        match self.severity {
            Severity::Error => 'E',
            Severity::Warning => 'W',
            Severity::Info => 'I',
            _ => 'D',
        }
    }

    /// `E|W|I` and the index into a row's counts, or `None` below information.
    fn slot(&self) -> Option<usize> {
        match self.severity {
            Severity::Error => Some(0),
            Severity::Warning => Some(1),
            Severity::Info => Some(2),
            _ => None,
        }
    }

    fn line(&self) -> String {
        format!(
            "{} {} {} | {} | {} | {} | {}",
            self.pass.word(),
            self.letter(),
            self.code,
            self.folder,
            self.path,
            self.json_path,
            self.text
        )
    }

    fn tsv(&self) -> String {
        [
            self.pass.word(),
            &self.letter().to_string(),
            self.code,
            &self.folder,
            &self.path,
            &self.json_path,
            &self.text.replace(['\t', '\n'], " "),
        ]
        .join("\t")
    }
}

/// Every pack folder directly below `dir`, sorted.
fn folders_below(dir: &Path) -> Vec<PathBuf> {
    let mut folders: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("{}: {error}", dir.display()))
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_dir())
        .collect();
    folders.sort();
    folders
}

/// The name of a folder.
fn name_of(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// The rows of the packs that opened, the open findings by folder, the folders that did not.
type Opened = (
    Vec<(PackIdVersion, PackRow)>,
    Vec<(String, Diagnostic)>,
    Vec<String>,
);

/// Opens every pack folder of `dirs` on its own, for the table: its identity, files and the
/// wall time of the open; the open findings of each pack; the folders that do not open.
fn open_each(dirs: &[PathBuf]) -> Opened {
    let mut driver = gate_driver();
    let mut rows = Vec::new();
    let mut open_findings = Vec::new();
    let mut invalid = Vec::new();
    for dir in dirs {
        let started = Instant::now();
        match driver.pack_dir(dir) {
            Ok(pack) => {
                // `behavior_packs/vanilla`: a behaviour pack and a resource pack share folder
                // names.
                let folder = dir.parent().map_or_else(
                    || name_of(dir),
                    |parent| format!("{}/{}", name_of(parent), name_of(dir)),
                );
                for finding in pack.report().diagnostics.iter() {
                    open_findings.push((folder.clone(), finding.clone()));
                }
                rows.push((
                    pack.id().clone(),
                    PackRow {
                        folder,
                        kind: pack.pack_type(),
                        files: pack.paths().count(),
                        open: started.elapsed(),
                        counts: [0; 3],
                    },
                ));
            }
            Err(error) => invalid.push(format!("{}: {error:?}", dir.display())),
        }
    }
    (rows, open_findings, invalid)
}

/// The table of `rows` and the findings counted into it, as the report prints it.
fn table(rows: &[(PackIdVersion, PackRow)], unattributed: [usize; 3]) -> String {
    let mut out =
        String::from("| pack | kind | files | errors | warnings | informations | open |\n");
    out.push_str("| --- | --- | ---: | ---: | ---: | ---: | ---: |\n");
    for (_, row) in rows {
        let _ = writeln!(
            out,
            "| {} | {:?} | {} | {} | {} | {} | {:.1?} |",
            row.folder, row.kind, row.files, row.counts[0], row.counts[1], row.counts[2], row.open
        );
    }
    let _ = writeln!(
        out,
        "| (no pack) | - | - | {} | {} | {} | - |",
        unattributed[0], unattributed[1], unattributed[2]
    );
    out
}

/// Counts `found` into the rows by folder; what names no known pack goes to the last slot.
fn count_into(rows: &mut [(PackIdVersion, PackRow)], found: &[Found]) -> [usize; 3] {
    let mut unattributed = [0; 3];
    for finding in found {
        let Some(slot) = finding.slot() else { continue };
        match rows
            .iter_mut()
            .find(|(_, row)| row.folder == finding.folder)
        {
            Some((_, row)) => row.counts[slot] += 1,
            None => unattributed[slot] += 1,
        }
    }
    unattributed
}

/// Writes `found` as `<name>.tsv` into the folder [`DUMP`] names, if it names one.
fn dump(name: &str, found: &[Found]) {
    let Some(folder) = std::env::var_os(DUMP) else {
        return;
    };
    let folder = PathBuf::from(folder);
    std::fs::create_dir_all(&folder).expect("the dump folder");
    let path = folder.join(format!("{name}.tsv"));
    let text: String = found.iter().map(|finding| finding.tsv() + "\n").collect();
    std::fs::write(&path, text).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
}

/// What the gate's load came to.
struct GateRun {
    found: Vec<Found>,
    rows: Vec<(PackIdVersion, PackRow)>,
    invalid: Vec<String>,
    /// The packs of the install on neither stack (hidden, or shadowed by a pack of one
    /// identity): their files are not read.
    off_stack: Vec<String>,
    packs: (usize, usize),
    wall: (Duration, Duration, Duration),
}

/// The install at `root`, composed and loaded as the gate loads it.
fn run_gate(root: &Path) -> GateRun {
    let mut dirs = folders_below(&root.join("behavior_packs"));
    dirs.extend(folders_below(&root.join("resource_packs")));
    let (rows, open_findings, mut invalid) = open_each(&dirs);
    let folders: HashMap<PackIdVersion, String> = rows
        .iter()
        .map(|(id, row)| (id.clone(), row.folder.clone()))
        .collect();

    let started = Instant::now();
    let options = gate_options(root);
    let mut driver = gate_driver();
    let repository = driver
        .repository([
            DirSource::packs(root.join("behavior_packs"), PackOrigin::Package),
            DirSource::packs(root.join("resource_packs"), PackOrigin::Package),
        ])
        .expect("the repository");
    invalid.extend(
        repository
            .invalid()
            .iter()
            .map(|pack| format!("repository: {pack:?}")),
    );
    let world = base_game_world(&repository, &options);
    let set: StackSet = driver
        .load_stacks(
            repository.stacks(&world, options),
            StackOptions {
                behavior: Residency::Transient,
                resource: Residency::Transient,
                ..StackOptions::default()
            },
        )
        .expect("the stacks load");
    let composed = started.elapsed();
    let report = set.load_all();
    let loaded = started.elapsed();
    let link_started = Instant::now();
    let linked = set.link();
    let link_wall = link_started.elapsed();

    let mut found: Vec<Found> = open_findings
        .iter()
        .map(|(folder, finding)| {
            let mut found = Found::of(Pass::Open, finding, &folders);
            found.folder.clone_from(folder);
            found
        })
        .collect();
    found.extend(
        report
            .iter()
            .map(|finding| Found::of(Pass::Load, finding, &folders)),
    );
    found.extend(
        linked
            .iter()
            .map(|finding| Found::of(Pass::Link, finding, &folders)),
    );
    let on_stack: Vec<PackIdVersion> = set
        .behavior
        .packs()
        .chain(set.resource.packs())
        .map(|pack| pack.id().clone())
        .collect();
    let off_stack = rows
        .iter()
        .filter(|(id, _)| !on_stack.contains(id))
        .map(|(_, row)| row.folder.clone())
        .collect();
    GateRun {
        found,
        rows,
        invalid,
        off_stack,
        packs: (set.behavior.packs().count(), set.resource.packs().count()),
        wall: (composed, loaded, link_wall),
    }
}

/// Counts of `found` by pass and severity, as one summary line.
fn summary(found: &[Found]) -> String {
    let mut counts: BTreeMap<(Pass, char), usize> = BTreeMap::new();
    for finding in found {
        *counts.entry((finding.pass, finding.letter())).or_insert(0) += 1;
    }
    let mut line = String::new();
    for pass in [Pass::Open, Pass::Load, Pass::Link] {
        let get = |letter| counts.get(&(pass, letter)).copied().unwrap_or(0);
        let _ = write!(
            line,
            "{}: {} errors, {} warnings, {} informations; ",
            pass.word(),
            get('E'),
            get('W'),
            get('I')
        );
    }
    line
}

/// Findings above information, grouped by pass, severity, code and folder, with a count.
fn groups(found: &[Found]) -> String {
    let mut by: BTreeMap<(Pass, char, &str, &str), (usize, &str)> = BTreeMap::new();
    for finding in found
        .iter()
        .filter(|finding| finding.severity > Severity::Info)
    {
        let entry = by
            .entry((
                finding.pass,
                finding.letter(),
                finding.code,
                finding.folder.as_str(),
            ))
            .or_insert((0, finding.path.as_str()));
        entry.0 += 1;
    }
    let mut out = String::new();
    for ((pass, letter, code, folder), (count, example)) in by {
        let _ = writeln!(
            out,
            "{count:6} {} {letter} {code} @ {folder} (e.g. {example})",
            pass.word()
        );
    }
    out
}

// ---- expected findings ----------------------------------------------------------------------

/// A finding the game logs to the content log for its own file, pinned as expected: the gate
/// does not fail on it, and fails when it does not occur exactly `count` times.
#[derive(Clone, Copy, Debug)]
struct Expected {
    /// The pack folder, as the table names it (`behavior_packs/vanilla`).
    pack: &'static str,
    /// When the game logs it: opening the pack, loading the documents or linking them.
    pass: Pass,
    /// The file's path in the pack, whole.
    path: &'static str,
    /// The `DiagnosticCode` name.
    code: &'static str,
    /// The severity the game logs it at, which the crate mirrors.
    severity: Severity,
    /// The code's text, whole.
    text: &'static str,
    /// How often it occurs.
    count: usize,
}

impl Expected {
    fn matches(&self, finding: &Found) -> bool {
        finding.folder == self.pack
            && finding.pass == self.pass
            && finding.path == self.path
            && finding.code == self.code
            && finding.severity == self.severity
            && finding.message == self.text
    }

    fn line(&self) -> String {
        format!(
            "{} {} {:?} @ {} {} \"{}\" x{}",
            self.pass.word(),
            self.code,
            self.severity,
            self.pack,
            self.path,
            self.text,
            self.count
        )
    }
}

/// The findings the game logs for its own files that the gate expects.
const EXPECTED: &[Expected] = &[
    // The game's loot-table loader logs this text at the content log's error level for
    // vanilla's pool `{}` and keeps the table.
    Expected {
        pack: "behavior_packs/vanilla",
        pass: Pass::Load,
        path: "loot_tables/entities/armor_stand.json",
        code: "MissingRequiredField",
        severity: Severity::Error,
        text: "There must be an \"entries\" array inside the loot table pool.",
        count: 1,
    },
    Expected {
        pack: "behavior_packs/vanilla",
        pass: Pass::Load,
        path: "loot_tables/entities/boat.json",
        code: "MissingRequiredField",
        severity: Severity::Error,
        text: "There must be an \"entries\" array inside the loot table pool.",
        count: 1,
    },
    // The texture-set loader's content-log warning (rule R7) for a `description.textures`
    // value no pack of the install has under any resolution rule.
    r7(
        "resource_packs/experimental_y_2026_drop_4",
        "entity/lingering_potion.entity.json",
        "Failed to find either texture set or image file of name (textures/items/potion_bottle_lingering_digSlowdown)",
    ),
    r7(
        "resource_packs/experimental_y_2026_drop_4",
        "entity/lingering_potion.entity.json",
        "Failed to find either texture set or image file of name (textures/items/potion_bottle_lingering_digSpeed)",
    ),
    r7(
        "resource_packs/experimental_y_2026_drop_4",
        "entity/lingering_potion.entity.json",
        "Failed to find either texture set or image file of name (textures/items/potion_bottle_lingering_confusion)",
    ),
    r7(
        "resource_packs/experimental_y_2026_drop_4",
        "entity/lingering_potion.entity.json",
        "Failed to find either texture set or image file of name (textures/items/potion_bottle_lingering_resistance)",
    ),
    r7(
        "resource_packs/experimental_y_2026_drop_4",
        "entity/lingering_potion.entity.json",
        "Failed to find either texture set or image file of name (textures/items/potion_bottle_lingering_blindness)",
    ),
    r7(
        "resource_packs/experimental_y_2026_drop_4",
        "entity/lingering_potion.entity.json",
        "Failed to find either texture set or image file of name (textures/items/potion_bottle_lingering_hunger)",
    ),
    r7(
        "resource_packs/experimental_y_2026_drop_4",
        "entity/lingering_potion.entity.json",
        "Failed to find either texture set or image file of name (textures/items/potion_bottle_lingering_healthBoost)",
    ),
    r7(
        "resource_packs/experimental_y_2026_drop_4",
        "entity/lingering_potion.entity.json",
        "Failed to find either texture set or image file of name (textures/items/potion_bottle_lingering_absorption)",
    ),
    r7(
        "resource_packs/experimental_y_2026_drop_4",
        "entity/lingering_potion.entity.json",
        "Failed to find either texture set or image file of name (textures/items/potion_bottle_lingering_saturation)",
    ),
    r7(
        "resource_packs/experimental_y_2026_drop_4",
        "entity/lingering_potion.entity.json",
        "Failed to find either texture set or image file of name (textures/items/potion_bottle_lingering_levitation)",
    ),
    r7(
        "resource_packs/vanilla_1.21.0",
        "entity/trial_spawner.entity.json",
        "Failed to find either texture set or image file of name (textures/blocks/trial_spawner_base)",
    ),
    r7(
        "resource_packs/vanilla_1.21.0",
        "attachables/mace.json",
        "Failed to find either texture set or image file of name (textures/item/mace)",
    ),
];

/// The texture-set loader's warning `text` at `path` of `pack`: the game logs it at the content
/// log's warning level when the client reads an entity or attachable definition that names a
/// texture no pack has.
const fn r7(pack: &'static str, path: &'static str, text: &'static str) -> Expected {
    Expected {
        pack,
        pass: Pass::Link,
        path,
        code: "TextureNotFound",
        severity: Severity::Warning,
        text,
        count: 1,
    }
}

/// The findings above information, sorted against `expected`.
#[derive(Debug, Default)]
struct Sorted<'a> {
    /// Findings no entry expects: each fails the gate.
    unexpected: Vec<&'a Found>,
    /// Findings an entry expects.
    expected: Vec<&'a Found>,
    /// Entries that do not occur exactly as pinned: each fails the gate.
    broken: Vec<String>,
}

/// Sorts `found` (above information) against `expected`.
fn sort_expected<'a>(found: &[&'a Found], expected: &[Expected]) -> Sorted<'a> {
    let mut sorted = Sorted::default();
    let mut counts = vec![0_usize; expected.len()];
    for finding in found {
        match expected.iter().position(|entry| entry.matches(finding)) {
            Some(index) => {
                counts[index] += 1;
                sorted.expected.push(finding);
            }
            None => sorted.unexpected.push(finding),
        }
    }
    for (entry, count) in expected.iter().zip(counts) {
        if count != entry.count {
            sorted.broken.push(format!(
                "pinned x{}, found x{count}: {}",
                entry.count,
                entry.line()
            ));
        }
    }
    sorted
}

#[test]
fn every_vanilla_pack_loads_and_links_with_no_error_and_no_warning() {
    let Some(root) = install_root() else {
        return;
    };
    let at = root.clone();
    let mut run = support::on_a_thread(RECOMMENDED_JOB_STACK, move || run_gate(&at));
    let unattributed = count_into(&mut run.rows, &run.found);
    dump("vanilla", &run.found);

    let above_info: Vec<&Found> = run
        .found
        .iter()
        .filter(|finding| finding.severity > Severity::Info)
        .collect();
    let sorted = sort_expected(&above_info, EXPECTED);
    for finding in &sorted.unexpected {
        eprintln!("{}", finding.line());
    }
    eprintln!("\nexpected (pinned findings the game logs itself):");
    for finding in &sorted.expected {
        eprintln!("expected {}", finding.line());
    }
    eprintln!("\n{}", table(&run.rows, unattributed));
    eprintln!("{}", groups(&run.found));
    eprintln!(
        "{}: {} + {} packs on the stacks, composed in {:.1?}, loaded in {:.1?} (from the start), \
         linked in {:.1?}",
        root.display(),
        run.packs.0,
        run.packs.1,
        run.wall.0,
        run.wall.1,
        run.wall.2
    );
    eprintln!(
        "packs of the install on no stack: {}",
        run.off_stack.join(", ")
    );
    eprintln!("summary: {}", summary(&run.found));
    assert!(
        run.invalid.is_empty(),
        "packs of the install that do not open:\n{}",
        run.invalid.join("\n")
    );
    assert!(
        sorted.broken.is_empty(),
        "vanilla gate: expected findings that do not occur as pinned:\n{}",
        sorted.broken.join("\n")
    );
    let errors = sorted
        .unexpected
        .iter()
        .filter(|finding| finding.severity >= Severity::Error)
        .count();
    assert!(
        sorted.unexpected.is_empty(),
        "vanilla gate: the game's own packs must load and link with no error and no warning but \
         the expected ones; found {errors} errors and {} warnings (listed above), {} expected",
        sorted.unexpected.len() - errors,
        sorted.expected.len()
    );
}

/// Every folder below `dir` that holds a `manifest.json`, not looking inside a pack.
fn pack_folders(dir: &Path, out: &mut Vec<PathBuf>) {
    if dir.join("manifest.json").is_file() {
        out.push(dir.to_path_buf());
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut below: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_dir())
        .collect();
    below.sort();
    for path in below {
        pack_folders(&path, out);
    }
}

#[test]
fn the_install_s_test_packs_are_read_and_reported() {
    let Some(root) = install_root() else {
        return;
    };
    let test = root.join("test");
    if !test.is_dir() {
        eprintln!("skipped: {} has no test/ folder", root.display());
        return;
    }
    let at = root.clone();
    let (rows, found, invalid) = support::on_a_thread(RECOMMENDED_JOB_STACK, move || {
        let mut dirs = Vec::new();
        pack_folders(&at.join("test"), &mut dirs);
        // The install's `definitions/` are left out: each test pack is alone on its stack,
        // and the app data's own references (textures and features of the vanilla packs)
        // would be told once per test pack.
        let mut options = gate_options(&at);
        options.app_data = None;
        let mut driver = gate_driver();
        let mut rows = Vec::new();
        let mut found = Vec::new();
        let mut invalid = Vec::new();
        for dir in &dirs {
            let folder = dir
                .strip_prefix(at.join("test"))
                .map_or_else(|_| name_of(dir), |path| path.display().to_string());
            let started = Instant::now();
            let pack = match driver.pack_dir(dir) {
                Ok(pack) => pack,
                Err(error) => {
                    invalid.push(format!("{folder}: {error:?}"));
                    continue;
                }
            };
            let folders = HashMap::from([(pack.id().clone(), folder.clone())]);
            for finding in pack.report().diagnostics.iter() {
                found.push(Found::of(Pass::Open, finding, &folders));
                found.last_mut().expect("pushed").folder.clone_from(&folder);
            }
            let id = pack.id().clone();
            let (kind, files) = (pack.pack_type(), pack.paths().count());
            let (behavior, resource) = if kind == PackType::Resource {
                (Vec::new(), vec![pack])
            } else {
                (vec![pack], Vec::new())
            };
            let set = StackSet {
                behavior: PackStack::of(behavior, options.clone()).expect("a stack"),
                resource: PackStack::of(resource, options.clone()).expect("a stack"),
            };
            let set = match driver.load_stacks(set, StackOptions::default()) {
                Ok(set) => set,
                Err(error) => {
                    invalid.push(format!("{folder}: the stack does not load: {error:?}"));
                    continue;
                }
            };
            let report = set.load_all();
            found.extend(
                report
                    .iter()
                    .map(|finding| Found::of(Pass::Load, finding, &folders)),
            );
            let linked = set.link();
            found.extend(
                linked
                    .iter()
                    .map(|finding| Found::of(Pass::Link, finding, &folders)),
            );
            rows.push((
                id,
                PackRow {
                    folder,
                    kind,
                    files,
                    open: started.elapsed(),
                    counts: [0; 3],
                },
            ));
        }
        (rows, found, invalid)
    });
    let mut rows = rows;
    let unattributed = count_into(&mut rows, &found);
    dump("vanilla_test_packs", &found);
    for finding in found
        .iter()
        .filter(|finding| finding.severity > Severity::Info)
    {
        eprintln!("{}", finding.line());
    }
    eprintln!("\n{}", table(&rows, unattributed));
    eprintln!("{}", groups(&found));
    eprintln!("packs that do not open or load:\n{}", invalid.join("\n"));
    eprintln!(
        "test/: {} packs read, {} not; summary: {}",
        rows.len(),
        invalid.len(),
        summary(&found)
    );
}

/// A finding for the tests of [`sort_expected`].
fn found_for_test(pack: &str, path: &str, code: &'static str, message: &str) -> Found {
    Found {
        pass: Pass::Load,
        severity: Severity::Warning,
        code,
        folder: pack.to_owned(),
        path: path.to_owned(),
        json_path: String::new(),
        text: message.to_owned(),
        message: message.to_owned(),
    }
}

/// A fabricated entry, pinned once.
const FABRICATED: Expected = Expected {
    pack: "behavior_packs/vanilla",
    pass: Pass::Load,
    path: "loot_tables/entities/boat.json",
    code: "MissingRequiredField",
    severity: Severity::Warning,
    text: "There must be an \"entries\" array inside the loot table pool.",
    count: 1,
};

#[test]
fn an_expected_entry_that_does_not_occur_fails_the_gate() {
    let sorted = sort_expected(&[], &[FABRICATED]);
    assert_eq!(
        sorted.broken.len(),
        1,
        "a vanished pin must fail: {sorted:?}"
    );
}

#[test]
fn an_expected_entry_that_occurs_as_pinned_is_set_apart_and_the_rest_fails() {
    let pinned = found_for_test(
        "behavior_packs/vanilla",
        "loot_tables/entities/boat.json",
        "MissingRequiredField",
        "There must be an \"entries\" array inside the loot table pool.",
    );
    let other = found_for_test(
        "behavior_packs/vanilla_1.20.0",
        "loot_tables/entities/boat.json",
        "MissingRequiredField",
        "There must be an \"entries\" array inside the loot table pool.",
    );
    let sorted = sort_expected(&[&pinned, &other], &[FABRICATED]);
    assert!(sorted.broken.is_empty(), "{sorted:?}");
    assert_eq!(sorted.expected.len(), 1);
    assert_eq!(sorted.unexpected.len(), 1);
    assert_eq!(sorted.unexpected[0].folder, "behavior_packs/vanilla_1.20.0");
}

#[test]
fn an_expected_entry_that_occurs_more_often_than_pinned_fails_the_gate() {
    let pinned = found_for_test(
        "behavior_packs/vanilla",
        "loot_tables/entities/boat.json",
        "MissingRequiredField",
        "There must be an \"entries\" array inside the loot table pool.",
    );
    let sorted = sort_expected(&[&pinned, &pinned], &[FABRICATED]);
    assert_eq!(sorted.broken.len(), 1, "{sorted:?}");
}

#[test]
fn a_pin_matches_the_whole_path_the_whole_text_and_its_pass_only() {
    let finding = found_for_test(
        "behavior_packs/vanilla",
        "loot_tables/entities/boat.json",
        "MissingRequiredField",
        "There must be an \"entries\" array inside the loot table pool.",
    );
    assert!(FABRICATED.matches(&finding), "the pin as written matches");
    let prefix = Expected {
        text: "There must be an",
        ..FABRICATED
    };
    assert!(
        !prefix.matches(&finding),
        "a text that is a prefix of the finding's matches"
    );
    let suffix = Expected {
        path: "entities/boat.json",
        ..FABRICATED
    };
    assert!(
        !suffix.matches(&finding),
        "a path that is a suffix of the finding's matches"
    );
    let other_pass = Expected {
        pass: Pass::Link,
        ..FABRICATED
    };
    assert!(
        !other_pass.matches(&finding),
        "a pin of another pass matches"
    );
}
