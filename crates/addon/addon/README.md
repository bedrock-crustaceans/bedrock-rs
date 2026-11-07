# bedrock_addon

Minecraft Bedrock Edition behaviour and resource packs as typed Rust data: manifests, containers (zip, encrypted, brarchive), pack repositories and stacks, every content family the game reads from a pack (`ENGINE_VERSION` is 1.26.60, the newest game version the crate reads), diagnostics shaped like the content log, a link pass over the references between definitions, and write-back. Part of [bedrock-rs](https://github.com/bedrock-crustaceans/bedrock-rs).

The crate is sans-IO: it never reads a file, starts a thread or asks the clock. A pack is opened from bytes the caller already has, or by a loader that asks for bytes through an `Output` and takes the answer as an `Input`; the CPU work is a `Job` the caller runs where it likes. [`bedrock_addon_std`](../std) is the driver that answers the loaders with `std::fs` and, optionally, a rayon pool, and writes a pack to a directory or an archive file.

## Installation

The crate uses edition 2024 and needs Rust 1.98 or newer:

```toml
[dependencies]
bedrock_addon = { git = "https://github.com/bedrock-crustaceans/bedrock-rs.git" }
```

The default features enable every content module, zip and encrypted containers, the Molang compile check and the network NBT methods. A tool that reads one family writes `default-features = false, features = ["zip", "block"]`; the flags are listed under [Features](#features).

## Quick start

`Pack::from_files` opens a pack from files already in memory, `PackStack::of` puts packs on a stack, top first, and an accessor of the stack (`blocks()`) parses its family on first call and caches the result. A content file never makes these calls fail: a problem in one is a finding in the stack's report. A document of a family with a version history (a block, an entity, an item: the families of [Versions and upgrades](#versions-and-upgrades)) whose own load raises an error has no value and is listed under `rejected()` of its `Registry` or `Files` (a block without a `description`, below). What does fail is opening something that is no pack (`OpenError`: no `manifest.json`, an unreadable archive) and stacking a pack on the wrong side (`WrongPackType`).

```rust
use bedrock_addon::id::BlockId;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{Behavior, OpenOptions, Pack, PackStack};
use bedrock_addon::storage::PackPath;

const MANIFEST: &str = r#"{
    "format_version": 2,
    "header": {
        "name": "Ruby lamps", "description": "", "version": [1, 0, 0],
        "uuid": "3b2c7a52-8f0e-4b7d-9a35-1c6e5d0f4a11", "min_engine_version": [1, 26, 20]
    },
    "modules": [{ "type": "data", "uuid": "8d4f1e63-2a9b-4c0d-b1e7-5f3a6c2d9e22", "version": [1, 0, 0] }]
}"#;

const RUBY_LAMP: &str = r#"{
    "format_version": "1.26.20",
    "minecraft:block": {
        "description": { "identifier": "demo:ruby_lamp", "states": { "demo:lit": [false, true] } },
        "components": { "minecraft:light_emission": 0 },
        "permutations": [
            { "condition": "q.block_state('demo:lit')", "components": { "minecraft:light_emission": 15 } }
        ]
    }
}"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pack = Pack::from_files(
        [
            (PackPath::from_static("manifest.json"), MANIFEST),
            (PackPath::from_static("blocks/ruby_lamp.json"), RUBY_LAMP),
        ],
        &OpenOptions::default(),
    )?;
    let stack = PackStack::<Behavior>::of([pack], LoadOptions::default())?;

    let lamp = stack.blocks().get(&BlockId::from_static("demo:ruby_lamp")).expect("loaded");
    assert_eq!(lamp.origin().path, PackPath::from_static("blocks/ruby_lamp.json"));
    assert_eq!(lamp.permutations.len(), 1);
    assert!(!stack.report().has_errors(), "{}", stack.report());
    Ok(())
}
```

## Packs, repositories and stacks

All of these are in `bedrock_addon::pack`.

- A `Pack` is one opened pack: its `manifest()`, its identity (`id()`, a UUID and a version), its `report()` and its files (`paths()`, `read(&PackPath)`). `Pack::from_files` takes files in memory and, with the `zip` feature, `Pack::from_archive` takes the bytes of a `.mcpack` or `.zip`.
- A `PackRepository` is every pack a host knows of. `add_pack(pack, origin)` registers one; the first registration of an identity and origin wins.
- `PackRepository::stacks(&WorldPacks, LoadOptions)` composes the two stacks of a world as vanilla does: the packs the world's `world_behavior_packs.json` and `world_resource_packs.json` choose (`WorldPacks::parse`), their dependencies, subpacks and the base-game layer. It never fails: a missing dependency hides the pack and is a finding in `StackSet::selection()`.
- A `StackSet` is the pair `behavior: PackStack<Behavior>` and `resource: PackStack<Resource>`. `load_all()` parses every collection of both stacks on the calling thread and returns the `StackReport`.
- `PackStack::<S>::of(packs, LoadOptions)` is a stack of exactly the packs given, for a tool that looks at one pack.
- `StackSet::link()` resolves the references between the definitions of both stacks ([Linking](#linking)), and a `PackWriter` builds a pack to write ([Writing packs](#writing-packs)).

```rust
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{OpenOptions, Pack, PackOrigin, PackRepository, WorldPacks};
use bedrock_addon::storage::PackPath;

/// A manifest with one module of `module` type.
fn manifest(name: &str, uuid: &str, module: &str, module_uuid: &str) -> String {
    format!(
        r#"{{
            "format_version": 2,
            "header": {{ "name": "{name}", "description": "", "uuid": "{uuid}", "version": [1, 0, 0], "min_engine_version": [1, 26, 20] }},
            "modules": [{{ "type": "{module}", "uuid": "{module_uuid}", "version": [1, 0, 0] }}]
        }}"#
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const BEHAVIOR: &str = "3b2c7a52-8f0e-4b7d-9a35-1c6e5d0f4a11";
    const RESOURCE: &str = "c5e8a914-6d3b-4f2a-8c70-9b1d4e7f3a33";
    let open = |manifest: String| {
        Pack::from_files([(PackPath::from_static("manifest.json"), manifest)], &OpenOptions::default())
    };

    let mut repository = PackRepository::new();
    repository.add_pack(
        open(manifest("Ruby behaviour", BEHAVIOR, "data", "8d4f1e63-2a9b-4c0d-b1e7-5f3a6c2d9e22"))?,
        PackOrigin::World,
    );
    repository.add_pack(
        open(manifest("Ruby resources", RESOURCE, "resources", "f1a2b3c4-5d6e-4f70-8192-a3b4c5d6e744"))?,
        PackOrigin::World,
    );

    // The world's two pack lists, as the files hold them.
    let world = WorldPacks::parse(
        Some(format!(r#"[{{ "pack_id": "{BEHAVIOR}", "version": [1, 0, 0] }}]"#).as_bytes()),
        Some(format!(r#"[{{ "pack_id": "{RESOURCE}", "version": [1, 0, 0] }}]"#).as_bytes()),
    )?;
    let stacks = repository.stacks(&world, LoadOptions::default());
    assert!(!stacks.selection().has_errors());

    let names: Vec<String> = stacks.behavior.packs().map(|pack| pack.name().to_owned()).collect();
    assert_eq!(names, ["Ruby behaviour"]);
    assert_eq!(stacks.resource.packs().len(), 1);

    let report = stacks.load_all();
    assert!(!report.has_errors(), "{report}");
    Ok(())
}
```

Packs that are on disk are opened by the loaders of `bedrock_addon::load`: `PackLoader` (one pack), `RepositoryLoader` (every pack below a set of sources) and `StackLoader` (the collections of a `StackSet`, with their parse jobs). Each is a state machine a caller drives through `Sans::poll` and `Sans::handle` (the trait `bedrock_addon::load::Sans`, which every `Loader` is); `bedrock_addon_std::Driver` is that caller for `std::fs`.

A host that runs the jobs on threads of its own needs no stack that grows with a file. JSON nesting is capped at 128 levels (`json::MAX_JSON_DEPTH`; a deeper file is refused with a finding), and the reads and writes that go one call deeper per level run on a stack made for them, on the same thread, where the thread is short of one: a document within the limit never overflows a thread with one megabyte of stack, in a debug build too. `load::RECOMMENDED_JOB_STACK` (8 MiB) is the margin for what does not grow with a file. Three cases are outside that rule. On WebAssembly a module has the one stack it was linked with and none can be made, so the embedder sizes it for the typed reads (up to 11 KiB a level optimised, 87 KiB unoptimised). On a target the `stacker` crate has no stack switch for, the read runs where it is, as on WebAssembly. And where the host cannot map the extra stack (an address-space limit, strict overcommit), the read panics; the stack for a text at the limit is 65 MiB of address space.

## Content modules

One module per game document family under `families`, each behind the feature of its name, and one accessor per collection on the stack it is read from: `bp` is `PackStack<Behavior>`, `rp` is `PackStack<Resource>`, `both` is either. An accessor returns a `Registry<T>` (definitions by identifier), a `Files<T>` (documents by path), a `Merged<T>` (one value merged over the stack) or a group of these (two differ: `both.scripts()` is a slice of `ScriptBundle`, and `both.translations(&language)` is an `Option` of the merged table of that language), and it is present only with the family's feature.

A `Registry` and a `Files` answer for the effective copy of each definition (`get`, `iter`); the copies below it are its layers. `all_layers()` on either lists every identifier or path a copy is kept of with all of its layers, also one that has no effective copy because a failed higher copy claimed it (kept with `LoadOptions::keep_shadowed`).

| Family and feature | What | Accessors |
| --- | --- | --- |
| `block` | custom block definitions | `bp.blocks()` |
| `voxel_shape` | voxel shapes | `bp.voxel_shapes()` |
| `blocks_json` | the client's `blocks.json` | `rp.client_blocks()` |
| `block_culling` | block culling rules | `rp.block_culling()` |
| `item` | items; the armour trims compiled into the game (`families::item::trim::TrimData::vanilla()`) | `bp.items()` |
| `crafting_catalog` | the crafting item catalogue | `bp.crafting_catalog()` |
| `attachable` | attachables | `rp.attachables()` |
| `entity` | entities with their components and AI goals | `bp.entities()` |
| `client_entity` | client entities | `rp.client_entities()` |
| `atomic_client_entity` | atomic client entities | `rp.atomic_client_entities()` |
| `spawn_rules` | spawn rules | `bp.spawn_rules()` |
| `spawn_groups` | the spawn groups of the app data | `bp.spawn_groups()` |
| `biome` | biomes | `bp.biomes()` |
| `client_biome` | client biomes | `rp.client_biomes()` |
| `biomes_client` | the legacy `biomes_client.json` | `rp.legacy_client_biomes()` |
| `feature` | features | `bp.features()` |
| `feature_rules` | feature rules | `bp.feature_rules()` |
| `jigsaw` | jigsaw structures with their pools, sets and processors | `bp.jigsaw()` |
| `dimension` | dimensions | `bp.dimensions()` |
| `structure` | `.mcstructure` files | `bp.structures()` |
| `loot` | loot tables | `bp.loot_tables()` |
| `recipe` | recipes, one registry per recipe namespace | `bp.recipes()` |
| `trade` | trade tables | `bp.trade_tables()` |
| `dialogue` | NPC dialogue scenes | `bp.dialogue()` |
| `camera` | camera presets, lists and splines | `bp.cameras()` |
| `aim_assist` | aim assist presets and categories | `bp.aim_assist()` |
| `animation` | animations | `both.animations()` |
| `animation_controller` | animation controllers | `both.animation_controllers()` |
| `geometry` | geometries | `rp.geometries()` |
| `render_controller` | render controllers | `rp.render_controllers()` |
| `fog` | fog settings | `rp.fogs()` |
| `material` | materials | `rp.materials()` |
| `deferred_settings` | the deferred renderer's settings | `rp.deferred_settings()` |
| `texture_file` | the texture files of the stack: which pack has which image or texture set | `rp.texture_files()` |
| `texture_atlas` | the terrain and item atlases | `rp.texture_atlases()` |
| `flipbooks` | flipbooks | `rp.flipbooks()` |
| `texture_list` | texture lists | `rp.texture_lists()` |
| `texture_set` | texture sets | `rp.texture_sets()` |
| `legacy_path_mapping` | `texture_backwards_compatibility_mapping.json`, which the game reads from its app package only | none |
| `particle` | particle effects | `rp.particles()` |
| `sound_definitions` | `sound_definitions.json` | `rp.sound_definitions()` |
| `sounds_json` | `sounds.json` | `rp.sounds_json()` |
| `music` | music definitions | `rp.music()` |
| `sound_file` | the audio files | `rp.sound_files()` |
| `server_sound_definitions` | the server's sound definitions of a behaviour pack | `bp.server_sound_definitions()` |
| `languages` | `languages.json` and `language_names.json` | `both.languages()` |
| `lang` | the `.lang` files | `both.translations(&Language)` |
| `function` | `.mcfunction` files and `tick.json` | `bp.functions()`, `bp.tick_functions()` |
| `json_ui` | JSON UI | `rp.ui()` |
| `ddui` | data-driven UI | `rp.data_driven_ui()` |
| `splashes` | `splashes.json` | `rp.splashes()` |
| `loading_messages` | `loading_messages.json` | `rp.loading_messages()` |
| `credits` | `credits.json` | `rp.credits()` |
| `font` | fonts | `rp.fonts()` |
| `ui_texture_info` | the side files of UI images | `rp.ui_texture_infos()` |
| `skin` | skin packs, which are never on a world's stack | none |
| `persona` | persona pieces (`families::persona::PersonaPiece::load_all(&pack)`) | none |
| `poi` | point-of-interest types and tags | `bp.poi()` |
| `behavior_tree` | behavior trees | `bp.behavior_trees()` |
| `script` | script modules and the script files of each pack, one `ScriptBundle` a pack, with the plugin its script module starts; module metadata (`families::script::meta`), which is no pack content | `both.scripts()` |
| `server_config` | the server's script configuration (`families::server_config`: the `config/` folder and `server.properties`), which is no pack content; turns on `script` | none |

A component of a block, an item, a biome or an entity is read by its type: `block.components.get::<LightEmission>()`. The entity components vanilla itself still parses at run time (`minecraft:health`, `minecraft:movement`, `minecraft:type_family` among them) have no type: `components.iter()` gives each as a `ComponentRef::Dynamic` with its JSON as written, as the [`entity_components`](examples/entity_components.rs) example shows.

## Diagnostics

Loading never fails for what a file says. Every finding is a `diagnostic::Diagnostic`: a `code` (a `Code`: a definition's, with its arguments, asked by `is(&DEF)` or by its general `kind()`, never a message to parse), a `severity` (`Debug`, `Info`, `Warning`, `Error`), the `phase` that raised it and a `location` (pack, file, JSON path, byte span). Its `Display` is one line in the shape of the game's content log.

The JSON path of a location (`location.json_path`, a `diagnostic::JsonPath`) is data too. Its `Display` joins the segments with `/` and writes a member key as a JSON pointer token (RFC 6901: `~` as `~0`, `/` as `~1`), so a key that holds a `/` stays one piece; that text is for a reader. Code asks the path: `segments()`, `last_key()`, `is_key(key)`, `starts_with(&prefix)`, `ends_with(&suffix)`, `index_after(key)` (the 3 of `permutations/3/condition`) and `strip_prefix(&prefix)`; `JsonPath::of_keys(["minecraft:block", "components"])` builds one to compare with. What a finding names beyond its code is its `detail`: the reference of a link finding (`DiagnosticDetail::Reference`), the Molang compiler's message (`Molang`, with the `molang` feature), the paths a file lookup tried (`TriedPaths`), or a note in fixed words where the game has no text (`Note`: which member a written document carries a value as, or why an item has no text).

A document of a family with a version history whose own load raises an error is not in its collection: it is in `rejected()` of its `Registry` or `Files`, with every finding about it (the block of the example below; texture sets call theirs `rejected_sets()`). `blocks.json`, which is one file and no such family, keeps its file with an error at an entry it refuses; only a file that is no JSON object is rejected. And an error is not always a rejection: one the stack tells when it combines the packs' definitions can leave the document in the registry. A dialogue scene tag that two packs define, for example, is told with an error, and the scene stays as a shadowed layer. A member the game reads strictly refuses a value of another JSON type (`InvalidType`). Elsewhere a scalar of another JSON type than its member's (`7` where a string belongs, `"7"` where a number does) is converted, the member holds the converted value, and an `InvalidType` warning at the member says so. Three codes say what a load did with a value it does not hold as the file has it: `DroppedField` (the file's value at the member is gone: a rule or an upgrade step removes or replaces it), `MemberNotKept` (a member no schema declares, at a place that keeps no unknown members: read, not held, not written back) and `ValueRewritten` (the member stays and its value is held, and written, in another form than the file has: a name in another case, a boolean for a number; also a value the written format has no spelling for, written as the nearest it has, and a value a written table carries under another member, which the finding's note names). `LossyConversion` is a number that a 32-bit float does not hold as written; it is raised by the readers of fog and the render documents read the same way, of geometry and of legacy trade tables, and by no other, so its absence says nothing elsewhere. A `StackReport` (`PackStack::report()`, `StackSet::load_all()`) lists the findings of everything loaded so far and filters them with `at_least(Severity)`, `has_errors()`, `by_pack()` and `by_file()`.

```rust
use bedrock_addon::diagnostic::{Kind, Severity};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{Behavior, OpenOptions, Pack, PackStack};
use bedrock_addon::storage::PackPath;

const MANIFEST: &str = r#"{
    "format_version": 2,
    "header": {
        "name": "Ruby lamps", "description": "", "version": [1, 0, 0],
        "uuid": "3b2c7a52-8f0e-4b7d-9a35-1c6e5d0f4a11", "min_engine_version": [1, 26, 20]
    },
    "modules": [{ "type": "data", "uuid": "8d4f1e63-2a9b-4c0d-b1e7-5f3a6c2d9e22", "version": [1, 0, 0] }]
}"#;

// No `description`, so no identifier: the block cannot load.
const BROKEN: &str = r#"{ "format_version": "1.26.20", "minecraft:block": { "components": {} } }"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pack = Pack::from_files(
        [
            (PackPath::from_static("manifest.json"), MANIFEST),
            (PackPath::from_static("blocks/broken.json"), BROKEN),
        ],
        &OpenOptions::default(),
    )?;
    let stack = PackStack::<Behavior>::of([pack], LoadOptions::default())?;

    let blocks = stack.blocks();
    assert!(blocks.is_empty());
    let [rejected] = blocks.rejected() else { panic!("one rejected document") };
    assert_eq!(rejected.origin.path, PackPath::from_static("blocks/broken.json"));

    let report = stack.report();
    assert!(report.has_errors());
    for finding in report.at_least(Severity::Warning) {
        eprintln!("{finding}");
    }
    let missing = report
        .iter()
        .filter(|finding| finding.kind() == Kind::MissingRequired)
        .count();
    assert_eq!(missing, 1);
    Ok(())
}
```

## Versions and upgrades

The type at a module's root is the document at the newest format the crate reads (`families::block::Block`). A file written at an older `format_version` is parsed by the struct of its own version (the module's `history`) and upgraded step by step, the way the game upgrades it, so a caller sees one shape whatever the pack's age. A `Registry<T>` holds its entries as `Document<T>`, which dereferences to the value and keeps where it came from (`origin()`) and what was done to it (`version()`): the version as `written`, the struct it was `parsed_as`, and whether it was `upgraded`.

```rust
use bedrock_addon::id::BlockId;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{Behavior, OpenOptions, Pack, PackStack};
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::FormatVersion;

const MANIFEST: &str = r#"{
    "format_version": 2,
    "header": {
        "name": "Ruby lamps", "description": "", "version": [1, 0, 0],
        "uuid": "3b2c7a52-8f0e-4b7d-9a35-1c6e5d0f4a11", "min_engine_version": [1, 21, 0]
    },
    "modules": [{ "type": "data", "uuid": "8d4f1e63-2a9b-4c0d-b1e7-5f3a6c2d9e22", "version": [1, 0, 0] }]
}"#;

const OLD_BLOCK: &str = r#"{
    "format_version": "1.20.80",
    "minecraft:block": {
        "description": { "identifier": "demo:old_lamp" },
        "components": { "minecraft:light_emission": 7 }
    }
}"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pack = Pack::from_files(
        [
            (PackPath::from_static("manifest.json"), MANIFEST),
            (PackPath::from_static("blocks/old_lamp.json"), OLD_BLOCK),
        ],
        &OpenOptions::default(),
    )?;
    let stack = PackStack::<Behavior>::of([pack], LoadOptions::default())?;
    let lamp = stack.blocks().get(&BlockId::from_static("demo:old_lamp")).expect("loaded");

    let version = lamp.version();
    assert_eq!(version.written, Some(FormatVersion { major: 1, minor: 20, patch: 80 }));
    assert!(version.upgraded);
    assert!(version.parsed_as <= FormatVersion { major: 1, minor: 20, patch: 80 });
    Ok(())
}
```

`LoadOptions` carries what the game's world settings decide: the experiments, the Beta APIs toggle and the base-game version. A document that says `"format_version": "beta"` is permitted when one of four things holds: `LoadOptions::beta_apis` is set, the experiments hold `Experiment::BetaApis` (which sets it), the pack is a base-game pack, or the family does not put the word behind the toggle at all (block culling rules, atomic client entities and saved pack settings do not). Otherwise it is refused with `BetaFeaturesNotEnabled`. Permitted is not the same as read: a family without a beta format (block culling rules and saved pack settings among them) refuses the document as an unsupported version. Beta definitions are kept apart from the stable ones, in the registry's `beta()`.

## Linking

`StackSet::link()` resolves the references between definitions, after the load. Most go from one family to another: the block an entity's trail names, the loot table of a block, the geometry and textures of a client entity, the atlas key of a `blocks.json` entry. Some stay inside one family: the blocks of a block's placement filter, the features a feature names, the loot tables a loot table names, the subtree of a behaviour tree, the entities an entity's components name. The pass reads the collections through the stacks' accessors (loading, from bytes already there, what is not loaded yet), issues no request and performs no I/O (all it consults of the host is the `BuiltIns` of the `LoadOptions`, below), never fails, and returns its findings as `Diagnostics`, each of `Phase::Link`. It changes no registry: a definition with a reference that does not resolve stays where the load put it.

The rules are in five groups: entities and spawn rules; blocks, items, loot, trades, recipes and the item catalogue; world generation, cameras and the small behaviour-pack families; client entities, attachables and render controllers; and the resource data (`blocks.json`, atlases, flipbooks, particles, client biomes, sounds, UI). (A sixth group module, for scripts, holds no rule: the script rules are told by the load or are functions of their own, below.) Rules are compiled by feature: the one between entities and blocks, for example, exists only with both `entity` and `block`.

A name carried as the argument of a Molang query is left to the game: a component group or a property that an entity names in a query, for example, is not resolved. The rules of the client visuals do read Molang text, in two ways. A render controller's expression that is exactly `geometry.<name>`, `texture.<name>` or `material.<name>` is taken as the slot it names; and for an atomic client entity, a controller's `light_color_multiplier` is asked whether it is a constant, by its text and, with the `molang` feature, by compiling it.

The severity of a link finding is graded by what the rule records of the game's handling of the document that holds the reference, not by the level the game logs at. (The middle column is what the rule assumes of the game; nothing in this repository runs the game.)

| Severity | What the game does | In the crate |
| --- | --- | --- |
| `Error` | drops the document or definition | the definition stays in its registry; the finding says the game would not have it |
| `Warning` | drops the entry or member, puts a default in its place, or logs the miss and keeps the value | nothing changes |
| `Info` | keeps the reference as written and logs nothing | a lint: only the link pass says it |

A name the game checks when it links its own assets is the exception to the middle column: the game logs each miss at the content log's error level and removes nothing, so the finding is an `Error`, and a link finding never refuses a document. The block a placement filter names is one: in the example below `demo:pedestal` is an `Error`. A state of the block that the block lacks is the crate's own check, no line of the game's, and an `Info`.

A name that resolves to nothing is a finding of `Kind::UnresolvedReference` (a texture file, a sound event and a translation key among them), one that resolves to something the use cannot accept `Kind::InvalidReference`, a cycle `Kind::Cycle`. What was named is data, in the finding's `detail`: `DiagnosticDetail::Reference { kind, id, hint }`, the hint being a near miss when there is one, such as the same name in another case.

`link()` runs every rule. `link_with(&LinkOptions { lints: false, ..Default::default() })` returns no finding of `Info` severity: the pass's sink drops them. That leaves out the lints, and with them the two kinds of information that are no lint, a miss under a partial view and `FileNotLoaded` (both below). A server that links on every start turns the lints off; a tool for pack authors keeps them. `LinkOptions { client: false, ..Default::default() }` links the stacks as a dedicated server loads them: the rules of what only the client reads (R7, the textures of a client entity, an attachable or an atomic client entity) say nothing; the default links as the client.

```rust
use std::sync::Arc;

use bedrock_addon::diagnostic::{DiagnosticDetail, Diagnostics, Kind, Phase, Severity};
use bedrock_addon::load::{BuiltIns, LoadOptions};
use bedrock_addon::pack::{OpenOptions, Pack, PackOrigin, PackRepository, WorldPacks};
use bedrock_addon::storage::PackPath;

const MANIFEST: &str = r#"{
    "format_version": 2,
    "header": {
        "name": "Ruby lamps", "description": "", "version": [1, 0, 0],
        "uuid": "3b2c7a52-8f0e-4b7d-9a35-1c6e5d0f4a11", "min_engine_version": [1, 26, 20]
    },
    "modules": [{ "type": "data", "uuid": "8d4f1e63-2a9b-4c0d-b1e7-5f3a6c2d9e22", "version": [1, 0, 0] }]
}"#;

// The lamp may be placed on stone, which is the game's, and on a pedestal no pack defines.
const RUBY_LAMP: &str = r#"{
    "format_version": "1.26.20",
    "minecraft:block": {
        "description": { "identifier": "demo:ruby_lamp" },
        "components": {
            "minecraft:placement_filter": {
                "conditions": [{ "allowed_faces": ["up"], "block_filter": ["minecraft:stone", "demo:pedestal"] }]
            }
        }
    }
}"#;

/// A host that knows one block and nothing else.
struct OnlyThePedestal;

impl BuiltIns for OnlyThePedestal {
    fn contains(&self, kind: &'static str, id: &str) -> bool {
        kind == "block" && id == "demo:pedestal"
    }
}

/// Links a world that uses the one pack, and gives the blocks the pass could not resolve.
fn unresolved_blocks(options: LoadOptions) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let pack = Pack::from_files(
        [
            (PackPath::from_static("manifest.json"), MANIFEST),
            (PackPath::from_static("blocks/ruby_lamp.json"), RUBY_LAMP),
        ],
        &OpenOptions::default(),
    )?;
    let mut repository = PackRepository::new();
    repository.add_pack(pack, PackOrigin::World);
    let world = WorldPacks::parse(
        Some(br#"[{ "pack_id": "3b2c7a52-8f0e-4b7d-9a35-1c6e5d0f4a11", "version": [1, 0, 0] }]"#),
        None,
    )?;
    let stacks = repository.stacks(&world, options);
    assert!(!stacks.load_all().has_errors());

    let links: Diagnostics = stacks.link();
    Ok(links
        .iter()
        .map(|finding| {
            assert_eq!(finding.kind(), Kind::UnresolvedReference);
            assert_eq!((finding.severity, finding.phase), (Severity::Error, Phase::Link));
            match finding.detail.as_deref() {
                Some(DiagnosticDetail::Reference { kind: "block", id, .. }) => id.clone(),
                other => panic!("not a block reference: {other:?}"),
            }
        })
        .collect())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Without a host the crate's own list of the game's blocks answers: stone is known.
    assert_eq!(unresolved_blocks(LoadOptions::default())?, ["demo:pedestal"]);

    // A host is asked in place of the crate's list and believed, yes or no: this one does not know stone.
    let hosted = LoadOptions {
        built_ins: Some(Arc::new(OnlyThePedestal)),
        ..LoadOptions::default()
    };
    assert_eq!(unresolved_blocks(hosted)?, ["minecraft:stone"]);
    Ok(())
}
```

### The game's own content

A name no pack of the stack defines can still be the game's.

- **A host** answers through `LoadOptions::built_ins` (`load::BuiltIns`, one method: `contains(kind, id)`). It is believed, yes or no, but for the names of the third point: the compiled block above shows a host's "no" for `minecraft:stone`. For a registry keyed by its identifier the name is passed in the kind's canonical form (`minecraft:` in front of a bare name of a kind that defaults to it, an item without its `:aux` value, an entity without its `<event>`, a geometry or render-controller name in lower case); a loot table is passed by its path in lower case, without what stands before a last `:`.
- **Without a host**, lists of the crate answer in its place, and a host replaces them. The generated identifier lists (`vanilla::ids()`) do for `block`, `item` (block items included: every block of the stack has an item of its name), `entity`, `biome` and `jigsaw structure`; lists of the crate's own do for `level sound event`, for `aim assist preset` and `aim assist category` (the ones the game's behaviour pack defines) and for `feature` (the features the game registers in code). A lookup of a kind without a list has no such answer.
- **Some names stand whatever a host says.** Among them: the item and block names of older versions that the game's own files still write, the entities the game has without a file, the item tags the game's recipes use, and the default cubemap. The block rules know the voxel shapes `minecraft:empty` and `minecraft:unit_cube` and the geometries `minecraft:geometry.cross`, `minecraft:geometry.full_block` and `minecraft:geometry.full_block_v1` before they look anything up. The nine behavior trees the game has in code are documents of the stack's registry; a host is asked for any other tree no pack defines.

Two kinds are registries of the game that an add-on does not carry, `material` and `atlas key` (a tile of the terrain or the item atlas), and a reference to one is judged by the view the pass has of the kind:

| View | When | A name the stack lacks |
| --- | --- | --- |
| none | the stack holds no source of the kind (no material list, no atlas file) and there is no host | nothing is said: the name cannot be told from one of the game's |
| partial | the only sources are the add-on's own, and there is no host | an information, with the rule's code and detail: the name may be the game's (left out with `lints: false`) |
| whole | a pack the game ships provides a source that was read into the stack, or the stacks were built with a host | told with the rule's own severity |

One rule is narrower than the table: a flipbook's `atlas_tile` is compared only when the stack has a terrain atlas, so with a host and no atlas it says nothing. A source of the game's pack that is listed and not loaded gives no whole view; the unread file is told once, as a `FileNotLoaded` information.

The kinds a rule of the link pass passes to a host: `block`, `item`, `entity`, `biome`, `feature`, `jigsaw structure`, `template pool`, `processor list`, `structure`, `voxel shape`, `culling rules`, `loot table`, `trade table`, `poi type`, `poi tag`, `behavior tree`, `aim assist preset`, `aim assist category`, `geometry`, `render controller`, `material`, `texture`, `atlas key`, `particle`, `animation`, `fog`, `water settings`, `lighting settings`, `atmosphere settings`, `color grading settings`, `cubemap settings`, `sound event`, `level sound event`, `lang key`, `item tag` and `crafting tag`. No test holds this list. The load asks a host too, for kinds of its own (a camera list's presets and a script-registered dimension, for example), and `link()` runs the loads it needs.

Linking the game's own packs with their app data needs no host for blocks, items, entities, biomes, dimensions, camera presets, structure features and features; a host (`LoadOptions::built_ins`) quiets what is left: `item tag` (the tags of the game's items beyond its recipes'), `entity` (actors defined in code with a client entity and no behaviour file) and, on a dedicated server, the client kinds `atlas key`, `texture`, `geometry`, `animation`, `render controller`. An add-on linked without the game's packs needs a host for every other kind the game defines in files (features, jigsaw kinds, loot and trade tables, geometries, animations, render controllers, particles, sounds, fogs, localisation keys, item tags); materials and atlas tiles are not judged until the stack or a host has them. Four more things such a link of the game's packs still tells without a host, each a reference in the game's own files that names nothing: `minecraft:oak_button` in the aim-assist categories, a culling rule, a particle event and three loot tables. No committed test repeats this.

A rule that asks the pass for the bytes of a file (the header of a flipbook image, for example) gets them only when they are loaded: for a file that is indexed and not resident it gets none, and one `FileNotLoaded` information says so (nothing says so with `lints: false`). The lookup of a localisation key reads `.lang` files itself and says nothing of one that is not loaded: a key it cannot judge for that reason is left unjudged.

Three rules about scripts and saved settings take an input that is no stack content, so they are functions of their own and not part of `link()`: `families::server_config::ScriptConfig::unmatched_folders` (a `config/` folder that no script which starts uses: it takes the plugins of the stack, `stack.scripts().iter().filter_map(|bundle| bundle.plugin.as_ref())`, so the folder of a module whose script does not start is reported too; `unmatched_folders_of_modules` takes every script module of the stack by its identity, started or not), `ScriptConfig::parse` (a permission level for a name that is no command) and `pack::settings::unmatched_entries` (a saved setting for a pack the world does not have). The first and the last return findings of `Phase::Link`.

The [`diagnostics_gate`](examples/diagnostics_gate.rs) example is a CI gate over a load and a link.

## Writing packs

`pack::PackWriter` builds a pack in memory. `PackWriter::new(manifest)` starts from a manifest (`Manifest::new(name, pack_type)` is a format-2 manifest with fresh random UUIDs, which takes randomness from the operating system and is the one call here that is not a function of its arguments; `Manifest::with_ids` is the same with UUIDs the caller chooses), `PackWriter::from_pack(&pack)` from a copy of a pack's files. `add(&document)` writes a document at its default path (`blocks/<namespace>/<name>.json`) and returns the path, and refuses a path that is taken (`WriteError::DuplicatePath`); `put(path, &document)` and `put_bytes(path, bytes)` write at a path and replace what was there; `remove(&path)` takes a file away. `manifest.json` is not a file to put or remove: it is written from `manifest_mut()`, in the format the manifest was read in (a format-0 manifest is written as format 1).

The writer gives the pack three ways: `files()` (every file, `manifest.json` first, then in bytewise path order; a manifest its format cannot hold is left out, and `to_archive` and `into_pack` then fail), `to_archive()` with the `zip` feature (deflated zip bytes that are the same for the same files, on any host) and `into_pack(origin)` (a `Pack` with every file resident, to put on a stack). A writer reads and writes no file: `bedrock_addon_std::write_dir` and `write_archive` put the files on disk.

```rust
use bedrock_addon::families::block::component::LightEmission;
use bedrock_addon::families::block::{Block, Permutation, StateValues};
use bedrock_addon::id::BlockId;
use bedrock_addon::load::{LoadContext, LoadOptions};
use bedrock_addon::manifest::Manifest;
use bedrock_addon::pack::{Behavior, PackOrigin, PackStack, PackType, PackWriter, WriteError};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut lamp = Block::new(BlockId::from_static("demo:ruby_lamp"));
    lamp.description.states.insert("demo:lit".to_owned(), StateValues::bools());
    lamp.permutations
        .push(Permutation::when("q.block_state('demo:lit')").with(LightEmission(15)));
    // A value built in code is checked by the rules a loaded one is checked by.
    assert!(!lamp.validate(&LoadContext::detached()).has_errors());

    let mut writer = PackWriter::new(Manifest::new("Ruby lamps", PackType::Behavior));
    let path = writer.add(&lamp)?;
    assert_eq!(path.as_str(), "blocks/demo/ruby_lamp.json");
    assert!(matches!(writer.add(&lamp), Err(WriteError::DuplicatePath { .. })));
    let paths: Vec<&str> = writer.files().map(|(path, _)| path.as_str()).collect();
    assert_eq!(paths, ["manifest.json", "blocks/demo/ruby_lamp.json"]);

    // The written pack loads back, and the block is the one that was added.
    let stack = PackStack::<Behavior>::of([writer.into_pack(PackOrigin::World)?], LoadOptions::default())?;
    let loaded = stack.blocks().get(&BlockId::from_static("demo:ruby_lamp")).expect("loaded");
    assert_eq!(loaded.value(), &lamp);
    Ok(())
}
```

What `from_pack` does not copy as it is: the files a `__brarchive` archive serves become loose files and the archive is left out; a binary (`\x7FMCB`) document whose layout the crate decodes is copied as the JSON text it stands for, at the same path (any other binary document keeps its bytes, and a stack that loads the copy reports it as an unsupported file format, as a stack over the source does); `contents.json` and `signatures.json` of the pack root are left out, since they list and sign the bytes a writer is there to change; an encrypted pack is refused (`WriteError::Encrypted`), with or without its key, because the copy would be the pack in plain text. A file that is indexed and not loaded is `WriteError::Storage`: copy a pack that was loaded with `load::Residency::All`.

### What a round trip keeps

A value read from a file and written back (`Content::to_json`, `PackWriter::put`) is held to one of these levels. The level of each document type is checked over a stack that holds every family. Beside the levels, test probes hold every document type with a writer to this: a document that loads can be written, and where the written text differs from the file in a value (a number clamped, a member dropped, a type changed), the load has a finding at that place, or at the member around it. A second set of probes holds the upgrade of older documents to the same. It is what the probes hold and no more than that. They replace one value at a time in small documents and in a capped sample of the game's files; a float held as the nearest 32-bit float counts as held; and they carry lists of recorded exceptions. Among them: places where a value changes its form with no finding (`EXEMPT`: a filter subject given as a number is written by its name; a legacy trade quantity given as text is written as a range), and a document that loads and is not written (`TOLD_NOT_WRITTEN`: the item without a registered component, below). The lists are in `tests/writable/harness.rs` and `tests/writable_legacy/harness.rs`; the header of `tests/writable.rs` states the rule in full.

| Level | What it means | Document types |
| --- | --- | --- |
| Exact | the bytes are the same | `.mcstructure` files; a file no document type reads (an image, a script) is copied by a writer byte for byte |
| Value | the written file reads back as an equal value (but for the members that record the file, below), and with the same findings but those about members the load told and did not keep, which the written file no longer has. Unknown members are kept where the type keeps them, in the document's `Unknown` table beside its value (`Document::unknown`), and a document writes them after the known members of their objects (`Document::write_json`, `PackWriter::put` of the document; the value alone writes none); where a place keeps none the load tells the member (an unknown member of a legacy manifest's `metadata` is `MemberNotKept`; one inside a map value is `UnrecognizedField`, reported and not kept). Map order is kept, with two exceptions that the game reads as the same file: a bare key of `biomes_client.json` other than `default` is written with `minecraft:` and moves behind the other keys, and the members of a `.material` body are written in the reader's order. Comments, whitespace, a BOM, trailing commas and the spelling of escapes are not kept. A real negative zero (`-0.0`) keeps its sign in a member that holds a float. Where the member holds an integer the sign has nowhere to go, and what happens is the reader's: an entity's `hud_range` given `-0.0` is written `0` with no finding; a material's `stencilRef`, `stencilReadMask` and `stencilWriteMask` are written `0` and the load notes it with `ValueRewritten` (`samplerIndex` is read by the same function); a member read strictly as an integer takes integer tokens only, so a real token is a type finding. The integer token `-0` is the integer zero and is written `0` | blocks, items, the crafting catalogue, entities, biomes, client biomes and `biomes_client.json`, spawn rules and groups, features, feature rules, jigsaw structures with their pools, sets and processors, dimensions, recipes, loot tables, trade tables, dialogue, cameras, aim assist, voxel shapes, behavior trees, POI types and tags, `tick.json`, animations and controllers, geometry, render controllers, fog, materials, the deferred renderer's settings, particles, client entities, atomic client entities, attachables, block culling rules, texture lists and texture sets, the server's sound definitions, `languages.json` and `language_names.json`, data-driven UI, splashes, loading messages, credits, font metadata, emoticons, the side files of UI images |
| Value (raw) | as Value, and the JSON is held as written, so member order and number spelling are kept too | `blocks.json`, the terrain and item atlases, flipbooks, sound definitions, `sounds.json`, music definitions, JSON UI files, `_ui_defs.json`, `_global_variables.json` |
| Upgraded | a document of an older format version is written at the newest and reads back as the value the load gave, apart from the mark of a component the upgrade itself made (`synthesized`), which the written file has as its own. Its findings may differ: the written file is of the newest format and no longer has what the upgrade told of. `history::<Name>History::to_json` writes it at the version it was read at | the types with a version chain that the suite holds to it: blocks, items, entities, biomes, client biomes, features, dimensions, trade tables, geometry, camera presets, aim assist presets, texture sets, fog, client entities, attachables, particles, and, of the Value (raw) row, sound definitions |
| Lines | the line structure is kept; a last line without a line end is written with one | `.lang` files (comments, blank lines and order kept; CRLF written as LF); `.mcfunction` files (the command lines only: comments and blank lines are not written) |

What is outside those levels:

- **`\x7FMCB` documents**, the binary form of a JSON document, as a `.brarchive` entry or as a loose file. They are decoded into JSON text when a stack loads the file, for the shapes the game itself ships (items, particle effects, trade tables, voxel shapes, and the one camera document the game has, recognised whole); any other body is not decoded: a stack that loads the file reports it as an unsupported file format. The binary form is read only: the crate has no encoder for it. `from_pack` copies a document the decoder reads as that JSON text, and `put` of the loaded document writes it in its family's style.
- **Members that record the file**, which make a value read from a rewritten file compare unequal by `PartialEq` although nothing of the content changed. For any document: `BehaviorTree::raw` (the file's text, byte for byte) and `TextureList::skipped_non_strings`. For a document of an older format alone, which is written at the newest: `Block::source_format_version`, `Dimension::source_format_version`, `GeometryFile::format_version`, and the mark of a component the upgrade made.
- **Fog settings that break a rule of the newest format** (a file of an older format that the newest would refuse) are written in the older schema, at the oldest version that has their members; settings that fit no schema are not written (`EncodeError`).
- **A block of an older format that the newest format refuses** is written at its own format version when that text reads back as the same block, and is refused otherwise.
- **An item without a registered component** has no text as a component-based item (custom components alone, unknown keys alone): the load says so with a warning at `components`, and the writer refuses it. An item that holds nothing at all (`Item::new`) is written as the legacy document it is.
- **An item of an older format** carries every member of its legacy reading through both writers, written or told by a finding; written as it was read (`history::<Name>History::to_json`) it has the members its own format reads and the unknown members the file had, and none that only a later format has.
- **A client entity or an attachable** is written only as text its own loader reads: a description the schema refuses gives no text from either writer. An empty conditional object in `render_controllers` or `scripts.animate` is kept where the file has it, names nothing and is told with a warning; it is written as `{}` and reads back equal, with the same warning.
- **Encrypted packs** are not written, and **persona pieces** and **script metadata** are no content of a pack on a stack: a persona piece document is written with `put` like any other.

## Features

Every public item has one path, the module this table names (`bedrock_addon::families::block::Block`); no item is re-exported at the crate root.

| Feature | Contents | Implies |
| --- | --- | --- |
| (always) | `id`, `version`, `diagnostic`, `document`, `load`, `json`, `molang`, `visual`, `shared`, `vanilla`, `storage`, `manifest`, `pack` | |
| `zip` | in-memory decoding and encoding of `.mcpack` / `.zip` / `.mcaddon` / `.mctemplate` (`Pack::from_archive`, `Pack::to_archive`) | |
| `encrypted` | marketplace content keys: the encrypted-zip header and `contents.json` decryption, with keys the caller supplies | `zip` |
| `molang` | the load-time Molang compile check (`molang::MolangCheck`, `LoadOptions::molang`) and the compiler types it uses | |
| `nbt` | the client-network NBT methods (`Block::to_network_nbt`, `Item::network_properties`) | |
| `aim_assist`, `animation`, `animation_controller`, `atomic_client_entity`, `attachable`, `behavior_tree`, `biome`, `biomes_client`, `block`, `block_culling`, `blocks_json`, `camera`, `client_biome`, `client_entity`, `crafting_catalog`, `credits`, `ddui`, `deferred_settings`, `dialogue`, `dimension`, `entity`, `feature`, `feature_rules`, `flipbooks`, `fog`, `font`, `function`, `geometry`, `item`, `jigsaw`, `json_ui`, `lang`, `languages`, `legacy_path_mapping`, `loading_messages`, `loot`, `material`, `music`, `particle`, `persona`, `poi`, `recipe`, `render_controller`, `script`, `server_sound_definitions`, `skin`, `sound_definitions`, `sound_file`, `sounds_json`, `spawn_groups`, `spawn_rules`, `splashes`, `structure`, `texture_atlas`, `texture_file`, `texture_list`, `texture_set`, `trade`, `ui_texture_info`, `voxel_shape` | one family module each, with its stack accessors | |
| `full` | every content module | the 25 module features |
| `script-syntax` | a full JavaScript syntax check of script files; off by default and outside `full` | `script` |

## Examples

Seven examples ship with the two crates. Each runs with no argument and exits 0: the four of this crate on packs they hold as literals, reading no file, and the three of [`bedrock_addon_std`](../std) on packs they write into a temporary directory.

| Example | Crate | What it covers | Features |
| --- | --- | --- | --- |
| [`in_memory_pack`](examples/in_memory_pack.rs) | `bedrock_addon` | open a pack from bytes, build a `PackStack<Behavior>`, look up a block | `block` |
| [`entity_components`](examples/entity_components.rs) | `bedrock_addon` | an entity's components: typed access, and the JSON of a component kept as written | `entity` |
| [`client_visuals`](examples/client_visuals.rs) | `bedrock_addon` | client entities and their geometry and texture references through `HasVisuals` | `client_entity`, `entity`, `geometry`, `texture_file`, `texture_set` |
| [`diagnostics_gate`](examples/diagnostics_gate.rs) | `bedrock_addon` | a CI gate over `Severity` and the findings' codes: `load_all()`, `link()`, two errors (one the load's, one the link pass's) and one lint, and `LinkOptions { lints: false, ..Default::default() }` | `block`, `entity`, `loot` |
| [`load_stacks`](../std/examples/load_stacks.rs) | `bedrock_addon_std` | the `Driver` loads a world's stacks, then the custom blocks are registered with their Molang permutation conditions compiled | |
| [`rayon_driver`](../std/examples/rayon_driver.rs) | `bedrock_addon_std` | the driver with its default rayon strategy, in three lines, and the same load in order | `rayon` |
| [`ruby_pack`](../std/examples/ruby_pack.rs) | `bedrock_addon_std` | build a block in code, `PackWriter::add`, `write_dir`, and load the directory back | |

```sh
cargo run -p bedrock_addon --example in_memory_pack
cargo run -p bedrock_addon_std --example ruby_pack
```

## Tests

`cargo nextest run -p bedrock_addon --all-features` runs the unit tests and the test binaries under `tests/`, most of them one family's. The binaries that check the crate as a whole:

| Binary (`--test <name>`) | What it holds |
| --- | --- |
| `link` | the link pass: its frame and one module per rule group |
| `writer` | `PackWriter`: its files in order, its zip bytes, the pack it opens as |
| `writer_round_trip` | the round-trip level of every document type, through `PackWriter`, over a stack that holds every family |
| `writable`, `writable_2`, `writable_3`, `writable_4` | every document that loads is writable, and no value is rewritten or dropped without a finding. One probe in four binaries, since a test binary runs under a limit of CPU seconds: `writable_2`, `writable_3` and `writable_4` share out the twelve document types with the most texts, and a test of `writable` holds the four lists against each other |
| `writable_legacy` and the `writable_legacy_*` binaries | the same for documents of older formats: no upgrade step and no legacy reader loses an author's value in silence |
| `determinism` | a load gives the same result for every order in which the caller completes requests and jobs |
| `module_independence` | no content module imports another (a family imports only the core vocabulary, the derive and the shared crates; the three biome families also support's `biome_json`) |

The tests that read the game's own files run only with `BEDROCK_ADDON_CORPUS` set to a folder holding them, and skip without it; nothing of the game is in the repository.

The game's own packs have a gate of their own: the `vanilla_gate` test of `bedrock_addon_std` (`cargo nextest run -p bedrock_addon_std --test vanilla_gate --no-capture`). It reads the variable `BEDROCK_ADDON_VANILLA_1_26_60`, the root of a game install (the folder holding `behavior_packs/`, `resource_packs/`, `definitions/` and `test/`, every pack with its `manifest.json`). It loads every pack of both folders on stacks composed as the game composes them, with the install's app data, every experiment on and every Molang field compiled, links them, and requires no error and no warning, apart from the 14 pins of its `EXPECTED` list: the two vanilla loot tables `armor_stand.json` and `boat.json` (`MissingRequiredField`, each with the game's own content-log error) and twelve `TextureNotFound` warnings of the link pass. Informations are counted and printed, never asserted. A pin that does not occur exactly as often as pinned also fails the gate. Findings are printed per pack; the packs below `test/` are loaded alone and reported, never asserted. Without the variable the tests print `skipped:` and pass, which is no pass of the gate. `crates/addon/addon/tests/pack_vanilla.rs` reads the same variable for the experimental-pack rows that ship only in the newest game version.

The generator of the script-module metadata, of the vanilla identifier lists and of the command table is `cargo xtask script-meta --samples <dir>`, `<dir>` being a checkout of Mojang's `bedrock-samples` repository at a release tag; its tests run under `cargo nextest run -p xtask`.

## Contributing

See [CONTRIBUTING.md](../../../CONTRIBUTING.md).

## License

Licensed under the **Apache License 2.0**; see [LICENSE](../../../LICENSE).
