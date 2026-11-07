//! Specific codes: a finding declared as a `static` next to the rule that raises it, carrying
//! its owner, name, general [`Kind`], grading [`Class`] and, when the game prints one, the game's
//! sentence. Its arity is a const generic, so [`CodeDef::with`] takes exactly the arguments the
//! sentence has placeholders for.
//!
//! ```
//! use bedrock_addon_core::diagnostic::{Class, CodeDef, Kind};
//!
//! static DENSITY_OUT_OF_UNIT: CodeDef<1> = CodeDef::new(
//!     "fog/density_out_of_unit",
//!     Kind::InvalidValue,
//!     Class::Error,
//!     "fog density {} is outside [0, 1]",
//! );
//!
//! let code = DENSITY_OUT_OF_UNIT.with(["1.5"]);
//! assert!(code.is(&DENSITY_OUT_OF_UNIT));
//! assert_eq!(code.id(), "fog/density_out_of_unit");
//! assert_eq!(code.kind(), Kind::InvalidValue);
//! assert_eq!(code.vanilla(), None); // the toolkit's own: the game prints no sentence
//! assert_eq!(code.to_string(), "fog density 1.5 is outside [0, 1]");
//! ```
//!
//! # Where definitions live
//!
//! Each family declares its codes in its own `codes.rs`, a public module of `pub static`
//! definitions, one per sentence, beside the rules that raise them (`families/fog/src/codes.rs`
//! is the model):
//!
//! - the id is `<family>/<the const's name in lower case>` (`fog/end_below_start`);
//! - a code the game prints has its sentence as [`game`](CodeDef::game), byte for byte, and a
//!   line of the crate's own; a code of the toolkit's own has the line alone;
//! - its [`Kind`] says what sort of finding it is, and its [`Class`] the severity its sites raise it at
//!   (a site at another severity keeps it with [`Diagnostic::severity`](super::Diagnostic::severity));
//! - a helper that took a text (`report(d, severity, text, at)`) takes a `&'static CodeDef<0>`,
//!   and a reader that still takes a text takes [`CodeDef::template`];
//! - the family's `texts.rs` goes: its consts become the definitions.
//!
//! A finding no family owns (the door, the envelope, the kit, the builders, the link pass) is
//! a definition of [`codes`](super::codes), the core's: one generic definition per kind of
//! finding, and a `_WORDED` one taking the game's sentence whole for a site whose sentence has
//! no definition of its own yet.

use std::fmt;
use std::hash::{Hash, Hasher};

use super::Severity;

/// What a definition holds, whatever its arity: what a [`Code`] points at, and what a list of
/// definitions of several arities holds ([`CodeDef::meta`], [`codes::ALL`](super::codes::ALL)).
#[derive(Clone, Copy, Debug)]
pub struct CodeMeta {
    id: &'static str,
    owner: &'static str,
    name: &'static str,
    kind: Kind,
    class: Class,
    game: Option<&'static str>,
    line: &'static str,
}

impl CodeMeta {
    /// The stable id, `owner/name`.
    pub const fn id(&self) -> &'static str {
        self.id
    }

    /// Who declares the code.
    pub const fn owner(&self) -> &'static str {
        self.owner
    }

    /// The name within its owner: the id's tail (`density_out_of_unit`).
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// The general kind of finding.
    pub const fn kind(&self) -> Kind {
        self.kind
    }

    /// How the finding is graded.
    pub const fn class(&self) -> Class {
        self.class
    }

    /// The game's sentence, `{}` where each argument goes; `None` for a code of the toolkit's
    /// own.
    pub const fn game_template(&self) -> Option<&'static str> {
        self.game
    }

    /// The crate's own line, `{}` where each argument goes when it has any.
    pub const fn line(&self) -> &'static str {
        self.line
    }

    /// The number of arguments the code takes: the placeholders of its game sentence, else of
    /// its line.
    pub const fn arity(&self) -> usize {
        match self.game {
            Some(game) => placeholders(game),
            None => placeholders(self.line),
        }
    }
}

/// The definition of one specific code, taking `N` arguments.
///
/// Declared as a `static` (or `const`) beside the rule that raises it. [`new`](Self::new) and
/// [`game`](Self::game) are `const fn`s that check the definition when it is evaluated, so a
/// wrong one does not compile:
///
/// - the id is `owner/name`, one `/` between two lower snake case words
///   (`fog/density_out_of_unit`): the code's stable id, written as one literal;
/// - the game's sentence has exactly `N` placeholders (`{}`), one per argument, in order;
/// - the crate's own line is not empty and has either no placeholder or `N` of them.
///
/// An id is unique: no two definitions of one owner share a name. Nothing checks it when a
/// definition is evaluated: the tombstone test (`core/tests/diagnostic_codes.rs`, the list of
/// code names) is the guard, and a new definition adds its id there.
///
/// ```compile_fail
/// use bedrock_addon_core::diagnostic::{Class, CodeDef, Kind};
/// // Two arguments, one placeholder: refused when the static is evaluated.
/// static WRONG: CodeDef<2> =
///     CodeDef::new("fog/wrong", Kind::InvalidValue, Class::Error, "only {}");
/// ```
#[derive(Clone, Copy, Debug)]
pub struct CodeDef<const N: usize> {
    meta: CodeMeta,
}

/// Two definitions are the same when their ids are (a `const` definition has one address per
/// use): what lets a value that holds a definition, a family's `Envelope`, compare.
impl<const N: usize> PartialEq for CodeDef<N> {
    fn eq(&self, other: &Self) -> bool {
        self.meta.id == other.meta.id
    }
}

impl<const N: usize> Eq for CodeDef<N> {}

impl<const N: usize> CodeDef<N> {
    /// A code of the toolkit's own: `line` is the sentence `Display` renders, with no
    /// placeholder or one `{}` per argument. Add the game's sentence with
    /// [`game`](Self::game) when the game prints one.
    ///
    /// # Panics
    ///
    /// When the definition breaks a rule of [`CodeDef`]: a compile-time check in a `const` or
    /// `static`, a panic at run time otherwise.
    #[must_use]
    pub const fn new(id: &'static str, kind: Kind, class: Class, line: &'static str) -> Self {
        let Some(slash) = split_id(id) else {
            panic!("a code's id is `owner/name`, with one `/`");
        };
        let (owner, rest) = id.split_at(slash);
        let (_, name) = rest.split_at(1);
        assert!(is_snake_case(owner), "a code's owner is lower snake case");
        assert!(is_snake_case(name), "a code's name is lower snake case");
        assert!(!line.is_empty(), "a code's line is not empty");
        let holes = placeholders(line);
        assert!(
            holes == 0 || holes == N,
            "a code's line has no placeholder or one per argument"
        );
        Self {
            meta: CodeMeta {
                id,
                owner,
                name,
                kind,
                class,
                game: None,
                line,
            },
        }
    }

    /// The definition with the game's sentence: the content-log text the game prints, one
    /// `{}` per argument. [`Diagnostic::vanilla`](super::Diagnostic::vanilla) renders it.
    ///
    /// # Panics
    ///
    /// When `template` does not have exactly `N` placeholders: a compile-time check in a
    /// `const` or `static`, a panic at run time otherwise.
    #[must_use]
    pub const fn game(mut self, template: &'static str) -> Self {
        assert!(
            placeholders(template) == N,
            "a code's game sentence has one placeholder per argument"
        );
        self.meta.game = Some(template);
        self
    }

    /// The code with its arguments, in the order of the placeholders.
    pub fn with<A: Into<String>>(&'static self, args: [A; N]) -> Code {
        Code {
            meta: &self.meta,
            args: args.into_iter().map(Into::into).collect(),
        }
    }

    /// The stable id, `owner/name` (`fog/density_out_of_unit`), as [`new`](Self::new) got it.
    pub const fn id(&self) -> &'static str {
        self.meta.id
    }

    /// Who declares the code: a family (`"fog"`), `"core"`, `"addon"`.
    pub const fn owner(&self) -> &'static str {
        self.meta.owner
    }

    /// The name within its owner: the id's tail (`density_out_of_unit`).
    pub const fn name(&self) -> &'static str {
        self.meta.name
    }

    /// The general kind of finding.
    pub const fn kind(&self) -> Kind {
        self.meta.kind
    }

    /// How the finding is graded.
    pub const fn class(&self) -> Class {
        self.meta.class
    }

    /// The sentence a code of the definition renders, `{}` where each argument goes: the
    /// game's when there is one, else the crate's line. A `const`, so a reader that still
    /// takes a text (an envelope's `missing` rule) takes it from the definition.
    pub const fn template(&self) -> &'static str {
        match self.meta.game {
            Some(game) => game,
            None => self.meta.line,
        }
    }

    /// What the definition holds, whatever its arity: the entry of a list of definitions.
    pub const fn meta(&self) -> &CodeMeta {
        &self.meta
    }
}

/// A code without arguments, as is.
impl From<&'static CodeDef<0>> for Code {
    fn from(def: &'static CodeDef<0>) -> Self {
        def.with::<String>([])
    }
}

/// A specific code with its arguments: what a [`CodeDef`] gives.
///
/// Two codes are equal when they have the same id and arguments; the definition is not
/// compared by address (a `const` definition has one address per use).
#[derive(Clone, Debug)]
pub struct Code {
    meta: &'static CodeMeta,
    args: Box<[String]>,
}

impl Code {
    /// Who declares the code.
    pub fn owner(&self) -> &'static str {
        self.meta.owner
    }

    /// The name within its owner: the id's tail (`density_out_of_unit`).
    pub fn name(&self) -> &'static str {
        self.meta.name
    }

    /// The stable id: `owner/name` (`fog/density_out_of_unit`).
    pub fn id(&self) -> &'static str {
        self.meta.id
    }

    /// The general kind of finding.
    pub fn kind(&self) -> Kind {
        self.meta.kind
    }

    /// How the finding is graded.
    pub fn class(&self) -> Class {
        self.meta.class
    }

    /// The arguments, in the order of the placeholders.
    pub fn args(&self) -> &[String] {
        &self.args
    }

    /// Whether this is a code of `def`, whatever its arguments.
    pub fn is<const N: usize>(&self, def: &CodeDef<N>) -> bool {
        self.meta.id == def.meta.id
    }

    /// The arguments of a code of `def`; `None` for a code of another definition. What a
    /// site that reads a code's arguments asks.
    pub fn args_of<const N: usize>(&self, def: &CodeDef<N>) -> Option<&[String]> {
        self.is(def).then_some(self.args())
    }

    /// The game's sentence, `{}` where each argument goes; `None` for a code of the toolkit's
    /// own.
    pub(super) fn vanilla_template(&self) -> Option<&'static str> {
        self.meta.game
    }

    /// The game's sentence with the arguments in it; `None` for a code of the toolkit's own.
    pub fn vanilla(&self) -> Option<String> {
        // `Display` renders the game's sentence whenever there is one.
        self.meta.game.map(|_| self.to_string())
    }
}

impl PartialEq for Code {
    fn eq(&self, other: &Self) -> bool {
        self.meta.id == other.meta.id && self.args == other.args
    }
}

impl Eq for Code {}

impl Hash for Code {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.meta.id.hash(state);
        self.args.hash(state);
    }
}

/// The game's sentence when there is one, else the crate's line; the arguments in the
/// placeholders.
impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let template = self.meta.game.unwrap_or(self.meta.line);
        fill(f, template, &self.args)
    }
}

/// `template` with each `{}` replaced by the next of `args` (none left: nothing): a sentence of
/// the game's built from its format string, for a site that holds one as a text (the link
/// pass's sentences, which a `_WORDED` definition carries whole).
///
/// ```
/// use bedrock_addon_core::diagnostic::sentence;
/// assert_eq!(sentence("{} is not a valid item name.", &["a:b"]), "a:b is not a valid item name.");
/// assert_eq!(sentence("no {} here", &[] as &[&str]), "no  here");
/// ```
#[must_use]
pub fn sentence(template: &str, args: &[impl AsRef<str>]) -> String {
    let mut out = String::with_capacity(template.len());
    let _ = fill(&mut out, template, args);
    out
}

/// Writes `template` with each `{}` replaced by the next argument (none left: nothing).
fn fill(out: &mut impl fmt::Write, template: &str, args: &[impl AsRef<str>]) -> fmt::Result {
    let mut args = args.iter();
    let mut parts = template.split("{}");
    if let Some(first) = parts.next() {
        out.write_str(first)?;
    }
    for part in parts {
        out.write_str(args.next().map_or("", AsRef::as_ref))?;
        out.write_str(part)?;
    }
    Ok(())
}

/// The number of `{}` in `text`.
const fn placeholders(text: &str) -> usize {
    let bytes = text.as_bytes();
    let mut count = 0;
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'{' && bytes[i + 1] == b'}' {
            count += 1;
            i += 2;
        } else {
            i += 1;
        }
    }
    count
}

/// The index of the one `/` in `id`; `None` when it has none or more than one.
const fn split_id(id: &str) -> Option<usize> {
    let bytes = id.as_bytes();
    let mut slash = None;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'/' {
            if slash.is_some() {
                return None;
            }
            slash = Some(i);
        }
        i += 1;
    }
    slash
}

/// Not empty, of `a-z`, `0-9` and `_`, starting with a letter.
const fn is_snake_case(text: &str) -> bool {
    let bytes = text.as_bytes();
    if bytes.is_empty() || !bytes[0].is_ascii_lowercase() {
        return false;
    }
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if !(b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_') {
            return false;
        }
        i += 1;
    }
    true
}

/// How a code is graded: the severity a finding of it gets unless the raise site says
/// otherwise ([`Diagnostic::severity`](super::Diagnostic::severity)).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Class {
    /// The game rejects what the finding is about: an error.
    Error,
    /// The game logs an error and keeps the document: an error that is
    /// [`kept`](super::Diagnostic::kept).
    KeptError,
    /// An unknown member or component, graded where it is raised by the reading of the slice
    /// that reads it ([`Reading::grade`](crate::versioning::Reading::grade), through
    /// `LoadContext::grade`, `UpgradeCx::grade` or `ValidateCx::grade`): an error under a
    /// strict [`UnknownMask`](crate::versioning::UnknownMask), and the document is refused; a
    /// warning under an unstrict one. Nothing raises its severity after the read: a site that
    /// tells one at another severity says so where it raises it.
    Unknown,
    /// The document loads: a warning.
    Warning,
    /// The toolkit's own advice, which the game does not print: information.
    Lint,
}

impl Class {
    /// The severity a finding of this class gets by default.
    pub const fn default_severity(self) -> Severity {
        match self {
            Self::Error | Self::KeptError => Severity::Error,
            Self::Unknown | Self::Warning => Severity::Warning,
            Self::Lint => Severity::Info,
        }
    }
}

/// The general kind of a finding, for the question "is this an invalid value" whichever
/// rule raised it; [`Diagnostic::is`](super::Diagnostic::is) asks the specific one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Kind {
    /// The text is not the document's shape: a JSON syntax error, an empty file, a root or
    /// node of the wrong shape.
    Syntax,
    /// The envelope around the payload: a missing, extra or misnamed payload key.
    Envelope,
    /// The closing line of a document that failed: the finding after the ones that rejected
    /// it, which says the document as a whole did not load.
    Closing,
    /// The `format_version` member: absent, malformed, unusual, a prerelease, above the engine.
    FormatVersion,
    /// A version no parser of the family covers.
    UnsupportedVersion,
    /// A member or value form against the engine version (`min_engine_version`, the target
    /// engine, a manifest version bound).
    EngineVersion,
    /// Content that needs a toggle, an experiment or a capability.
    Toggle,
    /// A member or component the schema does not declare, and what became of it: kept, not
    /// kept, dropped by a rule, or ignored as the game ignores it. One kind, four facts: the
    /// question `matches!(d.kind(), Kind::UnknownMember(_))` is the general one.
    UnknownMember(Disposition),
    /// A deprecated member or format.
    Deprecated,
    /// A required member is absent.
    MissingRequired,
    /// A value of the wrong JSON type or shape.
    WrongType,
    /// A value the member does not accept.
    InvalidValue,
    /// A string outside its enum, or a node type no reader knows.
    InvalidEnumValue,
    /// A value outside its domain, dropped.
    OutOfRange,
    /// A value clamped into its range.
    Clamped,
    /// A value or name the load holds in another form than the file has it in.
    Rewritten,
    /// A Molang expression: a parser message, a lint, or one not checked.
    Molang,
    /// An identifier against a namespace rule.
    Identifier,
    /// Two of a thing where one is allowed.
    Duplicate,
    /// A definition that does not take effect because another one of its name does: one of
    /// higher priority, or the lower one a failed definition falls back to.
    Shadowed,
    /// A name that resolves to nothing.
    UnresolvedReference,
    /// A name that resolves to something the use cannot accept.
    InvalidReference,
    /// References that form a cycle.
    Cycle,
    /// What a higher pack's content replaces or clears in the lower packs'.
    Overridden,
    /// Content that never takes effect where it is written.
    NeverApplies,
    /// The toolkit's own advice on style.
    Lint,
    /// An asset file: an image, a sound or a pack icon.
    Asset,
    /// A pack-level rule: a skin pack, a misplaced pack, an added dependency.
    Pack,
    /// A file that could not be read or is not read.
    Io,
    /// A bug or panic inside the toolkit.
    Internal,
    /// A script module dependency.
    ScriptModule,
    /// A script source file or import.
    ScriptSource,
    /// A server configuration file.
    Config,
}

/// What became of a member or component the schema does not declare: the payload of
/// [`Kind::UnknownMember`].
///
/// The reader's policy, [`kit::Disposition`](crate::json::kit::Disposition), converts into it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Disposition {
    /// Told and kept: the member goes to the document's unknown members and a written document
    /// has it (`UnrecognizedField`, `UnrecognizedComponent`).
    Kept,
    /// Told and not kept: nothing reads it and a written document lacks it (`MemberNotKept`).
    NotKept,
    /// Dropped by a rule (`DroppedField`, `DroppedComponent`, the older of two versions of one
    /// component).
    Dropped,
    /// Read and ignored, as the game ignores it (`IgnoredMember`, an ignored property,
    /// literal, setting, legacy entry or filter).
    Ignored,
}

impl From<crate::json::kit::Disposition> for Disposition {
    fn from(disposition: crate::json::kit::Disposition) -> Self {
        use crate::json::kit::Disposition as Policy;
        match disposition {
            Policy::Keep => Self::Kept,
            Policy::Report => Self::NotKept,
            Policy::Reject => Self::Dropped,
            Policy::Ignore => Self::Ignored,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{Diagnostic, Location, Phase, codes};
    use std::collections::hash_map::DefaultHasher;

    static FILE_MISSING: CodeDef<1> = CodeDef::new(
        "core/file_missing",
        Kind::UnresolvedReference,
        Class::Error,
        "no file {}",
    )
    .game("Failed to load file: {}");

    /// The same id as `FILE_MISSING`, declared again as a `const`: one address per use.
    const FILE_MISSING_AGAIN: CodeDef<1> = CodeDef::new(
        "core/file_missing",
        Kind::UnresolvedReference,
        Class::Error,
        "no file {}",
    );

    static TRAILING_COMMA: CodeDef<0> = CodeDef::new(
        "core/trailing_comma",
        Kind::Lint,
        Class::Lint,
        "a trailing comma",
    );

    /// A family's definition.
    static DENSITY_OUT_OF_UNIT: CodeDef<1> = CodeDef::new(
        "fog/density_out_of_unit",
        Kind::InvalidValue,
        Class::Error,
        "fog density {} is outside [0, 1]",
    );

    fn finding(code: impl Into<Code>) -> Diagnostic {
        Diagnostic::new(code, Severity::Error, Phase::Parse, Location::default())
    }

    fn hash_of(code: &Code) -> u64 {
        let mut hasher = DefaultHasher::new();
        code.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn a_game_sentence_is_rendered_with_its_arguments() {
        let code = FILE_MISSING.with(["textures/a.png"]);
        assert_eq!(
            code.vanilla().as_deref(),
            Some("Failed to load file: textures/a.png")
        );
        assert_eq!(code.to_string(), "Failed to load file: textures/a.png");
    }

    #[test]
    fn a_code_of_the_toolkit_renders_its_line_and_no_game_sentence() {
        let code = Code::from(&TRAILING_COMMA);
        assert_eq!(code.vanilla(), None);
        assert_eq!(code.to_string(), "a trailing comma");
        assert_eq!(FILE_MISSING_AGAIN.with(["a"]).to_string(), "no file a");
    }

    #[test]
    fn codes_are_equal_by_owner_name_and_arguments() {
        let a = FILE_MISSING.with(["a"]);
        let again = FILE_MISSING_AGAIN.with(["a"]);
        assert_eq!(a, again);
        assert_eq!(hash_of(&a), hash_of(&again));
        assert_ne!(a, FILE_MISSING.with(["b"]));
        assert_ne!(Code::from(&TRAILING_COMMA), a);
    }

    #[test]
    fn the_stable_id_is_owner_slash_name() {
        let code = FILE_MISSING.with(["a"]);
        assert_eq!(code.id(), "core/file_missing");
        assert_eq!(code.name(), "file_missing");
        assert_eq!(code.owner(), "core");
    }

    /// A definition compares by its id: the static and a `const` of the same id are one
    /// definition, another id is another (what lets a family's `Envelope` holding one compare).
    #[test]
    fn two_definitions_are_equal_when_their_ids_are() {
        assert_eq!(FILE_MISSING, FILE_MISSING_AGAIN);
        assert_ne!(
            codes::MISSING_FORMAT_VERSION,
            codes::NO_VALID_VERSION,
            "two ids"
        );
    }

    /// The arguments of a code of one definition, asked of the code: none for another
    /// definition.
    #[test]
    fn a_codes_arguments_are_given_for_its_own_definition_only() {
        let code = codes::INVALID_MOLANG.with(["1 +"]);
        assert_eq!(
            code.args_of(&codes::INVALID_MOLANG),
            Some(&["1 +".to_owned()][..])
        );
        assert_eq!(code.args_of(&codes::MOLANG_LINT), None);
    }

    #[test]
    fn a_definitions_id_is_its_literal_split_at_the_slash() {
        assert_eq!(FILE_MISSING.id(), "core/file_missing");
        assert_eq!(FILE_MISSING.owner(), "core");
        assert_eq!(FILE_MISSING.name(), "file_missing");
        let id: &'static str = FILE_MISSING.with(["a"]).id();
        assert_eq!(id, "core/file_missing");
        assert_eq!(split_id("a/b"), Some(1));
        assert_eq!(split_id("ab"), None);
        assert_eq!(split_id("a/b/c"), None);
    }

    #[test]
    fn a_definitions_name_is_the_tail_of_its_id() {
        assert_eq!(DENSITY_OUT_OF_UNIT.name(), "density_out_of_unit");
        assert_eq!(
            DENSITY_OUT_OF_UNIT.with(["2"]).name(),
            "density_out_of_unit"
        );
        assert_eq!(DENSITY_OUT_OF_UNIT.meta().name(), "density_out_of_unit");
        assert_eq!(DENSITY_OUT_OF_UNIT.owner(), "fog");
    }

    #[test]
    fn a_definitions_meta_holds_what_it_declares_whatever_its_arity() {
        let meta = FILE_MISSING.meta();
        assert_eq!(meta.id(), "core/file_missing");
        assert_eq!(meta.owner(), "core");
        assert_eq!(meta.name(), "file_missing");
        assert_eq!(meta.kind(), Kind::UnresolvedReference);
        assert_eq!(meta.class(), Class::Error);
        assert_eq!(meta.game_template(), Some("Failed to load file: {}"));
        assert_eq!(meta.line(), "no file {}");
        assert_eq!(meta.arity(), 1);
        assert_eq!(FILE_MISSING.template(), "Failed to load file: {}");
        assert_eq!(TRAILING_COMMA.template(), "a trailing comma");
        let metas: [&CodeMeta; 2] = [TRAILING_COMMA.meta(), DENSITY_OUT_OF_UNIT.meta()];
        assert_eq!(metas[0].arity(), 0);
        assert_eq!(metas[0].game_template(), None);
        assert_eq!(metas[1].name(), "density_out_of_unit");
    }

    #[test]
    fn is_asks_for_the_definition_whatever_the_arguments() {
        let d = finding(FILE_MISSING.with(["a"]));
        assert!(d.is(&FILE_MISSING));
        assert!(d.is(&FILE_MISSING_AGAIN));
        assert!(!d.is(&TRAILING_COMMA));
        assert!(!finding(&codes::INVALID_VALUE).is(&FILE_MISSING));
        assert!(FILE_MISSING.with(["a"]).is(&FILE_MISSING_AGAIN));
        assert!(!FILE_MISSING.with(["a"]).is(&TRAILING_COMMA));
        assert!(!Code::from(&codes::INVALID_VALUE).is(&DENSITY_OUT_OF_UNIT));
    }

    #[test]
    fn kind_is_the_definitions() {
        assert_eq!(
            finding(FILE_MISSING.with(["a"])).kind(),
            Kind::UnresolvedReference
        );
        assert_eq!(finding(&TRAILING_COMMA).kind(), Kind::Lint);
        assert_eq!(finding(&codes::INVALID_VALUE).kind(), Kind::InvalidValue);
        assert_eq!(
            finding(&codes::UNRECOGNIZED_FIELD).kind(),
            Kind::UnknownMember(Disposition::Kept)
        );
    }

    /// The unknown-member codes are one kind, told apart by what became of the member; the
    /// neighbours that drop or ignore a member go with them.
    #[test]
    fn the_unknown_member_codes_are_one_kind_with_their_disposition() {
        use Disposition as D;
        let rows: [(&CodeMeta, D); 12] = [
            (codes::UNRECOGNIZED_FIELD.meta(), D::Kept),
            (codes::UNRECOGNIZED_COMPONENT.meta(), D::Kept),
            (codes::MEMBER_NOT_KEPT.meta(), D::NotKept),
            (codes::DROPPED_FIELD.meta(), D::Dropped),
            (codes::DROPPED_COMPONENT.meta(), D::Dropped),
            (codes::DUPLICATE_COMPONENT_VERSION.meta(), D::Dropped),
            (codes::IGNORED_MEMBER.meta(), D::Ignored),
            (codes::IGNORED_PROPERTY.meta(), D::Ignored),
            (codes::LITERAL_IGNORED.meta(), D::Ignored),
            (codes::LEGACY_SETTING_IGNORED.meta(), D::Ignored),
            (codes::ATOMIC_FILTER_LIGHTING_IGNORED.meta(), D::Ignored),
            (codes::LEGACY_FOG_IGNORED.meta(), D::Ignored),
        ];
        for (meta, disposition) in rows {
            assert_eq!(
                meta.kind(),
                Kind::UnknownMember(disposition),
                "{}",
                meta.id()
            );
        }
    }

    #[test]
    fn the_readers_policy_converts_into_the_disposition() {
        use crate::json::kit::Disposition as Policy;
        assert_eq!(Disposition::from(Policy::Keep), Disposition::Kept);
        assert_eq!(Disposition::from(Policy::Report), Disposition::NotKept);
        assert_eq!(Disposition::from(Policy::Reject), Disposition::Dropped);
        assert_eq!(Disposition::from(Policy::Ignore), Disposition::Ignored);
    }

    #[test]
    fn a_closing_line_an_unknown_node_and_a_fallback_have_their_own_kinds() {
        assert_eq!(codes::DOCUMENT_REJECTED.kind(), Kind::Closing);
        assert_eq!(codes::UNKNOWN_NODE.kind(), Kind::InvalidEnumValue);
        assert_eq!(codes::RENDER_CONTROLLER_FALLBACK.kind(), Kind::Shadowed);
    }

    #[test]
    fn vanilla_is_the_game_sentence_of_the_definition() {
        let with = finding(codes::UNRECOGNIZED_FIELD_WORDED.with(["child 'zz' not valid here."]));
        assert_eq!(
            with.vanilla().as_deref(),
            Some("child 'zz' not valid here.")
        );
        assert_eq!(finding(&codes::UNRECOGNIZED_FIELD).vanilla(), None);
        assert_eq!(
            finding(FILE_MISSING.with(["x"])).vanilla().as_deref(),
            Some("Failed to load file: x")
        );
    }

    #[test]
    fn a_specific_finding_renders_in_the_content_log_line() {
        let d = finding(FILE_MISSING.with(["x"]));
        assert_eq!(d.to_string(), "[error] Failed to load file: x");
    }

    #[test]
    fn each_class_grades_to_its_default_severity() {
        assert_eq!(Class::Error.default_severity(), Severity::Error);
        assert_eq!(Class::KeptError.default_severity(), Severity::Error);
        assert_eq!(Class::Unknown.default_severity(), Severity::Warning);
        assert_eq!(Class::Warning.default_severity(), Severity::Warning);
        assert_eq!(Class::Lint.default_severity(), Severity::Info);
    }

    #[test]
    fn placeholders_are_counted_as_pairs_of_braces() {
        assert_eq!(placeholders(""), 0);
        assert_eq!(placeholders("{} and {}"), 2);
        assert_eq!(placeholders("{x} {"), 0);
        assert!(is_snake_case("density_out_of_unit2"));
        assert!(!is_snake_case("Density"));
        assert!(!is_snake_case("a/b"));
        assert!(!is_snake_case(""));
    }
}
