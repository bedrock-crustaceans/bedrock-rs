//! The determinism rule for the link pass (: `link()` is a pure call
//! over the loaded stacks): the fixture stack of `determinism.rs`, with the cross-references of
//! this test's own fragment, is loaded with completions in issue order, in reverse and in three
//! seeded random orders, and after each `StackSet::link()` returns the same findings in the same
//! order, with and without its lints.
//!
//! The stack is the two packs of `determinism.rs` with every pack fragment a family left under
//! `tests/fixtures/<module>/<family>/determinism/`, plus `tests/fixtures/pack/determinism_link/`:
//! per rule group one reference that resolves and one that nothing defines, so that the output
//! compared is not empty and holds findings of every group that tells at link time. The
//! registries a rule compares with (the terrain and the item atlas, the material list) are the
//! families' own fragments; this fragment names keys they define and keys they lack.
//!
//! The script group raises nothing in `link()` (its rules are rules of the load, 20 S1 to S7):
//! the behaviour pack's manifest asks for one module the game has and one it lacks, and the
//! findings of `scripts()` are compared across the orders beside the pass.
//!
//! This is a file of its own, and not a test of `determinism.rs`, so that the helpers are
//! repeated here: the two files were worked on at the same time.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![allow(
    dead_code,
    unused_imports,
    reason = "the test of the fragment's references needs every content feature"
)]

#[path = "common/responder.rs"]
mod responder;

use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{Diagnostic, DiagnosticDetail, Diagnostics, Severity};
use bedrock_addon::load::{
    LoadOptions, RepositoryLoader, Residency, RootId, Source, StackLoader, StackOptions,
};
use bedrock_addon::pack::{LinkOptions, OpenOptions, PackOrigin, StackSet, WorldPacks};
use responder::{Order, Responder};

/// Every file below `dir`, as `(path relative to dir, bytes)`, sorted.
fn files_below(dir: &Path) -> Vec<(String, Vec<u8>)> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(root, &path, out);
            } else {
                let relative = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                out.push((relative, std::fs::read(&path).unwrap()));
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, dir, &mut out);
    out.sort();
    out
}

/// The `determinism/` fragments of every family, sorted by path, then this test's own.
fn fragments() -> Vec<PathBuf> {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut found = Vec::new();
    for module in std::fs::read_dir(&fixtures).unwrap() {
        let module = module.unwrap().path();
        if !module.is_dir() {
            continue;
        }
        for family in std::fs::read_dir(&module).unwrap() {
            let fragment = family.unwrap().path().join("determinism");
            if fragment.is_dir() {
                found.push(fragment);
            }
        }
    }
    found.sort();
    found.push(fixtures.join("pack/determinism_link"));
    found
}

/// The manifest of pack `n`: a script module that starts `scripts/main.js` (the script bundle
/// fragment's entry file), then the module that makes the pack a behaviour (`data`) or a
/// resource (`resources`) pack, and `dependencies`.
fn pack_manifest(n: u128, script_module: &str, module: &str, dependencies: &str) -> String {
    format!(
        r#"{{
  "format_version": 2,
  "header": {{ "name": "pack {n}", "description": "", "uuid": "{}",
              "version": [1, 0, 0], "min_engine_version": [1, 21, 0] }},
  "modules": [
    {{ "type": "{script_module}", "language": "javascript", "entry": "scripts/main.js",
      "uuid": "{}", "version": [1, 0, 0] }},
    {{ "type": "{module}", "uuid": "{}", "version": [1, 0, 0] }}
  ],
  "dependencies": [{dependencies}]
}}"#,
        uuid::Uuid::from_u128(n),
        uuid::Uuid::from_u128(n | 0xeeee_0000_0000_0000_0000),
        uuid::Uuid::from_u128(n | 0xffff_0000_0000_0000_0000),
    )
}

/// The module of the game the entry file imports.
const SERVER: &str = r#"{ "module_name": "@minecraft/server", "version": "2.0.0" }"#;
/// A module the game does not have: the script group's unresolved reference.
const NO_MODULE: &str = r#"{ "module_name": "@minecraft/linked-no-module", "version": "1.0.0" }"#;

/// The packs directory at root 1: `bp/` and `rp/` with every fragment. A fragment's files go
/// to both packs, except those under `behavior_pack/` or `resource_pack/`, which go to that
/// pack only; its `app_data/` belongs to no pack and is left out here.
fn fixture_stack() -> Responder {
    let mut responder = Responder::new(Order::Issue);
    responder
        .file(
            1,
            "bp/manifest.json",
            pack_manifest(1, "script", "data", &format!("{SERVER}, {NO_MODULE}")),
        )
        .file(1, "bp/texts/en_US.lang", &b"pack.name=Determinism\n"[..])
        .file(
            1,
            "rp/manifest.json",
            pack_manifest(2, "client_script", "resources", SERVER),
        )
        .file(1, "rp/texts/en_US.lang", &b"pack.name=Determinism\n"[..]);
    for fragment in fragments() {
        for (path, bytes) in files_below(&fragment) {
            if path.starts_with("app_data/") {
                continue;
            }
            if let Some(rest) = path.strip_prefix("behavior_pack/") {
                responder.file(1, &format!("bp/{rest}"), bytes);
            } else if let Some(rest) = path.strip_prefix("resource_pack/") {
                responder.file(1, &format!("rp/{rest}"), bytes);
            } else {
                responder.file(1, &format!("bp/{path}"), bytes.clone());
                responder.file(1, &format!("rp/{path}"), bytes);
            }
        }
    }
    responder
}

/// The world list naming both packs.
fn world() -> WorldPacks {
    let entry = |n: u128| {
        format!(
            r#"[{{"pack_id": "{}", "version": [1, 0, 0]}}]"#,
            uuid::Uuid::from_u128(n)
        )
    };
    WorldPacks::parse(Some(entry(1).as_bytes()), Some(entry(2).as_bytes())).unwrap()
}

/// The stacks of the fixture, the repository and every collection loaded with completions in
/// `order`.
fn loaded(template: &Responder, order: Order, residency: Residency) -> StackSet {
    let mut responder = template.reordered(order);
    let mut repository = RepositoryLoader::new(
        vec![Source::Directory {
            root: RootId(1),
            origin: PackOrigin::World,
        }],
        OpenOptions::default(),
    );
    let repository = responder.drive(&mut repository);
    let set = repository.stacks(&world(), LoadOptions::default());
    let mut loader = StackLoader::new(
        set,
        StackOptions {
            behavior: residency,
            resource: residency,
            ..StackOptions::default()
        },
    );
    responder.drive(&mut loader)
}

/// What the link pass and the script load of one world came to, as text.
#[derive(Debug, PartialEq)]
struct Linked {
    /// `{:#?}` of `link()`.
    link: String,
    /// `{:#?}` of `link_with(&LinkOptions { lints: false, ..Default::default() })`.
    without_lints: String,
    /// `{:?}` of every finding of `scripts()` of both stacks, in order.
    scripts: Vec<String>,
}

fn linked(set: &StackSet) -> (Linked, Diagnostics) {
    let found = set.link();
    #[cfg(feature = "script")]
    let scripts = set
        .behavior
        .scripts()
        .iter()
        .chain(set.resource.scripts())
        .flat_map(|bundle| bundle.findings())
        .map(|finding| format!("{finding:?}"))
        .collect();
    #[cfg(not(feature = "script"))]
    let scripts = Vec::new();
    (
        Linked {
            link: format!("{found:#?}"),
            without_lints: format!(
                "{:#?}",
                set.link_with(&LinkOptions {
                    lints: false,
                    ..Default::default()
                })
            ),
            scripts,
        },
        found,
    )
}

/// The five completion orders, as `determinism.rs` pins them.
const ORDERS: [Order; 5] = [
    Order::Issue,
    Order::Reverse,
    Order::Seeded(1),
    Order::Seeded(0x5eed),
    Order::Seeded(0xdead_beef),
];

/// The name a reference finding is about, with its severity.
fn reference(finding: &Diagnostic) -> Option<(&str, Severity)> {
    match finding.detail.as_deref() {
        Some(DiagnosticDetail::Reference { id, .. }) => Some((id.as_str(), finding.severity)),
        _ => None,
    }
}

#[test]
fn every_completion_order_gives_the_same_link_findings() {
    assert_eq!(Order::ALL, ORDERS);
    let template = fixture_stack();
    for residency in [Residency::Transient, Residency::All] {
        let (reference, _) = linked(&loaded(&template, Order::Issue, residency));
        // With every content feature the fragment's references are told (the test below
        // says which); with fewer features fewer rules run, down to none.
        assert!(
            !cfg!(feature = "full") || reference.link.len() > 1_000,
            "the pass finds something to compare: {}",
            reference.link
        );
        for order in ORDERS {
            let set = loaded(&template, order, residency);
            let (found, _) = linked(&set);
            assert_eq!(found, reference, "{order:?}, {residency:?}");
            // And the pass again over the same stacks.
            assert_eq!(linked(&set).0, reference, "{order:?}, {residency:?}, twice");
        }
    }
}

#[cfg(feature = "full")]
#[test]
fn the_fixture_stack_holds_a_resolved_and_an_unresolved_reference_of_every_rule_group() {
    let set = loaded(&fixture_stack(), Order::Issue, Residency::Transient);
    let (text, found) = linked(&set);
    let told: Vec<(&str, Severity)> = found.iter().filter_map(reference).collect();
    let times = |name: &str| told.iter().filter(|(told, _)| *told == name).count();
    // Per group: a name nothing defines is told once, and the name beside it that the stack
    // defines is not told.
    for (group, missing, defined) in [
        ("entity (E15)", "linked:no_treat", "linked:wand"),
        ("entity (E14)", "linked:no_grown", "linked:pet"),
        ("content (I3)", "linked:no_placed", "linked:pet"),
        ("content (L1)", "linked:no_drop", "linked:wand"),
        ("content (B9)", "linked_no_tile", "demo_stone"),
        ("world (W1)", "linked:no_feature", "linked:rock"),
        ("visual (R1)", "linked_no_material", "determinism_base"),
        ("visual (R2)", "linked:no_puff", "linked:spark"),
        ("visual (R10)", "linked:no_entity", "linked:pet"),
        ("resource (R32)", "linked_no_icon", "demo_gem"),
        (
            "resource (R38)",
            "linked_no_particle_material",
            "determinism_base",
        ),
    ] {
        assert_eq!(
            times(missing),
            1,
            "{group}: `{missing}` is told once; told: {told:#?}"
        );
        assert_eq!(times(defined), 0, "{group}: `{defined}` resolves");
    }
    // The loot table both the block and the entity name is the fragment's own.
    assert_eq!(times("loot_tables/linked_drop.json"), 0);
    // The script group: the module the game lacks is told by the load, once, and the pass
    // tells nothing of a manifest or a script file.
    let unresolved: Vec<&String> = text
        .scripts
        .iter()
        .filter(|finding| finding.contains("linked-no-module"))
        .collect();
    assert_eq!(unresolved.len(), 1, "script: {:#?}", text.scripts);
    assert!(
        !text
            .scripts
            .iter()
            .any(|finding| finding.contains("\"@minecraft/server\"")
                && finding.contains("Unresolved")),
        "script: `@minecraft/server` resolves: {:#?}",
        text.scripts
    );
    let about_scripts: Vec<&Diagnostic> = found
        .iter()
        .filter(|finding| {
            finding.location.path.as_ref().is_some_and(|path| {
                path.as_str() == "manifest.json" || path.as_str().ends_with(".js")
            })
        })
        .collect();
    assert!(about_scripts.is_empty(), "{about_scripts:#?}");
    // Without the lints the pass returns what is above the information level, in order.
    let above: Vec<&Diagnostic> = found
        .iter()
        .filter(|finding| finding.severity > Severity::Info)
        .collect();
    let without = set.link_with(&LinkOptions {
        lints: false,
        ..Default::default()
    });
    assert_eq!(
        format!("{:?}", without.iter().collect::<Vec<_>>()),
        format!("{above:?}")
    );
    assert!(!above.is_empty() && above.len() < found.iter().count());
}
