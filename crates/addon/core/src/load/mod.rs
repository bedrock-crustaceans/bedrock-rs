//! What loading depends on: the world-level options, the per-pack load context, experiments and
//! load requirements, the built-in identifier set, app data, and the virtual locations and
//! limits the loaders in `bedrock_addon::load` speak in.

use std::fmt;
use std::sync::Arc;

use crate::version::BaseGameVersion;

mod app_data;
mod collection;
mod constructors;
mod context;
mod experiment;
mod limits;
mod place;

pub use app_data::AppData;
pub use collection::ListedFile;
pub use collection::NamedFiles;
#[doc(hidden)]
pub use collection::{
    Accept, AppDataPlace, CollectionLoader, Registered, Registration, Reported, StackCx, StackPack,
    Walk, WalkFile, WalkOrder, unread_files,
};
#[doc(hidden)]
pub use constructors::{FileCopy, files, merged, registry};
pub use context::{BiomeRenderIdentifiers, DocumentSource, LoadContext};
pub use experiment::{Experiment, Experiments, LoadRequirement, SaveName};
pub use limits::{
    JSON_DOCUMENT_WARNING_BYTES, JsonDocumentSize, Limits, MAX_JSON_DOCUMENT_BYTES,
    json_document_size, json_document_size_against,
};
pub use place::{ListDepth, Place, ReadRange, RootId};

/// Run jobs on threads with at least this much stack: the driver's own pool has it.
///
/// No document can overflow a job's stack by its nesting. JSON nesting is capped at 128
/// levels ([`crate::json::MAX_JSON_DEPTH`]; a deeper file is rejected with a finding), the reads
/// and writes of typed values that go a call deeper for each level run on a stack of their own
/// where the thread is short of one. A world whose files nest to the limit loads on a
/// thread with one megabyte of stack, in a debug build too
/// (`bedrock_addon_std/tests/deep_documents.rs`, `small_stack.rs`). The eight are the margin
/// for what does not grow with a file: the frames of the loaders themselves and the Molang
/// compiler, which needs up to 384 KiB unoptimised. Sequential callers on the main thread
/// (8 MiB on Linux) are fine, and so is a spawned thread at its default of 2 MiB.
///
/// On WebAssembly a module has one stack, sized when it is linked, and no reader can make
/// another: size it for the typed reads ([`crate::json::MAX_JSON_DEPTH`]).
pub const RECOMMENDED_JOB_STACK: usize = 8 * 1024 * 1024;

/// World-level inputs every document load depends on (vanilla's level settings and toggles).
/// Fixed for the lifetime of a stack; build a new stack to change them.
/// Built as `LoadOptions { beta_apis: true, ..LoadOptions::default() }`.
#[derive(Clone, Default)]
#[allow(clippy::struct_excessive_bools)] // the world's toggles, one field each
pub struct LoadOptions {
    /// The world's enabled experiments.
    pub experiments: Experiments,
    /// The Beta APIs toggle: allows `"format_version": "beta"` documents.
    pub beta_apis: bool,
    /// The Upcoming Creator Features toggle.
    pub upcoming_creator_features: bool,
    /// The world's base-game version.
    pub base_game_version: BaseGameVersion,
    /// The world's education features: the
    /// composer appends the app-bundled chemistry behaviour packs to the world's stack.
    pub education: bool,
    /// Keep shadowed definitions for `Registry::layers` (default `false`: most callers do not need
    /// them).
    pub keep_shadowed: bool,
    /// Built-in identifiers that count as defined without a document.
    pub built_ins: Option<Arc<dyn BuiltIns>>,
    /// App data outside packs: `definitions/attachables`, `features`, `feature_rules`,
    /// `spawn_groups`. In-memory files; `bedrock_addon_std::read_app_data` reads the directory.
    pub app_data: Option<AppData>,
    /// Shared, so `LoadOptions` stays `Clone` and one compile cache serves both stacks.
    #[cfg(feature = "molang")]
    #[cfg_attr(docsrs, doc(cfg(feature = "molang")))]
    pub molang: Option<Arc<crate::molang::MolangCheck>>,
}

impl fmt::Debug for LoadOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let present = |present: bool| if present { "present" } else { "absent" };
        let mut debug = f.debug_struct("LoadOptions");
        debug
            .field("experiments", &self.experiments)
            .field("beta_apis", &self.beta_apis)
            .field("upcoming_creator_features", &self.upcoming_creator_features)
            .field("base_game_version", &self.base_game_version)
            .field("education", &self.education)
            .field("keep_shadowed", &self.keep_shadowed)
            .field("built_ins", &present(self.built_ins.is_some()))
            .field("app_data", &self.app_data);
        #[cfg(feature = "molang")]
        debug.field("molang", &present(self.molang.is_some()));
        debug.finish()
    }
}

/// What the game has without a document: identifiers, and the values of its command and
/// enchantment tables. Chorus passes its vanilla palette.
///
/// A host's provider is believed, yes or no, for what it answers. The value lookups default
/// to the generated tables ([`GeneratedBuiltIns`](crate::vanilla::GeneratedBuiltIns)), so a
/// host that answers only [`contains`](Self::contains) keeps the game's commands; `None` from
/// a lookup the host implements means the game has no such value.
///
/// What the link pass asks a provider is not all it answers: it answers the code-registered
/// features from addon's `CODE_FEATURES` (`addon/src/pack/link/lookup.rs`), so
/// `contains("feature", ..)` of the generated tables is `false`, and it checks the legacy
/// block and item names and the fileless entities itself before asking the host
/// (`lookup.rs:722-728`). A host's [`command`](Self::command) is asked with the word as
/// written; the generated tables fold ASCII case, as function's `find_command` does.
pub trait BuiltIns: Send + Sync {
    /// Whether `id` of kind `kind` (`Kind::NAME`) is built in.
    fn contains(&self, kind: &'static str, id: &str) -> bool;
    /// The command typed as `name`: its name or one of its aliases, without `/`. The default
    /// compares without ASCII case.
    fn command(&self, name: &str) -> Option<&crate::vanilla::Command> {
        crate::vanilla::GeneratedBuiltIns.command(name)
    }
    /// The command enum called `name`, compared without ASCII case.
    fn command_enum(&self, name: &str) -> Option<&crate::vanilla::CommandEnum> {
        crate::vanilla::GeneratedBuiltIns.command_enum(name)
    }
    /// The enchantment an alias names (`minecraft:bow_infinity` → `minecraft:infinity`).
    fn enchantment_alias(&self, alias: &str) -> Option<&str> {
        crate::vanilla::GeneratedBuiltIns.enchantment_alias(alias)
    }
}
