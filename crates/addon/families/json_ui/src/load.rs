//! Loading JSON UI: `Content` of the file kinds, the walk of `ui/`, and the stack's merge:
//! the united lists, each listed file merged from the lowest pack up, the global variables,
//! the definition table and the checks over it.

use std::collections::HashSet;

use bedrock_addon_core::load::{
    AppDataPlace, CollectionLoader, ListDepth, LoadContext, StackCx, Walk, WalkFile, WalkOrder,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use indexmap::IndexMap;

use super::globals::{GlobalEvent, merge_globals};
use super::merge::{CopyContent, Event, FileCopy, merge_file};
use super::resolve::{ResolveEvent, resolve};
use super::value::{member, string_or_empty};
use super::{
    DefKey, GLOBAL_VARIABLES_PATH, GlobalVariables, UI_DEFS_PATH, UiDefinitions, UiDefsList,
    UiFile, UiPackError, UiPackErrorKind, UiPackReport, lint, write,
};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticDetail, Diagnostics, JsonPath, Location, Phase, Severity,
    VanillaText,
};
use bedrock_addon_core::document::{
    Content, Document, Merge, Merged, Origin, Placed, RawLayer, Rejected, VersionInfo,
};
use bedrock_addon_core::json::{EncodeError, Value, WriteStyle, write_json};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_support::jsoncpp::read;

/// The directory the listed files live in.
const UI: &str = "ui";

/// The text of a file that is not JSON: the path and the reader's formatted errors.
const PARSE_ERROR: &str = "JSON Parse error in file: {}\n\n{}\n";
/// The per-file task's text for a listed file no pack has.
const NOT_LOADED: &str = "The following file could not be loaded: {}. Make sure the file exists or remove the reference in '_ui_defs.json'";
/// The text for empty or repeated child names: the names one per line, then the path.
const INVALID_NAMES: &str = "We have invalid names in the resource files. Make sure all your control children have unique names that aren't empty!\n\nControls:\n{}\nFile name:{}";
/// The text for a global variables file at the top of the stack that does not parse.
const GLOBALS_NOT_PARSED: &str = "Could not parse file at the top of the stack: {}.";

impl bedrock_addon_core::__private::Sealed for UiDefinitions {}
impl bedrock_addon_core::__private::Sealed for UiFile {}
impl bedrock_addon_core::__private::Sealed for UiDefsList {}
impl bedrock_addon_core::__private::Sealed for GlobalVariables {}

/// Keyed by the path of a read file (`ui/hud_screen.json`, `ui/_global_variables.json`):
/// `Merged::origin` names the highest pack whose copy is part of the merged file, with the
/// lower ones as contributors.
impl Merge for UiDefinitions {
    type Key = String;
}

impl UiFile {
    /// A file from its root object: `namespace` and the definitions. The flag says the
    /// `namespace` member is not a string (it is then kept among the definitions).
    pub(crate) fn from_value(root: &Value) -> Option<(Self, bool)> {
        let root = root.as_object()?;
        let mut file = Self::default();
        let mut bad_namespace = false;
        for (key, value) in root {
            if key.as_str() == "namespace" && !value.is_null() {
                match value.as_string() {
                    Some(namespace) => {
                        file.namespace = Some(namespace.as_str().to_owned());
                        continue;
                    }
                    None => bad_namespace = true,
                }
            }
            file.defs.insert(DefKey::parse(key.as_str()), value.clone());
        }
        Some((file, bad_namespace))
    }
}

/// The origin of a file read on its own: no pack.
fn lone(path: &PackPath) -> Origin {
    Origin::new(
        PackRef::app_data(),
        PackSide::Resource,
        path.clone(),
        None,
        Vec::new(),
    )
}

/// A document of this unversioned family: no file carries a `format_version`.
fn document<T>(value: T, origin: Origin, mut diagnostics: Diagnostics) -> Document<T> {
    locate(&mut diagnostics, &origin);
    let none = FormatVersion::new(0, 0, 0);
    Document::new(
        value,
        origin,
        VersionInfo::new(None, false, none, none, false),
        diagnostics,
    )
}

/// A rejection for a file that is not the JSON object it has to be.
fn reject(origin: Origin, message: String) -> Rejected {
    let mut diagnostics = Diagnostics::default();
    diagnostics.push(Diagnostic::new(
        DiagnosticCode::MalformedInput {
            message,
            text: None,
        },
        Severity::Error,
        Phase::Parse,
        Location::default(),
    ));
    locate(&mut diagnostics, &origin);
    Rejected::new(origin, diagnostics)
}

/// Gives every finding the document's pack and file.
fn locate(diagnostics: &mut Diagnostics, origin: &Origin) {
    let found = std::mem::take(diagnostics);
    for mut finding in found {
        finding.location.pack = Some(origin.pack.clone());
        finding.location.path = Some(origin.path.clone());
        diagnostics.push(finding);
    }
}

/// The root object of a file, or the rejection.
fn root_object(origin: &Origin, bytes: &[u8]) -> Result<Value, Rejected> {
    let root = read(bytes).map_err(|message| reject(origin.clone(), message.to_string()))?;
    if root.is_object() {
        Ok(root)
    } else {
        Err(reject(
            origin.clone(),
            "the root is not an object".to_owned(),
        ))
    }
}

fn json_text(value: &Value, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
    write_json(value, out, style)
}

fn json_string(value: &Value) -> Result<String, EncodeError> {
    let mut out = Vec::new();
    json_text(value, &mut out, WriteStyle::default())?;
    Ok(String::from_utf8(out).unwrap_or_default())
}

/// A lint of the crate's own about one file: an information, because the game says nothing about it.
fn lint(code: DiagnosticCode, path: &[&str]) -> Diagnostic {
    Diagnostic::new(
        code,
        Severity::Info,
        Phase::Parse,
        Location::new(None, None, json_path(path), None),
    )
}

impl Content for UiFile {
    const NAME: &'static str = "ui file";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// Reads one UI file on its own: its namespace and its definitions, as written. The
    /// checks of the merge and of control creation belong to the stack ([`UiDefinitions`]).
    fn parse(path: &PackPath, bytes: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        let origin = lone(path);
        let mut root = root_object(&origin, bytes)?;
        let mut diagnostics = Diagnostics::default();
        // A number no double holds is read as the largest finite one, with a warning.
        bedrock_addon_core::json::clamp_non_finite_warned(
            &mut root,
            None,
            &JsonPath::new(),
            &mut diagnostics,
        );
        let (file, bad_namespace) = Self::from_value(&root).unwrap_or_default();
        if bad_namespace {
            // The game converts the value to a string; the crate puts the file in `_root`
            // instead and says so.
            diagnostics.push(lint(
                DiagnosticCode::InvalidType { text: None },
                &["namespace"],
            ));
        }
        Ok(document(file, origin, diagnostics))
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        json_text(&write::file_value(self), out, style)
    }

    fn to_json(&self) -> Result<String, EncodeError> {
        json_string(&write::file_value(self))
    }
}

impl Content for UiDefsList {
    const NAME: &'static str = "ui definition list";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// Reads one `ui/_ui_defs.json`.
    fn parse(path: &PackPath, bytes: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        let origin = lone(path);
        let mut root = root_object(&origin, bytes)?;
        let mut list = Self::default();
        let mut diagnostics = Diagnostics::default();
        // A number no double holds is read as the largest finite one, with a warning.
        bedrock_addon_core::json::clamp_non_finite_warned(
            &mut root,
            None,
            &JsonPath::new(),
            &mut diagnostics,
        );
        for (key, value) in root
            .as_object()
            .into_iter()
            .flat_map(|object| object.iter())
        {
            if key.as_str() != "ui_defs" {
                list.extra.insert(key.as_str().to_owned(), value.clone());
                continue;
            }
            list.ui_defs = write::read_ui_defs(value);
            if !value.is_array() {
                // The game reads no path from a list that is not an array; the crate says so.
                diagnostics.push(lint(
                    DiagnosticCode::InvalidType { text: None },
                    &["ui_defs"],
                ));
            }
            // What the typed list cannot say (no array, an element that is no string, an
            // empty array against no member at all) is kept as written beside it.
            let exact = value.as_array().is_some_and(|paths| {
                !paths.is_empty() && paths.as_slice().iter().all(Value::is_string)
            });
            if !exact {
                list.extra.insert(key.as_str().to_owned(), value.clone());
            }
        }
        Ok(document(list, origin, diagnostics))
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        json_text(&write::defs_list_value(self), out, style)
    }

    fn to_json(&self) -> Result<String, EncodeError> {
        json_string(&write::defs_list_value(self))
    }
}

/// The paths of a `ui_defs` array: each element through `asString("")`.
pub(super) fn listed(paths: &[Value]) -> Vec<String> {
    paths
        .iter()
        .map(|path| string_or_empty(Some(path)).to_owned())
        .collect()
}

impl Placed for UiDefsList {
    /// `ui/_ui_defs.json`: the one path the game reads the list from.
    fn default_path(&self) -> PackPath {
        PackPath::from_static(UI_DEFS_PATH)
    }
}

impl Content for GlobalVariables {
    const NAME: &'static str = "ui global variables";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// Reads one `ui/_global_variables.json`.
    fn parse(path: &PackPath, bytes: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        let origin = lone(path);
        let mut root = root_object(&origin, bytes)?;
        let mut diagnostics = Diagnostics::default();
        // A number no double holds is read as the largest finite one, with a warning.
        bedrock_addon_core::json::clamp_non_finite_warned(
            &mut root,
            None,
            &JsonPath::new(),
            &mut diagnostics,
        );
        Ok(document(Self::from_value(&root), origin, diagnostics))
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        json_text(&write::globals_value(self), out, style)
    }

    fn to_json(&self) -> Result<String, EncodeError> {
        json_string(&write::globals_value(self))
    }
}

impl Placed for GlobalVariables {
    /// `ui/_global_variables.json`: the one path the game reads the variables from.
    fn default_path(&self) -> PackPath {
        PackPath::from_static(GLOBAL_VARIABLES_PATH)
    }
}

// ---- the stack ------------------------------------------------------------------------------

/// What the `ui()` line reads of one file below `ui/`.
#[derive(Debug)]
pub struct UiCopy {
    origin: Origin,
    /// The path the stack keys the file by (no subpack folder).
    key: String,
    content: CopyContent,
    /// What reading the copy found, in its pack's file: the warnings of the numbers no
    /// double holds, each read as the largest finite one.
    found: Diagnostics,
}

/// Whether a path names a JSON file: the files below `ui/` a list may name and have read. A
/// picture or an archive left there is never read.
fn is_json(path: &str) -> bool {
    path.rsplit_once('.')
        .is_some_and(|(_, extension)| extension.eq_ignore_ascii_case("json"))
}

/// Whether the loader reads the file a list names by `path`: a JSON file below `ui/`: the game reads only `ui/` for JSON UI.
fn is_read(path: &str) -> bool {
    path.starts_with("ui/") && is_json(path)
}

/// The two files the game reads by their fixed paths: the list and the global variables.
fn fixed_file(path: &PackPath) -> bool {
    matches!(path.as_str(), UI_DEFS_PATH | GLOBAL_VARIABLES_PATH)
}

/// The second walk stage: the files the lists of the stack name. Every pack's copy of
/// `ui/_ui_defs.json` that is JSON counts, as in the merge; the loader reads each named file
/// from every pack that has it.
// The only enumeration of files is the `ui_defs` list.
fn listed_files(parsed: &[&UiCopy]) -> Vec<PackPath> {
    parsed
        .iter()
        .filter(|copy| copy.key == UI_DEFS_PATH)
        .filter_map(|copy| match &copy.content {
            CopyContent::Json(json) => member(json, "ui_defs").and_then(Value::as_array),
            _ => None,
        })
        .flat_map(|paths| listed(paths.as_slice()))
        .filter(|path| is_read(path))
        .filter_map(|path| PackPath::new(&path).ok())
        .collect()
}

/// Whether a pack of the stack has a JSON file below `ui/`, read or not: a stack without one
/// has no JSON UI to talk about. Decided from the packs' listings, bytes unread.
fn has_ui_file(stack: &StackCx<'_>) -> bool {
    stack.packs.iter().any(|pack| {
        let mut below = pack.files_below(UI).iter();
        below.any(|file| is_json(file.key.as_str()))
    })
}

/// The `Parse` job: the file's JSON, or why it is none.
fn parse(file: &WalkFile, _: &LoadContext) -> UiCopy {
    let origin = Origin::new(
        file.pack.clone(),
        file.side,
        file.path.clone(),
        None,
        Vec::new(),
    );
    // The copy is read with the numbers no double holds clamped, so no layer, file,
    // variable or control of the merge keeps an infinity; the warning names this pack's file.
    let finite = bedrock_addon_core::document::FiniteText::of(&file.bytes);
    let content = if file.bytes.is_empty() {
        CopyContent::Empty
    } else {
        match read(finite.bytes()) {
            Ok(json) => CopyContent::Json(json),
            Err(message) => CopyContent::Malformed(message.to_string()),
        }
    };
    let mut found = Diagnostics::default();
    finite.settle(&mut found, Some(&origin));
    UiCopy {
        origin,
        key: file.key.as_str().to_owned(),
        content,
        found,
    }
}

/// The loader of `ui()`: the list and the global variables of every resource pack, lowest
/// pack first, then the files the lists name (the second walk stage). A file below
/// `ui/` that no list names is never asked for.
// The game hands the merge the copies of a file lowest priority first.
pub const fn load() -> CollectionLoader<Merged<UiDefinitions, RawLayer>, UiCopy> {
    CollectionLoader {
        walk: Walk {
            directories: &[UI],
            depth: ListDepth::Recursive,
            accept: fixed_file,
            order: WalkOrder::BottomUp,
            directory_major: false,
            app_data: &[],
            app_data_place: AppDataPlace::After,
            in_pack: None,
            shadowed: None,
            read_error: None,
        },
        parse,
        combine,
        needs: &[],
        then: Some(listed_files),
    }
}

fn json_path(segments: &[&str]) -> JsonPath {
    let mut path = JsonPath::new();
    for segment in segments {
        match segment.parse::<usize>() {
            Ok(index) if !segment.is_empty() => path.push_index(index),
            _ => path.push(segment),
        }
    }
    path
}

/// The stack's findings and pack reports as they accumulate.
#[derive(Default)]
struct Report {
    diagnostics: Diagnostics,
    reports: IndexMap<PackRef, Vec<UiPackError>>,
}

impl Report {
    fn push(
        &mut self,
        code: DiagnosticCode,
        severity: Severity,
        origin: Option<&Origin>,
        path: &[&str],
        detail: Option<DiagnosticDetail>,
    ) {
        let location = Location::new(
            origin.map(|origin| origin.pack.clone()),
            origin.map(|origin| origin.path.clone()),
            json_path(path),
            None,
        );
        let phase = if matches!(code, DiagnosticCode::MalformedInput { .. }) {
            Phase::Parse
        } else {
            Phase::Validate
        };
        let mut diagnostic = Diagnostic::new(code, severity, phase, location);
        diagnostic.detail = detail.map(Box::new);
        self.diagnostics.push(diagnostic);
    }

    /// A content-log error after which the game goes on with the file: a kept error.
    fn push_kept(&mut self, code: DiagnosticCode, origin: Option<&Origin>) {
        let location = Location::new(
            origin.map(|origin| origin.pack.clone()),
            origin.map(|origin| origin.path.clone()),
            json_path(&[]),
            None,
        );
        let diagnostic = Diagnostic::new(code, Severity::Error, Phase::Validate, location);
        self.diagnostics.push_kept_error(diagnostic);
    }

    /// A lint of the crate's own (vanilla is silent): an information.
    fn lint(&mut self, code: DiagnosticCode, origin: Option<&Origin>, path: &[&str]) {
        self.push(code, Severity::Info, origin, path, None);
    }

    /// One `uiPackError`: into the pack's report and the findings.
    fn pack_error(&mut self, error: UiPackError, origin: &Origin) {
        let text = Some(VanillaText::Formatted {
            template: error.kind.template(),
            args: error.args.clone(),
        });
        // A pack error has no code of its own: each is reported under the code of its nature.
        let code = match error.kind {
            UiPackErrorKind::InvalidChildNames => DiagnosticCode::InvalidValue { text },
            UiPackErrorKind::ParseError => DiagnosticCode::MalformedInput {
                message: error.args.get(1).cloned().unwrap_or_default(),
                text,
            },
            UiPackErrorKind::MissingControl | UiPackErrorKind::MissingControlTarget => {
                DiagnosticCode::UnresolvedReference { text }
            }
            UiPackErrorKind::MissingArrayName
            | UiPackErrorKind::MissingCondition
            | UiPackErrorKind::MissingValue
            | UiPackErrorKind::MissingOperation => DiagnosticCode::MissingRequiredField { text },
            UiPackErrorKind::InvalidOperationName => DiagnosticCode::InvalidEnumValue { text },
        };
        // The overlay key, when the text has one, is where the finding is.
        let key = match error.kind {
            UiPackErrorKind::InvalidChildNames | UiPackErrorKind::ParseError => None,
            _ => error.args.get(1).cloned(),
        };
        let path: Vec<&str> = key.as_deref().into_iter().collect();
        self.push(code, error.severity, Some(origin), &path, None);
        self.reports
            .entry(origin.pack.clone())
            .or_default()
            .push(error);
    }

    /// A file that is not JSON, with vanilla's text for it.
    fn malformed(
        &mut self,
        template: &'static str,
        args: Vec<String>,
        message: String,
        origin: &Origin,
    ) {
        self.push(
            DiagnosticCode::MalformedInput {
                message,
                text: Some(VanillaText::Formatted { template, args }),
            },
            Severity::Error,
            Some(origin),
            &[],
            None,
        );
    }
}

/// The stack position of a pack: the top pack first, the app data last.
fn rank(pack: &PackRef) -> usize {
    pack.index().unwrap_or(usize::MAX)
}

/// The reader's message of a copy that is not JSON.
fn message_of(content: &CopyContent) -> Option<String> {
    match content {
        CopyContent::Json(_) => None,
        CopyContent::Empty => Some("empty document".to_owned()),
        CopyContent::Malformed(message) => Some(message.clone()),
    }
}

/// The `Combine` job: see the module documentation.
fn combine(parsed: Vec<UiCopy>, stack: &StackCx<'_>) -> Merged<UiDefinitions, RawLayer> {
    let has_ui = !parsed.is_empty() || has_ui_file(stack);
    merge_stack(parsed, has_ui)
}

/// The whole merge of a stack's `ui/` files, given lowest pack first: the fixed files, then
/// the listed ones. `has_ui` says whether a pack of the stack has a JSON file below `ui/`: a
/// stack without one has no JSON UI to talk about.
#[allow(clippy::too_many_lines, reason = "the load order, step by step")]
fn merge_stack(parsed: Vec<UiCopy>, has_ui: bool) -> Merged<UiDefinitions, RawLayer> {
    // A subpack's file replaces its pack's file of the same key.
    let mut by_path: IndexMap<String, Vec<UiCopy>> = IndexMap::new();
    for copy in parsed {
        let copies = by_path.entry(copy.key.clone()).or_default();
        match copies.last_mut() {
            Some(last) if last.origin.pack == copy.origin.pack => *last = copy,
            _ => copies.push(copy),
        }
    }
    let mut report = Report::default();
    // What reading each copy that takes part found, lowest pack first.
    for copy in by_path.values_mut().flatten() {
        report.diagnostics.extend(std::mem::take(&mut copy.found));
    }
    let mut layers: Vec<RawLayer> = Vec::new();
    let mut origins: IndexMap<String, Origin> = IndexMap::new();
    let mut value = UiDefinitions::default();

    // The union of every pack's list, sorted, each path once.
    let mut paths: Vec<String> = Vec::new();
    // Each listed path with the list that names it and its index there (of several, the
    // highest pack's last).
    let mut listers: IndexMap<String, (Origin, usize)> = IndexMap::new();
    let lists = by_path.shift_remove(UI_DEFS_PATH).unwrap_or_default();
    for copy in &lists {
        match &copy.content {
            CopyContent::Json(json) => {
                let listed_here = member(json, "ui_defs")
                    .and_then(Value::as_array)
                    .map(|listed_here| listed(listed_here.as_slice()))
                    .unwrap_or_default();
                for (index, path) in listed_here.into_iter().enumerate() {
                    listers.insert(path.clone(), (copy.origin.clone(), index));
                    paths.push(path);
                }
                layers.push(RawLayer::new(copy.origin.clone(), json.clone()));
            }
            // That pack's list is skipped; the others still contribute.
            content => {
                let message = message_of(content).unwrap_or_default();
                let args = vec![copy.origin.path.to_string(), message.clone()];
                report.malformed(PARSE_ERROR, args, message, &copy.origin);
            }
        }
    }
    paths.sort();
    paths.dedup();
    // A list may name any file, the two fixed ones too: the game loads every listed path
    // as a UI file, whatever else it reads it as.
    let is_listed = |fixed: &str| {
        paths
            .binary_search_by(|path| path.as_str().cmp(fixed))
            .is_ok()
    };
    if is_listed(UI_DEFS_PATH) && !lists.is_empty() {
        by_path.insert(UI_DEFS_PATH.to_owned(), lists);
    }
    let globals_listed = is_listed(GLOBAL_VARIABLES_PATH);
    if paths.is_empty() && has_ui {
        // No pack lists a file although the stack has JSON UI files.
        report.push(
            DiagnosticCode::NoUiDefinitions,
            Severity::Info,
            None,
            &[],
            None,
        );
    }

    // The global variables.
    let globals = by_path
        .shift_remove(GLOBAL_VARIABLES_PATH)
        .unwrap_or_default();
    let global_origins: Vec<Origin> = globals.iter().map(|copy| copy.origin.clone()).collect();
    let mut inputs = Vec::new();
    let mut listed_globals = Vec::new();
    for (slot, copy) in globals.into_iter().enumerate() {
        if let CopyContent::Json(json) = &copy.content {
            layers.push(RawLayer::new(copy.origin.clone(), json.clone()));
        }
        if globals_listed {
            listed_globals.push(UiCopy {
                origin: copy.origin,
                key: copy.key,
                content: copy.content.clone(),
                found: Diagnostics::default(),
            });
        }
        inputs.push((slot, copy.content));
    }
    if !listed_globals.is_empty() {
        by_path.insert(GLOBAL_VARIABLES_PATH.to_owned(), listed_globals);
    }
    let (root, merged, raised) = merge_globals(inputs);
    for (slot, event) in raised {
        let origin = &global_origins[slot];
        match event {
            // The lowest copy of the variables does not parse.
            GlobalEvent::BaseMalformed(message) => report.malformed(
                GLOBALS_NOT_PARSED,
                vec![origin.path.to_string()],
                message,
                origin,
            ),
            // Silent in vanilla.
            GlobalEvent::OverlayMalformed(message) => report.lint(
                DiagnosticCode::MalformedInput {
                    message,
                    text: None,
                },
                Some(origin),
                &[],
            ),
        }
    }
    value.global_variables = GlobalVariables::from_value(&root);
    if let Some(origin) = merged_origin(&global_origins, &merged) {
        origins.insert(GLOBAL_VARIABLES_PATH.to_owned(), origin);
    }

    // Each listed file, merged from the lowest pack up.
    for path in &paths {
        let Some(copies) = by_path.shift_remove(path) else {
            let (lister, index) = match listers.get(path) {
                Some((lister, index)) => (Some(lister), index.to_string()),
                None => (None, String::new()),
            };
            // At the element of the list that names the file.
            let at: &[&str] = if lister.is_some() {
                &["ui_defs", &index]
            } else {
                &["ui_defs"]
            };
            if path.is_empty() || is_read(path) {
                // The game's line for a listed file no pack has goes to the developer log
                // only, not the content log: an information that carries its text.
                report.push(
                    DiagnosticCode::UnresolvedReference {
                        text: Some(VanillaText::Formatted {
                            template: NOT_LOADED,
                            args: vec![path.clone()],
                        }),
                    },
                    Severity::Info,
                    lister,
                    at,
                    None,
                );
            } else {
                // Only the JSON files of `ui/` are read, so a path elsewhere or of another
                // kind is never asked for; the crate says so.
                report.lint(DiagnosticCode::FileNotLoaded, lister, at);
            }
            continue;
        };
        // The two fixed files have their layers already.
        let fixed = path == UI_DEFS_PATH || path == GLOBAL_VARIABLES_PATH;
        let file_origins: Vec<Origin> = copies.iter().map(|copy| copy.origin.clone()).collect();
        let mut inputs = Vec::new();
        for (slot, copy) in copies.into_iter().enumerate() {
            if let CopyContent::Json(json) = &copy.content
                && !fixed
            {
                layers.push(RawLayer::new(copy.origin, json.clone()));
            }
            inputs.push(FileCopy {
                slot,
                content: copy.content,
            });
        }
        let merged = merge_file(path, inputs);
        for raised in merged.raised {
            let origin = &file_origins[raised.slot];
            match raised.event {
                // The base copy does not parse.
                Event::BaseMalformed(message) => report.malformed(
                    PARSE_ERROR,
                    vec![origin.path.to_string(), message.clone()],
                    message,
                    origin,
                ),
                // Empty or repeated child names in the base copy are an error in the game's
                // content log, and the merge goes on with the file: a kept error.
                Event::BaseInvalidNames(names) => report.push_kept(
                    DiagnosticCode::InvalidValue {
                        text: Some(VanillaText::Formatted {
                            template: INVALID_NAMES,
                            args: vec![names.join("\n"), path.clone()],
                        }),
                    },
                    Some(origin),
                ),
                Event::Pack(error) => report.pack_error(error, origin),
                // Silent in vanilla: an overlay member that is no object, and `modifications`
                // below a control's top level.
                Event::IgnoredValue(key) => report.lint(
                    DiagnosticCode::IgnoredMember { text: None },
                    Some(origin),
                    &[&key],
                ),
                Event::NestedModifications(key) => report.lint(
                    DiagnosticCode::IgnoredMember { text: None },
                    Some(origin),
                    &[&key, "modifications"],
                ),
                // An overlay `name@base` key whose short name the base already has.
                Event::Rebased(key) => {
                    report.lint(DiagnosticCode::RebasedOverride, Some(origin), &[&key]);
                }
                // A condition that matches nothing is silent in vanilla.
                Event::Unmatched(key) => report.lint(
                    DiagnosticCode::UnmatchedCondition,
                    Some(origin),
                    &[&key, "modifications"],
                ),
                Event::WhereArray(key) => report.lint(
                    DiagnosticCode::WhereArrayNeverMatches,
                    Some(origin),
                    &[&key, "modifications"],
                ),
            }
        }
        let Some(root) = merged.root else {
            continue;
        };
        match UiFile::from_value(&root) {
            Some((file, bad_namespace)) => {
                let origin = merged_origin(&file_origins, &merged.committed);
                if bad_namespace {
                    report.lint(
                        DiagnosticCode::InvalidType { text: None },
                        origin.as_ref(),
                        &["namespace"],
                    );
                }
                if let Some(origin) = origin {
                    origins.entry(path.clone()).or_insert(origin);
                }
                value.files.insert(path.clone(), file);
            }
            // A root that is no object defines nothing; the game says nothing, the crate
            // reports an error.
            None => report.push(
                DiagnosticCode::MalformedInput {
                    message: "the root is not an object".to_owned(),
                    text: None,
                },
                Severity::Error,
                file_origins.first(),
                &[],
                None,
            ),
        }
    }

    // The definition table.
    let (namespaces, resolved) = resolve(&value.files, &value.global_variables);
    value.namespaces = namespaces;
    let mut unresolved: HashSet<(String, String)> = HashSet::new();
    for found in resolved {
        let origin = origins.get(&found.file);
        let key = found.key.to_string();
        match found.event {
            // Silent in vanilla; the crate reports an information.
            ResolveEvent::Duplicate => report.push(
                DiagnosticCode::DuplicateIdentifier { text: None },
                Severity::Info,
                origin,
                &[&key],
                None,
            ),
            // Silent in vanilla at load; the crate reports an information.
            ResolveEvent::MissingBase(reference) => {
                unresolved.insert((found.namespace, found.key.name));
                report.push(
                    DiagnosticCode::UnresolvedReference { text: None },
                    Severity::Info,
                    origin,
                    &[&key],
                    Some(DiagnosticDetail::Reference {
                        kind: "ui definition",
                        id: reference,
                        hint: None,
                    }),
                );
            }
            // Silent in vanilla at load (the self-base walk stops); the crate reports an
            // information.
            ResolveEvent::Cycle => {
                unresolved.insert((found.namespace, found.key.name));
                report.push(
                    DiagnosticCode::ReferenceCycle { text: None },
                    Severity::Info,
                    origin,
                    &[&key],
                    None,
                );
            }
        }
    }

    // The control-creation checks, statically: informations, since the game logs them only
    // when a screen builds the control, never at load.
    for finding in lint::lint(&value, &unresolved) {
        let path: Vec<&str> = finding.path.iter().map(String::as_str).collect();
        report.push(
            finding.code,
            Severity::Info,
            origins.get(&finding.file),
            &path,
            finding.detail,
        );
    }

    let Report {
        diagnostics,
        reports,
    } = report;
    let mut reports: Vec<UiPackReport> = reports
        .into_iter()
        .map(|(pack, errors)| UiPackReport { pack, errors })
        .collect();
    reports.sort_by_key(|report| rank(&report.pack));
    value.reports = reports;
    // Top first; within one pack the later file first.
    layers.reverse();
    layers.sort_by_key(|layer| rank(&layer.origin.pack));
    Merged::new(value, origins, layers, diagnostics)
}

/// The origin of a merged file: the highest pack whose copy is part of it, the lower ones as
/// contributors.
fn merged_origin(file_origins: &[Origin], committed: &[usize]) -> Option<Origin> {
    let (top, lower) = committed.split_last()?;
    let top = file_origins.get(*top)?;
    Some(Origin::new(
        top.pack.clone(),
        top.side,
        top.path.clone(),
        None,
        lower
            .iter()
            .rev()
            .filter_map(|slot| file_origins.get(*slot))
            .map(|origin| origin.pack.clone())
            .collect(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `UiCopy` can be printed in a failure message.
    #[test]
    fn ui_copy_implements_debug() {
        fn assert_debug<T: std::fmt::Debug>() {}
        assert_debug::<UiCopy>();
    }

    /// `UiPackError` is an error, so `?` can box it.
    #[test]
    fn ui_pack_error_implements_error() {
        fn assert_error<T: std::error::Error>() {}
        assert_error::<UiPackError>();
    }

    /// The walk asks for the two fixed files and nothing else (`Walk::matches`); every other
    /// file is asked for only when a list names it.
    #[test]
    fn the_walk_asks_for_the_fixed_files_and_the_lists_name_the_rest() {
        let loader = load();
        let asked =
            |path: &'static str| loader.walk.matches(&PackPath::from_static(path)).is_some();
        assert!(asked(UI_DEFS_PATH));
        assert!(asked(GLOBAL_VARIABLES_PATH));
        // A file no list names is not asked for.
        assert!(!asked("ui/unlisted.json"));
        assert!(!asked("ui/deep/_ui_defs.json"));
        assert!(!asked("ui/picture.png"));
        assert!(!asked("textures/ui/a.json"));

        let copy = |key: &str, text: &str| {
            let path = PackPath::new(key).unwrap();
            UiCopy {
                origin: lone(&path),
                key: key.to_owned(),
                content: match read(text.as_bytes()) {
                    Ok(json) => CopyContent::Json(json),
                    Err(message) => CopyContent::Malformed(message.to_string()),
                },
                found: Diagnostics::default(),
            }
        };
        let lists = [
            copy(
                UI_DEFS_PATH,
                r#"{ "ui_defs": ["ui/a.json", "ui/deep/b.JSON", "ui/a.json", 7, "",
                                 "ui/note.txt", "textures/ui/c.json", "uix/d.json"] }"#,
            ),
            // The global variables name no file, whatever they hold.
            copy(GLOBAL_VARIABLES_PATH, r#"{ "ui_defs": ["ui/g.json"] }"#),
            // A list that is no JSON, and one whose `ui_defs` is no array, name nothing.
            copy(UI_DEFS_PATH, "{"),
            copy(UI_DEFS_PATH, r#"{ "ui_defs": "ui/e.json" }"#),
            copy(UI_DEFS_PATH, r#"{ "ui_defs": ["ui/z.json"] }"#),
        ];
        let then = loader.then.expect("the second stage");
        let named: Vec<String> = then(&lists.iter().collect::<Vec<_>>())
            .iter()
            .map(ToString::to_string)
            .collect();
        // Only the JSON files below `ui/` are named.
        assert_eq!(
            named,
            ["ui/a.json", "ui/deep/b.JSON", "ui/a.json", "ui/z.json"]
        );
    }
}
