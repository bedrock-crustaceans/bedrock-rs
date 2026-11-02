//! Functions: `Function`, `TickList`.
//!
//! Functions (`functions/**/*.mcfunction` of a behaviour pack) and the tick list
//! (`functions/tick.json`). Neither file has a `format_version`: there is one model, and the only
//! version input is the pack's `min_engine_version`, mapped to a command version (8 to 46) that
//! every function and tick entry carries. Command lines are kept raw; parsing them is the command
//! system's (chorus).

#![cfg_attr(docsrs, feature(doc_cfg))]

mod grammar;
mod lint;
mod load;
mod new;
mod parse;
mod texts;
mod write;

#[cfg(test)]
mod tests;

use bedrock_addon_core::document::Extra;
use bedrock_addon_core::id::FunctionPath;

pub use grammar::{GrammarRange, grammar_ranges};
#[doc(hidden)]
pub use load::{load_registry, load_tick};

/// A function: the command lines of one `.mcfunction` file and the command version they are
/// parsed under.
///
/// The file is plain text, one command per line, without a leading `/`. A line that is empty
/// once trimmed, or whose first character is then `#`, is not a command and is not kept.
/// Written back, a function is its command lines only: comments, blank lines and dropped
/// lines are not written.
#[derive(Clone, Debug, PartialEq, Eq)]
#[doc(alias = "mcfunction")]
pub struct Function {
    /// The name `/function` and `tick.json` call it by: the file's path between `functions/`
    /// and `.mcfunction`, as written (`a/b` for `functions/a/b.mcfunction`). Vanilla compares
    /// it byte for byte, so `Foo/Bar` and `foo/bar` are two functions.
    pub name: FunctionPath,
    /// The pack's command version (8 to 46, from its `min_engine_version`), which selects the
    /// command grammar of every line. `None` when the pack has no usable `min_engine_version`:
    /// the function exists, shadows lower packs' functions of its name, and fails when called
    /// (vanilla's `commands.function.noEngineVersionSpecified`); its lines are then kept
    /// unchecked.
    pub command_version: Option<u8>,
    /// The command lines, in file order.
    pub commands: Vec<FunctionLine>,
    /// The lines vanilla dropped because their first word names no command. Only a pack below
    /// command version 12 (`min_engine_version` below 1.16.0) keeps a function with such
    /// lines; from 12 on the whole function is rejected. Not written back.
    pub dropped: Vec<FunctionLine>,
}

/// One command line of a [`Function`].
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FunctionLine {
    /// The one-based number of the line in its file, blank and comment lines counted (the
    /// number vanilla's messages give); 0 for a line that was not read from a file.
    pub line: u32,
    /// The line as written, without its line ending and not trimmed: vanilla hands the command
    /// parser the untrimmed line.
    pub raw: String,
}

/// Whether a [`Function`] can run.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum FunctionState {
    /// The function runs.
    Valid,
    /// The pack has no usable `min_engine_version`: calling the function fails with
    /// `commands.function.noEngineVersionSpecified`, and a tick entry naming it is skipped.
    EngineVersionNotInitialized,
}

/// Where the commands of a function called from a running function go in the command queue.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum QueueOrder {
    /// After the rest of the calling function: command version 18 (`min_engine_version`
    /// 1.18.10) and above.
    AfterCaller,
    /// Before the rest of the calling function: below command version 18.
    NextToExecute,
}

/// The functions that run every tick (`functions/tick.json`).
///
/// JSON: `{"values": ["<function name>", …]}`. As one pack's file, the names are as written.
/// As the stack's merged value, they are the entries vanilla runs, in run order: every pack's
/// entries, the lowest-priority pack's first, a name listed twice kept twice, and an entry
/// naming no loaded function left out.
#[derive(Clone, Debug, Default, PartialEq)]
#[doc(alias = "tick.json")]
pub struct TickList {
    /// The functions to run, in order (JSON `values`).
    pub values: Vec<TickFunction>,
    /// The other members of the file, which vanilla does not read.
    pub extra: Extra,
}

/// One entry of a [`TickList`].
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TickFunction {
    /// The name of the function to run.
    pub function: FunctionPath,
    /// The command version of the pack whose `tick.json` lists the entry; 46 for a pack
    /// without a usable `min_engine_version`. Not part of the JSON.
    pub command_version: u8,
}
