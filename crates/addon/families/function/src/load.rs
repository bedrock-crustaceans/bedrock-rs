//! Loading functions and tick lists: `Content` / `Definition` / `Merge`, the walks of
//! `functions/`, and how the files of several packs combine.

use super::parse::{not_found, parse_function, parse_tick};
use super::{Function, TickList};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics};
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, JsonContent, Merge, Merged, NoBeta,
    Origin, Placed, Registry, RegistryBuilder, Rejected, VersionInfo,
};
use bedrock_addon_core::id::{FunctionPath, Kind, kind};
use bedrock_addon_core::json::{EncodeError, WriteStyle};
use bedrock_addon_core::load::{
    Accept, CollectionLoader, LoadContext, Registration, StackCx, Walk, WalkFile,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::{CURRENT_COMMAND_VERSION, MinEngineVersion};

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
pub(super) fn function_name(path: &PackPath) -> Option<&str> {
    path.as_str()
        .strip_prefix("functions/")?
        .strip_suffix(".mcfunction")
        // The game ignores a file whose name is empty (`functions/.mcfunction`).
        .filter(|name| !name.is_empty())
}

/// The origin of a file of `pack`.
fn origin(pack: PackRef, side: PackSide, path: &PackPath) -> Origin {
    Origin::file(pack, side, path.clone())
}

/// Neither file kind has a `format_version`: the version is the engine's.
fn unversioned(cx: &LoadContext) -> VersionInfo {
    VersionInfo::new(None, false, cx.engine_version, cx.engine_version, false)
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
    let origin = origin(pack, side, path);
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
            unversioned(cx),
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
    let origin = origin(pack, side, path);
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
            Ok(Document::new(list, origin, unversioned(cx), diagnostics))
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
    let mut builder = RegistryBuilder::new(
        CombineRule::top_wins(),
        CombineRule::top_wins(),
        stack.options,
    );
    for contribution in parsed {
        builder.push(contribution);
    }
    builder.finish()
}

/// The loader of `bp.functions()`: `functions/**/*.mcfunction` of every behaviour pack, by
/// name.
pub const fn load_registry() -> CollectionLoader<Registry<Function>, Contribution<Function>> {
    CollectionLoader {
        walk: WALK,
        parse: parse_walk_file,
        combine,
        needs: &[],
        then: None,
    }
}

/// One file of the tick walk: a pack's `functions/tick.json`. The tick walk reads
/// `functions/tick.json` alone and asks the stack's function registry (`StackCx::registered`)
/// which names exist, instead of walking the function files a second time.
#[derive(Debug)]
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
pub const fn load_tick() -> CollectionLoader<Merged<TickList>, TickWalkFile> {
    CollectionLoader {
        walk: TICK_WALK,
        parse: parse_tick_walk_file,
        combine: combine_tick,
        needs: &[<kind::Function as Kind>::NAME],
        then: None,
    }
}
