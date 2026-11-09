//! Vanilla smoke for the script side of manifests: every behaviour pack of the server install
//! and of both client installs that has a script module is discovered and resolved against the compiled-in module catalog with zero error-level
//! findings. The packs are those of the world the other smokes load (`common::world`): the
//! install as a repository of `Package`-origin packs, every behaviour pack on the stack, top
//! first, which is the order discovery wants. Being bundled, their `beta` dependencies pass
//! the Beta APIs gate. Outside an editor world their `@minecraft/server-editor` dependencies
//! are dropped with an info; in one they resolve to the editor module the engine serves.
//!
//! One pack cannot run by vanilla's own rules: the `experimental_spawn_rules` pack of client
//! install A names the entry `scripts/ExperimentalSpawnRules.js` while the pack ships
//! `scripts/scripts.js` only, and it depends on `@minecraft/server` `2.2.0-internal`, a version
//! no published table has. It is the one error of the game's files and is asserted as such, and
//! so is the second fault with the entry file supplied.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo nextest run -p bedrock_addon_std --test
//! smoke_script_module`; without the variable every test skips.

#[macro_use]
mod common;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use bedrock_addon::families::script::{
    Discovery, ModuleCatalog, ScriptContext, ScriptDiscovery, ScriptFiles, ScriptModule,
};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::manifest::Dependency;
use bedrock_addon::pack::PackOrigin;
use bedrock_addon_std::{DirSource, Driver, Options};

use common::game_files;
use common::world::base_game_world;

/// `allowed_modules` of the server install's `config/default/permissions.json`.
const SERVER_ALLOWED_MODULES: [&str; 6] = [
    "@minecraft/server-gametest",
    "@minecraft/server",
    "@minecraft/server-ui",
    "@minecraft/server-admin",
    "@minecraft/server-editor",
    "@minecraft/debug-utilities",
];

/// The children of `dir`, sorted.
fn children(dir: &Path) -> Vec<PathBuf> {
    let mut children: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|entry| entry.unwrap().path())
        .collect();
    children.sort();
    children
}

/// The entry names of a brarchive: a 16-byte header with the entry count at 8, then one
/// 256-byte descriptor per entry that starts with the name's length and the name.
fn brarchive_names(bytes: &[u8]) -> Vec<String> {
    let count = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
    (0..count)
        .map(|index| {
            let at = 0x10 + index * 256;
            let len = usize::from(bytes[at]);
            String::from_utf8(bytes[at + 1..at + 1 + len].to_vec()).expect("a UTF-8 name")
        })
        .collect()
}

/// Every file of the pack folder `dir` as a pack path: the loose files, and the entries of
/// `__brarchive/<folder>.brarchive` below `<folder>/` (vanilla's script file set includes
/// them).
fn pack_paths(dir: &Path) -> Vec<String> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<String>) {
        for path in children(dir) {
            if path.is_dir() {
                walk(root, &path, out);
            } else {
                let relative = path.strip_prefix(root).unwrap();
                out.push(relative.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    let mut paths = Vec::new();
    walk(dir, dir, &mut paths);
    let archives = dir.join("__brarchive");
    if archives.is_dir() {
        for archive in children(&archives) {
            let folder = archive.file_stem().unwrap().to_string_lossy().into_owned();
            for name in brarchive_names(&fs::read(&archive).unwrap()) {
                paths.push(format!("{folder}/{name}"));
            }
        }
    }
    paths
}

/// What one source's behaviour packs came to.
#[derive(Default)]
struct Outcome {
    /// Behaviour packs on the world's stack.
    packs: usize,
    /// Behaviour pack folders the world does not load.
    off_stack: usize,
    /// Packs without a script module whose manifest still names a native module.
    libraries: Vec<String>,
    /// The packs with a script module: folder name, discovery.
    scripts: Vec<(String, Discovery)>,
}

impl Outcome {
    fn errors(&self) -> Vec<String> {
        self.scripts
            .iter()
            .flat_map(|(pack, found)| {
                found
                    .diagnostics()
                    .iter()
                    .filter(|d| d.severity == bedrock_addon::diagnostic::Severity::Error)
                    .map(move |d| format!("{pack}: {}", d.code))
            })
            .collect()
    }

    /// Findings by code name.
    fn by_code(&self) -> BTreeMap<String, usize> {
        let mut counts = BTreeMap::new();
        for (_, found) in &self.scripts {
            for diagnostic in found.diagnostics() {
                let name: String = diagnostic.code.id().to_owned();
                *counts.entry(name).or_insert(0) += 1;
            }
        }
        counts
    }

    /// `<pack>: <module> <version>` for every resolved dependency.
    fn resolved(&self) -> Vec<String> {
        self.scripts
            .iter()
            .flat_map(|(pack, found)| {
                found
                    .plugin()
                    .into_iter()
                    .flat_map(|plugin| plugin.dependencies.iter())
                    .map(move |d| format!("{pack}: {} {}", d.module_name, d.resolved))
            })
            .collect()
    }

    fn running(&self) -> usize {
        self.scripts
            .iter()
            .filter(|(_, found)| found.plugin().is_some())
            .count()
    }
}

/// Discovers the plugin of every behaviour pack of the install `root` that has a script
/// module, against `catalog` with the gates of `cx`. The packs are the behaviour stack of the
/// world that lists every pack of the install ([`base_game_world`]), top first.
fn discover_all(root: &Path, catalog: &ModuleCatalog, cx: &ScriptContext<'_>) -> Outcome {
    let mut driver = Driver::new(Options::default()).unwrap();
    let repository = driver
        .repository([
            DirSource::packs(root.join("behavior_packs"), PackOrigin::Package),
            DirSource::packs(root.join("resource_packs"), PackOrigin::Package),
        ])
        .unwrap();
    let options = LoadOptions::default();
    let stacks = repository.stacks(&base_game_world(&repository, &options), options);
    let mut discovery = ScriptDiscovery::new(catalog);
    let mut outcome = Outcome::default();
    for listed in stacks.behavior.packs() {
        let pack = repository
            .packs()
            .find(|pack| pack.id() == listed.id())
            .unwrap_or_else(|| panic!("{} is not in the repository", listed.name()));
        let dir = driver
            .roots()
            .host_path(pack.location())
            .expect("a pack folder");
        let name = dir.file_name().unwrap().to_string_lossy().into_owned();
        outcome.packs += 1;
        let Some(module) = ScriptModule::from_manifest(pack.manifest()) else {
            let names_native = pack
                .manifest()
                .dependencies
                .iter()
                .any(|d| matches!(d, Dependency::NativeModule { .. }));
            if names_native {
                outcome.libraries.push(name);
            }
            continue;
        };
        // The file listing of the opened pack and the folder's agree on the loose files.
        let paths = pack_paths(&dir);
        for path in pack.paths() {
            assert!(
                paths.iter().any(|p| p == path.as_str()),
                "{name}: {path} is not on disk"
            );
        }
        let files = ScriptFiles::from_pack_paths(paths.iter().map(String::as_str));
        outcome
            .scripts
            .push((name, discovery.discover(&module, &files, cx)));
    }
    // The pack folders the world does not load (in the server install the six chemistry
    // slices): none has a script module, so none is left unchecked.
    let on_stack: Vec<_> = stacks
        .behavior
        .packs()
        .map(|pack| pack.id().clone())
        .collect();
    for dir in children(&root.join("behavior_packs")) {
        let pack = driver
            .pack_dir(&dir)
            .unwrap_or_else(|e| panic!("{}: {e:?}", dir.display()));
        if !on_stack.contains(pack.id()) {
            assert!(
                ScriptModule::from_manifest(pack.manifest()).is_none(),
                "{}: a script module off the stack",
                dir.display()
            );
            outcome.off_stack += 1;
        }
    }
    outcome
}

fn report(source: &str, outcome: &Outcome) {
    eprintln!(
        "{source}: {} behaviour packs on the stack ({} folders off it), {} with a script module ({} run), {} more name a native module; findings {:?}",
        outcome.packs,
        outcome.off_stack,
        outcome.scripts.len(),
        outcome.running(),
        outcome.libraries.len(),
        outcome.by_code(),
    );
    for line in outcome.resolved() {
        eprintln!("{source}: {line}");
    }
}

const PACKAGE: ScriptContext<'static> = ScriptContext {
    beta_apis: false,
    editor_world: false,
    allow_internal: false,
    is_package_origin: true,
    allowed_modules: None,
};

#[test]
fn server_install_script_packs_resolve() {
    let server = skip!(game_files::SERVER_INSTALL);
    // The list the server ships is the one the probes ran under.
    let permissions = fs::read_to_string(server.join("config/default/permissions.json")).unwrap();
    for module in SERVER_ALLOWED_MODULES {
        assert!(permissions.contains(&format!("\"{module}\"")), "{module}");
    }
    assert_eq!(
        permissions.matches("@minecraft/").count(),
        SERVER_ALLOWED_MODULES.len()
    );
    let allowed: Vec<String> = SERVER_ALLOWED_MODULES
        .iter()
        .map(|m| (*m).to_owned())
        .collect();
    let cx = ScriptContext {
        allowed_modules: Some(&allowed),
        ..PACKAGE
    };
    let catalog = ModuleCatalog::shipped();
    let outcome = discover_all(&server, &catalog, &cx);
    report("server install", &outcome);
    assert_eq!(outcome.errors(), [] as [&str; 0]);
    // 68 folders: 62 on the stack, 6 the world does not load.
    assert_eq!(
        (outcome.packs, outcome.off_stack),
        (62, 6),
        "behaviour packs"
    );
    let scripts: Vec<&str> = outcome
        .scripts
        .iter()
        .map(|(name, _)| name.as_str())
        .collect();
    assert_eq!(scripts, ["editor"]);
    assert_eq!(outcome.running(), 1);
    assert_eq!(outcome.libraries, ["server_editor_library"]);
    assert_eq!(
        outcome.resolved(),
        ["editor: @minecraft/server 2.11.0-beta"]
    );
    assert_eq!(
        outcome.by_code(),
        BTreeMap::from([("script/plugin_not_editor_world".to_owned(), 1)])
    );

    // The gates are what lets it through: as a pack that is not bundled it is refused.
    let unbundled = ScriptContext {
        is_package_origin: false,
        ..cx
    };
    let refused = discover_all(&server, &catalog, &unbundled);
    assert_eq!(refused.running(), 0);
    assert_eq!(refused.by_code().get("script/plugin_beta_apis"), Some(&1));
}

#[test]
fn client_install_b_script_packs_resolve() {
    let client = skip!(game_files::CLIENT_INSTALL_B);
    let catalog = ModuleCatalog::shipped();
    let outcome = discover_all(&client, &catalog, &PACKAGE);
    report("client install B", &outcome);
    assert_eq!(outcome.errors(), [] as [&str; 0]);
    // 71 folders: 64 on the stack, 7 the world does not load.
    assert_eq!(
        (outcome.packs, outcome.off_stack),
        (64, 7),
        "behaviour packs"
    );
    let scripts: Vec<&str> = outcome
        .scripts
        .iter()
        .map(|(name, _)| name.as_str())
        .collect();
    assert_eq!(scripts, ["editor"]);
    assert_eq!(outcome.running(), 1);
    assert_eq!(outcome.libraries, ["server_editor_library"]);
    assert_eq!(
        outcome.resolved(),
        ["editor: @minecraft/server 2.11.0-beta"]
    );
    assert_eq!(
        outcome.by_code(),
        BTreeMap::from([("script/plugin_not_editor_world".to_owned(), 1)])
    );
}

#[test]
fn client_install_a_script_packs_resolve() {
    let client = skip!(game_files::CLIENT_INSTALL_A);
    let catalog = ModuleCatalog::shipped();
    let outcome = discover_all(&client, &catalog, &PACKAGE);
    report("client install A", &outcome);
    // 75 folders: 69 on the stack, 6 the world does not load.
    assert_eq!(
        (outcome.packs, outcome.off_stack),
        (69, 6),
        "behaviour packs"
    );
    let scripts: Vec<&str> = outcome
        .scripts
        .iter()
        .map(|(name, _)| name.as_str())
        .collect();
    assert_eq!(
        scripts,
        [
            "editor",
            "editor-internal",
            "editor-samples",
            "experimental_spawn_rules"
        ]
    );
    assert_eq!(outcome.libraries, ["server_editor_library"]);
    // The three editor packs run; one dropped editor-module dependency each.
    assert_eq!(outcome.running(), 3);
    assert_eq!(
        outcome.resolved(),
        [
            "editor: @minecraft/server 2.11.0-beta",
            "editor-internal: @minecraft/server 2.11.0-beta",
            "editor-samples: @minecraft/server 2.11.0-beta",
        ]
    );
    assert_eq!(
        outcome.by_code(),
        BTreeMap::from([
            ("script/plugin_not_editor_world".to_owned(), 3),
            ("script/plugin_missing_main_file".to_owned(), 1)
        ])
    );
    // The fourth names an entry file it does not ship.
    assert_eq!(
        outcome.errors(),
        [
            "experimental_spawn_rules: Plugin [behaviorPack.spawn_rules.name - 0.0.1] - does not contain main file [ExperimentalSpawnRules.js]."
        ]
    );
}

/// In an editor world the editor packs get the editor module: `@minecraft/server-editor`
/// `beta` is the `0.1.0-beta` the install's `server_editor_library` ships, and client install A's
/// `editor-internal` gets the `0.1.0-internal` beside it. No pack is refused for a module the
/// catalog does not know.
#[test]
fn in_an_editor_world_the_editor_packs_get_the_editor_module() {
    const EDITOR_WORLD: ScriptContext<'static> = ScriptContext {
        editor_world: true,
        ..PACKAGE
    };
    let catalog = ModuleCatalog::shipped();
    let mut ran = 0;
    for (source, packs, off_stack, expected) in [
        (
            game_files::SERVER_INSTALL,
            62,
            6,
            vec![
                "editor: @minecraft/server-editor 0.1.0-beta",
                "editor: @minecraft/server 2.11.0-beta",
            ],
        ),
        (
            game_files::CLIENT_INSTALL_B,
            64,
            7,
            vec![
                "editor: @minecraft/server-editor 0.1.0-beta",
                "editor: @minecraft/server 2.11.0-beta",
            ],
        ),
        (
            game_files::CLIENT_INSTALL_A,
            69,
            6,
            vec![
                "editor: @minecraft/server-editor 0.1.0-beta",
                "editor: @minecraft/server 2.11.0-beta",
                "editor-internal: @minecraft/server-editor 0.1.0-internal",
                "editor-internal: @minecraft/server 2.11.0-beta",
                "editor-samples: @minecraft/server-editor 0.1.0-beta",
                "editor-samples: @minecraft/server 2.11.0-beta",
            ],
        ),
    ] {
        let Some(root) = game_files::source(source) else {
            eprintln!("skipped: {}", game_files::missing(source));
            continue;
        };
        let outcome = discover_all(&root, &catalog, &EDITOR_WORLD);
        report(source, &outcome);
        assert_eq!(
            (outcome.packs, outcome.off_stack),
            (packs, off_stack),
            "{source}: behaviour packs"
        );
        let mut resolved = outcome.resolved();
        resolved.sort_by_key(|line| line.split(':').next().map(str::to_owned));
        assert_eq!(resolved, expected, "{source}");
        // The server's files are the libraries the two versions are served from.
        let library = root.join("behavior_packs/server_editor_library/scripts");
        for version in ["0.1.0-beta", "0.1.0-internal"] {
            assert!(
                library
                    .join(format!("minecraft-server-editor.{version}.js"))
                    .is_file(),
                "{source}: {version}"
            );
        }
        let codes = outcome.by_code();
        // No finding of a member that is dropped, under any of the definitions that drop one.
        assert!(
            !codes.keys().any(|code| [
                "block/bone_visibility_boolean",
                "block/members_thrown_away",
                "core/dropped_field",
                "core/dropped_field_worded",
                "entity/actor_properties_not_loaded",
                "entity/member_removed_by_upgrade",
                "function/failed_header_dropped",
                "geometry/cape_dropped",
                "script/plugin_not_editor_world",
                "visual/failed_to_parse",
            ]
            .contains(&code.as_str())),
            "{source}: {codes:?}"
        );
        // Client install A's spawn-rules pack keeps its one error; nothing else.
        let errors = outcome.errors();
        if source == game_files::CLIENT_INSTALL_A {
            assert_eq!(errors.len(), 1, "{errors:?}");
            assert!(
                errors[0].starts_with("experimental_spawn_rules: "),
                "{errors:?}"
            );
        } else {
            assert_eq!(errors, [] as [&str; 0], "{source}");
        }
        ran += 1;
    }
    eprintln!("editor world: {ran} installs");
}

/// The spawn-rules pack of client install A, looked at alone: its entry is in no file of the pack,
/// loose or archived, and with the entry supplied its `2.2.0-internal` request is the next
/// fault, since no table says the engine serves that version.
#[test]
fn the_dev_spawn_rules_pack_has_two_faults() {
    let client = skip!(game_files::CLIENT_INSTALL_A);
    let dir = client.join("behavior_packs/experimental_spawn_rules");
    let paths = pack_paths(&dir);
    let scripts: Vec<&String> = paths.iter().filter(|p| p.starts_with("scripts/")).collect();
    // The loose file and the archive's entry of the same name.
    assert_eq!(scripts, ["scripts/scripts.js", "scripts/scripts.js"]);
    assert_eq!(
        brarchive_names(&fs::read(dir.join("__brarchive/scripts.brarchive")).unwrap()),
        ["scripts.js"]
    );

    let mut driver = Driver::new(Options::default()).unwrap();
    let pack = driver.pack_dir(&dir).unwrap();
    let module = ScriptModule::from_manifest(pack.manifest()).expect("a script module");
    assert_eq!(
        module.entry.as_deref(),
        Some("scripts/ExperimentalSpawnRules.js")
    );
    let catalog = ModuleCatalog::shipped();
    let supplied = ScriptFiles::from_members(["ExperimentalSpawnRules.js"]);
    let found = ScriptDiscovery::new(&catalog).discover(&module, &supplied, &PACKAGE);
    assert!(found.plugin().is_none());
    let texts: Vec<String> = found
        .diagnostics()
        .iter()
        .map(|d| d.code.to_string())
        .collect();
    assert_eq!(texts.len(), 1, "{texts:?}");
    assert!(
        texts[0].starts_with(
            "Plugin [behaviorPack.spawn_rules.name] - module [behaviorPack.spawn_rules.name - 0.0.1] requested invalid version [2.2.0-internal] of [@minecraft/server].\nCurrent [internal] version is [2.11.0-internal].\nAvailable versions:\n0.1.0\n1.0.0\n"
        ),
        "{texts:?}"
    );
    // The internal version is resolvable and not listed.
    assert!(!texts[0].contains("-internal\n") && !texts[0].ends_with("-internal"));
    assert!(
        texts[0].ends_with("\n2.11.0-beta\n3.0.0-alpha"),
        "{texts:?}"
    );
}

/// An entry inside `__brarchive/scripts.brarchive` counts as a script file: the server install's editor
/// pack has its entry both loose and archived.
#[test]
fn an_entry_inside_a_brarchive_is_a_script_file() {
    let server = skip!(game_files::SERVER_INSTALL);
    let dir = server.join("behavior_packs/editor");
    let archived = brarchive_names(&fs::read(dir.join("__brarchive/scripts.brarchive")).unwrap());
    assert!(
        archived.iter().any(|name| name == "Main.js"),
        "{archived:?}"
    );
    let only_archived: Vec<String> = archived
        .iter()
        .map(|name| format!("scripts/{name}"))
        .collect();
    let files = ScriptFiles::from_pack_paths(only_archived.iter().map(String::as_str));
    assert!(files.contains("Main.js"));
}
