//! Minecraft Bedrock Edition behaviour and resource packs as typed Rust data: manifests,
//! containers, pack repositories and stacks, every content family the game reads from a pack,
//! diagnostics shaped like the content log, a link pass over the references between definitions,
//! and write-back.
//!
//! # Sans-IO
//!
//! A function belongs here only if its result depends on nothing but its arguments and it
//! chooses neither where bytes come from nor which thread runs it: the crate reads no file,
//! starts no thread and asks no clock. A pack is opened from bytes the caller has
//! ([`pack::Pack::from_files`]), or by a loader that asks for bytes through a [`load::Output`]
//! and takes the answer as a [`load::Input`]; CPU work is a [`load::Job`] the caller runs.
//! `bedrock_addon_std` is the driver that answers the loaders with `std::fs`, runs the jobs
//! on the calling thread or a rayon pool, and writes packs to disk.
//!
//! A host that runs jobs on threads of its own needs no stack that grows with a file. JSON
//! nesting is capped at 128 levels ([`json::MAX_JSON_DEPTH`]; a deeper file is refused with a
//! finding), and a read or a write that goes a call deeper for each level runs on a stack
//! made for it, on the same thread, where the thread is short of one: a document within the
//! limit never overflows a thread with one megabyte of stack, in a debug build too.
//! [`load::RECOMMENDED_JOB_STACK`] is the margin for what does not grow with a file. Three
//! cases are outside the rule. On WebAssembly no stack can be made: a module has the one it
//! was linked with, and the embedder sizes it for the typed reads (up to 11 KiB a level in an
//! optimised build, 87 KiB in an unoptimised one). On a target without a stack switch the
//! read runs where it is, as on WebAssembly. And where the host cannot map the stack (an
//! address-space limit, strict overcommit) the read panics; the stack for a text at the limit
//! is 65 MiB of address space.
//!
//! # From a pack to findings
//!
//! - [`pack::Pack`] is one opened pack; [`pack::PackRepository`] every pack a host knows;
//!   [`pack::PackRepository::stacks`] composes the two stacks of a world, a
//!   [`pack::StackSet`] of a [`pack::PackStack`] per side.
//! - An accessor of a stack (`blocks()`, `entities()`, …) parses its family on first call and
//!   caches the result; [`pack::StackSet::load_all`] parses everything. A content file does
//!   not make loading fail: a problem is a [`diagnostic::Diagnostic`] in the report, and a
//!   document of a family with a version history whose own load raises an error is in
//!   `rejected()` of its `Registry` or `Files` (texture sets: `rejected_sets()`). `blocks.json`,
//!   which is no such family, keeps its file with an error at an entry it refuses; only a
//!   file that is no JSON object is rejected.
//! - [`pack::StackSet::link`] resolves the references between definitions, of two families or
//!   of one, and returns its findings; it changes no collection. Its informations are left
//!   out with [`pack::LinkOptions`]. A host answers for the game's own content through
//!   [`load::LoadOptions::built_ins`]; it is believed, yes or no, but some names stand
//!   whatever a host says (the item names of older versions, for example).
//! - [`pack::PackWriter`] builds a pack in memory, from a manifest or from a copy of a pack,
//!   and gives its files, its zip bytes or a `Pack` for a stack.
//!
//! # One path per item
//!
//! A public item has its path in the module it lives in (`molang` also re-exports the
//! `molang` crate, whose items so have a second path). Nothing is re-exported at the
//! crate root. Modules, with the feature a module needs:
//!
//! - `id`: typed identifiers (`Id<K>` and one alias per kind).
//! - `version`: pack versions, document format versions and the engine version.
//! - `diagnostic`: findings as data (`Diagnostic`, `Diagnostics`, `DiagnosticCode`, `Severity`).
//! - `document`: a loaded value with its provenance (`Document<T>`) and the collections
//!   (`Registry`, `Files`, `Merged`).
//! - `json`: the untyped JSON value (`Value`), the strict scalar wrappers many component
//!   members are made of (`StrictInt`, `StrictFloat`, `StrictBool`, `StrictString`), the
//!   nesting limit (`MAX_JSON_DEPTH`), the write style and the encode error.
//! - `molang`: the Molang types pack content holds; with `molang`, the compile check.
//! - `visual`: the description slots client entities and attachables share.
//! - `shared`: value types several content modules use (descriptors, ranges, filters).
//! - `vanilla`: generated identifier lists of the game, its command table and `ActorType`.
//! - `load`: the loaders (`PackLoader`, `RepositoryLoader`, `StackLoader`), their inputs,
//!   outputs and jobs, and what loading depends on (`LoadOptions`, `LoadContext`, `BuiltIns`).
//! - `storage`: pack paths, storage errors and content keys.
//! - `manifest`: `Manifest` and its parts.
//! - `pack`: `Pack`, `PackRepository`, `PackStack<S>`, `StackSet`, `StackReport`,
//!   `LinkOptions`, `PackWriter`, and the saved pack settings (`pack::settings`).
//! - `families`: one module per game document family (`families::block`,
//!   `families::attachable`, `families::fog`, …), each behind the feature of its name and each
//!   described on its own page. A family imports the core vocabulary and nothing else of the
//!   crate; what two families share that is no vocabulary lives in the crate `bedrock_addon_support`
//!   (`support`).
//!
//! The first nine are the shared vocabulary, defined in `bedrock_addon_core` and documented
//! here.
//!
//! # Features
//!
//! All on by default except `script-syntax`:
//!
//! - `zip`: in-memory decoding and encoding of `.mcpack` / `.zip` / `.mcaddon` / `.mctemplate`
//!   (`Pack::from_archive`, `Pack::to_archive`, `PackWriter::to_archive`).
//! - `encrypted`: marketplace content keys, with keys supplied by the caller (implies `zip`).
//! - `molang`: the load-time Molang compile check.
//! - `nbt`: the client-network NBT methods (`Block::to_network_nbt`, …).
//! - one feature per family (`block`, `attachable`, …, `script`) and `full` for all of them.
//!   A link rule between two families runs only with both features on.
//! - `script-syntax`: a full JavaScript syntax check of script files (implies `script`; outside
//!   `full`).
//!
//! # Examples
//!
//! The README has a quick start and one compiled block per topic (packs and stacks,
//! diagnostics, versions, linking, writing), and its examples table lists the runnable
//! examples under `examples/`: `in_memory_pack`, `entity_components`, `client_visuals` and
//! `diagnostics_gate` here, `load_stacks`, `rayon_driver` and `ruby_pack` in
//! `bedrock_addon_std`.

// `#[must_use]` goes on methods returning a changed copy and on values whose loss is a bug, not
// on every pure getter.
// The error enums' variants document each failure.
// A panic here is a broken invariant of the crate, never an outcome a caller handles, so no
// function carries a `# Panics` section.
// Names such as `PackPath` in `pack` stay descriptive.
#![cfg_attr(docsrs, feature(doc_cfg))]
// The entity component enum carries 145 `#[component]` rows on its `Dynamic` variant, one
// nested expansion each.
#![recursion_limit = "512"]

/// Every public item has its path in its module; none resolves at the crate root.
///
/// ```compile_fail,E0432
/// use bedrock_addon::Block;
/// ```
/// ```compile_fail,E0432
/// use bedrock_addon::Pack;
/// ```
/// ```compile_fail,E0432
/// use bedrock_addon::Diagnostic;
/// ```
#[cfg(doctest)]
struct NoItemAtTheRoot;

/// Compiles the README's code blocks as doctests; they read blocks, so they need `block`.
#[cfg(all(doctest, feature = "block"))]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;

// The shared vocabulary, defined in core and documented at these paths.
#[doc(inline)]
pub use bedrock_addon_core::diagnostic;
#[doc(inline)]
pub use bedrock_addon_core::document;
#[doc(inline)]
pub use bedrock_addon_core::id;
#[doc(inline)]
pub use bedrock_addon_core::json;
#[doc(inline)]
pub use bedrock_addon_core::manifest;
#[doc(inline)]
pub use bedrock_addon_core::molang;
#[doc(inline)]
pub use bedrock_addon_core::shared;
#[doc(inline)]
pub use bedrock_addon_core::vanilla;
#[doc(inline)]
pub use bedrock_addon_core::version;
#[doc(hidden)]
pub use bedrock_addon_core::versioning;
#[doc(inline)]
pub use bedrock_addon_core::visual;

pub mod load;
pub mod pack;
pub mod storage;
pub(crate) use bedrock_addon_support as support;
#[cfg(test)]
mod test_cost;

/// One module per game document family, each behind the feature of its name.
pub mod families;
