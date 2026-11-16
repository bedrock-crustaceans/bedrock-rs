//! Reading the `config/` folder: the folders in name order, the shared front of every JSON
//! file (byte-order mark, depth, members written twice, the root) and the reading of the
//! per-folder files.

use crate::codes;
use std::collections::HashMap;

use uuid::Uuid;

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Code, CodeDef, Diagnostic, DiagnosticDetail, Diagnostics, JsonPath, Location, Phase, Severity,
    Span,
};
use bedrock_addon_core::json::kit::{Node, Profile, Read, Recovery};
use bedrock_addon_core::json::parse::{Options, parse_slice};
use bedrock_addon_core::json::{self, JsonSlice, PrepareError, Prepared, Winner};
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::versioning::{MemberFailure, NullHandling, Reading};

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
    pub fn push(&mut self, code: Code, severity: Severity, at: JsonPath, span: Option<Span>) {
        self.push_in(Phase::Parse, code, severity, at, span);
    }

    fn push_in(
        &mut self,
        phase: Phase,
        code: Code,
        severity: Severity,
        at: JsonPath,
        span: Option<Span>,
    ) {
        self.push_with(phase, code, severity, at, span, None);
    }

    /// [`push_in`](Self::push_in) with what the finding carries beyond its code.
    fn push_with(
        &mut self,
        phase: Phase,
        code: Code,
        severity: Severity,
        at: JsonPath,
        span: Option<Span>,
        detail: Option<DiagnosticDetail>,
    ) {
        let span = span.filter(|_| !self.secret);
        let mut finding = Diagnostic::new(
            code,
            severity,
            phase,
            Location::new(None, self.path.clone(), at, span),
        );
        finding.detail = detail.map(Box::new);
        self.d.push(finding);
    }

    /// A note of the crate's own that the `kind` named `id` at `at` resolves to nothing: an
    /// `UnresolvedReference` information of the validate phase, with the reference as its
    /// detail.
    pub fn reference(&mut self, kind: &'static str, id: &str, at: JsonPath, span: Option<Span>) {
        let span = span.filter(|_| !self.secret);
        let mut finding = Diagnostic::new(
            Code::from(&core_codes::UNRESOLVED_REFERENCE),
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

    /// An information of the crate's own.
    pub fn info(&mut self, code: Code, at: JsonPath, span: Option<Span>) {
        self.push(code, Severity::Info, at, span);
    }

    /// The line the server prints when the file cannot be parsed, under the variant of the
    /// finding: "Failed to parse JSON in file '%s'", and for `commands.json` followed by
    /// ":  malformed input". Vanilla names the sibling `variables.json` for a `permissions.json`.
    fn parse_failed(&self, empty: bool) -> Code {
        let file = self.name.as_str();
        match (self.commands, empty) {
            (true, false) => codes::COMMANDS_MALFORMED_MALFORMED_INPUT.with([file]),
            (true, true) => codes::COMMANDS_MALFORMED_EMPTY_DOCUMENT.with([file]),
            (false, false) => codes::CONFIG_PARSE_FAILED_MALFORMED_INPUT.with([file]),
            (false, true) => codes::CONFIG_PARSE_FAILED_EMPTY_DOCUMENT.with([file]),
        }
    }

    /// The file exists but could not be opened.
    // The text is "Failed to open file '%s'".
    pub fn unreadable(&mut self, kind: std::io::ErrorKind) {
        self.push_with(
            Phase::Read,
            codes::CONFIG_OPEN_FAILED.with([self.name.as_str()]),
            Severity::Error,
            JsonPath::new(),
            None,
            Some(DiagnosticDetail::Io(kind)),
        );
    }

    /// The file is refused: a syntax error.
    fn malformed(&mut self, message: String, span: Option<Span>) {
        let message = if self.secret {
            "malformed JSON".to_owned()
        } else {
            message
        };
        let code = self.parse_failed(false);
        self.push_with(
            Phase::Parse,
            code,
            Severity::Error,
            JsonPath::new(),
            span,
            Some(DiagnosticDetail::Syntax(message)),
        );
    }
}

/// The per-folder files' reading, the server's jsoncpp walk: every member of the wrong type
/// (a `null` too) counts as absent and is an information, as an unknown member is, an element
/// or an entry that fails is left out, scalars are read as written, and a kept number no
/// double holds is clamped and told.
pub(super) const FOLDER_READING: Reading = Reading::OWN
    .strict()
    .with_null(NullHandling::Rejected)
    .with_unknown(Severity::Info)
    .with_on_member_failure(MemberFailure::DefaultAndTell);

/// The order the per-folder files are read and told in: the file's, every member where it
/// stands, the innermost value that fails given up.
pub(super) const FOLDER_PROFILE: Profile = Profile::EVERY_FAILURE.with_recovery(Recovery::Value);

/// A read of a per-folder file into `found`.
pub(super) fn folder_read(found: &mut Diagnostics) -> Read<'_> {
    Read::new(FOLDER_READING, found)
        .profile(FOLDER_PROFILE)
        .clamp_kept()
}

/// A note of the crate's own at `node`, an information: the server reads on in silence.
pub(super) fn note(read: &mut Read<'_>, node: &Node<'_>, code: &'static CodeDef<0>) {
    let mut finding = Diagnostic::of(code)
        .severity(Severity::Info)
        .at(node.path().clone());
    finding.location.span = node.span();
    read.push(finding);
}

/// Runs `with` on the root of the file `bytes`, telling into the findings it is given, when
/// the file is JSON, and says whether it was.
///
/// The front every file shares: the byte-order mark is skipped, an empty file and a syntax
/// error refuse the file (an error; `with` is not run), nesting past `MAX_JSON_DEPTH`
/// too, of a member written twice the copy `winner` says stands, and content after the root
/// is a warning. Offsets count from the start of the file, the mark included. Every finding
/// is put in the file.
pub(super) fn with_root(
    bytes: &[u8],
    winner: Winner,
    sink: &mut Sink<'_>,
    with: impl FnOnce(&Node<'_>, &mut Diagnostics),
) -> bool {
    // The mark is blanked, so every offset is the file's own.
    let text = match Prepared::new(bytes) {
        Ok(prepared) => prepared.into_text(),
        // Bytes that are no UTF-8 are no JSON text, so the crate refuses the file.
        Err(PrepareError::NotUtf8(_)) => {
            sink.malformed("the file is not UTF-8".to_owned(), None);
            return false;
        }
        // A blank text holds no bracket, so a text too deep is never the empty one below.
        // Its byte counts from after a mark, as when the reader stripped it.
        Err(PrepareError::TooDeep(too_deep)) => {
            let mark = bytes.len() - json::strip_bom(bytes).len();
            let too_deep = json::TooDeep {
                offset: too_deep.offset - mark,
            };
            sink.malformed(too_deep.to_string(), None);
            return false;
        }
        // `new` reads zero bytes as the empty text; were it to refuse them, they are that
        // text all the same, told blank below.
        Err(PrepareError::Empty) => std::borrow::Cow::Borrowed(""),
    };
    let text = &*text;
    if text.trim().is_empty() {
        // An empty file is a parse failure.
        let code = sink.parse_failed(true);
        sink.push(code, Severity::Error, JsonPath::new(), None);
        return false;
    }
    let root = JsonSlice::new(text);
    // The faults are the text's whichever copy stands: a copy that does not is text too.
    let parsed = match parse_slice(root, Options::winner(winner)) {
        Ok(parsed) => parsed,
        Err(fault) => {
            sink.malformed(fault.kind.to_string(), Some(fault.span));
            return false;
        }
    };
    if let Some(mut finding) = json::trailing_content_finding(root) {
        finding.location.path.clone_from(&sink.path);
        if sink.secret {
            finding.location.span = None;
        }
        sink.d.push(finding);
    }
    let mut found = Diagnostics::default();
    with(&Node::parsed(&parsed, JsonPath::new()), &mut found);
    for finding in found.iter_mut() {
        finding.location.path.clone_from(&sink.path);
    }
    sink.d.append(found);
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
            Code::from(&core_codes::IGNORED_FILE),
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
            Sink::new(Some(name), file, d).info(
                Code::from(&core_codes::PACK_CONFIG_SHADOWS_DEFAULT),
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
