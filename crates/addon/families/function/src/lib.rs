//! Functions: `Function`, `TickList`.
//!
//! Functions (`functions/**/*.mcfunction` of a behaviour pack) and the tick list
//! (`functions/tick.json`). Neither file has a `format_version`: there is one model, and the only
//! version input is the pack's `min_engine_version`, mapped to a command version (8 to 46) that
//! every function and tick entry carries. Command lines are kept raw; parsing them is the command
//! system's (chorus).
//!
//! The reader stays text: command lines are kept raw, and `tick.json` is a list of names read by
//! hand as vanilla's function manager reads it.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod grammar;
mod lint;
mod read;
mod texts;
mod write;

#[cfg(test)]
mod tests;

use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::id::FunctionPath;

use crate::read::{first_word, trimmed};
use crate::read::{not_found, parse_function, parse_tick};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics};
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, JsonContent, Merge, Merged, NoBeta,
    Placed, Registry, RegistryBuilder, Rejected, unversioned,
};
use bedrock_addon_core::id::{Kind, kind};
use bedrock_addon_core::json::{EncodeError, WriteStyle};
use bedrock_addon_core::load::{
    Accept, CollectionLoader, LoadContext, Registration, StackCx, Walk, WalkFile,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::{CURRENT_COMMAND_VERSION, MinEngineVersion};
pub use grammar::{GrammarRange, grammar_ranges};

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
    pub extra: UnknownMembers,
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

/// Where a pack keeps its tick list.
const TICK_PATH: &str = "functions/tick.json";

impl bedrock_addon_core::__private::Sealed for Function {}

impl Content for Function {
    const NAME: &'static str = "function";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    /// Reads one `.mcfunction` file as a file of a pack loaded with `cx`. The function's name
    /// comes from `path`, which must be `functions/<name>.mcfunction`; the pack's
    /// `min_engine_version` gives the command version.
    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        parse_function_file(
            PackRef::app_data(),
            PackSide::Behavior,
            path,
            path,
            bytes,
            cx,
        )
        .1
    }

    /// Writes the function as `.mcfunction` text, not JSON: its command lines, one per line.
    /// Only `style.trailing_newline` applies.
    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        self.write_into(out, style)
    }
}

impl Definition for Function {
    type Kind = kind::Function;
    type File = Function;
    type Beta = NoBeta<kind::Function>;
}

impl Placed for Function {
    /// `functions/<name>.mcfunction`.
    fn default_path(&self) -> PackPath {
        PackPath::new(&format!("functions/{}.mcfunction", self.name.as_str()))
            .unwrap_or_else(|_| PackPath::from_static("functions/_.mcfunction"))
    }
}

impl bedrock_addon_core::__private::Sealed for TickList {}

impl Content for TickList {
    const NAME: &'static str = "tick list";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    /// Reads one `tick.json` as a file of a pack loaded with `cx`. Whether its entries name
    /// loaded functions is decided on a stack, not here.
    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        parse_tick_file(PackRef::app_data(), PackSide::Behavior, path, bytes, cx).0
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        self.write_into(out, style)
    }
}

impl JsonContent for TickList {}

impl Placed for TickList {
    /// `functions/tick.json`.
    fn default_path(&self) -> PackPath {
        PackPath::from_static(TICK_PATH)
    }
}

/// The key is the position of an entry in the merged `values`.
impl Merge for TickList {
    type Key = usize;
}

/// The name of the function the file at `path` (relative to its pack) defines: the text between
/// `functions/` and `.mcfunction`. `None` when the file is not a function file.
pub(crate) fn function_name(path: &PackPath) -> Option<&str> {
    path.as_str()
        .strip_prefix("functions/")?
        .strip_suffix(".mcfunction")
        // The game ignores a file whose name is empty (`functions/.mcfunction`).
        .filter(|name| !name.is_empty())
}

/// Loads one function file of `pack`: the name `key` gives it (the file's path without a
/// subpack folder) and the document or its rejection.
fn parse_function_file(
    pack: PackRef,
    side: PackSide,
    path: &PackPath,
    key: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> (Option<FunctionPath>, Result<Document<Function>, Rejected>) {
    let origin = unversioned::origin(pack, side, path);
    let Some(name) = function_name(key) else {
        let mut diagnostics = Diagnostics::default();
        diagnostics.push(Diagnostic::error(DiagnosticCode::IgnoredFile {
            text: None,
        }));
        diagnostics.locate(&origin);
        return (None, Err(Rejected::new(origin, diagnostics)));
    };
    let name = FunctionPath::parse(name);
    // A pack without a valid `min_engine_version` gives no command version: the game creates
    // the function in its error state.
    let version = cx
        .min_engine_version
        .as_ref()
        .map(MinEngineVersion::command_version);
    let (function, mut diagnostics) = parse_function(name.clone(), bytes, version);
    diagnostics.locate(&origin);
    let result = if diagnostics.has_errors() {
        Err(Rejected::new(origin, diagnostics))
    } else {
        Ok(Document::new(
            function,
            origin,
            // A function has no `format_version`: the version is the engine's.
            unversioned::version_at(cx.engine_version),
            diagnostics,
        ))
    };
    (Some(name), result)
}

/// Loads one tick list of `pack`: the document or its rejection, and the index in the file's
/// `values` of each entry the document keeps.
fn parse_tick_file(
    pack: PackRef,
    side: PackSide,
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> (Result<Document<TickList>, Rejected>, Vec<usize>) {
    let origin = unversioned::origin(pack, side, path);
    // A pack without `min_engine_version` gets the default version's command version, 46.
    let version = cx
        .min_engine_version
        .as_ref()
        .map_or(CURRENT_COMMAND_VERSION, MinEngineVersion::command_version);
    // A number no double holds is read as the largest finite one, with a warning, so the list can always be written back.
    let finite = bedrock_addon_core::document::FiniteText::of(bytes);
    let parsed = parse_tick(finite.bytes(), version);
    let mut diagnostics = parsed.diagnostics;
    finite.settle(&mut diagnostics, None);
    diagnostics.locate(&origin);
    let result = match parsed.list {
        Some(list) if !diagnostics.has_errors() => {
            // A tick list has no `format_version`: the version is the engine's.
            let version = unversioned::version_at(cx.engine_version);
            Ok(Document::new(list, origin, version, diagnostics))
        }
        _ => Err(Rejected::new(origin, diagnostics)),
    };
    (result, parsed.indices)
}

// ---- the collections ---------------------------------------------------------------------

/// A function file of the walk.
fn is_function_file(path: &PackPath) -> bool {
    function_name(path).is_some()
}

/// `functions/**/*.mcfunction`, packs top-down: vanilla walks the stack from index 0 and the
/// first function created under a name keeps it. The game descends into sub-folders.
const WALK: Walk = Walk::json_tree_in(&["functions"])
    .accept(Accept::Fn(is_function_file))
    .top_down();

/// One function file's contribution.
fn parse_walk_file(file: &WalkFile, cx: &LoadContext) -> Contribution<Function> {
    match parse_function_file(
        file.pack.clone(),
        file.side,
        &file.path,
        &file.key,
        &file.bytes,
        cx,
    ) {
        (_, Ok(document)) => Contribution::Stable(document.name.clone(), document),
        (name, Err(rejected)) => Contribution::Rejected(name, rejected),
    }
}

/// The registry of the stack's functions.
fn combine(parsed: Vec<Contribution<Function>>, stack: &StackCx<'_>) -> Registry<Function> {
    RegistryBuilder::combine(
        parsed,
        CombineRule::top_wins(),
        CombineRule::top_wins(),
        stack.options,
    )
}

/// The loader of `bp.functions()`: `functions/**/*.mcfunction` of every behaviour pack, by
/// name. The family's own: a function is a text file (no `JsonDocument`) named by its walked
/// key, not by anything in the file, which `load::registry` cannot read.
#[doc(hidden)]
pub const LOADER: CollectionLoader<Registry<Function>, Contribution<Function>> = CollectionLoader {
    walk: WALK,
    parse: parse_walk_file,
    combine,
    needs: &[],
    then: None,
};

/// One file of the tick walk: a pack's `functions/tick.json`. The tick walk reads
/// `functions/tick.json` alone and asks the stack's function registry (`StackCx::registered`)
/// which names exist, instead of walking the function files a second time.
#[derive(Debug)]
#[doc(hidden)]
pub struct TickWalkFile {
    /// The list, or why the file was skipped.
    result: Box<Result<Document<TickList>, Rejected>>,
    /// The index in the file's `values` of each entry the list keeps.
    indices: Vec<usize>,
}

/// `functions/tick.json`; packs bottom-up, as vanilla reads the tick files. Whether an entry
/// names a function is asked of the stack's function registry (`StackCx::registered`): the
/// function files are read once, for `bp.functions()`.
const TICK_WALK: Walk = Walk::json_children_in(&["functions"]).accept(Accept::Exact(&[TICK_PATH]));

/// One file of the tick walk.
fn parse_tick_walk_file(file: &WalkFile, cx: &LoadContext) -> TickWalkFile {
    let (result, indices) =
        parse_tick_file(file.pack.clone(), file.side, &file.path, &file.bytes, cx);
    TickWalkFile {
        result: Box::new(result),
        indices,
    }
}

/// The stack's tick list: nothing is overridden or de-duplicated, every listed entry of every
/// pack is appended.
fn combine_tick(parsed: Vec<TickWalkFile>, stack: &StackCx<'_>) -> Merged<TickList> {
    let mut walk = Vec::new();
    let mut indices = Vec::new();
    for file in parsed {
        if file.result.is_ok() {
            indices.push(file.indices);
        }
        walk.push(*file.result);
    }
    let mut indices = indices.into_iter();
    Merged::merge(walk, TickList::default(), |merged, layer, cx| {
        let of_file = indices.next().unwrap_or_default();
        for (position, entry) in layer.values.iter().enumerate() {
            // Vanilla looks the name up by its bytes among every function some pack creates,
            // whatever its state, which is what the stack's function registry holds (a
            // rejected file creates none).
            // The registry compares names as identifiers (`a//b` is `a/b`); vanilla's find
            // compares bytes, so the registered spelling decides.
            let found = matches!(
                stack.registered(&entry.function),
                Registration::By { written, .. } if written == entry.function.as_str()
            );
            if found {
                cx.claim(merged.values.len());
                merged.values.push(entry.clone());
                continue;
            }
            // The game's text is `Function listed in tick.json with name %s not found.`,
            // logged as a warning; the entry is skipped.
            let warning = not_found(entry.function.as_str(), of_file.get(position).copied());
            cx.push_diagnostic(warning.in_file(cx.origin()));
        }
    })
}

/// The loader of `bp.tick_functions()`: every behaviour pack's `functions/tick.json`, merged.
/// The family's own: each copy carries the indices of the entries it keeps (`TickWalkFile`),
/// which `load::merged`'s `FileCopy` cannot carry.
#[doc(hidden)]
pub const TICK_LOADER: CollectionLoader<Merged<TickList>, TickWalkFile> = CollectionLoader {
    walk: TICK_WALK,
    parse: parse_tick_walk_file,
    combine: combine_tick,
    needs: &[<kind::Function as Kind>::NAME],
    then: None,
};

/// From this command version on (`min_engine_version` 1.18.10), the commands of a nested
/// function call are queued after the caller's remaining commands.
const FUNCTION_EXECUTE_ORDER: u8 = 18;

impl Function {
    /// A function called `name` with no commands, at the current command version.
    pub fn new(name: FunctionPath) -> Function {
        Function {
            name,
            command_version: Some(CURRENT_COMMAND_VERSION),
            commands: Vec::new(),
            dropped: Vec::new(),
        }
    }

    /// Whether the function can run: it cannot when its pack has no usable
    /// `min_engine_version`: the game creates such a function in the not-initialised state
    /// when the pack's version is not valid.
    pub fn state(&self) -> FunctionState {
        match self.command_version {
            Some(_) => FunctionState::Valid,
            None => FunctionState::EngineVersionNotInitialized,
        }
    }

    /// Whether vanilla offers the name for auto-completion (its `FunctionName` soft enum): only
    /// a function that can run and loaded without a dropped line is listed, because the game
    /// adds the name to the list only when loading the file gave no error.
    pub fn is_auto_completed(&self) -> bool {
        self.command_version.is_some() && self.dropped.is_empty()
    }

    /// Whether `/function <name>` can name the function: every byte of the name must be an
    /// ASCII letter or digit or one of `( ) - . / _` (vanilla's
    /// `commands.function.invalidCharacters` otherwise). A function with another character in
    /// its name still loads and can be listed in `tick.json`.
    pub fn is_callable(&self) -> bool {
        self.name.as_str().bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'(' | b')' | b'-' | b'.' | b'/' | b'_')
        })
    }
}

impl FunctionLine {
    /// A command line that was not read from a file (line number 0).
    pub fn new(raw: impl Into<String>) -> FunctionLine {
        FunctionLine {
            line: 0,
            raw: raw.into(),
        }
    }

    /// The command: the line trimmed, as vanilla trims it before it looks at the first word.
    pub fn command(&self) -> &str {
        trimmed(&self.raw)
    }

    /// The command's name: the first word of the trimmed line, up to its first space.
    pub fn name(&self) -> &str {
        first_word(self.command())
    }
}

impl QueueOrder {
    /// The queue order of nested function calls under `command_version`: the version of the
    /// calling command for `/function`, of the listing pack for a tick entry. Command version 18
    /// is the one that introduced the order after the caller (together with a chest slot fix).
    pub fn from_command_version(command_version: u8) -> QueueOrder {
        if command_version < FUNCTION_EXECUTE_ORDER {
            QueueOrder::NextToExecute
        } else {
            QueueOrder::AfterCaller
        }
    }
}

impl TickFunction {
    /// An entry running `function`, at the current command version.
    pub fn new(function: FunctionPath) -> TickFunction {
        TickFunction {
            function,
            command_version: CURRENT_COMMAND_VERSION,
        }
    }

    /// Where the commands of functions this entry's function calls are queued.
    pub fn queue_order(&self) -> QueueOrder {
        QueueOrder::from_command_version(self.command_version)
    }
}
