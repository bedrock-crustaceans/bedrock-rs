//! The shared vocabulary of the `bedrock_addon` crate family: identifiers, versions,
//! diagnostics, documents, the load context, JSON helpers, the Molang re-exports and the
//! version-chain machinery.
//!
//! No I/O: nothing here reads a file, starts a thread or asks the clock. Users reach these
//! types through `bedrock_addon`, which re-exports the public modules below at one documented
//! path each; this crate exists so that `bedrock_addon` and its proc-macro share one set of
//! types. Depend on `bedrock_addon`.
//!
//! # One path per item
//!
//! A public item of this crate has its path in the module it lives in (the `molang` module
//! also re-exports the `molang` crate, whose items so have a second path). No type or
//! function is re-exported at the crate root (the hidden macros `version_chain!` and
//! `impl_no_molang!` are exported there, as `#[macro_export]` places them). Modules, with the feature a part of one needs:
//!
//! - `id`: typed identifiers (`Id<K>`, `Kind`, the `kind` markers and one alias per kind).
//! - `version`: pack versions (`SemVersion`) and document versions (`FormatVersion`), the
//!   engine version and the manifest version types.
//! - `diagnostic`: findings as data (`Diagnostic`, `Diagnostics`, `DiagnosticCode`,
//!   `Severity`, `Phase`, `PackReport`).
//! - `document`: a loaded value with its provenance (`Document<T>`), the collections
//!   (`Registry`, `Files`, `Merged`), components and the `Content` / `Definition` traits.
//! - `load`: what loading depends on (`LoadOptions`, `LoadContext`, `BuiltIns`, experiments,
//!   limits and the virtual locations loaders speak in).
//! - `json`: the public part of the facet helpers (`Value`, the scalar shape and its error,
//!   `MAX_JSON_DEPTH`, the envelope of a versioned document, `WriteStyle`, `EncodeError`);
//!   the other helpers `bedrock_addon` reads and writes with are named from here and not
//!   listed on the module's page.
//! - `molang`: the molang types pack content holds; with `molang`, the compile check.
//! - `shared`: value types several content modules use (descriptors, ranges, filters).
//! - `vanilla`: the game's tables (identifier lists, commands, name lists, level sound
//!   events) re-exported from `bedrock_addon_vanilla`, `GeneratedBuiltIns` over them, and
//!   `ActorType`.
//! - `pack_identity`: the pack-identity vocabulary (`PackPath`, `PackRef`, `PackIdVersion`,
//!   `PackType`, `PackSide`, `PackOrigin`); `bedrock_addon::storage` and `bedrock_addon::pack`
//!   are its documented paths.
//! - `schema`: the `schema::` attribute grammar (`bedrock_addon_schema`, re-exported) that a
//!   typed document struct states its per-field facts in; hidden at the facade.
//! - `versioning`: the version-chain machinery (`Family`, `Upgrade`, `version_chain!`);
//!   hidden at the facade.
//!
//! # Features
//!
//! - `molang`: the load-time Molang compile check (`molang::MolangCheck`), which enables
//!   molang's `compiler`, `stdlib` and `cache`.
//!
//! # The stack of a JSON read
//!
//! A typed value is read from JSON, and written to it, by a call for each level of nesting.
//! A text may nest 128 levels deep ([`json::MAX_JSON_DEPTH`]); the readers and writers of `json`
//! run such a walk on the calling thread's stack where enough of it is left, and on a stack
//! made for the walk, on the same thread, where it is not. So a document within the limit
//! never overflows a thread with one megabyte of stack, and a deeper one is refused with an
//! error. The stack is made with the `stacker` crate, a dependency on every target but
//! WebAssembly; a build script tells the crate whether it is built without optimisation,
//! where a level takes about eight times the stack. Three cases are outside the rule:
//!
//! - WebAssembly. A module has the one stack it was linked with and none can be made: the
//!   embedder sizes it for the typed reads (up to 11 KiB a level in an optimised build, 87
//!   KiB in an unoptimised one).
//! - A target `stacker` has no stack switch for: the walk runs where it is, as on
//!   WebAssembly, and nothing says so.
//! - A host that cannot map the stack (an address-space limit such as `RLIMIT_AS`, strict
//!   overcommit, a full 32-bit address space): `stacker` panics, so a typed read or a write
//!   panics where it would have returned. The stack for a text at the limit is 65 MiB of
//!   address space; its pages are taken as they are touched.
//!
//! # Examples
//!
//! The crate has no example program of its own: the README of `bedrock_addon` has the
//! compiled quick start and the examples table, and the examples there name these types by
//! their `bedrock_addon` path. The doctests of this crate name its own paths.

// Names such as `FormatVersion` in `version` stay descriptive.
#![cfg_attr(docsrs, feature(doc_cfg))]

/// Every public item has its path in its module; none resolves at the crate root.
///
/// ```compile_fail,E0432
/// use bedrock_addon_core::Id;
/// ```
/// ```compile_fail,E0432
/// use bedrock_addon_core::Diagnostic;
/// ```
/// ```compile_fail,E0432
/// use bedrock_addon_core::PackPath;
/// ```
#[cfg(doctest)]
struct NoItemAtTheRoot;

/// Compiles the README's code blocks as doctests.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;

// The paths the derives emit, `::bedrock_addon_core::schema::…`, resolve inside this crate too.
extern crate self as bedrock_addon_core;

#[doc(hidden)]
pub mod __private;
pub mod diagnostic;
pub mod document;
pub mod id;
pub mod json;
pub mod load;
pub mod manifest;
pub mod molang;
pub mod pack_identity;
pub mod schema;
pub mod shared;
#[doc(hidden)]
pub mod test_cost;
pub mod vanilla;
pub mod version;
pub mod versioning;
