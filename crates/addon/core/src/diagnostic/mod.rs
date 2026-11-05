//! Findings as data: one typed `Diagnostic` per thing a vanilla content log would print, never a
//! log line. Tests match on `code` variants, never on text.

mod builder;
pub mod cereal;
mod code;
mod def;
mod deprecated;
mod event;
mod facet;
mod origin;
pub mod testing;
pub mod validator;

pub use code::{DiagnosticCode, NotFoundHint, VanillaText};
pub use def::{Class, Code, CodeDef, Kind};
pub use deprecated::ENUM_VALUE_DEPRECATED;
pub use event::EVENT_BOTH_RANDOMIZE_AND_SEQUENCE;
#[doc(hidden)]
pub use facet::code_for_deserialize_error_kind;
#[doc(hidden)]
pub use origin::Origins;

use std::fmt;

use crate::document::Origin;
use crate::load::Place;
use crate::pack_identity::{PackIdVersion, PackOrigin, PackPath, PackRef, PackType};
use crate::version::{BaseGameVersion, FormatVersion};
use crate::visual::ShadowReason;

/// One finding: a typed code, a severity, where, and in which phase.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Diagnostic {
    /// What happened, with typed payload. `Display` renders vanilla's text.
    pub code: DiagnosticCode,
    /// Chosen by the emitting rule; the same code can be an error in one family and a warning
    /// in another.
    pub severity: Severity,
    /// The loading phase that raised it.
    pub phase: Phase,
    /// Where.
    pub location: Location,
    /// Molang compiler detail and similar attachments.
    pub detail: Option<Box<DiagnosticDetail>>,
    /// A kept error: an error-level finding after which the game keeps the document. It counts
    /// as an error wherever findings are counted, and rejects nothing:
    /// [`Diagnostics::has_errors`] and the loads' "any error rejects" checks pass it by. Set
    /// by [`Diagnostic::of`] for a code of [`Class::KeptError`], by
    /// [`Diagnostics::push_kept_error`], and by the doc-hidden `Diagnostic::into_kept` that it
    /// uses, for a compile pass that marks a finding before it is pushed;
    /// [`Diagnostic::severity`] below error level clears it.
    pub kept: bool,
}

impl Diagnostic {
    /// An error-level finding that rejects the document it is about: an error that is not
    /// [`kept`](Self::kept). A loader's "any error rejects" check asks this.
    pub fn rejects(&self) -> bool {
        self.severity == Severity::Error && !self.kept
    }

    /// This finding as a kept error: at error level, rejecting nothing. A legacy code is made
    /// [`kept`](Self::kept) here: [`Diagnostics::push_kept_error`] goes through it, and so
    /// does a compile pass that marks a Molang finding kept before it is pushed; a specific
    /// code is kept by its [`Class::KeptError`] in [`of`](Self::of). Both halves must hold for
    /// the finding: it is an error in the game's content log, and the game's loader goes on
    /// with the document.
    #[doc(hidden)]
    #[must_use]
    pub fn into_kept(mut self) -> Self {
        self.severity = Severity::Error;
        self.kept = true;
        self
    }

    /// The general kind of the finding: a specific code's own, or the kind a legacy code
    /// belongs to. For the question `d.kind() == Kind::InvalidValue`, whichever rule raised it.
    pub fn kind(&self) -> Kind {
        def::kind_of(&self.code)
    }

    /// Whether the finding is a code of `def`, whatever its arguments: the specific question.
    /// A legacy code is no code of any definition.
    pub fn is<const N: usize>(&self, def: &CodeDef<N>) -> bool {
        matches!(&self.code, DiagnosticCode::Specific { code } if code.is(def))
    }

    /// The game's sentence for the finding, with its arguments; `None` when the game prints
    /// none and the finding is the toolkit's own. A legacy code gives the game text it carries
    /// in its `text` field.
    pub fn vanilla(&self) -> Option<String> {
        match &self.code {
            DiagnosticCode::Specific { code } => code.vanilla(),
            legacy => legacy.game_text().map(ToString::to_string),
        }
    }

    /// A finding without detail. The struct is `#[non_exhaustive]`, so `bedrock_addon` builds
    /// it here.
    #[doc(hidden)]
    pub fn new(code: DiagnosticCode, severity: Severity, phase: Phase, location: Location) -> Self {
        Self {
            code,
            severity,
            phase,
            location,
            detail: None,
            kept: false,
        }
    }
}

/// Vanilla's content-log shape:
/// `[warning] <pack name> | blocks/ruby.json | minecraft:block/components/minecraft:friction | <message>`.
///
/// A part of the location that is not known (no pack, no file, the document root) is left out
/// with its separator. The message is the code's text followed by the detail: a Molang
/// compiler message after ` | ` (vanilla's `<expression> | <message>` scope chain), a
/// reference as `: <kind> '<id>'` with its hint.
impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] ", self.severity)?;
        let Location {
            pack,
            path,
            json_path,
            span: _,
        } = &self.location;
        if let Some(pack) = pack {
            write!(f, "{} | ", pack.name())?;
        }
        if let Some(path) = path {
            write!(f, "{path} | ")?;
        }
        if !json_path.is_empty() {
            write!(f, "{json_path} | ")?;
        }
        write!(f, "{}", self.code)?;
        match self.detail.as_deref() {
            None => Ok(()),
            Some(detail) => write!(f, "{detail}"),
        }
    }
}

/// An attachment of a [`Diagnostic`]: what the code alone does not say.
///
/// A [`CodeDef`] that [`was`](CodeDef::was) a legacy code with a typed payload (a
/// `ShadowReason`, a `PackIdVersion`, an `io::ErrorKind`, a UUID, a `NotFoundHint`, a
/// `FormatVersion`) puts that payload here: the definition's arguments are the text the line
/// says, the detail is the data a caller matches on. Those payload variants are data only and
/// add nothing to the content-log line.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum DiagnosticDetail {
    /// The Molang compiler's diagnostic behind an `InvalidMolang` / `MolangLint`: its code,
    /// parser message and span in the expression.
    #[cfg(feature = "molang")]
    #[cfg_attr(docsrs, doc(cfg(feature = "molang")))]
    Molang(molang::diag::Diagnostic),
    /// The name behind an `UnresolvedReference` / `InvalidReference`: what kind of thing it
    /// names, the name as written, and a near miss when one exists.
    Reference {
        /// The kind of thing the name refers to (`"geometry"`, `"material"`, …).
        kind: &'static str,
        /// The name as written.
        id: String,
        /// A near miss, when one exists.
        hint: Option<ReferenceHint>,
    },
    /// The paths a file lookup tried and found in no pack, behind a `FontFileMissing`: what
    /// kind of file was looked for and each path, in the order the game tries them.
    TriedPaths {
        /// The kind of file looked for (`"font file"`).
        kind: &'static str,
        /// Each path tried, in the game's order.
        paths: Vec<String>,
    },
    /// What a finding of this crate says beyond its code, in fixed words (the game has no
    /// text for it): which member a written document carries a value as, or why an item has
    /// no text.
    Note(&'static str),
    /// Why the shared identifier map ignored a document (`ClientEntityShadowed`'s and
    /// `AttachableShadowed`'s `reason`). Data only.
    Shadow(ShadowReason),
    /// The packs of a stack finding: the pack, and the dependency it names when it names one
    /// (`MissingDependency`, `DependencyAutoAdded`, `PackNotFound`). Data only.
    Packs {
        /// The pack the finding is about.
        pack: PackIdVersion,
        /// The dependency it names, when it names one.
        dependency: Option<PackIdVersion>,
    },
    /// What the host reported for a file it could not read (`ReadError`'s `kind`). Data only.
    Io(std::io::ErrorKind),
    /// The UUID a finding names (`UnmatchedConfigFolder`'s `expected`): a finding with none
    /// has no such detail. Data only.
    Uuid(uuid::Uuid),
    /// Why an import likely resolved to no file (`ScriptImportNotFound`'s `hint`). Data only.
    NotFound(NotFoundHint),
    /// The format version a finding names (`BiomeFormatHint`'s `version`, `FieldSinceEngine`'s
    /// `since`). Data only.
    FormatVersion(FormatVersion),
}

/// The suffix a detail adds to the diagnostic's message: ` | <compiler message>` for Molang,
/// `: <kind> '<id>'` and the hint for a reference, `: <kind> '<path>, <path>'` for tried paths,
/// `: <note>` for a note; nothing for a typed payload.
impl fmt::Display for DiagnosticDetail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "molang")]
            Self::Molang(diagnostic) => write!(f, " | {diagnostic}"),
            Self::Note(note) => write!(f, ": {note}"),
            Self::Reference { kind, id, hint } => {
                write!(f, ": {kind} '{id}'")?;
                match hint {
                    None => Ok(()),
                    Some(ReferenceHint::CaseMismatch(target)) => {
                        write!(f, " (did you mean '{target}'?)")
                    }
                    Some(ReferenceHint::MissingNamespace) => f.write_str(" (namespace missing)"),
                    Some(ReferenceHint::Shadowed) => {
                        f.write_str(" (shadowed by a higher-priority definition)")
                    }
                }
            }
            Self::TriedPaths { kind, paths } => write!(f, ": {kind} '{}'", paths.join(", ")),
            // Data only: the code's arguments carry what the line says.
            Self::Shadow(_)
            | Self::Packs { .. }
            | Self::Io(_)
            | Self::Uuid(_)
            | Self::NotFound(_)
            | Self::FormatVersion(_) => Ok(()),
        }
    }
}

/// A near miss for a reference that did not resolve as written.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ReferenceHint {
    /// A target exists whose name differs only in case: the target's name.
    CaseMismatch(String),
    /// The name lacks a namespace that the target has.
    MissingNamespace,
    /// The target exists but is shadowed by a higher-priority definition.
    Shadowed,
}

/// How bad a finding is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    /// Debug output.
    Debug,
    /// Information.
    Info,
    /// A warning: the document loads.
    Warning,
    /// An error: the document (or entry) is dropped.
    Error,
}

/// The content-log level word: `debug`, `info`, `warning`, `error`.
impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        })
    }
}

/// The phases of the game's versioned loaders, plus the crate's own I/O and link phases.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Phase {
    /// Reading bytes from the host.
    Read,
    /// JSON and typed parsing.
    Parse,
    /// Version upgrades.
    Upgrade,
    /// Validation on the newest struct.
    Validate,
    /// The cross-family link pass.
    Link,
}

/// Where a finding is.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct Location {
    /// The pack, when known.
    pub pack: Option<PackRef>,
    /// The file, when known.
    pub path: Option<PackPath>,
    /// Breadcrumb inside the file: `minecraft:block/components/minecraft:friction`.
    pub json_path: JsonPath,
    /// Byte range in the file, when the reader knows it (typed findings yes, `Value` findings no).
    pub span: Option<Span>,
}

impl Location {
    /// A location from its parts. The struct is `#[non_exhaustive]`, so `bedrock_addon` builds
    /// it here.
    #[doc(hidden)]
    pub fn new(
        pack: Option<PackRef>,
        path: Option<PackPath>,
        json_path: JsonPath,
        span: Option<Span>,
    ) -> Self {
        Self {
            pack,
            path,
            json_path,
            span,
        }
    }
}

/// A breadcrumb inside a JSON document: `minecraft:block/components/minecraft:friction`.
///
/// Member names and array indices, from the document root down, as vanilla's error path stack
/// records them while it reads. `Display` joins them with `/`; an index prints as its number,
/// and a `/` or `~` inside a key is written `~1` or `~0` (the JSON pointer escapes).
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct JsonPath {
    segments: Vec<Segment>,
}

/// One step of a [`JsonPath`].
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Segment {
    Key(Box<str>),
    Index(usize),
}

impl JsonPath {
    /// The empty path: the document root.
    pub fn new() -> Self {
        Self::default()
    }

    /// Steps into the member `key`.
    pub fn push(&mut self, key: &str) {
        self.segments.push(Segment::Key(key.into()));
    }

    /// Steps into the array element `index`.
    pub fn push_index(&mut self, index: usize) {
        self.segments.push(Segment::Index(index));
    }

    /// Continues the path with every segment of `other` (a path relative to this one, such as
    /// `SliceError::path` below a component key).
    pub fn append(&mut self, other: &JsonPath) {
        self.segments.extend(other.segments.iter().cloned());
    }

    /// The path of the member `key` below this one.
    #[must_use]
    pub fn child(&self, key: &str) -> Self {
        let mut path = self.clone();
        path.push(key);
        path
    }

    /// The path of the array element `index` below this one.
    #[must_use]
    pub fn index(&self, index: usize) -> Self {
        let mut path = self.clone();
        path.push_index(index);
        path
    }

    /// This path continued by every segment of `relative`.
    #[must_use]
    pub fn join(&self, relative: &JsonPath) -> Self {
        let mut path = self.clone();
        path.append(relative);
        path
    }

    /// The path of the members `keys` below this one, from the outermost down, each taken as
    /// written: a key of digits stays a key (an array element is [`index`](Self::index)).
    #[must_use]
    pub fn with<K: AsRef<str>>(&self, keys: &[K]) -> Self {
        let mut path = self.clone();
        for key in keys {
            path.push(key.as_ref());
        }
        path
    }

    /// Steps out of the last segment; `false` when the path was already the root.
    pub fn pop(&mut self) -> bool {
        self.segments.pop().is_some()
    }

    /// The number of segments.
    pub fn len(&self) -> usize {
        self.segments.len()
    }

    /// The path is the document root.
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// The first `length` segments of the path.
    #[doc(hidden)]
    #[must_use]
    pub fn prefix(&self, length: usize) -> JsonPath {
        JsonPath {
            segments: self.segments[..length.min(self.segments.len())].to_vec(),
        }
    }

    /// The rest of the path below `prefix`; `None` when the path is not at or below it.
    #[must_use]
    pub fn strip_prefix(&self, prefix: &JsonPath) -> Option<JsonPath> {
        self.segments
            .strip_prefix(prefix.segments.as_slice())
            .map(|rest| JsonPath {
                segments: rest.to_vec(),
            })
    }

    /// The path of the members `keys`, from the document root down, each taken as written.
    #[must_use]
    pub fn of_keys<K: AsRef<str>>(keys: impl IntoIterator<Item = K>) -> Self {
        let mut path = Self::new();
        for key in keys {
            path.push(key.as_ref());
        }
        path
    }

    /// The last segment when it is a member key, as the file has it; `None` for the root and
    /// for a path that ends at an array element.
    #[must_use]
    pub fn last_key(&self) -> Option<&str> {
        match self.segments.last()? {
            Segment::Key(key) => Some(key),
            Segment::Index(_) => None,
        }
    }

    /// Whether the path is the one member `key` of the root.
    #[must_use]
    pub fn is_key(&self, key: &str) -> bool {
        matches!(self.segments.as_slice(), [Segment::Key(only)] if **only == *key)
    }

    /// Whether the path is `prefix` or lies below it, segment for segment (the member
    /// `ab` is not below `a`).
    #[must_use]
    pub fn starts_with(&self, prefix: &JsonPath) -> bool {
        self.segments.starts_with(&prefix.segments)
    }

    /// Whether the path's last segments are those of `suffix`, segment for segment.
    #[must_use]
    pub fn ends_with(&self, suffix: &JsonPath) -> bool {
        self.segments.ends_with(&suffix.segments)
    }

    /// The array index that follows the first member `key` on the path
    /// (`permutations/3/condition` has 3 after `permutations`); `None` when no segment is
    /// that key or an index does not follow it.
    #[must_use]
    pub fn index_after(&self, key: &str) -> Option<usize> {
        let at = self
            .segments
            .iter()
            .position(|segment| matches!(segment, Segment::Key(k) if **k == *key))?;
        match self.segments.get(at + 1)? {
            Segment::Index(index) => Some(*index),
            Segment::Key(_) => None,
        }
    }

    /// The segments, from the document root down: each key as the file has it (the `Display`
    /// text escapes `/` and `~` in a key) and each index as a number.
    pub fn segments(&self) -> impl ExactSizeIterator<Item = JsonPathSegment<'_>> + '_ {
        self.segments.iter().map(|segment| match segment {
            Segment::Key(key) => JsonPathSegment::Key(key),
            Segment::Index(index) => JsonPathSegment::Index(*index),
        })
    }
}

/// One step of a [`JsonPath`], as [`JsonPath::segments`] lists it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JsonPathSegment<'a> {
    /// The member of this key.
    Key(&'a str),
    /// The array element at this index.
    Index(usize),
}

/// Writes a member key as a JSON pointer token (RFC 6901): `~` as `~0` and `/` as `~1`, so a
/// key that holds the separator stays one segment of the text.
///
/// Nearly every key holds neither and is written in one piece. It is looked through as bytes
/// first: the search of a `str` for one of two characters goes a character at a time and is
/// built with this crate, and in the dev profile it took ten times the writing of the key.
fn write_key(f: &mut fmt::Formatter<'_>, key: &str) -> fmt::Result {
    let bytes = key.as_bytes();
    if !bytes.contains(&b'~') && !bytes.contains(&b'/') {
        return f.write_str(key);
    }
    let mut rest = key;
    while let Some(at) = rest.find(['~', '/']) {
        f.write_str(&rest[..at])?;
        f.write_str(if rest.as_bytes()[at] == b'~' {
            "~0"
        } else {
            "~1"
        })?;
        rest = &rest[at + 1..];
    }
    f.write_str(rest)
}

/// The segments joined with `/`; an index as its number, a key as a JSON pointer token (RFC
/// 6901: `~` as `~0`, `/` as `~1`), without the pointer's leading `/`.
///
/// The text is for a reader, not for code: splitting it at `/` and undoing the two escapes
/// gives the keys back, with two exceptions the text cannot tell apart. A key made of digits
/// prints like an index (`["0"]` and element 0 are both `0`), and a path of one empty key
/// prints like the root (both are the empty text; an empty key after other segments is an
/// empty piece, `a//b`). Code that needs a key, a prefix or a step reads
/// [`JsonPath::segments`], [`last_key`](JsonPath::last_key),
/// [`starts_with`](JsonPath::starts_with) or [`strip_prefix`](JsonPath::strip_prefix).
impl fmt::Display for JsonPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, segment) in self.segments.iter().enumerate() {
            if i > 0 {
                f.write_str("/")?;
            }
            match segment {
                Segment::Key(key) => write_key(f, key)?,
                Segment::Index(index) => write!(f, "{index}")?,
            }
        }
        Ok(())
    }
}

/// A byte range in a file.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Span {
    /// Byte offset of the start.
    pub offset: u32,
    /// Length in bytes.
    pub len: u32,
}

impl Span {
    /// The one-based line and column of the span's start in `text`, the file as written.
    ///
    /// Lines end at `\n`, so LF and CRLF text give the same line numbers; columns count
    /// characters, not bytes. An offset past the end, or inside a character, counts up to the
    /// last whole character before it. A byte-order mark at the start of `text` is no
    /// column: the first character after it is column 1, as an editor shows it (the span's
    /// offset does count the mark's three bytes).
    pub fn line_col(&self, text: &str) -> (u32, u32) {
        let mut end = usize::try_from(self.offset).map_or(text.len(), |o| o.min(text.len()));
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        let before = &text[..end];
        let line_start = before.rfind('\n').map_or(0, |i| i + 1);
        let line = before.bytes().filter(|&b| b == b'\n').count() + 1;
        let on_line = &before[line_start..];
        let on_line = if line_start == 0 {
            on_line.strip_prefix('\u{feff}').unwrap_or(on_line)
        } else {
            on_line
        };
        let column = on_line.chars().count() + 1;
        (saturate(line), saturate(column))
    }
}

fn saturate(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

/// An ordered list of findings.
#[derive(Clone, Default)]
pub struct Diagnostics(Option<Box<Held>>);

/// What a [`Diagnostics`] holds, behind one pointer (`None`: nothing yet), so that a list of
/// findings is small wherever it is carried (a `Rejected` is the `Err` of every parse).
#[derive(Clone, Default)]
struct Held {
    findings: Vec<Diagnostic>,
    /// The moves of an upgrade in progress.
    moves: Origins,
    /// The moves already undone for the findings, in the order made: where each member of
    /// the loaded document is in the file.
    places: Origins,
    /// `places` by the place a member was moved to, made when a place is first asked for:
    /// a rule asks once per member it names.
    places_index: std::sync::OnceLock<origin::ByPlace<'static>>,
}

impl Diagnostics {
    /// The findings.
    fn list(&self) -> &[Diagnostic] {
        self.0.as_ref().map_or(&[], |held| held.findings.as_slice())
    }

    /// What is held, made when nothing was.
    fn held(&mut self) -> &mut Held {
        self.0.get_or_insert_with(Box::default)
    }
}

/// The findings alone: the moves of an upgrade in progress and the places of the lists held
/// without some of their elements are bookkeeping.
impl PartialEq for Diagnostics {
    fn eq(&self, other: &Self) -> bool {
        self.list() == other.list()
    }
}

impl fmt::Debug for Diagnostics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Diagnostics").field(&self.list()).finish()
    }
}

impl Diagnostics {
    /// The findings in order.
    pub fn iter(&self) -> std::slice::Iter<'_, Diagnostic> {
        self.list().iter()
    }

    /// The findings in order, each open to a change in place.
    #[doc(hidden)]
    #[allow(
        clippy::iter_without_into_iter,
        reason = "a loader's door, hidden: `&mut Diagnostics` is no public iterator"
    )]
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Diagnostic> {
        match self.0.as_mut() {
            Some(held) => held.findings.iter_mut(),
            None => [].iter_mut(),
        }
    }

    /// How many findings.
    pub fn len(&self) -> usize {
        self.list().len()
    }

    /// No findings.
    pub fn is_empty(&self) -> bool {
        self.list().is_empty()
    }

    /// At least one error-level finding that rejects: a kept error does not
    /// count here, though it is an error everywhere findings are counted
    /// ([`at_least`](Self::at_least), [`first_error`](Self::first_error), the severity).
    pub fn has_errors(&self) -> bool {
        self.list().iter().any(Diagnostic::rejects)
    }

    /// The most severe finding's severity, if any.
    pub fn max_severity(&self) -> Option<Severity> {
        self.list().iter().map(|d| d.severity).max()
    }

    /// Findings at or above `severity`.
    pub fn at_least(&self, severity: Severity) -> impl Iterator<Item = &Diagnostic> + '_ {
        self.list().iter().filter(move |d| d.severity >= severity)
    }

    /// Appends a finding.
    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.held().findings.push(diagnostic);
    }

    /// Records `diagnostic` as a kept error: `Severity::Error`, marked
    /// [`kept`](Diagnostic::kept), so the document it is about loads all the same.
    ///
    /// Use it only where the game does both: it writes the line to the content log at error
    /// level AND its loader then continues with the document. A finding of the crate's own,
    /// which the game does not report, never uses it; a game error after which the game drops
    /// the document is a plain [`push`](Self::push) of an error.
    pub fn push_kept_error(&mut self, diagnostic: Diagnostic) {
        self.push(diagnostic.into_kept());
    }

    /// Records that an upgrade step moved the member at `from` to `to` (paths from the
    /// document root): a finding told from here on at or below `to` is about the member the
    /// file has at `from` ([`Diagnostics::name_as_the_file_has_it`]).
    #[doc(hidden)]
    pub fn moved(&mut self, from: JsonPath, to: JsonPath) {
        let told = self.list().len();
        self.held().moves.moved(from, to, told);
    }

    /// Records that the list at `list` now holds, in order, the elements it had at the
    /// indices `kept` (ascending: a reader, a step or a check left the others out): one
    /// move for each element that is no longer at its index. A finding told from here on
    /// of an element of the list, at the index it is held at, is named at the index the
    /// file has ([`Diagnostics::name_as_the_file_has_it`]), and so is a member a rule names
    /// once the document has loaded ([`Diagnostics::place_of`]). A list kept whole records
    /// nothing.
    ///
    /// Two rules, which the moves cannot check afterwards:
    ///
    /// - Call it after everything the caller tells of the list at the indices it had
    ///   before (a reader: the file's). A finding told at such an index after the record
    ///   would be renumbered a second time.
    /// - `list` is the path the list has at the moment of the call, every index of it as
    ///   the elements above are held then. A reader records an inner list before the outer
    ///   one, by the file's index of the outer element; a step that shortens an inner list
    ///   after the outer one was recorded uses the index the outer element is held at.
    ///   Both orders are named as the file has it. What is wrong is an inner list recorded
    ///   after its outer list under the outer element's FILE index: the outer move is
    ///   undone first, and the inner one is then never reached. Such a path cannot be told
    ///   from a held one (the file's index of one element may be the held index of
    ///   another), so it is not checked.
    #[doc(hidden)]
    pub fn left_out(&mut self, list: &JsonPath, kept: &[usize]) {
        for (now, was) in kept.iter().enumerate() {
            if *was != now {
                let (mut from, mut to) = (list.clone(), list.clone());
                from.push_index(*was);
                to.push_index(now);
                self.moved(from, to);
            }
        }
    }

    /// The path the member at `at` (a path of the document as the steps have made it so
    /// far) has in the file: every recorded move undone, the latest first. For a step that
    /// holds where the file has a member (a block's write refusal names it).
    #[doc(hidden)]
    pub fn source_of(&self, at: &JsonPath) -> JsonPath {
        self.0
            .as_ref()
            .map_or_else(|| at.clone(), |held| held.moves.source_of(at))
    }

    /// The findings and the recorded moves, in the order they were made: for a reader that
    /// gathers the findings of one member with paths relative to it, and hands them on
    /// below the member's own path (a move goes on through [`Diagnostics::moved`]).
    #[doc(hidden)]
    pub fn into_told(self) -> Vec<Told> {
        let held = self.0.map(|held| *held).unwrap_or_default();
        let mut moves = held.moves.into_moves().into_iter().peekable();
        let mut out = Vec::with_capacity(held.findings.len());
        for (index, finding) in held.findings.into_iter().enumerate() {
            while let Some((_, from, to)) = moves.next_if(|(told, _, _)| *told <= index) {
                out.push(Told::Moved { from, to });
            }
            out.push(Told::Finding(Box::new(finding)));
        }
        out.extend(moves.map(|(_, from, to)| Told::Moved { from, to }));
        out
    }

    /// Gives every finding told after a recorded move the path its member has in the file
    /// ([`Origins`]), and forgets the moves. Called once, when the chain and the
    /// checks on its result are done.
    #[doc(hidden)]
    pub fn name_as_the_file_has_it(&mut self) {
        let Some(held) = self.0.as_mut() else {
            return;
        };
        let moves = std::mem::take(&mut held.moves);
        if moves.is_empty() {
            return;
        }
        let by_place = moves.by_place();
        for (index, finding) in held.findings.iter_mut().enumerate() {
            finding.location.json_path = moves.undo(&by_place, &finding.location.json_path, index);
        }
        drop(by_place);
        held.places.append(moves, 0);
        held.places_index = std::sync::OnceLock::new();
    }

    /// The path the member at `at` (a path of the loaded document) has in the file, by the
    /// moves [`Diagnostics::name_as_the_file_has_it`] has undone for the findings: for what
    /// names a member of the document once it has loaded.
    ///
    /// Every index and key of `at` is the one the document HOLDS, at every level: a path
    /// whose outer indices are already the file's may be moved a second time. A walk builds
    /// the held path of each member and asks once per member. Before the moves are undone
    /// (a document from `parse_as_written`, whose upgrade has not run) nothing is known yet
    /// and `at` comes back unchanged; so does a path no move touches.
    #[doc(hidden)]
    pub fn place_of(&self, at: &JsonPath) -> JsonPath {
        let Some(held) = self.0.as_ref().filter(|held| !held.places.is_empty()) else {
            return at.clone();
        };
        let index = held
            .places_index
            .get_or_init(|| held.places.by_place().into_owned());
        held.places.undo(index, at, usize::MAX)
    }

    /// Appends the findings of `other`, with the moves recorded among them (each still
    /// before the findings told after it) and the places of the moves it has undone.
    #[doc(hidden)]
    pub fn append(&mut self, other: Diagnostics) {
        let Some(other) = other.0 else {
            return;
        };
        let held = self.held();
        let told = held.findings.len();
        held.findings.extend(other.findings);
        held.moves.append(other.moves, told);
        held.places.append(other.places, 0);
        held.places_index = std::sync::OnceLock::new();
    }

    /// Puts every finding in the pack and file of `origin`, in place: the moves and places
    /// the list holds stay, so the link pass can still ask where a member of the document is
    /// in the file ([`place_of`](Self::place_of)). Rebuilding the list from its findings (a
    /// `for` over the list, pushing each) would drop them.
    pub fn locate(&mut self, origin: &Origin) {
        for finding in self.iter_mut() {
            finding.location.pack = Some(origin.pack.clone());
            finding.location.path = Some(origin.path.clone());
        }
    }

    /// [`locate`](Self::locate), by value.
    #[must_use]
    pub fn located(mut self, origin: &Origin) -> Self {
        self.locate(origin);
        self
    }

    /// Keeps the findings `keep` says to keep, in order. A recorded move stays before the
    /// findings told after it, and the places stay.
    #[doc(hidden)]
    pub fn retain(&mut self, mut keep: impl FnMut(&Diagnostic) -> bool) {
        let Some(held) = self.0.as_mut() else {
            return;
        };
        // How many of the first `n` findings stay, for each `n`.
        let mut kept = Vec::with_capacity(held.findings.len() + 1);
        kept.push(0);
        held.findings.retain(|finding| {
            let stays = keep(finding);
            kept.push(kept.last().copied().unwrap_or_default() + usize::from(stays));
            stays
        });
        held.moves.retold(&kept);
    }

    /// The first error-level finding, if any.
    pub fn first_error(&self) -> Option<&Diagnostic> {
        self.list().iter().find(|d| d.severity == Severity::Error)
    }

    /// For error messages: the first error, else the most severe finding, else "no findings".
    pub fn summary(&self) -> impl fmt::Display + '_ {
        Summary(self)
    }

    /// The finding [`Diagnostics::summary`] names: the first of the most severe.
    fn headline(&self) -> Option<&Diagnostic> {
        let max = self.max_severity()?;
        self.list().iter().find(|d| d.severity == max)
    }
}

/// One entry of [`Diagnostics::into_told`].
#[doc(hidden)]
#[derive(Clone, Debug, PartialEq)]
pub enum Told {
    /// A finding.
    Finding(Box<Diagnostic>),
    /// The member at `from` was moved to `to`.
    Moved {
        /// Where the member was.
        from: JsonPath,
        /// Where it is now.
        to: JsonPath,
    },
}

/// [`Diagnostics::summary`].
struct Summary<'a>(&'a Diagnostics);

impl fmt::Display for Summary<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.headline() {
            Some(diagnostic) => diagnostic.fmt(f),
            None => f.write_str("no findings"),
        }
    }
}

impl Extend<Diagnostic> for Diagnostics {
    fn extend<I: IntoIterator<Item = Diagnostic>>(&mut self, iter: I) {
        self.held().findings.extend(iter);
    }
}

impl FromIterator<Diagnostic> for Diagnostics {
    fn from_iter<I: IntoIterator<Item = Diagnostic>>(iter: I) -> Self {
        let findings: Vec<Diagnostic> = iter.into_iter().collect();
        if findings.is_empty() {
            return Self(None);
        }
        Self(Some(Box::new(Held {
            findings,
            moves: Origins::default(),
            places: Origins::default(),
            places_index: std::sync::OnceLock::new(),
        })))
    }
}

impl IntoIterator for Diagnostics {
    type Item = Diagnostic;
    type IntoIter = std::vec::IntoIter<Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.0
            .map(|held| held.findings)
            .unwrap_or_default()
            .into_iter()
    }
}

impl<'a> IntoIterator for &'a Diagnostics {
    type Item = &'a Diagnostic;
    type IntoIter = std::slice::Iter<'a, Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.list().iter()
    }
}

/// Manifest and container findings of one pack: the field set of the game's pack report.
///
/// Built by the pack loader; `has_errors` is what decides, in vanilla, whether a manifest was
/// valid.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct PackReport {
    /// Errors and warnings about the manifest and the container (vanilla keeps two lists; the
    /// severity tells them apart).
    pub diagnostics: Diagnostics,
    /// The manifest was upgraded from an older format.
    pub was_upgraded: bool,
    /// An upgrade was attempted.
    pub attempted_upgrade: bool,
    /// Where the pack was found.
    pub location: Place,
    /// The base-game version the pack requires, when it declares one.
    pub required_base_game_version: Option<BaseGameVersion>,
    /// The header name as written.
    pub original_name: String,
    /// The header version as written.
    pub original_version: String,
    /// `(uuid, version)` of the header; `None` when the manifest yields no identity.
    pub identity: Option<PackIdVersion>,
    /// What the pack holds.
    pub pack_type: PackType,
    /// Where the pack comes from.
    pub pack_origin: PackOrigin,
}

impl PackReport {
    /// An empty report for a pack found at `location`. The struct is `#[non_exhaustive]`, so
    /// `bedrock_addon` builds it here and fills the fields.
    #[doc(hidden)]
    pub fn new(location: Place, pack_type: PackType, pack_origin: PackOrigin) -> Self {
        Self {
            diagnostics: Diagnostics::default(),
            was_upgraded: false,
            attempted_upgrade: false,
            location,
            required_base_game_version: None,
            original_name: String::new(),
            original_version: String::new(),
            identity: None,
            pack_type,
            pack_origin,
        }
    }

    /// The findings about the manifest and the container.
    pub fn diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }

    /// At least one rejecting error-level finding (a kept error does not count), as the
    /// game's pack report counts errors.
    pub fn has_errors(&self) -> bool {
        self.diagnostics.has_errors()
    }

    /// For error messages: the first error, else the most severe finding, else "no findings".
    pub fn summary(&self) -> impl fmt::Display + '_ {
        self.diagnostics.summary()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn components() -> JsonPath {
        JsonPath::of_keys(["minecraft:block", "components"])
    }

    /// Each typed payload a legacy code carries, as a detail.
    fn typed_details() -> Vec<DiagnosticDetail> {
        let pack = |n: u128| {
            PackIdVersion::new(
                uuid::Uuid::from_u128(n),
                crate::version::SemVersion::new(1, 0, 0),
                PackType::Resource,
            )
        };
        vec![
            DiagnosticDetail::Shadow(crate::visual::ShadowReason::OlderMinEngineVersion),
            DiagnosticDetail::Packs {
                pack: pack(1),
                dependency: Some(pack(2)),
            },
            DiagnosticDetail::Packs {
                pack: pack(1),
                dependency: None,
            },
            DiagnosticDetail::Io(std::io::ErrorKind::NotFound),
            DiagnosticDetail::Uuid(uuid::Uuid::from_u128(3)),
            DiagnosticDetail::NotFound(NotFoundHint::CaseMismatch),
            DiagnosticDetail::FormatVersion(crate::version::FormatVersion::new(1, 21, 0)),
        ]
    }

    /// A typed payload rides on the finding as data: compared, cloned, and not in the line.
    #[test]
    fn a_typed_payload_is_data_on_the_finding_and_not_in_its_line() {
        for detail in typed_details() {
            let bare = Diagnostic::warn(DiagnosticCode::InternalError);
            let with = bare.clone().detail(detail.clone());
            assert_eq!(with.detail.as_deref(), Some(&detail));
            assert_eq!(with.clone(), with);
            assert_ne!(with, bare);
            assert_eq!(with.to_string(), bare.to_string(), "{detail:?}");
        }
        let details = typed_details();
        for (i, a) in details.iter().enumerate() {
            for (j, b) in details.iter().enumerate() {
                assert_eq!(a == b, i == j, "{a:?} vs {b:?}");
            }
        }
    }

    #[test]
    fn child_steps_into_a_member_and_leaves_the_base() {
        let base = components();
        let child = base.child("minecraft:friction");
        assert_eq!(
            child.to_string(),
            "minecraft:block/components/minecraft:friction"
        );
        assert_eq!(base, components());
        let mut pushed = components();
        pushed.push("minecraft:friction");
        assert_eq!(child, pushed);
    }

    #[test]
    fn index_steps_into_an_element_unlike_a_key_of_its_digits() {
        let element = components().index(0);
        assert_eq!(element.to_string(), "minecraft:block/components/0");
        let mut pushed = components();
        pushed.push_index(0);
        assert_eq!(element, pushed);
        assert_ne!(element, components().child("0"), "an index is no key");
    }

    #[test]
    fn join_continues_the_base_with_a_relative_path() {
        let relative = JsonPath::new().child("a").index(2);
        let joined = components().join(&relative);
        assert_eq!(joined.to_string(), "minecraft:block/components/a/2");
        let mut appended = components();
        appended.append(&relative);
        assert_eq!(joined, appended);
        assert_eq!(components().join(&JsonPath::new()), components());
    }

    #[test]
    fn with_steps_into_each_key_as_written() {
        let path = JsonPath::new().with(&["minecraft:block", "components", "0"]);
        assert_eq!(path, components().child("0"));
        assert_eq!(path.to_string(), "minecraft:block/components/0");
        assert_eq!(JsonPath::new().with(&["a/b"]).to_string(), "a~1b");
        assert_eq!(components().with::<&str>(&[]), components());
        let owned = vec![String::from("x")];
        assert_eq!(JsonPath::new().with(&owned), JsonPath::of_keys(["x"]));
    }
}
