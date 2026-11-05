// The script bundle: the `.js` files below a pack's `scripts/` folder,
// the entry file its script module names, and what each `import` specifier resolves to. There
// is no document format and no version chain; the rules are the engine's path rules, reimplemented
// byte for byte (`normalize`), and a lexer of the crate's own that finds the import sites.

#[cfg(any(feature = "script-syntax", test))]
mod bound;
mod lexer;
mod load;
mod normalize;
mod scan;
#[cfg(feature = "script-syntax")]
mod syntax;

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;
use std::fmt;

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, NotFoundHint};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use indexmap::IndexMap;

pub use load::LOADER;
pub use normalize::{EngineCrash, normalize};

/// The folder a pack's script files live in: `scripts/`, for a `script` module of a behaviour
/// pack and a `client_script` module of a resource pack alike.
// The game's script source for a pack is rooted at `scripts/`, whatever the module's type.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ScriptRoot;

impl ScriptRoot {
    /// The folder with its trailing slash: `scripts/`.
    pub const PREFIX: &'static str = "scripts/";
    /// The folder's name: `scripts`.
    pub const DIRECTORY: &'static str = "scripts";

    /// `scripts/`.
    pub fn as_str(self) -> &'static str {
        Self::PREFIX
    }
}

impl fmt::Display for ScriptRoot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(Self::PREFIX)
    }
}

/// The name of a script file as the engine knows it: its path below `scripts/`, with `/`
/// between folders (`lib/b.js`).
///
/// Names are compared byte for byte on every platform: `Upper.js` and `upper.js` are two
/// files. The order is the byte order.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ScriptPath(String);

impl ScriptPath {
    /// The name `name`, taken as written.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// The name of the file at `path` of a pack, when it is a script file: `path` lies below
    /// `scripts/` at any depth and ends in the three bytes `.js`. `.JS`, `.mjs`, `.ts` and
    /// `.json` files are no script files, nor is any file outside `scripts/`.
    // The game compares the file's extension, byte for byte, with `.js`.
    pub fn of_pack_path(path: &PackPath) -> Option<Self> {
        let name = path.as_str().strip_prefix(ScriptRoot::PREFIX)?;
        (normalize::extension(name) == ".js").then(|| Self(name.to_owned()))
    }

    /// The name as text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ScriptPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Where a script file's bytes stop being UTF-8.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, thiserror::Error)]
#[error("invalid UTF-8 at byte {offset}")]
#[non_exhaustive]
pub struct InvalidUtf8 {
    /// The offset in the file of the first byte that is not UTF-8.
    pub offset: u64,
    /// The line that byte is on, from 1: the line the engine names. The engine's own
    /// message for a byte on the first line of a main file reads `at main.js:1`; for a later
    /// line, and for the name of a file in a sub-folder, the crate assumes the line where the
    /// engine's tokenizer stops.
    pub line: u32,
}

/// A place in a script file.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct ScriptPosition {
    /// The byte offset in the file.
    pub offset: u64,
    /// The line, from 1, as the engine prints it (`main.js:2`).
    pub line: u32,
    /// The column in characters, from 1.
    pub column: u32,
}

/// Why the scan of a script file stopped.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ScriptSyntaxKind {
    /// A string literal that its line ends before its closing quote.
    UnterminatedString,
    /// A template literal the file ends inside.
    UnterminatedTemplate,
    /// A `/* … */` comment the file ends inside.
    UnterminatedComment,
    /// What the JavaScript parser of the `script-syntax` feature reports: checked against
    /// ES2025, not against the engine's `QuickJS`.
    Parser {
        /// The parser's message.
        message: Box<str>,
    },
}

/// A syntax error in a script file, and where.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct ScriptSyntax {
    /// What is wrong.
    pub kind: ScriptSyntaxKind,
    /// Where: the opening quote, backquote or `/*`, or the place the parser names.
    pub at: ScriptPosition,
}

/// One script file of a bundle: where it is, and what it holds when it was read.
///
/// A load reads the script files of a pack whose script module starts a plugin, and of no
/// other pack: nothing of a pack without one ever runs, so its files are listed by name and
/// [`content`](Self::content) is `None`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ScriptFile {
    /// The file's path in its pack (`scripts/lib/b.js`; with the subpack folder when a subpack
    /// supplies it).
    pub path: PackPath,
    /// What the file holds; `None` for a file that was not read: one of a pack whose script
    /// module does not run (the pack has none, or vanilla drops it for its language or its
    /// entry before it looks at a file), or one the load did not get the bytes of.
    pub content: Option<ScriptContent>,
}

impl ScriptFile {
    /// The file's import sites, in file order; none for a file that was not read.
    pub fn imports(&self) -> &[ImportSite] {
        self.content
            .as_ref()
            .map_or(&[], |content| content.imports.as_slice())
    }
}

/// What a script file holds: its size and encoding, and what the scan found in it.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ScriptContent {
    /// The file's size in bytes.
    pub size: u64,
    /// The file starts with a UTF-8 byte order mark, which the engine accepts.
    pub bom: bool,
    /// Whether the file is UTF-8. The engine refuses a file that is not, when it loads it
    /// (`SyntaxError: invalid UTF-8 sequence`); a file nothing imports is never read.
    pub utf8: Result<(), InvalidUtf8>,
    /// The lexical error that stopped the scan of the file, or, with the `script-syntax`
    /// feature, the first error of the JavaScript parser. `None` for a file that scans whole.
    /// Of a file that is not UTF-8 only the text before the first bad byte is scanned, for a
    /// string its line ends inside: the error the engine would meet before the byte.
    pub syntax: Option<ScriptSyntax>,
    /// The JavaScript parser of the `script-syntax` feature read the file.
    ///
    /// `false` without the feature, and with it for a file the parser is not given, which
    /// has the lexer's findings only:
    ///
    /// - a file the entry does not reach, and every file of a bundle without a
    ///   [`graph`](ScriptBundle::graph): vanilla reads a file when an import first names it;
    /// - a file the lexer stopped in, or that is not UTF-8;
    /// - a file that could nest more deeply than the parser, which recurses, has stack for:
    ///   the nesting is bounded from the file's bytes, whatever their reading, and held to
    ///   a quarter of a megabyte of stack.
    ///   A bundle with such files among those its entry reaches says how many in one note
    ///   (`ScriptSyntaxNotChecked`).
    pub parsed: bool,
    /// The file's import sites, in file order: empty for a file that is not UTF-8, and the
    /// sites before the error for a file with a lexical error.
    pub imports: Vec<ImportSite>,
}

/// The entry of a bundle: the file its script module starts with.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct EntryRef {
    /// The module's `entry` as written (`scripts/main.js`).
    pub raw: String,
    /// The main file: `entry` without the `scripts/` prefix, `\` read as `/`. An `entry`
    /// without the prefix is taken whole.
    pub main: ScriptPath,
    /// `entry` had the `scripts/` prefix. Vanilla accepts one without it when the file
    /// exists.
    pub prefix_stripped: bool,
}

impl EntryRef {
    /// The entry a module's `entry` member `raw` names.
    // The game reads `\` in the entry as a path separator, and an entry without the
    // `scripts/` prefix is kept as it is.
    pub(crate) fn of(raw: &str) -> Self {
        let path = raw.replace('\\', "/");
        let stripped = path.strip_prefix(ScriptRoot::PREFIX);
        Self {
            raw: raw.to_owned(),
            main: ScriptPath::new(stripped.unwrap_or(&path)),
            prefix_stripped: stripped.is_some(),
        }
    }
}

/// How an import is written.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ImportKind {
    /// `import x from "s"`, `import { a } from "s"`, `import * as ns from "s"`.
    Static,
    /// `export * from "s"`, `export * as ns from "s"`, `export { a } from "s"`.
    ReExport,
    /// `import "s"`.
    SideEffect,
    /// `import("s")` with a string literal, or a template literal without substitutions.
    Dynamic,
    /// `import(…)` with any other argument: the specifier is not known before the call runs.
    DynamicNonLiteral,
}

impl ImportKind {
    /// The import is linked with the importing file: `Static`, `ReExport`, `SideEffect`. A
    /// failure of one is a failure of every file that statically reaches it.
    pub fn is_static(self) -> bool {
        matches!(self, Self::Static | Self::ReExport | Self::SideEffect)
    }
}

/// One `import` or re-export of a script file.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ImportSite {
    /// How it is written.
    pub kind: ImportKind,
    /// The specifier as the engine sees it, escapes decoded; empty for `DynamicNonLiteral`.
    pub raw: String,
    /// The line of the specifier's opening quote, from 1 (of `import` for
    /// `DynamicNonLiteral`).
    pub line: u32,
    /// The column of the same place in characters, from 1.
    pub column: u32,
    /// The byte offset of the same place.
    pub offset: u64,
    /// `with { … }` or `assert { … }` follows the specifier: the engine's `QuickJS` refuses
    /// import attributes (`SyntaxError: expecting ';'`).
    pub attributes: bool,
    /// What the specifier names.
    pub resolution: Resolution,
}

/// What an import specifier names, by the engine's own rules ([`normalize`]).
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Resolution {
    /// A script file of the same bundle.
    File(ScriptPath),
    /// A native module the script module may import: one it declares, or one a declared module
    /// brings.
    Native {
        /// The module name.
        name: String,
    },
    /// A native module no pack file may import (`@minecraft/server-bindings`).
    Restricted {
        /// The module name.
        name: String,
    },
    /// An `@…` name that is neither a native module of the context nor a file.
    Unrecognized {
        /// The name.
        name: String,
    },
    /// No file of the bundle has the name the specifier normalises to.
    NotFound {
        /// The name looked up.
        looked_up: String,
        /// Why it likely failed, when a near miss exists.
        hint: Option<NotFoundHint>,
        /// The file the specifier likely meant, for the hints that find one
        /// (`CaseMismatch`, `DirectoryIndex`, `ScriptsPrefix`).
        near: Option<ScriptPath>,
    },
    /// The specifier's leading run of `./` and `../` reaches its end (`.`, `..`, `./`,
    /// `../`): vanilla's normaliser throws and the server terminates.
    CrashesEngine,
    /// Not looked at: a non-literal `import()`, a file the entry does not reach, or a bundle
    /// whose imports are not scanned.
    NotChecked,
}

/// Which files of a bundle its entry reaches.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct ImportGraph {
    /// The entry and every file it reaches through static imports and re-exports: the files
    /// the engine links before the entry runs. A cycle is no error.
    pub reachable_static: BTreeSet<ScriptPath>,
    /// The files reached only through at least one literal `import()`.
    pub reachable_dynamic: BTreeSet<ScriptPath>,
}

impl ImportGraph {
    /// Whether the entry reaches `file` at all.
    pub fn reaches(&self, file: &ScriptPath) -> bool {
        self.reachable_static.contains(file) || self.reachable_dynamic.contains(file)
    }
}

/// What a native module name is to a script module's context.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum NativeModuleAccess {
    /// The module may be imported: the manifest declares it, or a declared module brings it.
    Available,
    /// The module exists and pack files may not import it.
    Restricted,
    /// The context has no module of the name.
    Unknown,
}

/// What a bundle needs to know of the script module that runs it: the manifest side of
/// scripting, which the bundle does not read itself.
///
/// A stack answers with the plugin discovery made of the pack's script module
/// ([`ScriptPlugin`](crate::ScriptPlugin), which implements this); a pack whose module
/// starts no plugin has no host.
pub trait ScriptHost {
    /// The name vanilla prints the plugin by: `<pack header name> - <module version>`.
    fn plugin(&self) -> &str;
    /// The module's `entry` as written; `None` when the manifest has none.
    fn entry(&self) -> Option<&str>;
    /// What the native module `name` is to the module's context.
    fn native(&self, name: &str) -> NativeModuleAccess;
    /// The pack has the `script_eval` capability, which enables `eval` and `Function(string)`.
    fn script_eval(&self) -> bool;
}

/// How a bundle is scanned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ScanOptions {
    /// Resolve the imports of the files the entry reaches. Off, the bundle has no
    /// [`graph`](ScriptBundle::graph) and says so once (`ScriptsNotChecked`).
    pub imports: bool,
}

impl ScanOptions {
    /// These options with import resolution switched on or off.
    #[must_use]
    pub const fn with_imports(mut self, imports: bool) -> Self {
        self.imports = imports;
        self
    }
}

impl Default for ScanOptions {
    /// Imports are resolved.
    fn default() -> Self {
        Self { imports: true }
    }
}

/// The script files of one pack: every `.js` file below `scripts/`, the entry of the pack's
/// script module, and what the imports of the files the entry reaches resolve to.
///
/// A stack has one bundle per pack that has a `script` / `client_script` module or any file
/// below `scripts/`, the highest pack first (the stack accessor `scripts()`).
/// Packs never share script files: an import resolves inside its own bundle or not at all.
///
/// On a stack the bundle is its pack's whole script side: the pack's script module is read
/// once ([`ScriptModule`](crate::ScriptModule)), discovered with the modules of the
/// packs above it against the catalog compiled in, and the plugin that makes
/// ([`plugin`](Self::plugin)) is what the files are scanned against. The findings of the
/// module (its manifest members, its dependencies, the versions it asks for) are the
/// bundle's first findings, at their place in `manifest.json`.
///
/// The engine loads lazily: it reads a file when an import first names it, so a file the
/// entry does not reach is never read and whatever is wrong in it is harmless. The bundle
/// follows that: only what the entry reaches raises more than a note.
///
/// Hand-written: one walk on each side reads every pack's manifest and script files into one
/// bundle per pack (the collection is `Vec<ScriptBundle>`), and a script is no versioned JSON
/// document, so there is no `Family`.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
#[doc(alias = "scripts")]
pub struct ScriptBundle {
    /// The pack.
    pub pack: PackRef,
    /// The stack side: behaviour for a `script` module, resources for a `client_script` one.
    pub side: PackSide,
    /// The folder: `scripts/`.
    pub root: ScriptRoot,
    /// The script files by name, bytewise by path.
    pub files: IndexMap<ScriptPath, ScriptFile>,
    /// The other files below `scripts/` (`.mjs`, `.ts`, `.json`, `.JS`, no extension): no
    /// import can name them.
    pub others: Vec<PackPath>,
    /// The entry of the pack's script module; `None` for a pack without a script module, and
    /// for one whose module has no `entry` or another language than JavaScript.
    pub entry: Option<EntryRef>,
    /// What the entry reaches; `None` when no plugin starts, or imports are not scanned.
    pub graph: Option<ImportGraph>,
    /// The plugin discovery made of the pack's script module: the script that runs. `None`
    /// for a pack without a script module, for one a finding disabled, and for a bundle
    /// scanned outside a stack.
    pub plugin: Option<crate::ScriptPlugin>,
    /// The script module the pack's manifest declares for this side of the stack, whether
    /// or not its script starts: what the server keys the pack's configuration by. `None`
    /// for a pack without one, for a manifest that was refused, and for a bundle scanned
    /// outside a stack.
    pub module: Option<crate::ScriptModuleRef>,
    /// The findings.
    diagnostics: Diagnostics,
}

impl ScriptBundle {
    /// Whether script files are also parsed as JavaScript: the `script-syntax` feature. Without
    /// it only the import lexer runs, and a general syntax error is the engine's to find.
    /// With it, [`ScriptContent::parsed`] says of each file whether the parser read it.
    pub const SYNTAX_CHECKED: bool = cfg!(feature = "script-syntax");

    /// The bundle of a pack outside any stack, from its files.
    ///
    /// `files` are the pack's files by path with their bytes; only the bytes of script files
    /// (`scripts/**/*.js`) are looked at. `host` is the pack's script module, `None` for a
    /// pack without one.
    pub fn scan<'a>(
        files: impl IntoIterator<Item = (&'a PackPath, &'a [u8])>,
        host: Option<&dyn ScriptHost>,
        options: ScanOptions,
    ) -> Self {
        scan::scan_detached(files, host, options)
    }

    /// Every finding of the bundle, in order: the entry's, then each file's in name order.
    pub fn findings(&self) -> impl Iterator<Item = &Diagnostic> + '_ {
        self.diagnostics.iter()
    }

    /// The main file, when the entry names a file of the bundle.
    pub fn main(&self) -> Option<&ScriptFile> {
        self.files.get(&self.entry.as_ref()?.main)
    }

    /// The import sites of every file that was read, with the file each is in.
    pub fn imports(&self) -> impl Iterator<Item = (&ScriptPath, &ImportSite)> + '_ {
        self.files
            .iter()
            .flat_map(|(name, file)| file.imports().iter().map(move |site| (name, site)))
    }
}
