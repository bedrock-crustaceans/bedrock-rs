//! JSON UI smoke over the game's own files: the `ui/`
//! trees of the sample packs' `resource_pack/` and of the client installs' `vanilla`, `vanilla_base`
//! and `chemistry` resource packs load through the `Driver` into `rp.ui()` with zero
//! error-level diagnostics, every listed file that exists on disk merges, and the findings
//! vanilla's own files are known to raise are pinned. The `textures/ui/*.json` side files are
//! smoked with `rp_misc`.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo nextest run -p bedrock_addon_std --test smoke_ui`;
//! without the variable the tests skip.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::families::json_ui::ControlKind;
use bedrock_addon::json::Value;
use bedrock_addon::load::{LoadOptions, StackOptions};
use bedrock_addon::pack::{OpenOptions, PackOrigin, PackStack, Resource, StackSet};
use bedrock_addon_std::{Driver, Options};

use common::game_files;

/// The `.json` files below `dir`, at any depth.
fn json_files(dir: &Path) -> usize {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .map(|path| {
            if path.is_dir() {
                json_files(&path)
            } else {
                usize::from(path.extension().is_some_and(|e| e == "json"))
            }
        })
        .sum()
}

/// The resource stack of the base-game packs `dirs`, top first, loaded by the driver that
/// opened them. The packs with a `ui/` folder are stacked directly: no two of them list the
/// same file, so their order does not change what merges.
fn stack(dirs: &[PathBuf]) -> StackSet {
    let mut driver = Driver::new(Options::default().with_open(OpenOptions {
        origin: PackOrigin::Package,
        ..OpenOptions::default()
    }))
    .unwrap();
    let packs: Vec<_> = dirs
        .iter()
        .map(|dir| {
            driver
                .pack_dir(dir)
                .unwrap_or_else(|e| panic!("{}: {e:?}", dir.display()))
        })
        .collect();
    let options = LoadOptions::default();
    let set = StackSet {
        behavior: PackStack::of([], options.clone()).unwrap(),
        resource: PackStack::of(packs, options).unwrap(),
    };
    driver.load_stacks(set, StackOptions::default()).unwrap()
}

/// How often `value` holds a member `"font_size": "medium"`, at any depth.
fn count_medium(value: &Value) -> usize {
    if let Some(object) = value.as_object() {
        object
            .iter()
            .map(|(key, member)| {
                let medium = key.as_str() == "font_size"
                    && member
                        .as_string()
                        .is_some_and(|size| size.as_str() == "medium");
                usize::from(medium) + count_medium(member)
            })
            .sum()
    } else if let Some(array) = value.as_array() {
        array.as_slice().iter().map(count_medium).sum()
    } else {
        0
    }
}

/// What one tree gave.
struct Tally {
    files: usize,
    listed_missing: Vec<String>,
    unknown_types: Vec<String>,
    /// How often a file writes `"font_size": "medium"`.
    silent_medium: usize,
    warnings: BTreeMap<&'static str, usize>,
}

/// Loads the UI of `dirs` (top first) and requires no error, every file on disk that a list
/// names merged, and at least `definitions` definitions.
fn check(source: &str, dirs: &[PathBuf], definitions: usize) -> Tally {
    let on_disk: usize = dirs.iter().map(|dir| json_files(&dir.join("ui"))).sum();
    let mcb: usize = dirs
        .iter()
        .map(|dir| game_files::mcb_on_disk(&dir.join("ui")).total())
        .sum();
    let set = stack(dirs);
    let resource: &PackStack<Resource> = &set.resource;
    let ui = resource.ui();
    let mut errors = Vec::new();
    let mut tally = Tally {
        files: ui.files.len(),
        listed_missing: Vec::new(),
        unknown_types: Vec::new(),
        silent_medium: ui
            .layers()
            .iter()
            .map(|layer| count_medium(&layer.json))
            .sum(),
        warnings: BTreeMap::new(),
    };
    let mut texts: BTreeMap<String, usize> = BTreeMap::new();
    for finding in ui.diagnostics() {
        let text = finding.code.to_string();
        // A creation-time text (an unknown control type): an information, because the game
        // raises it only when it creates the control.
        if text.starts_with("Type '") && finding.severity < Severity::Error {
            tally.unknown_types.push(text.clone());
        }
        // The developer log's line about a file that could not be loaded: an information.
        if let Some(path) = text
            .strip_prefix("The following file could not be loaded: ")
            .and_then(|rest| rest.split_once(". Make sure"))
            .filter(|_| finding.severity < Severity::Error)
        {
            tally.listed_missing.push(path.0.to_owned());
        }
        match finding.severity {
            Severity::Error => errors.push(finding.to_string()),
            Severity::Warning => {
                *tally.warnings.entry(finding.code.id()).or_default() += 1;
                // one line per distinct text, without what differs per control
                let kind = text
                    .split(['\'', ':', '"'])
                    .next()
                    .unwrap_or_default()
                    .to_owned();
                *texts
                    .entry(format!("{} | {kind}", finding.code.id()))
                    .or_default() += 1;
                if std::env::var_os("BEDROCK_ADDON_SMOKE_VERBOSE").is_some() {
                    eprintln!("{source}: {finding}");
                }
            }
            _ => {}
        }
    }
    let controls = ui
        .controls()
        .filter(|control| matches!(control.kind(), ControlKind::Control(_)))
        .count();
    let animations = ui
        .controls()
        .filter(|control| matches!(control.kind(), ControlKind::Animation(_)))
        .count();
    eprintln!(
        "{source}: {} packs, {} listed files merged ({on_disk} .json files on disk below ui/, {mcb} MCB), {} namespaces, {} definitions ({controls} controls, {animations} animations), {} global variables, {} layers, {} pack reports, {} errors, warnings {:?}",
        resource.packs().len(),
        tally.files,
        ui.namespaces.len(),
        ui.controls().count(),
        ui.global_variables.variables.len(),
        ui.layers().len(),
        ui.reports.len(),
        errors.len(),
        tally.warnings,
    );
    for (text, count) in &texts {
        eprintln!("{source}:   {count:>5} × {text}");
    }
    for error in &errors {
        eprintln!("{source}: {error}");
    }
    assert!(errors.is_empty(), "{source}: {} errors", errors.len());
    assert_eq!(mcb, 0, "{source}: no UI file is an MCB document");
    assert!(
        ui.reports.is_empty(),
        "{source}: vanilla packs raise no uiPackError"
    );
    // l08: `editor_gizmo_renderer` (hud_screen.json) is accepted.
    assert!(
        !texts
            .keys()
            .any(|text| text.contains("Unrecognized renderer")),
        "{source}: a vanilla renderer is not known"
    );
    assert!(
        ui.controls().count() >= definitions,
        "{source}: {} definitions",
        ui.controls().count()
    );
    // every namespace is named: the `_root` default is the only fallback
    assert!(ui.namespaces.keys().all(|namespace| !namespace.is_empty()));
    tally
}

/// Holds a tree to its warnings by code: vanilla's own slips, counted. A count that moves
/// means a rule changed what it reports on vanilla's files.
fn pinned(tally: &Tally, counts: &[(&'static str, usize)]) {
    let expected: BTreeMap<&'static str, usize> = counts.iter().copied().collect();
    assert_eq!(tally.warnings, expected);
}

/// The packs of `packs` named `names` that exist, top first.
fn named(packs: &Path, names: &[&str]) -> Vec<PathBuf> {
    names
        .iter()
        .map(|name| packs.join(name))
        .filter(|dir| dir.join("ui").is_dir())
        .collect()
}

#[test]
fn sample_packs_ui_loads_without_errors() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    let pack = samples.join("resource_pack");
    let tally = check("sample packs", std::slice::from_ref(&pack), 5_000);
    // l08: the tag lacks two files its list names; both are reported and nothing else is.
    let mut missing = tally.listed_missing.clone();
    missing.sort();
    assert_eq!(
        missing,
        [
            "ui/csb_sections/csb_view_packs_screen.json",
            "ui/persona_sdl.json"
        ]
    );
    // 207 files on disk: the two lists (`_ui_defs.json`, `_global_variables.json`) and 205
    // UI files, of which the list names 202.
    assert_eq!(json_files(&pack.join("ui")), 207);
    // l08: vanilla's one definition of a type the game does not have.
    assert_eq!(
        tally.unknown_types,
        ["Type 'carousel_label' not recognized for control: rotating_text"],
    );
    // `font_size: "medium"` is no size the game knows and raises nothing. The folder counts
    // four; the files the list names hold three, because one listed path differs in case from
    // the file.
    assert_eq!(tally.silent_medium, 3);
    // The reference-cycle findings and the control-creation findings are informations, as are
    // the developer log's line about an unloadable file and the crate's own lints: the game
    // says nothing at load time about any of them.
    pinned(&tally, &[]);
}

#[test]
fn client_install_b_ui_loads_without_errors() {
    let client = skip!(game_files::CLIENT_INSTALL_B);
    let dirs = named(
        &client.join("resource_packs"),
        &["chemistry", "vanilla_base", "vanilla"],
    );
    assert_eq!(dirs.len(), 3, "client install B has three packs with ui/");
    let tally = check("client install B", &dirs, 5_000);
    // the pack ships `persona_sdl.json`; one listed file is absent from it too
    assert_eq!(
        tally.listed_missing,
        ["ui/csb_sections/csb_view_packs_screen.json"]
    );
    assert_eq!(tally.unknown_types.len(), 1, "{:?}", tally.unknown_types);
    // The reference-cycle findings and the control-creation findings are informations, as are
    // the developer log's line about an unloadable file and the crate's own lints: the game
    // says nothing at load time about any of them.
    pinned(&tally, &[]);
}

#[test]
fn client_install_a_ui_loads_without_errors() {
    let client = skip!(game_files::CLIENT_INSTALL_A);
    let dirs = named(
        &client.join("resource_packs"),
        &[
            "furnace_recipe_book",
            "chemistry",
            "vanilla_base",
            "vanilla",
        ],
    );
    if dirs.is_empty() {
        eprintln!("client install A: no resource pack with ui/; skipped");
        return;
    }
    let tally = check("client install A", &dirs, 5_000);
    assert_eq!(tally.unknown_types.len(), 1, "{:?}", tally.unknown_types);
    // The reference-cycle findings and the control-creation findings are informations, as are
    // the developer log's line about an unloadable file and the crate's own lints: the game
    // says nothing at load time about any of them.
    pinned(&tally, &[]);
}
