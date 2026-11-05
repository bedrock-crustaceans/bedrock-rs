//! Scripting: `ScriptModule`, `ScriptBundle`.
//!
//! The readers stay hand walks: a module is JavaScript, and the server's config files are read from
//! their slices with the server's own texts. Only the game metadata (`meta`) is a facet read.

#![cfg_attr(docsrs, feature(doc_cfg))]

/// Dedicated server script configuration (`ScriptConfig`, `ScriptConfigFiles`, `ConfigFolder`, `PackScriptConfig`, `CommandsConfig`, `ServerProperties`, `ScriptProperties`).
pub mod config;
/// Script module metadata (`ModuleMetadata`, `ModuleVersion`, `Privilege`, …).
pub mod meta;
mod module;
pub use module::ScriptModuleRef;
#[doc(hidden)]
pub use module::native_module_of;
#[doc(hidden)]
pub use module::starts_a_plugin;
pub use module::{
    CatalogModule, CatalogVersion, Discovery, ModuleCatalog, NativeDependency, ResolvedDependency,
    ScriptContext, ScriptDiscovery, ScriptFiles, ScriptModule, ScriptPlugin, ScriptRuntime,
    ScriptTarget, ScriptVersion, SkippedModule, UuidDependency,
};

mod bundle;

#[doc(hidden)]
pub use bundle::LOADER;
pub use bundle::{
    EngineCrash, EntryRef, ImportGraph, ImportKind, ImportSite, InvalidUtf8, NativeModuleAccess,
    Resolution, ScanOptions, ScriptBundle, ScriptContent, ScriptFile, ScriptHost, ScriptPath,
    ScriptPosition, ScriptRoot, ScriptSyntax, ScriptSyntaxKind, normalize,
};
