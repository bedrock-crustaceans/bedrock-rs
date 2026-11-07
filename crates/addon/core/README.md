# bedrock_addon_core

The shared vocabulary of the `bedrock_addon` crate family: typed identifiers, pack and document versions, diagnostics, the document and collection types, the load context, thin JSON helpers over facet, the Molang re-exports and the version-chain machinery. It performs no I/O. Part of [bedrock-rs](https://github.com/bedrock-crustaceans/bedrock-rs).

**This crate is an implementation detail: depend on [`bedrock_addon`](../addon).** `bedrock_addon` re-exports the public modules of this crate at one documented path each (`bedrock_addon::id`, `bedrock_addon::diagnostic`, …; the table below has the two exceptions), and that path is the one its documentation, its README and its examples use. The crate is separate only so that `bedrock_addon` and its proc-macro, [`bedrock_addon_derive`](../derive), name one set of types.

## Installation

The crate uses edition 2024 and needs Rust 1.98 or newer.

```toml
[dependencies]
bedrock_addon = { git = "https://github.com/bedrock-crustaceans/bedrock-rs.git" }
```

## Modules

No type or function is re-exported at the crate root. (One macro is there, hidden: `version_chain!`, which `#[macro_export]` places at the root.)

| Module | Contents | Path in `bedrock_addon` |
| --- | --- | --- |
| `id` | typed identifiers: `Id<K>`, the `Kind` markers and one alias per kind (`BlockId`, `EntityId`, …) | `id` |
| `version` | pack versions (`SemVersion`), document versions (`FormatVersion`), the engine version (`ENGINE_VERSION`, 1.26.60) | `version` |
| `diagnostic` | findings as data: `Diagnostic`, `Diagnostics`, `Code` and `CodeDef` (the definitions of `codes`), `Kind`, `Severity`, `PackReport`; `JsonPath`, whose display writes a key as an RFC 6901 token and whose accessors (`of_keys`, `last_key`, `is_key`, `starts_with`, `ends_with`, `index_after`, `strip_prefix`, `segments`) are what code reads; `DiagnosticDetail` (`Reference`, `TriedPaths`, `Note`, and `Molang` with the feature) | `diagnostic` |
| `document` | a loaded value with its provenance (`Document<T>`), the collections (`Registry`, `Files`, `Merged`; `Registry::all_layers` and `Files::all_layers` list every kept copy), `ComponentMap` | `document` |
| `load` | what loading depends on: `LoadOptions`, `LoadContext`, experiments, limits and the virtual locations loaders speak in | `load`, beside the loaders |
| `json` | `Value`; the strict scalar wrappers (`StrictInt`, `StrictFloat`, `StrictBool`, `StrictString`) with the sealed traits of their number types, the scalar shape they are read through (`ScalarShape`) and their error (`StrictError`, `JsonType`); `MAX_JSON_DEPTH`; the envelope of a versioned document (`RootProbe`, `EnvelopeVersion`, `probe_envelope`, whose signatures name the hidden `JsonSlice` and `SliceError`); `WriteStyle`, `EncodeError`. Hidden from the docs and outside SemVer, for `bedrock_addon`'s own readers: the other parse and write helpers, among them `JsonSlice`, `parse_at`, `to_facet_json_signed` (a value's text with a real negative zero written `-0.0`) and `too_deep_to_walk` (whether a value built by hand nests past the limit) | `json` |
| `molang` | the molang types pack content holds (`Molang`, `MolangSource`, `MolangVersion`); with the `molang` feature, the compile check | `molang` |
| `shared` | value types several content modules use (descriptors, ranges, filters) | `shared` |
| `vanilla` | the game's tables of the `bedrock_addon_vanilla` data crate (identifier lists, commands, name lists, level sound events), `GeneratedBuiltIns` over them, and `ActorType` | `vanilla` |
| `pack_identity` | `PackPath`, `PackRef`, `PackIdVersion`, `PackType`, `PackSide`, `PackOrigin` | `storage` and `pack` |
| `versioning` | the version-chain machinery behind every family's `history`: `Family`, `Upgrade`, `version_chain!` | hidden: not public API of `bedrock_addon` |

## Features

| Feature | Contents | Implies |
| --- | --- | --- |
| `molang` | the load-time Molang compile check (`molang::MolangCheck`), over molang's `compiler`, `stdlib` and `cache` | |

## Dependencies and the build script

The crate reads a typed value from JSON by a call per level of nesting, and gives that read the stack it needs on the thread that reads: on every target but WebAssembly it depends on [`stacker`](https://crates.io/crates/stacker) for that, called in one file (`src/json/stack.rs`). A document within the nesting limit (`json::MAX_JSON_DEPTH`, 128 levels) is therefore read on a thread with one megabyte of stack, in a debug build too. Two limits come with the dependency:

- where the host cannot map the extra stack (an address-space limit such as `RLIMIT_AS`, strict overcommit, a full 32-bit address space), `stacker` panics, so a typed read or a write panics where it would have returned. The stack for a text at the limit is 65 MiB of address space; its pages are taken as they are touched;
- on a target `stacker` has no stack switch for, the read runs on the stack there is and nothing says so.

On WebAssembly the dependency is not built: a module has the one stack it was linked with, and the embedder sizes it for the typed reads (up to 11 KiB a level in an optimised build, 87 KiB in an unoptimised one).

`build.rs` reads the profile's `opt-level` and sets the cfg `unoptimised` for a build without optimisation, where a level of a typed read takes about eight times the stack; the crate decides by it when a read may run on the stack there is. It reads nothing else and writes no file.

## License

Licensed under the **Apache License 2.0**; see [LICENSE](../../../LICENSE).
