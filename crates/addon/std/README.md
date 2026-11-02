# bedrock_addon_std

The `std` driver of [`bedrock_addon`](../addon): it answers the sans-IO loaders' requests with blocking `std::fs` calls, runs their jobs on the calling thread or on a rayon pool, reads the server-side files around packs (`world_*_packs.json`, saved pack settings, `config/`, `server.properties`, app data, key sidecars), and writes a pack built with `bedrock_addon::pack::PackWriter` to a directory or an archive file. `bedrock_addon` itself never touches the host; this crate is where that happens. Part of [bedrock-rs](https://github.com/bedrock-crustaceans/bedrock-rs).

## Installation

The crate uses edition 2024 and needs Rust 1.98 or newer. It depends on `bedrock_addon` without default features and re-exports nothing of it: name both crates, choose the content modules on `bedrock_addon`, and the features unify.

```toml
[dependencies]
bedrock_addon = { git = "https://github.com/bedrock-crustaceans/bedrock-rs.git" }
bedrock_addon_std = { git = "https://github.com/bedrock-crustaceans/bedrock-rs.git" }
```

## Quick start

A `Driver` keeps the table of host paths its loaders read from. `Driver::repository` scans packs directories (manifests and archive indexes only), `PackRepository::stacks` composes a world's two stacks from the world's pack lists, which `read_world_packs` reads, and `Driver::load_stacks` reads and parses every collection of both. A `LoadError` is one of four things: a pack that cannot be opened at all (`Open`), a rayon pool that cannot be built (`Pool`), and two that are bugs, of this driver (`Loader`) or of a loader (`Stalled`); an unreadable file or a broken document is a finding in the report.

The block touches the host's file system, so it is compiled and not run here; the [`load_stacks`](examples/load_stacks.rs) example runs the same calls on packs it writes into a temporary directory.

```rust,no_run
use bedrock_addon::diagnostic::Severity;
use bedrock_addon::load::{LoadOptions, StackOptions};
use bedrock_addon::pack::PackOrigin;
use bedrock_addon_std::{DirSource, Driver, Options, read_world_packs};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut driver = Driver::new(Options::default())?;
    let repository = driver.repository([
        DirSource::packs("behavior_packs", PackOrigin::World),
        DirSource::packs("resource_packs", PackOrigin::World),
    ])?;
    let world = read_world_packs("worlds/Bedrock level")?;
    let stacks = driver.load_stacks(
        repository.stacks(&world, LoadOptions::default()),
        StackOptions::default(),
    )?;

    // Already loaded: `load_all` returns the cached findings, the pack selection's included.
    for finding in stacks.load_all().at_least(Severity::Warning) {
        eprintln!("{finding}");
    }
    println!("{} custom block(s)", stacks.behavior.blocks().len());
    Ok(())
}
```

One pack is opened with `load_dir(path, &Options)` or, with the `zip` feature, `load_archive(path, &Options)`; on a driver they are `Driver::pack_dir` and `Driver::pack_archive`. Opening a folder reads its manifest and lists its files; the content files are read when a stack is loaded (`Driver::load_stacks`), and a stack built by hand over a folder pack that was only opened reports each content file as indexed but not loaded. A loader built by hand runs with `Driver::drive`, and a caller with a loop of its own (a game server's tick, an async runtime) uses `io::respond`, which answers one request of a loader with `std::fs`.

## Strategies

`Options::strategy` says where reads and parse jobs run.

| Strategy | Where the work runs | Feature |
| --- | --- | --- |
| `Strategy::Sequential` | on the calling thread, in the order the loader issues it: no thread, fully reproducible | |
| `Strategy::Rayon { threads }` | on a rayon pool the driver builds; `threads: None` is rayon's default count | `rayon` |
| `Strategy::RayonPool(pool)` | on a pool the caller built | `rayon` |

`Strategy::default()` is `Rayon { threads: None }` with the `rayon` feature and `Sequential` without it. The driver gives each thread of the pool it builds `bedrock_addon::load::RECOMMENDED_JOB_STACK` (8 MiB) of stack, and a pool passed in `RayonPool` should have as much. No file can overflow a smaller stack by its nesting, down to one megabyte, in a debug build too: the eight are the margin the loaders are tested with, for what does not grow with a file. The strategy says where the work runs and not what is loaded; the [`rayon_driver`](examples/rayon_driver.rs) example loads one world both ways and compares.

```rust
use bedrock_addon_std::{Driver, LoadError, Options, Strategy};

fn main() -> Result<(), LoadError> {
    // One thread, the same order every run.
    let options = Options::default().with_strategy(Strategy::Sequential);
    let _driver = Driver::new(options)?;
    Ok(())
}
```

## Readers

Each reader reads files with `std::fs` and hands the bytes to the pure parser of `bedrock_addon`; a `ReadError` is `Io` (the path and the `std::io::Error`) or `Rejected` (the file was read and refused).

| Reader | Reads | Returns | Feature |
| --- | --- | --- | --- |
| `read_world_packs(world_dir)` | `world_behavior_packs.json` and `world_resource_packs.json`; a missing file is an empty list | `pack::WorldPacks` | |
| `read_app_data(dir)` | the definitions outside packs (`definitions/attachables`, `definitions/features`, `definitions/feature_rules`, `definitions/spawn_groups`) and the game's `credits/credits.json`, for `LoadOptions::app_data` | `load::AppData` | |
| `read_saved_settings(path)` | a saved pack-settings file (`world_behavior_pack_settings.json`, `global_resource_pack_settings.json`); a missing file is the empty document | `pack::settings::SavedSettings` | |
| `read_script_config(config_dir)` | a dedicated server's `config/` folder; a missing folder is an empty configuration | `script::config::ScriptConfig` | `script` |
| `read_server_properties(path)` | `server.properties` | `script::config::ServerProperties` | `script` |

`read_saved_settings_document` returns the same settings as a document with the warnings the file gave (`None` for a file that is not there), and, with the `script` feature, `read_script_config_files` returns the files of `config/` unparsed. With the `encrypted` feature, `keys::read_sidecar_key` reads the content key beside a pack (`ruby.mcpack.key`), and `Options::sidecar_keys` makes the driver look there before it asks `Options::keys`.

## Writers

A `bedrock_addon::pack::PackWriter` holds a pack in memory; these two put it on disk. A `SaveError` is `Io` (the path and the `std::io::Error`), `NotEmpty` (the target directory holds something) or `Pack` (the writer's own `WriteError`).

| Writer | Writes | Feature |
| --- | --- | --- |
| `write_dir(&writer, dir)` | every file of `writer.files()` at its pack path below `dir`, with exactly its bytes. `dir` is created when absent and refused with `SaveError::NotEmpty` when it holds anything: a pack is never merged into an existing folder. A writer whose manifest its format cannot hold is `SaveError::Pack`, and nothing is created | |
| `write_archive(&writer, path)` | `writer.to_archive()` to a temporary file in `path`'s directory, synced, then renamed over `path`: a crash leaves the old archive or the new one, never a torn file | `zip` |

```rust
use bedrock_addon::manifest::Manifest;
use bedrock_addon::pack::{PackType, PackWriter};
use bedrock_addon_std::{Options, SaveError, load_dir, write_dir};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = Manifest::new("Ruby lamps", PackType::Behavior);
    let identity = manifest.identity();
    let writer = PackWriter::new(manifest);

    let out = tempfile::tempdir()?;
    let dir = out.path().join("ruby_lamps");
    write_dir(&writer, &dir)?;
    assert!(dir.join("manifest.json").is_file());
    assert!(matches!(write_dir(&writer, &dir), Err(SaveError::NotEmpty { .. })));

    let pack = load_dir(&dir, &Options::default())?;
    assert_eq!(*pack.id(), identity);
    Ok(())
}
```

The [`ruby_pack`](examples/ruby_pack.rs) example adds a block built in code and loads the written folder back through a `Driver`.

## Features

All four are on by default.

| Feature | Contents | Implies |
| --- | --- | --- |
| (always) | `io::{Roots, respond}`, `keys::ContentKeyProvider`, `Strategy::Sequential`, `Driver`, `DirSource`, `load_dir`, `read_world_packs`, `read_app_data`, `read_saved_settings`, `write_dir` | |
| `rayon` | `Strategy::Rayon` and `Strategy::RayonPool`: jobs and reads on a rayon pool built with `RECOMMENDED_JOB_STACK` | |
| `zip` | `load_archive`, `Driver::pack_archive`, `write_archive` | `bedrock_addon/zip` |
| `encrypted` | key sidecars (`keys::read_sidecar_key`, `Options::sidecar_keys`) | `bedrock_addon/encrypted` |
| `script` | `read_script_config`, `read_script_config_files`, `read_server_properties` | `bedrock_addon/script` |

## Examples

Each example runs with no argument and exits 0. It writes synthetic packs into a temporary directory and loads them from there; nothing else on the host is touched.

| Example | What it covers | Features |
| --- | --- | --- |
| [`load_stacks`](examples/load_stacks.rs) | the `Driver` loads a world's stacks, then the custom blocks are registered with their Molang permutation conditions compiled | |
| [`rayon_driver`](examples/rayon_driver.rs) | the driver with its default rayon strategy, in three lines; then the same world with `Strategy::Sequential`, and the two loads compared. It does not show which thread ran a job: the crates report none | `rayon` |
| [`ruby_pack`](examples/ruby_pack.rs) | build a block in code, add it to a new pack with `PackWriter::add`, write the pack with `write_dir`, load the directory back and find the same block | |

```sh
cargo run -p bedrock_addon_std --example load_stacks
```

The four examples of [`bedrock_addon`](../addon#examples) (`in_memory_pack`, `entity_components`, `client_visuals`, `diagnostics_gate`) read no file at all.

## Tests

`cargo test -p bedrock_addon_std` runs the tests that need nothing outside the repository: they write their packs into a temporary directory. The groups below that read the game's own files skip without their variable, print a `skipped:` line and pass; such a pass says nothing about what the test checks.

| Tests (`--test <name>`) | What they hold | Variable |
| --- | --- | --- |
| `driver`, `driver_packs`, `io`, `keys`, `read`, `write`, `pack_settings`, `script_config`, `world_order` | the `Driver` and its strategies, the root table and `io::respond`, key sidecars, the readers and the writers; `pack_settings` and `script_config` also hold a smoke test over the game's own files (saved-settings files, a dedicated server install) | `BEDROCK_ADDON_CORPUS`, for those smoke tests only |
| `small_stack`, `small_stack_feature`, `deep_documents` | a world loads on a thread with 1 MiB of stack, a tree feature with three quarters of a mebibyte, and documents nested to `MAX_JSON_DEPTH` load through every strategy | |
| `smoke_*` (one per content family) | the game's own files of the family load through the `Driver`, one pack at a time, with the findings the family is known to raise pinned exactly | `BEDROCK_ADDON_CORPUS` |
| `full`, `full_<tree>`, `full_<tree>_link`, `full_<tree>_write`, `full_checks`, `full_strategies*` | whole trees of the game's packs (a server install, two client installs, the sample packs, the split client and server pack trees) loaded as a server loads its world, linked and written back, with the counts pinned; `Strategy::Sequential` against `Strategy::Rayon` | `BEDROCK_ADDON_CORPUS` |
| `vanilla_gate` | the gate over the game's own packs: every pack of an install loads and links with no error and no warning, apart from the game's own content-log lines listed in its `EXPECTED` list; the packs below `test/` are reported, not asserted | `BEDROCK_ADDON_VANILLA_1_26_60` |

`BEDROCK_ADDON_CORPUS` names a folder with one entry per kind of install (the layout is in `tests/common/game_files.rs`; nothing of the game is in the repository). `BEDROCK_ADDON_VANILLA_1_26_60` names the root of a game install: the folder holding `behavior_packs/`, `resource_packs/`, `definitions/` and `test/`, every pack with its `manifest.json` and its `__brarchive` folders as shipped; a root whose packs carry no manifest is no install, and the test fails and says so. `BEDROCK_ADDON_VANILLA_1_26_60_DUMP`, set to a folder, also writes every finding of the gate there as tab-separated files. Run the gate with `BEDROCK_ADDON_VANILLA_1_26_60=<root> cargo test -p bedrock_addon_std --test vanilla_gate -- --nocapture`.

## Contributing

See [CONTRIBUTING.md](../../../CONTRIBUTING.md).

## License

Licensed under the **Apache License 2.0**; see [LICENSE](../../../LICENSE).
