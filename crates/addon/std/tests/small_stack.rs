#![allow(clippy::pedantic, missing_docs)]
//! A world's two stacks load on a thread with 1 MiB of stack, in whatever profile the test is
//! built with.
//!
//! Guards the stack use of a load. A walk of a version chain that kept every step's value of
//! every starting struct in one frame would take 1.5 MiB for the item chain in an unoptimised
//! build with full debug info, and a `StackSet` that carried its caches inline would be 40 KB a
//! copy; either would overflow the 2 MiB stack of a test thread during the sequential load of
//! a base-game install. The item below is in the legacy format,
//! the start of the longest chain of the crate; the stack test is on purpose its own binary: an
//! overflow aborts the process.
//!
//! Needs no game files.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::fs;
use std::path::Path;

use bedrock_addon::diagnostic::DiagnosticCode;
use bedrock_addon::families::script::{ScriptBundle, ScriptPath};
use bedrock_addon::load::{LoadOptions, StackOptions};
use bedrock_addon::pack::{PackOrigin, StackSet, WorldPacks};
use bedrock_addon_std::{DirSource, Driver, Options, Strategy};

/// The stack of the loading thread: half the default of a spawned thread.
const SMALL_STACK: usize = 1024 * 1024;

const BEHAVIOR: &str = "5e1f0a3c-3a61-4b1e-9e0e-3c1f5b0a6d21";
const RESOURCE: &str = "6f2a1b4d-4b72-4c2f-8f1f-4d2a6c1b7e32";

/// A manifest with one module of `module_type`.
fn manifest(name: &str, uuid: &str, module_type: &str, module: &str) -> String {
    format!(
        r#"{{
    "format_version": 2,
    "header": {{
        "name": "{name}",
        "description": "",
        "uuid": "{uuid}",
        "version": [1, 0, 0],
        "min_engine_version": [1, 26, 0]
    }},
    "modules": [{{ "type": "{module_type}", "uuid": "{module}", "version": [1, 0, 0] }}]
}}"#
    )
}

fn write(dir: &Path, path: &str, bytes: &[u8]) {
    let path = dir.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

/// A behaviour pack with one item in the legacy format and one in the 1.20.0 format (the
/// oldest component format a pack outside the base game may use), and a resource pack with
/// one material file.
fn small_world(dir: &Path) {
    write(
        dir,
        "ruby_bp/manifest.json",
        manifest(
            "ruby bp",
            BEHAVIOR,
            "data",
            "9a0d3c1e-3f7b-4c55-8a39-5b8e6c1d2f40",
        )
        .as_bytes(),
    );
    write(
        dir,
        "ruby_bp/items/old_ruby.json",
        br#"{"format_version": "1.10", "minecraft:item": {"description": {"identifier": "demo:old_ruby"}, "components": {"minecraft:max_stack_size": 8, "minecraft:hand_equipped": true}}}"#,
    );
    write(
        dir,
        "ruby_bp/items/ruby.json",
        br#"{"format_version": "1.20.0", "minecraft:item": {"description": {"identifier": "demo:ruby"}, "components": {"minecraft:max_stack_size": 16}}}"#,
    );
    write(
        dir,
        "ruby_rp/manifest.json",
        manifest(
            "ruby rp",
            RESOURCE,
            "resources",
            "0b1e4d2f-4a8c-4d66-9b4a-6c9f7d2e3a51",
        )
        .as_bytes(),
    );
    write(
        dir,
        "ruby_rp/materials/ruby.material",
        br#"{"materials": {"version": "1.0.0", "ruby": {"states": ["Blending"], "vertexShader": "shaders/uv.vertex", "fragmentShader": "shaders/texture.fragment"}}}"#,
    );
}

/// Both stacks of the world in `dir`, every collection loaded on the calling thread.
fn load(dir: &Path) -> StackSet {
    let mut driver = Driver::new(Options::default().with_strategy(Strategy::Sequential)).unwrap();
    let repository = driver
        .repository([DirSource::packs(dir.to_path_buf(), PackOrigin::World)])
        .unwrap();
    let world = WorldPacks::parse(
        Some(format!(r#"[{{"pack_id": "{BEHAVIOR}", "version": [1, 0, 0]}}]"#).as_bytes()),
        Some(format!(r#"[{{"pack_id": "{RESOURCE}", "version": [1, 0, 0]}}]"#).as_bytes()),
    )
    .unwrap();
    driver
        .load_stacks(
            repository.stacks(&world, LoadOptions::default()),
            StackOptions::default(),
        )
        .unwrap()
}

#[test]
fn a_world_loads_on_a_thread_with_one_mebibyte_of_stack() {
    let dir = tempfile::tempdir().unwrap();
    small_world(dir.path());
    let root = dir.path().to_path_buf();
    let (items, material_files) = std::thread::Builder::new()
        .stack_size(SMALL_STACK)
        .spawn(move || {
            let set = load(&root);
            (
                set.behavior.items().len(),
                set.resource.materials().files.len(),
            )
        })
        .unwrap()
        .join()
        .expect("the load does not panic");
    assert_eq!(items, 2, "both items load, each through the item chain");
    assert_eq!(material_files, 1);
}

/// A behaviour pack whose script module starts `scripts/main.js`, with `main` as its text.
fn script_world(dir: &Path, main: &str) {
    write(
        dir,
        "script_bp/manifest.json",
        format!(
            r#"{{
    "format_version": 2,
    "header": {{ "name": "deep bp", "description": "", "uuid": "{BEHAVIOR}", "version": [1, 0, 0],
        "min_engine_version": [1, 21, 0] }},
    "modules": [{{ "type": "script", "language": "javascript", "entry": "scripts/main.js",
        "uuid": "9a0d3c1e-3f7b-4c55-8a39-5b8e6c1d2f40", "version": [1, 0, 0] }}],
    "dependencies": [{{ "module_name": "@minecraft/server", "version": "2.0.0" }}]
}}"#
        )
        .as_bytes(),
    );
    write(dir, "script_bp/scripts/main.js", main.as_bytes());
}

/// What the load of the script world came to.
struct Loaded {
    /// The one file of the bundle was parsed.
    parsed: bool,
    /// `ScriptSyntaxNotChecked` files of the bundle, when it was raised.
    not_checked: Option<u32>,
}

/// Loads the world of [`script_world`] with `main` through the real pipeline, on a thread
/// with one mebibyte of stack: an overflow aborts the process.
fn load_script_world(main: String) -> Loaded {
    let dir = tempfile::tempdir().unwrap();
    script_world(dir.path(), &main);
    let root = dir.path().to_path_buf();
    std::thread::Builder::new()
        .stack_size(SMALL_STACK)
        .spawn(move || {
            let mut driver =
                Driver::new(Options::default().with_strategy(Strategy::Sequential)).unwrap();
            let repository = driver
                .repository([DirSource::packs(root, PackOrigin::World)])
                .unwrap();
            let world = WorldPacks::parse(
                Some(format!(r#"[{{"pack_id": "{BEHAVIOR}", "version": [1, 0, 0]}}]"#).as_bytes()),
                None,
            )
            .unwrap();
            let set = driver
                .load_stacks(
                    repository.stacks(&world, LoadOptions::default()),
                    StackOptions::default(),
                )
                .unwrap();
            let bundles = set.behavior.scripts();
            assert_eq!(bundles.len(), 1, "one pack, one bundle");
            let bundle = &bundles[0];
            let file = &bundle.files[&ScriptPath::new("main.js")];
            Loaded {
                parsed: file.content.as_ref().unwrap().parsed,
                not_checked: bundle.findings().find_map(|finding| match finding.code {
                    DiagnosticCode::ScriptSyntaxNotChecked { files } => Some(files),
                    _ => None,
                }),
            }
        })
        .unwrap()
        .join()
        .expect("the load does not abort or panic")
}

/// The scripts that nest `levels` deep, in the three families the parser spends most stack on
/// per unit of the bound's weight: a mix
/// of brackets, a template, a ternary, an arrow and an object method; `new (await …)` in an
/// async function; and a class heritage chain.
fn shapes(levels: usize) -> [(&'static str, String); 3] {
    [
        (
            "mixed",
            format!(
                "x = {}1{};",
                "[(a) => { if (b) return c ? d : `${({ m() { return ".repeat(levels),
                "} })}` }]".repeat(levels)
            ),
        ),
        (
            "new await",
            format!(
                "async function f() {{ x = {}a{}; }}",
                "(new (await ".repeat(levels),
                "))".repeat(levels)
            ),
        ),
        (
            "class heritage",
            format!(
                "{}B{}",
                "class A extends ".repeat(levels),
                " {}".repeat(levels)
            ),
        ),
    ]
}

/// The deepest nesting of each of [`shapes`] the bound admits, found by
/// searching with the load itself. A change of the weights moves them: the tests below say
/// when, and the new depths are the ones to put here.
const DEEPEST: [usize; 3] = [6, 19, 62];

#[test]
fn a_script_as_deep_as_the_bound_admits_loads_on_a_thread_with_one_mebibyte_of_stack() {
    for (k, depth) in DEEPEST.into_iter().enumerate() {
        let (name, source) = shapes(depth).into_iter().nth(k).unwrap();
        let loaded = load_script_world(source);
        // Without `script-syntax` nothing is parsed; with it the file is, so the load went
        // through the parser at the deepest nesting it is given.
        assert_eq!(
            loaded.parsed,
            ScriptBundle::SYNTAX_CHECKED,
            "{name} at {depth} levels"
        );
        assert_eq!(loaded.not_checked, None, "{name} at {depth} levels");
    }
}

#[test]
fn a_script_one_level_past_the_bound_loads_unparsed_and_says_so() {
    for (k, depth) in DEEPEST.into_iter().enumerate() {
        let (name, source) = shapes(depth + 1).into_iter().nth(k).unwrap();
        let loaded = load_script_world(source);
        assert!(!loaded.parsed, "{name} at {} levels", depth + 1);
        assert_eq!(
            loaded.not_checked,
            ScriptBundle::SYNTAX_CHECKED.then_some(1),
            "{name} at {} levels",
            depth + 1
        );
    }
}
