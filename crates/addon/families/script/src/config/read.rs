//! Reading the `config/` folder: the folders in name order, the shared front of every JSON
//! file (byte-order mark, depth, members written twice, the root) and the helpers that read
//! one level of a file at file offsets.

use std::collections::HashMap;

use facet_json::RawJson;
use indexmap::IndexMap;
use uuid::Uuid;

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity, Span, VanillaText,
};
use bedrock_addon_core::json::{self, JsonSlice, MAX_JSON_DEPTH, Value, parse_at};
use bedrock_addon_core::pack_identity::PackPath;

use super::{
    COMMANDS_FILE, CONFIG_DIR, ConfigFolder, ConfigSource, DEFAULT_DIR, PERMISSIONS_FILE,
    PackScriptConfig, SECRETS_FILE, ScriptConfig, ScriptConfigFiles, UnreadableFile,
    VARIABLES_FILE, commands, permissions, values,
};

/// Where the findings of one file go, and how the file is named in them.
pub(super) struct Sink<'d> {
    /// The file as the server prints it: `config/<folder>/<file>`.
    pub name: String,
    /// The same as a path, when the folder's name can be part of one.
    path: Option<PackPath>,
    /// Whether the file is `commands.json`, whose refusals print two lines.
    pub commands: bool,
    /// Whether the file's content must stay out of every finding (`secrets.json`): no
    /// message quotes it and no finding has a span, since a span addresses the file's bytes
    /// and the names of its members are all a finding may tell.
    pub secret: bool,
    d: &'d mut Diagnostics,
}

impl<'d> Sink<'d> {
    /// A sink for `config/<folder>/<file>`, or `config/<file>` without a folder.
    pub fn new(folder: Option<&str>, file: &str, d: &'d mut Diagnostics) -> Self {
        let name = match folder {
            Some(folder) => format!("{CONFIG_DIR}/{folder}/{file}"),
            None => format!("{CONFIG_DIR}/{file}"),
        };
        Self {
            path: PackPath::new(&name).ok(),
            name,
            commands: false,
            secret: false,
            d,
        }
    }

    /// One finding at `at`, whose value lies at `span`.
    pub fn push(
        &mut self,
        code: DiagnosticCode,
        severity: Severity,
        at: JsonPath,
        span: Option<Span>,
    ) {
        self.push_in(Phase::Parse, code, severity, at, span);
    }

    fn push_in(
        &mut self,
        phase: Phase,
        code: DiagnosticCode,
        severity: Severity,
        at: JsonPath,
        span: Option<Span>,
    ) {
        let span = span.filter(|_| !self.secret);
        self.d.push(Diagnostic::new(
            code,
            severity,
            phase,
            Location::new(None, self.path.clone(), at, span),
        ));
    }

    /// A note of the crate's own that the `kind` named `id` at `at` resolves to nothing: an
    /// `UnresolvedReference` information of the validate phase, with the reference as its
    /// detail.
    pub fn reference(&mut self, kind: &'static str, id: &str, at: JsonPath, span: Option<Span>) {
        let span = span.filter(|_| !self.secret);
        let mut finding = Diagnostic::new(
            DiagnosticCode::UnresolvedReference { text: None },
            Severity::Info,
            Phase::Validate,
            Location::new(None, self.path.clone(), at, span),
        );
        finding.detail = Some(Box::new(
            bedrock_addon_core::diagnostic::DiagnosticDetail::Reference {
                kind,
                id: id.to_owned(),
                hint: None,
            },
        ));
        self.d.push(finding);
    }

    /// A note of the crate's own: the server reads these silently, so it is an information.
    pub fn warn(&mut self, code: DiagnosticCode, at: JsonPath, span: Option<Span>) {
        self.push(code, Severity::Info, at, span);
    }

    /// A line the dedicated server writes to its log at warning level (`Secret '%s' in file
    /// '%s' is not a string…`): a warning.
    pub fn warn_logged(&mut self, code: DiagnosticCode, at: JsonPath, span: Option<Span>) {
        self.push(code, Severity::Warning, at, span);
    }

    /// An information of the crate's own.
    pub fn info(&mut self, code: DiagnosticCode, at: JsonPath, span: Option<Span>) {
        self.push(code, Severity::Info, at, span);
    }

    /// The text the server prints when the file cannot be parsed.
    // The text is "Failed to parse JSON in file '%s'"; for `commands.json` it is followed by
    // ":  malformed input". Vanilla names the sibling `variables.json` for a
    // `permissions.json`.
    fn parse_failed(&self) -> VanillaText {
        if self.commands {
            VanillaText::Formatted {
                template: commands::MALFORMED,
                args: vec![self.name.clone()],
            }
        } else {
            VanillaText::ConfigParseFailed {
                file: self.name.clone(),
            }
        }
    }

    /// The file exists but could not be opened.
    // The text is "Failed to open file '%s'".
    pub fn unreadable(&mut self, kind: std::io::ErrorKind) {
        let text = Some(VanillaText::ConfigOpenFailed {
            file: self.name.clone(),
        });
        self.push_in(
            Phase::Read,
            DiagnosticCode::ReadError { kind, text },
            Severity::Error,
            JsonPath::new(),
            None,
        );
    }

    /// The file is refused: a syntax error.
    fn malformed(&mut self, message: String, span: Option<Span>) {
        let message = if self.secret {
            "malformed JSON".to_owned()
        } else {
            message
        };
        let text = Some(self.parse_failed());
        self.push(
            DiagnosticCode::MalformedInput { message, text },
            Severity::Error,
            JsonPath::new(),
            span,
        );
    }
}

/// The JSON type of a value, from its first byte.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    Object,
    Array,
    String,
    Number,
    Bool,
    Null,
}

/// The JSON type of the value `slice` holds. The slice is a root that parsed, or a raw
/// member of one, so its first byte decides.
pub(super) fn kind(slice: JsonSlice<'_>) -> Kind {
    // Comments may stand before the value.
    let mut text = slice.text.trim_start();
    loop {
        let after = if text.starts_with("//") {
            text.find('\n').map(|end| &text[end..])
        } else if text.starts_with("/*") {
            text.find("*/").map(|end| &text[end + 2..])
        } else {
            break;
        };
        text = after.unwrap_or("").trim_start();
    }
    match text.as_bytes().first() {
        Some(b'{') => Kind::Object,
        Some(b'[') => Kind::Array,
        Some(b'"') => Kind::String,
        Some(b't' | b'f') => Kind::Bool,
        Some(b'n') => Kind::Null,
        _ => Kind::Number,
    }
}

/// The file offsets of `slice`.
pub(super) fn span(slice: JsonSlice<'_>) -> Option<Span> {
    Some(Span {
        offset: u32::try_from(slice.offset).ok()?,
        len: u32::try_from(slice.text.len()).ok()?,
    })
}

/// The members of the object `slice` holds, each a raw slice of the file; `None` when it is
/// not an object.
pub(super) fn object(slice: JsonSlice<'_>) -> Option<IndexMap<String, RawJson<'_>>> {
    (kind(slice) == Kind::Object)
        .then(|| parse_at(slice).ok())
        .flatten()
}

/// The elements of the array `slice` holds; `None` when it is not an array.
pub(super) fn array(slice: JsonSlice<'_>) -> Option<Vec<RawJson<'_>>> {
    (kind(slice) == Kind::Array)
        .then(|| parse_at(slice).ok())
        .flatten()
}

/// The string `slice` holds; `None` when it is not a string.
pub(super) fn string(slice: JsonSlice<'_>) -> Option<String> {
    (kind(slice) == Kind::String)
        .then(|| parse_at(slice).ok())
        .flatten()
}

/// The value `slice` holds, untyped and as read.
pub(super) fn raw_value(slice: JsonSlice<'_>) -> Option<Value> {
    json::from_str_jsonc(slice.text).ok()
}

/// The value `slice` holds, untyped, for keeping. A number too large for a double
/// (`1e999`) is kept as the largest finite double of its sign, with a warning at the number:
/// it has no JSON spelling otherwise.
pub(super) fn value(slice: JsonSlice<'_>, at: &JsonPath, sink: &mut Sink<'_>) -> Option<Value> {
    let mut value = raw_value(slice)?;
    for clamped in json::clamp_non_finite(&mut value, Some(slice)) {
        let mut path = at.clone();
        path.append(&clamped.path);
        sink.warn(
            DiagnosticCode::InvalidValue { text: None },
            path,
            clamped.span,
        );
    }
    Some(value)
}

/// `path` with `key` appended.
pub(super) fn child(path: &JsonPath, key: &str) -> JsonPath {
    let mut path = path.clone();
    path.push(key);
    path
}

/// `path` with `index` appended.
pub(super) fn element(path: &JsonPath, index: usize) -> JsonPath {
    let mut path = path.clone();
    path.push_index(index);
    path
}

/// How a file treats a member written twice.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Twice {
    /// The last value stands (jsoncpp: the per-folder files).
    Last,
    /// The first value stands (`commands.json`).
    First,
}

/// Runs `with` on the root of the file `bytes` when it is JSON, and says whether it was.
///
/// The front every file shares: the byte-order mark is skipped, an empty file and a syntax
/// error refuse the file (an error; `with` is not run), nesting past `MAX_JSON_DEPTH`
/// too, members written twice are reduced to the one that stands, and content after the root
/// is a warning. Offsets count from the start of the file, the mark included.
pub(super) fn with_root(
    bytes: &[u8],
    twice: Twice,
    sink: &mut Sink<'_>,
    with: impl FnOnce(JsonSlice<'_>, &mut Sink<'_>),
) -> bool {
    let stripped = json::strip_bom(bytes);
    let offset = bytes.len() - stripped.len();
    let Ok(text) = std::str::from_utf8(stripped) else {
        // Bytes that are no UTF-8 are no JSON text, so the crate refuses the file.
        sink.malformed("the file is not UTF-8".to_owned(), None);
        return false;
    };
    if text.trim().is_empty() {
        // An empty file is a parse failure.
        let text = Some(sink.parse_failed());
        sink.push(
            DiagnosticCode::EmptyDocument { text },
            Severity::Error,
            JsonPath::new(),
            None,
        );
        return false;
    }
    if let Err(too_deep) = json::check_depth(stripped, MAX_JSON_DEPTH) {
        sink.malformed(too_deep.to_string(), None);
        return false;
    }
    let clean = match twice {
        Twice::Last => json::last_wins(text),
        Twice::First => super::duplicates::first_wins(text),
    };
    if twice == Twice::First
        && matches!(clean, std::borrow::Cow::Owned(_))
        && let Err(failure) = parse_at::<Value>(JsonSlice { text, offset })
    {
        // `first_wins` follows brackets and keys, not the grammar: a copy it blanked may
        // hold the text's syntax fault. The server's reader reads the whole text, so the
        // text as written is read once more, and its fault is the file's.
        sink.malformed(failure.error.to_string(), Some(failure.span));
        return false;
    }
    let root = JsonSlice {
        text: &clean,
        offset,
    };
    // The whole root is read once, so that every slice of it read later is known to parse.
    if let Err(failure) = parse_at::<Value>(root) {
        sink.malformed(failure.error.to_string(), Some(failure.span));
        return false;
    }
    if let Some(mut finding) = json::trailing_content_finding(root) {
        finding.location.path.clone_from(&sink.path);
        if sink.secret {
            finding.location.span = None;
        }
        sink.d.push(finding);
    }
    with(root, sink);
    true
}

/// `ScriptConfig::parse`.
pub(super) fn parse(files: &ScriptConfigFiles) -> ScriptConfig {
    let mut config = ScriptConfig::default();
    let mut unreadable: HashMap<Option<&str>, Vec<&UnreadableFile>> = HashMap::new();
    for file in &files.unreadable {
        unreadable
            .entry(file.folder.as_deref())
            .or_default()
            .push(file);
    }
    let mut folders: Vec<&ConfigFolder> = files.folders.iter().collect();
    folders.sort_by(|a, b| a.name.cmp(&b.name));
    let mut gaps = Vec::new();
    for folder in folders {
        let failed = unreadable
            .get(&Some(&*folder.name))
            .map_or(&[][..], Vec::as_slice);
        let (pack, gap) = read_folder(folder, failed, &mut config.diagnostics);
        if &*folder.name == DEFAULT_DIR {
            config.default = pack;
        } else {
            // Folders of one name are neighbours here; the later one stands.
            if gaps
                .last()
                .is_some_and(|(name, _): &(&str, Gap)| *name == &*folder.name)
            {
                gaps.pop();
            }
            gaps.push((&*folder.name, gap));
            config.packs.insert(folder.name.to_string(), pack);
        }
    }
    shadowing(&config.default, &gaps, &mut config.diagnostics);
    let failed = unreadable.get(&None).map_or(&[][..], Vec::as_slice);
    config.commands = commands::read(files.commands.as_deref(), failed, &mut config.diagnostics);
    config
}

/// What a folder lacks that `default` may have.
#[derive(Clone, Copy, Default)]
struct Gap {
    allowed_modules: bool,
    variables: bool,
    secrets: bool,
}

/// One folder: its three files, each on its own.
fn read_folder(
    folder: &ConfigFolder,
    unreadable: &[&UnreadableFile],
    d: &mut Diagnostics,
) -> (PackScriptConfig, Gap) {
    let name = &*folder.name;
    let mut pack = PackScriptConfig {
        source: if name == DEFAULT_DIR {
            ConfigSource::Default
        } else {
            ConfigSource::Folder(name.to_owned())
        },
        ..PackScriptConfig::default()
    };
    let mut gap = Gap::default();
    // The server's order: permissions, variables, secrets.
    for file in ConfigFolder::FILES {
        let mut sink = Sink::new(Some(name), file, d);
        if let Some(failed) = unreadable.iter().find(|failed| &*failed.file == file) {
            sink.unreadable(failed.kind);
            continue;
        }
        match file {
            PERMISSIONS_FILE => match &folder.permissions {
                Some(bytes) => {
                    let read = permissions::read(bytes, &mut sink);
                    gap.allowed_modules = read.lacks_allowed_modules;
                    pack.permissions = read.permissions;
                }
                None => gap.allowed_modules = true,
            },
            VARIABLES_FILE => match &folder.variables {
                Some(bytes) => pack.variables = values::variables(bytes, &mut sink),
                None => gap.variables = true,
            },
            SECRETS_FILE => match &folder.secrets {
                Some(bytes) => {
                    sink.secret = true;
                    pack.secrets = values::secrets(bytes, &mut sink);
                }
                None => gap.secrets = true,
            },
            _ => {}
        }
    }
    if folder.stray_commands {
        // `config/default/commands.json` is not read.
        Sink::new(Some(name), COMMANDS_FILE, d).info(
            DiagnosticCode::IgnoredFile { text: None },
            JsonPath::new(),
            None,
        );
    }
    (pack, gap)
}

/// `PackConfigShadowsDefault`: a folder a script module can use (its name is a UUID in the
/// form the server looks up) replaces the whole of `default`; each part it lacks and
/// `default` has is a warning. The server is silent.
fn shadowing(default: &PackScriptConfig, gaps: &[(&str, Gap)], d: &mut Diagnostics) {
    let has = Gap {
        allowed_modules: !default.permissions.allowed_modules.is_empty(),
        variables: !default.variables.is_empty(),
        secrets: !default.secrets.is_empty(),
    };
    for (name, gap) in gaps {
        if !is_lookup_key(name) {
            continue;
        }
        let parts = [
            (gap.allowed_modules && has.allowed_modules, PERMISSIONS_FILE),
            (gap.variables && has.variables, VARIABLES_FILE),
            (gap.secrets && has.secrets, SECRETS_FILE),
        ];
        for (_, file) in parts.into_iter().filter(|(lacking, _)| *lacking) {
            Sink::new(Some(name), file, d).warn(
                DiagnosticCode::PackConfigShadowsDefault,
                JsonPath::new(),
                None,
            );
        }
    }
}

/// Whether `name` is a UUID exactly as the server builds its lookup key: lower-case and
/// hyphenated.
pub(super) fn is_lookup_key(name: &str) -> bool {
    Uuid::try_parse(name).is_ok_and(|uuid| {
        let mut buffer = Uuid::encode_buffer();
        uuid.hyphenated().encode_lower(&mut buffer) == name
    })
}
