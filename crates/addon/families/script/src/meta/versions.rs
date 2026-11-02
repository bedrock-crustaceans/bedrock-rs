//! The module catalog: every script module and version the metadata release ships, compiled
//! in from the generated table. Packs name these modules in their
//! manifest's dependencies; which version a request selects is `script::ScriptModule`'s rule.

use uuid::Uuid;

use super::generated::versions::{ENGINE_ONLY, MODULES, SAMPLES_TAG};
use bedrock_addon_core::version::SemVersion;

/// One script module of the catalog: a native module or a stub, with every version shipped.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct ModuleEntry {
    /// The module name (`@minecraft/server`), from the file names and `name`.
    pub name: &'static str,
    /// `uuid`, as a number (case does not matter in the files).
    pub uuid: u128,
    /// Native module or stub.
    pub shape: ModuleShape,
    /// The shipped versions in version order: releases, then the one beta.
    pub versions: &'static [ModuleVersion],
}

/// Whether a module declares its API or stands for a bindings module.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ModuleShape {
    /// A native module: its files have no `parentModule`.
    Native,
    /// A stub: its files name `parent` (the bindings module) as `parentModule`.
    Stub {
        /// The bindings module.
        parent: &'static str,
    },
}

/// The release channel of a shipped version.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Channel {
    /// A version without a pre-release tag.
    Stable,
    /// A `-beta` version; at most one per module, the newest.
    Beta,
}

/// One shipped version of a module.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct ModuleVersion {
    /// Major.
    pub major: u16,
    /// Minor.
    pub minor: u16,
    /// Patch.
    pub patch: u16,
    /// The pre-release tag of the file name (`beta`), if any.
    pub pre: Option<&'static str>,
    /// The release channel.
    pub channel: Channel,
    /// A stub's `wrapper_module`; `None` for a native module.
    pub wrapper: Option<&'static str>,
    /// For a stub, the index of its parent's same version in the parent's
    /// [`versions`](ModuleEntry::versions); `None` for a native module.
    pub parent_version: Option<u16>,
    /// The peer dependencies in effect: a native version's own; a stub version's parent's,
    /// with the stub's own entries replacing those of the same module.
    pub peers: &'static [PeerReq],
}

/// A peer dependency of a shipped version: a public module and the versions accepted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct PeerReq {
    /// The public module (`@minecraft/common`).
    pub module: &'static str,
    /// The accepted versions, in file order.
    pub versions: &'static [&'static str],
    /// An npm type package (`@minecraft/vanilla-data`): kept in the table, ignored by
    /// dependency resolution.
    pub types_only: bool,
    /// `wrapperName` of a stub's peer.
    pub wrapper_name: Option<&'static str>,
}

/// Every module of the catalog, sorted by name.
pub fn modules() -> &'static [ModuleEntry] {
    MODULES
}

/// The release tag of the metadata the catalog was generated from (`v1.26.50.4`): a pack
/// written for a newer game may name a version this tag does not ship.
pub fn samples_tag() -> &'static str {
    SAMPLES_TAG
}

/// Versions the engine knows but the metadata release ships no file for (`@minecraft/server` `3.0.0-alpha`,
/// `2.11.0-internal`), as (module, version): never selectable by packs.
pub fn engine_only() -> &'static [(&'static str, &'static str)] {
    ENGINE_ONLY
}

/// The module called `name` (`@minecraft/server`).
pub fn module(name: &str) -> Option<&'static ModuleEntry> {
    MODULES
        .binary_search_by(|entry| entry.name.cmp(name))
        .ok()
        .map(|index| &MODULES[index])
}

/// The module whose UUID is `uuid`.
pub fn module_by_uuid(uuid: Uuid) -> Option<&'static ModuleEntry> {
    MODULES.iter().find(|entry| entry.uuid == uuid.as_u128())
}

impl From<&ModuleVersion> for SemVersion {
    fn from(version: &ModuleVersion) -> Self {
        let mut sem = SemVersion::new(version.major, version.minor, version.patch);
        if let Some(pre) = version.pre {
            pre.clone_into(&mut sem.pre_release);
        }
        sem
    }
}
