//! Building a bundle: the entry rules, what each import of the files the entry reaches
//! resolves to, the reachability graph and the findings.
// Everything
// here is linear in the files and their import sites: names are looked up in maps, the graph
// is walked with a work list.

use crate::codes;
use std::collections::{BTreeSet, HashMap};

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticDetail, Diagnostics, JsonPath, Location, NotFoundHint,
    Phase, ReferenceHint, Severity, Span,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use indexmap::IndexMap;

use super::lexer::{Lexed, lex};
use super::normalize::{Form, Normalized, extension, normalize_full};
use super::{
    EntryRef, ImportGraph, ImportKind, ImportSite, InvalidUtf8, NativeModuleAccess, Resolution,
    ScanOptions, ScriptBundle, ScriptContent, ScriptFile, ScriptHost, ScriptPath, ScriptPosition,
    ScriptRoot, ScriptSyntax, ScriptSyntaxKind,
};
use crate::ScriptPlugin;

/// The UTF-8 byte order mark.
const BOM: &[u8] = b"\xEF\xBB\xBF";

/// One script file of a pack: as its `Parse` job leaves it (read, checked and lexed, its
/// imports not yet resolved: that needs the whole bundle), or listed by name alone.
#[derive(Clone, Debug)]
pub(super) struct Member {
    pub(super) name: ScriptPath,
    pub(super) file: ScriptFile,
    /// `eval(` / `Function(` call tokens.
    evals: Vec<ScriptPosition>,
    /// `import.meta` member reads.
    metas: Vec<ScriptPosition>,
    /// The text of a file the lexer scanned whole, for the parser: it reads the file only
    /// once the entry is known to reach it.
    #[cfg(feature = "script-syntax")]
    source: Option<Box<str>>,
}

impl Member {
    /// The script file `name` at `path` of its pack, listed and not read.
    pub(super) fn listed(name: ScriptPath, path: &PackPath) -> Member {
        Member {
            name,
            file: ScriptFile {
                path: path.clone(),
                content: None,
            },
            evals: Vec::new(),
            metas: Vec::new(),
            #[cfg(feature = "script-syntax")]
            source: None,
        }
    }
}

/// Reads the script file `name` at `path` of its pack.
pub(super) fn member(name: ScriptPath, path: &PackPath, bytes: &[u8]) -> Member {
    let mut content = ScriptContent {
        size: bytes.len() as u64,
        bom: bytes.starts_with(BOM),
        utf8: Ok(()),
        syntax: None,
        parsed: false,
        imports: Vec::new(),
    };
    let file = |content| ScriptFile {
        path: path.clone(),
        content: Some(content),
    };
    let text = match std::str::from_utf8(bytes) {
        Ok(text) => text,
        Err(error) => {
            let offset = error.valid_up_to();
            // The line of the byte, from 1: one more than the line feeds before it.
            let line = bytes[..offset].split(|byte| *byte == b'\n').count();
            content.utf8 = Err(InvalidUtf8 {
                offset: offset as u64,
                line: u32::try_from(line).unwrap_or(u32::MAX),
            });
            content.syntax = error_before_the_byte(&bytes[..offset]);
            return Member {
                name,
                file: file(content),
                evals: Vec::new(),
                metas: Vec::new(),
                #[cfg(feature = "script-syntax")]
                source: None,
            };
        }
    };
    let Lexed {
        imports,
        evals,
        metas,
        error,
    } = lex(text);
    content.imports = imports
        .into_iter()
        .map(|import| ImportSite {
            kind: import.kind,
            raw: import.raw,
            line: import.at.line,
            column: import.at.column,
            offset: import.at.offset,
            attributes: import.attributes,
            resolution: Resolution::NotChecked,
        })
        .collect();
    content.syntax = error.map(|(kind, at)| ScriptSyntax { kind, at });
    Member {
        name,
        #[cfg(feature = "script-syntax")]
        source: content.syntax.is_none().then(|| text.into()),
        file: file(content),
        evals,
        metas,
    }
}

/// The syntax error in `valid`, the UTF-8 text of a file up to its first byte that is none:
/// a string literal whose line ends, inside that text, before its closing quote. A string, a
/// template or a comment still open where the text stops is no error there: the bytes after
/// it may close it.
fn error_before_the_byte(valid: &[u8]) -> Option<ScriptSyntax> {
    let text = std::str::from_utf8(valid).ok()?;
    let (kind, at) = lex(text).error?;
    let from = usize::try_from(at.offset).ok()?;
    let line_ends = text.get(from..)?.contains('\n');
    (kind == ScriptSyntaxKind::UnterminatedString && line_ends).then_some(ScriptSyntax { kind, at })
}

/// The script module of the pack, as the bundle is built.
pub(super) enum Module<'a> {
    /// The pack has none.
    None,
    /// A pack of the game that has none: the engine reads the scripts of its own library
    /// packs itself, by the pack's folder name, so nothing says they never run.
    OfTheGame,
    /// It has one and no plugin starts: the findings that say why are the module's own and
    /// came with the parts.
    Dropped {
        /// The entry the module names, when discovery got as far as reading it.
        entry: Option<EntryRef>,
    },
    /// A host that is no discovered plugin (a bundle scanned outside a stack): the entry
    /// rules are checked here, since nothing else has.
    Host(&'a dyn ScriptHost),
    /// The plugin discovery made of the pack's module: its entry and main file are checked.
    Plugin(&'a ScriptPlugin),
}

/// Everything a bundle is built from.
pub(super) struct Parts<'a> {
    pub(super) pack: PackRef,
    pub(super) side: PackSide,
    /// The script files, bytewise by name.
    pub(super) members: Vec<Member>,
    /// The other files below `scripts/`.
    pub(super) others: Vec<PackPath>,
    /// The `.js` files outside `scripts/`.
    pub(super) outside: Vec<PackPath>,
    pub(super) module: Module<'a>,
    /// The findings of the manifest side, which come first.
    pub(super) findings: Vec<Diagnostic>,
    pub(super) options: ScanOptions,
}

/// The findings of one bundle, each placed in its pack.
struct Findings {
    pack: PackRef,
    out: Diagnostics,
}

impl Findings {
    /// A finding of this bundle's pack, at `offset` of the file at `path` when it has one.
    fn make(
        &self,
        code: DiagnosticCode,
        severity: Severity,
        path: Option<&PackPath>,
        offset: Option<u64>,
    ) -> Diagnostic {
        let span = offset.map(|offset| Span {
            offset: u32::try_from(offset).unwrap_or(u32::MAX),
            len: 1,
        });
        let location = Location::new(
            Some(self.pack.clone()),
            path.cloned(),
            JsonPath::default(),
            span,
        );
        Diagnostic::new(code, severity, Phase::Validate, location)
    }

    fn push(
        &mut self,
        code: DiagnosticCode,
        severity: Severity,
        path: Option<&PackPath>,
        offset: Option<u64>,
    ) {
        let finding = self.make(code, severity, path, offset);
        self.out.push(finding);
    }
}

/// The bundle of a pack outside any stack.
pub(super) fn scan_detached<'a>(
    files: impl IntoIterator<Item = (&'a PackPath, &'a [u8])>,
    host: Option<&dyn ScriptHost>,
    options: ScanOptions,
) -> ScriptBundle {
    let mut members = Vec::new();
    let mut others = Vec::new();
    let mut outside = Vec::new();
    for (path, bytes) in files {
        if let Some(name) = ScriptPath::of_pack_path(path) {
            members.push(member(name, path, bytes));
        } else if path.as_str().starts_with(ScriptRoot::PREFIX) {
            others.push(path.clone());
        } else if is_js(path) {
            outside.push(path.clone());
        }
    }
    members.sort_by(|a, b| a.name.cmp(&b.name));
    others.sort();
    outside.sort();
    build(Parts {
        pack: PackRef::app_data(),
        side: PackSide::Behavior,
        members,
        others,
        outside,
        module: host.map_or(Module::None, Module::Host),
        findings: Vec::new(),
        options,
    })
}

/// A `.js` file by the engine's own test: the extension is exactly `.js`.
pub(super) fn is_js(path: &PackPath) -> bool {
    extension(path.as_str()) == ".js"
}

/// Builds the bundle of one pack.
pub(super) fn build(parts: Parts<'_>) -> ScriptBundle {
    let Parts {
        pack,
        side,
        members,
        others,
        outside,
        module,
        findings: first,
        options,
    } = parts;
    let mut findings = Findings {
        pack: pack.clone(),
        out: Diagnostics::default(),
    };
    for finding in first {
        findings.out.push(finding);
    }
    let mut files = IndexMap::with_capacity(members.len());
    let mut tokens = Vec::with_capacity(members.len());
    #[cfg(feature = "script-syntax")]
    let mut sources = Vec::with_capacity(members.len());
    for member in members {
        files.insert(member.name, member.file);
        tokens.push((member.evals, member.metas));
        #[cfg(feature = "script-syntax")]
        sources.push(member.source);
    }
    let mut bundle = ScriptBundle {
        pack,
        side,
        root: ScriptRoot,
        files,
        others,
        entry: None,
        graph: None,
        plugin: None,
        module: None,
        diagnostics: Diagnostics::default(),
    };
    let scanned = Scanned {
        tokens: &tokens,
        #[cfg(feature = "script-syntax")]
        sources: &sources,
    };
    match module {
        Module::None => {
            // Nothing of a pack without a script module ever runs.
            if !bundle.files.is_empty() {
                findings.push(
                    DiagnosticCode::from(&core_codes::SCRIPT_FILE_NOT_LOADABLE),
                    Severity::Info,
                    None,
                    None,
                );
            }
        }
        // The members are listed and nothing is said of them.
        Module::OfTheGame => {}
        Module::Dropped { entry } => {
            bundle.entry = entry;
            not_loadable(&outside, &mut findings);
        }
        Module::Host(host) => {
            let main = entry(&mut bundle, host, &mut findings);
            not_loadable(&outside, &mut findings);
            if let Some(main) = main {
                run(&mut bundle, &scanned, main, host, options, &mut findings);
            }
        }
        Module::Plugin(plugin) => {
            // Discovery checked the entry's extension and found the main file.
            let entry = EntryRef::of(&plugin.entry);
            let main = bundle.files.get_index_of(&entry.main);
            if !entry.prefix_stripped && main.is_some() {
                entry_outside_root(&mut findings);
            }
            bundle.entry = Some(entry);
            not_loadable(&outside, &mut findings);
            if let Some(main) = main {
                run(&mut bundle, &scanned, main, plugin, options, &mut findings);
            }
            bundle.plugin = Some(plugin.clone());
        }
    }
    bundle.diagnostics = findings.out;
    bundle
}

/// What the lexer left of each file beside its import sites, by the file's index.
struct Scanned<'a> {
    /// The `eval(` / `Function(` call tokens and the `import.meta` member reads.
    tokens: &'a [(Vec<ScriptPosition>, Vec<ScriptPosition>)],
    /// The text of the files the lexer scanned whole.
    #[cfg(feature = "script-syntax")]
    sources: &'a [Option<Box<str>>],
}

/// A `.js` file outside `scripts/` is in no file set, whether the pack's plugin starts or
/// is dropped.
fn not_loadable(outside: &[PackPath], findings: &mut Findings) {
    for path in outside {
        findings.push(
            DiagnosticCode::from(&core_codes::SCRIPT_FILE_NOT_LOADABLE),
            Severity::Info,
            Some(path),
            None,
        );
    }
}

/// The rules of a bundle whose module vanilla starts, from the main file at `main`.
fn run(
    bundle: &mut ScriptBundle,
    scanned: &Scanned<'_>,
    main: usize,
    host: &dyn ScriptHost,
    options: ScanOptions,
    findings: &mut Findings,
) {
    if !options.imports {
        // The caller asked for no import checks, so the bundle says that nothing was checked.
        findings.push(
            DiagnosticCode::from(&core_codes::SCRIPTS_NOT_CHECKED),
            Severity::Info,
            None,
            None,
        );
        return;
    }
    let reach = resolve(bundle, main, host);
    #[cfg(feature = "script-syntax")]
    {
        let kept = parse_reached(bundle, scanned.sources, &reach);
        if kept > 0 {
            // One note for the bundle: `ScriptContent::parsed` says which files.
            findings.push(
                DiagnosticCode::from(
                    core_codes::SCRIPT_SYNTAX_NOT_CHECKED.with([kept.to_string()]),
                ),
                Severity::Info,
                None,
                None,
            );
        }
    }
    report(bundle, scanned.tokens, &reach, host, findings);
    let name = |index: &usize| bundle.files.get_index(*index).map(|(name, _)| name.clone());
    bundle.graph = Some(ImportGraph {
        reachable_static: reach.statically.iter().filter_map(name).collect(),
        reachable_dynamic: reach.dynamically.iter().filter_map(name).collect(),
    });
}

/// The entry rules for a host that is no discovered plugin: sets `bundle.entry`
/// and gives the index of the main file when the plugin starts.
// The extension test comes first, then whether the main file is among the members. On a
// stack discovery raises both findings once (`load.rs`), so they are not repeated there.
fn entry(
    bundle: &mut ScriptBundle,
    host: &dyn ScriptHost,
    findings: &mut Findings,
) -> Option<usize> {
    let not_supported = |entry: &str| {
        [
            host.plugin().to_owned(),
            entry.to_owned(),
            "JavaScript".to_owned(),
        ]
    };
    let Some(raw) = host.entry() else {
        // No entry at all: vanilla prints the empty entry.
        findings.push(
            DiagnosticCode::from(
                codes::PLUGIN_ENTRY_NOT_SUPPORTED_MISSING_REQUIRED_FIELD.with(not_supported("")),
            ),
            Severity::Error,
            None,
            None,
        );
        return None;
    };
    let entry = EntryRef::of(raw);
    let main = entry.main.clone();
    let prefix_stripped = entry.prefix_stripped;
    let found = bundle.files.get_index_of(&main);
    bundle.entry = Some(entry);
    // The extension of `entry` is not the three bytes `.js`.
    if extension(&raw.replace('\\', "/")) != ".js" {
        findings.push(
            DiagnosticCode::from(
                codes::PLUGIN_ENTRY_NOT_SUPPORTED_INVALID_VALUE.with(not_supported(raw)),
            ),
            Severity::Error,
            None,
            None,
        );
        return None;
    }
    let Some(found) = found else {
        // The main file is missing, with the file that differs only in case when there is one.
        let mut finding = findings.make(
            DiagnosticCode::from(
                codes::PLUGIN_MISSING_MAIN_FILE
                    .with([host.plugin().to_string(), main.as_str().to_string()]),
            ),
            Severity::Error,
            None,
            None,
        );
        finding.detail = Some(Box::new(missing_main(bundle.files.keys(), &main)));
        findings.out.push(finding);
        return None;
    };
    if !prefix_stripped {
        entry_outside_root(findings);
    }
    Some(found)
}

/// Vanilla takes an entry without the `scripts/` prefix, and says so in the engine log
/// only, so the crate reports it as an information.
fn entry_outside_root(findings: &mut Findings) {
    findings.push(
        DiagnosticCode::from(&core_codes::SCRIPT_ENTRY_OUTSIDE_ROOT),
        Severity::Info,
        None,
        None,
    );
}

/// What the bundle adds to the finding of a main file that is no script file of `names`
/// (the missing main file): the file asked for and, when one differs from it only in case,
/// that file as the hint.
pub(super) fn missing_main<'n>(
    names: impl IntoIterator<Item = &'n ScriptPath>,
    main: &ScriptPath,
) -> DiagnosticDetail {
    let near = Lowered::of(names).get(main.as_str()).cloned();
    DiagnosticDetail::Reference {
        kind: "script file",
        id: main.as_str().to_owned(),
        hint: near.map(|near| ReferenceHint::CaseMismatch(near.as_str().to_owned())),
    }
}

/// The file names in lower case, for the `CaseMismatch` hint: of two names that differ only in
/// case the first is kept.
struct Lowered(HashMap<String, ScriptPath>);

impl Lowered {
    fn of<'n>(files: impl IntoIterator<Item = &'n ScriptPath>) -> Self {
        let mut names = HashMap::new();
        for name in files {
            names
                .entry(name.as_str().to_lowercase())
                .or_insert_with(|| name.clone());
        }
        Self(names)
    }

    fn get(&self, name: &str) -> Option<&ScriptPath> {
        self.0.get(&name.to_lowercase())
    }
}

/// Which files the entry reaches, by index, and the sites whose normalised name overflowed.
struct Reach {
    statically: BTreeSet<usize>,
    dynamically: BTreeSet<usize>,
    /// `(file, site)` of the imports whose path was 1024 bytes or more.
    overflowed: BTreeSet<(usize, usize)>,
}

/// Resolves the imports of every file the entry reaches and says which files those are:
/// static imports first, then literal dynamic imports. Crash specifiers are marked in every
/// file.
fn resolve(bundle: &mut ScriptBundle, main: usize, host: &dyn ScriptHost) -> Reach {
    let mut reach = Reach {
        statically: BTreeSet::new(),
        dynamically: BTreeSet::new(),
        overflowed: BTreeSet::new(),
    };
    let known = |name: &str| host.native(name) != NativeModuleAccess::Unknown;
    let mut lowered = None;
    let mut visited = vec![false; bundle.files.len()];
    // Per visited file, the files its literal dynamic imports name.
    let mut later = Vec::new();
    let mut work = vec![main];
    let mut statically = true;
    loop {
        while let Some(index) = work.pop() {
            if std::mem::replace(&mut visited[index], true) {
                continue;
            }
            if statically {
                reach.statically.insert(index);
            } else {
                reach.dynamically.insert(index);
            }
            let Some((referrer, _)) = bundle.files.get_index(index) else {
                continue;
            };
            let referrer = referrer.clone();
            let sites = bundle.files[index].imports().len();
            for site in 0..sites {
                let import = &bundle.files[index].imports()[site];
                if import.kind == ImportKind::DynamicNonLiteral {
                    continue;
                }
                let kind = import.kind;
                let raw = import.raw.clone();
                let normalized = normalize_full(&referrer, &raw, &known);
                let resolution = match normalized {
                    Err(_) => Resolution::CrashesEngine,
                    Ok(normalized) => {
                        if normalized.overflow {
                            reach.overflowed.insert((index, site));
                        }
                        resolution(&bundle.files, &mut lowered, host, &raw, normalized)
                    }
                };
                if let Resolution::File(target) = &resolution
                    && let Some(target) = bundle.files.get_index_of(target)
                {
                    // A file reached only dynamically is linked whole when it is
                    // imported: its static imports stay on the dynamic side.
                    if kind.is_static() {
                        work.push(target);
                    } else {
                        later.push(target);
                    }
                }
                if let Some(content) = bundle.files[index].content.as_mut() {
                    content.imports[site].resolution = resolution;
                }
            }
        }
        if later.is_empty() {
            break;
        }
        statically = false;
        work.append(&mut later);
    }
    // A crash specifier is reported in every file: a crash specifier in a file nothing reaches only waits
    // for a code path that imports it.
    for (index, (referrer, file)) in bundle.files.iter_mut().enumerate() {
        if visited[index] {
            continue;
        }
        let Some(content) = file.content.as_mut() else {
            continue;
        };
        for import in &mut content.imports {
            if import.kind != ImportKind::DynamicNonLiteral
                && normalize_full(referrer, &import.raw, &known).is_err()
            {
                import.resolution = Resolution::CrashesEngine;
            }
        }
    }
    reach
}

/// Gives the parser of `script-syntax` the files the entry reaches: the only files vanilla
/// reads, and so the only ones a syntax error is more than a note in. A file whose nesting
/// could overflow the parser's stack is kept from it (`bound.rs`); returns how many were.
#[cfg(feature = "script-syntax")]
fn parse_reached(bundle: &mut ScriptBundle, sources: &[Option<Box<str>>], reach: &Reach) -> u32 {
    use super::bound::{STACK_BUDGET, parser_stack};
    let mut kept = 0u32;
    for index in reach.statically.iter().chain(&reach.dynamically) {
        let Some(text) = sources.get(*index).and_then(Option::as_deref) else {
            continue;
        };
        let Some(file) = bundle
            .files
            .get_index_mut(*index)
            .and_then(|(_, file)| file.content.as_mut())
        else {
            continue;
        };
        if parser_stack(text, STACK_BUDGET).is_some() {
            file.syntax = super::syntax::first_error(text);
            file.parsed = true;
        } else {
            kept = kept.saturating_add(1);
        }
    }
    kept
}

/// What a normalised specifier names in the bundle.
// The import check, then a native binding of the name, then a file of the
// plugin whose name is byte-equal, then the `ReferenceError`.
fn resolution(
    files: &IndexMap<ScriptPath, ScriptFile>,
    lowered: &mut Option<Lowered>,
    host: &dyn ScriptHost,
    raw: &str,
    normalized: Normalized,
) -> Resolution {
    let Normalized {
        name,
        form,
        appended_js,
        overflow,
    } = normalized;
    if form == Form::Verbatim && !name.is_empty() {
        match host.native(&name) {
            NativeModuleAccess::Available => return Resolution::Native { name },
            NativeModuleAccess::Restricted => return Resolution::Restricted { name },
            NativeModuleAccess::Unknown => {}
        }
    }
    let name = ScriptPath::new(name);
    if files.contains_key(&name) {
        return Resolution::File(name);
    }
    let looked_up = name.0;
    if form == Form::Verbatim && looked_up.starts_with('@') {
        return Resolution::Unrecognized { name: looked_up };
    }
    // What the engine looked up in place of an overlong path says nothing of the specifier.
    let (hint, near) = if overflow {
        (None, None)
    } else {
        hint(files, lowered, raw, &looked_up, form, appended_js)
    };
    Resolution::NotFound {
        looked_up,
        hint,
        near,
    }
}

/// The near miss of a specifier that names no file, and the file it likely meant.
fn hint(
    files: &IndexMap<ScriptPath, ScriptFile>,
    lowered: &mut Option<Lowered>,
    raw: &str,
    looked_up: &str,
    form: Form,
    appended_js: bool,
) -> (Option<NotFoundHint>, Option<ScriptPath>) {
    let file = |name: &str| {
        let name = ScriptPath::new(name);
        files.contains_key(&name).then_some(name)
    };
    if let Some(inner) = looked_up.strip_prefix(ScriptRoot::PREFIX) {
        return (Some(NotFoundHint::ScriptsPrefix), file(inner));
    }
    if form == Form::Escapes {
        return (Some(NotFoundHint::EscapesRoot), None);
    }
    let lowered = lowered.get_or_insert_with(|| Lowered::of(files.keys()));
    if let Some(near) = lowered.get(looked_up) {
        return (Some(NotFoundHint::CaseMismatch), Some(near.clone()));
    }
    if let Some(index) = looked_up
        .strip_suffix(".js")
        .and_then(|folder| file(&format!("{folder}/index.js")))
    {
        return (Some(NotFoundHint::DirectoryIndex), Some(index));
    }
    // Byte for byte, as every name here is: `.JSON` is no `.json`, so `.js` is appended to it.
    if extension(&raw.replace('\\', "/")) == ".json" {
        return (Some(NotFoundHint::ImportedAsJson), None);
    }
    if appended_js && extension(&raw.replace('\\', "/")).len() > 1 {
        return (Some(NotFoundHint::AppendedJs), None);
    }
    if looked_up.contains("/./") || looked_up.contains("/../") {
        return (Some(NotFoundHint::MidPathDots), None);
    }
    if form == Form::Bare && !raw.contains(['/', '\\']) {
        return (Some(NotFoundHint::BarePackage), None);
    }
    (None, None)
}

/// The findings of the files, in name order.
#[allow(
    clippy::too_many_lines,
    reason = "one block per rule, in the order a reader looks them up"
)]
fn report(
    bundle: &ScriptBundle,
    tokens: &[(Vec<ScriptPosition>, Vec<ScriptPosition>)],
    reach: &Reach,
    host: &dyn ScriptHost,
    findings: &mut Findings,
) {
    for (index, (name, file)) in bundle.files.iter().enumerate() {
        let Some(content) = &file.content else {
            continue;
        };
        let statically = reach.statically.contains(&index);
        let reached = statically || reach.dynamically.contains(&index);
        let path = Some(&file.path);
        // What is wrong with the file as a whole: load-fatal when the entry links it; a
        // rejected promise at run time when only an `import()` reaches it, nothing at load,
        // so that case is an information; harmless otherwise.
        let whole = if statically {
            Severity::Error
        } else {
            Severity::Info
        };
        if let Err(error) = content.utf8 {
            // An error for a file the entry reaches, an information for one it does not.
            let severity = if reached {
                Severity::Error
            } else {
                Severity::Info
            };
            let earlier = content
                .syntax
                .as_ref()
                .is_some_and(|syntax| syntax.at.offset < error.offset);
            let file = name.as_str().to_owned();
            let line = error.line.to_string();
            let code = if !reached {
                // The engine never reads the file, so it prints nothing about it.
                codes::UNREAD_NOT_UTF8.with([file, line])
            } else if earlier {
                // The engine stops at the file's first error, which is the one before.
                codes::NOT_UTF8_AFTER_ERROR.with([file, line])
            } else {
                // What the engine prints when it loads the file.
                codes::SCRIPT_INVALID_UTF8.with([file, line])
            };
            findings.push(
                DiagnosticCode::from(code),
                severity,
                path,
                Some(error.offset),
            );
        }
        if let Some(syntax) = &content.syntax {
            // The message is `QuickJS`'s own, which nothing here reproduces.
            findings.push(
                DiagnosticCode::from(&core_codes::SCRIPT_SYNTAX_ERROR),
                whole,
                path,
                Some(syntax.at.offset),
            );
        }
        for (site, import) in content.imports.iter().enumerate() {
            let at = Some(import.offset);
            if import.resolution == Resolution::CrashesEngine {
                // A crash specifier is reported in every file.
                findings.push(
                    DiagnosticCode::from(&core_codes::SCRIPT_IMPORT_CRASHES_ENGINE),
                    Severity::Error,
                    path,
                    at,
                );
                continue;
            }
            if !reached {
                continue;
            }
            // A static import of a file the entry links fails the plugin; anything else
            // rejects one promise at run time, nothing at load, so it is an information.
            let severity = if statically && import.kind.is_static() {
                Severity::Error
            } else {
                Severity::Info
            };
            if import.attributes {
                // The engine's `QuickJS` has no import attributes and wants the
                // statement to end after the specifier.
                findings.push(
                    DiagnosticCode::from(
                        codes::SCRIPT_EXPECTING_SEMICOLON
                            .with([name.as_str().to_string(), import.line.to_string()]),
                    ),
                    whole,
                    path,
                    at,
                );
            }
            match &import.resolution {
                Resolution::NotFound {
                    looked_up, hint, ..
                } => {
                    // A path the engine's buffer does not hold fails to be looked up just as a
                    // missing file does, so both get the same severity: an error for a static
                    // import of a statically reached file ("ReferenceError: Import [..] not
                    // found."), an information otherwise.
                    let _ = reach.overflowed.contains(&(index, site));
                    let mut finding = findings.make(
                        DiagnosticCode::from(
                            core_codes::SCRIPT_IMPORT_NOT_FOUND.with([looked_up.clone()]),
                        ),
                        severity,
                        path,
                        at,
                    );
                    finding.detail = hint.map(|hint| Box::new(DiagnosticDetail::NotFound(hint)));
                    findings.out.push(finding);
                }
                // An `@…` name that is neither a native module of the context nor a file.
                Resolution::Unrecognized { name: module } => {
                    findings.push(
                        DiagnosticCode::from(
                            core_codes::SCRIPT_MODULE_UNRECOGNIZED.with([module.clone()]),
                        ),
                        severity,
                        path,
                        at,
                    );
                }
                // A native module no pack file may import.
                Resolution::Restricted { name: module } => {
                    findings.push(
                        DiagnosticCode::from(
                            core_codes::SCRIPT_MODULE_RESTRICTED
                                .with([module.clone(), name.as_str().to_owned()]),
                        ),
                        severity,
                        path,
                        at,
                    );
                }
                // A dynamic import whose specifier is not a literal cannot be checked.
                Resolution::NotChecked if import.kind == ImportKind::DynamicNonLiteral => {
                    findings.push(
                        DiagnosticCode::from(&core_codes::SCRIPT_IMPORT_NOT_CHECKED),
                        Severity::Info,
                        path,
                        at,
                    );
                }
                _ => {}
            }
        }
        if !reached {
            continue;
        }
        let (evals, metas) = &tokens[index];
        if !host.script_eval() {
            // `eval` and `Function` without the capability fail at the call, not at load, so
            // the crate reports them as an information.
            for at in evals {
                findings.push(
                    DiagnosticCode::from(&core_codes::SCRIPT_EVAL_WITHOUT_CAPABILITY),
                    Severity::Info,
                    path,
                    Some(at.offset),
                );
            }
        }
        // `import.meta` is empty in the game, which writes no line for it, so the crate
        // reports it as an information.
        for at in metas {
            findings.push(
                DiagnosticCode::from(&core_codes::SCRIPT_IMPORT_META_EMPTY),
                Severity::Info,
                path,
                Some(at.offset),
            );
        }
    }
}
