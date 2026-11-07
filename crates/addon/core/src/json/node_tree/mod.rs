//! The walk the game's JSON schema node trees make of a file, for the loaders that validate with
//! one instead of a typed facet parse (render controllers, fog below 1.26.20, geometry, both
//! animation kinds).
//!
//! A node tree names the members of each object by name groups ([`Group`]: a literal name or a
//! regex, [`Pattern`], and whether a member must match it), types each value, and keeps
//! validating after the first failure. Each object level is read from the file text as
//! `IndexMap<String, RawJson>` and each value at its own slice ([`Node`]), so a failure in one
//! entry does not stop the others and findings carry file spans.
//!
//! The loaders' trees differ in five rules, which a walk is given as its [`Rules`]:
//!
//! | rule | what it decides |
//! |---|---|
//! | [`Names`] | a literal name and a prefix compare exactly, or without regard to ASCII case (then a member spelled in another case is noted in [`Walk::non_canonical`], and of two members one name in two cases the earlier is told as dropped) |
//! | [`Order`] | the members are visited in the sorted order of their keys (jsoncpp's `std::map`) or in the file's order |
//! | [`Validation`] | an undeclared or wrong-typed member is an error, or an information and the member is removed |
//! | [`Failure`] | an object or array fails when a member's read failed, or only when an error was raised below it |
//! | [`Lossy`] | a number a float does not hold is told with a `LossyConversion` information, or read silently |
//!
//! Each is the game's rule for its loader until it is known to be otherwise; the loaders name
//! their own `Rules`.

use std::sync::Arc;

use facet_json::RawJson;
use indexmap::IndexMap;

use super::{JsonSlice, Value, parse_at};
use crate::diagnostic::codes as core_codes;
use crate::diagnostic::{Code, Diagnostic, Diagnostics, JsonPath, Severity, Span, cereal};
use crate::molang::{JsonScalar, Molang, OtherJson, RawVersion, ReadField, ReaderKind};

// ---- vanilla's schema texts -------------------------------------------------------------------

/// A member no name group matches.
pub const CHILD_NOT_VALID: &str = core_codes::CHILD_NOT_VALID.template();
/// A matched name whose options all refuse the value's JSON type.
pub const UNKNOWN_OPTION_TYPE: &str = cereal::UNKNOWN_CHILD_OPTION_TEXT;
/// A required name group without a member.
pub const REQUIRED_CHILD: &str = core_codes::REQUIRED_CHILD.template();
/// A bool outside its allowed-value set.
pub const VALUE_NOT_ALLOWED: &str = core_codes::VALUE_NOT_ALLOWED.template();
/// The string node's valid-value regexes.
pub const NOT_ALLOWED_REGEX: &str = core_codes::NOT_ALLOWED_REGEX.template();
/// The enum node's valid keys.
pub const NOT_ALLOWED_KEYS: &str = core_codes::NOT_ALLOWED_KEYS.template();
/// The float and int nodes' range.
pub const OUTSIDE_RANGE: &str = core_codes::OUTSIDE_RANGE.template();
/// The array node's minimum element count.
pub const ARRAY_TOO_SMALL: &str = core_codes::ARRAY_TOO_SMALL.template();
/// The array node's maximum element count.
pub const ARRAY_TOO_LARGE: &str = core_codes::ARRAY_TOO_LARGE.template();
/// A version below the node's minimum.
pub const BELOW_MIN_VERSION: &str = "Version {} below min required version ({})";
/// A version above the node's maximum.
pub const ABOVE_MAX_VERSION: &str = "Version {} greater than max allowed version ({})";
/// The JSON reader failed on the file.
pub const PARSE_ERROR: &str = core_codes::PARSE_ERROR.template();

/// C's `%f` of a float: six decimals, no exponent.
pub fn percent_f(x: f32) -> String {
    format!("{:.6}", f64::from(x))
}

// ---- JSON nodes --------------------------------------------------------------------------------

/// The JSON type of a node, from its first byte.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    /// `null`, or no text at all.
    Null,
    /// `true` or `false`.
    Bool,
    /// A number.
    Number,
    /// A string.
    String,
    /// An array.
    Array,
    /// An object.
    Object,
}

/// One JSON value of the file, as its slice of the text (trimmed to the value's own tokens).
#[derive(Clone, Copy, Debug)]
pub struct Node<'a> {
    slice: JsonSlice<'a>,
}

impl<'a> Node<'a> {
    /// The value of a whole text (comments and whitespace around it skipped); `None` when the
    /// text is not JSON.
    pub fn root(text: &'a str) -> Option<Self> {
        Self::of(JsonSlice::new(text))
    }

    /// The value of a slice (comments and whitespace around it skipped); `None` when the text
    /// is not JSON.
    pub fn of(slice: JsonSlice<'a>) -> Option<Self> {
        let raw: RawJson<'a> = parse_at(slice).ok()?;
        slice.nested(&raw).map(|slice| Self { slice })
    }

    /// The JSON type of the node, from its first byte.
    pub fn kind(&self) -> Kind {
        match self.slice.text.as_bytes().first() {
            Some(b'{') => Kind::Object,
            Some(b'[') => Kind::Array,
            Some(b'"') => Kind::String,
            Some(b't' | b'f') => Kind::Bool,
            Some(b'n') | None => Kind::Null,
            Some(_) => Kind::Number,
        }
    }

    /// The value as a double, when it is a number the reader holds as one.
    pub fn number(&self) -> Option<f64> {
        (self.kind() == Kind::Number)
            .then(|| parse_at::<f64>(self.slice).ok())
            .flatten()
    }

    /// The value as a string with its escapes resolved, when it is a string.
    pub fn string(&self) -> Option<String> {
        (self.kind() == Kind::String)
            .then(|| parse_at::<String>(self.slice).ok())
            .flatten()
    }

    /// The value's text.
    pub fn text(&self) -> &'a str {
        self.slice.text
    }

    /// The node's place in the file, saturated to `u32`.
    pub fn span(&self) -> Span {
        let to_u32 = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
        Span {
            offset: to_u32(self.slice.offset),
            len: to_u32(self.slice.text.len()),
        }
    }

    fn nested(&self, raw: &RawJson<'a>) -> Option<Node<'a>> {
        self.slice.nested(raw).map(|slice| Node { slice })
    }
}

/// Whether two nodes hold one JSON value, whatever the spacing and the order of their members.
fn same_value(a: Node<'_>, b: Node<'_>) -> bool {
    match (parse_at::<Value>(a.slice), parse_at::<Value>(b.slice)) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

// ---- member-name patterns ---------------------------------------------------------------------

/// A member name or pattern of a schema object, matched in full unless it says otherwise. A
/// literal name and a prefix compare as the walk's [`Names`] say.
///
/// Two patterns are equal when they are one pattern; a [`Custom`](Self::Custom) one compares
/// its function by address. Rust does not promise one address per function, so that equality is
/// best-effort and serves only the hashability bound; no walk decides anything by it.
#[allow(
    unpredictable_function_pointer_comparisons,
    reason = "a `Custom` pattern is a loader's `const`, compared with itself"
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Pattern {
    /// A literal member name.
    Name(&'static str),
    /// `<prefix>` then one or more of `[a-zA-Z0-9]` and `extra`.
    Prefixed {
        /// The prefix, unescaped (`controller.render.`).
        prefix: &'static str,
        /// The bytes the tail takes beside letters and digits.
        extra: &'static [u8],
        /// The regex as vanilla prints it.
        regex: &'static str,
    },
    /// One or more of `[a-zA-Z0-9]` and `extra`.
    Charset {
        /// The bytes the name takes beside letters and digits.
        extra: &'static [u8],
        /// The regex as vanilla prints it.
        regex: &'static str,
    },
    /// A loader's own test of a name (a regex search, a time key), whatever the walk's
    /// [`Names`].
    Custom {
        /// Whether a name is one the pattern accepts.
        matches: fn(&str) -> bool,
        /// The regex as vanilla prints it.
        regex: &'static str,
    },
}

impl Pattern {
    /// `[a-zA-Z0-9_.-]+`.
    pub const IDENTIFIER: Self = Self::Charset {
        extra: b"_.-",
        regex: "[a-zA-Z0-9_.-]+",
    };

    /// Whether `key` is a name this pattern accepts, names and prefixes compared as `names`
    /// says.
    pub fn matches(self, key: &str, names: Names) -> bool {
        match (self, names) {
            (Self::Name(name), Names::Exact) => key == name,
            (Self::Name(name), Names::IgnoreAsciiCase) => key.eq_ignore_ascii_case(name),
            (Self::Prefixed { prefix, extra, .. }, Names::Exact) => key
                .strip_prefix(prefix)
                .is_some_and(|tail| charset(tail, extra)),
            (Self::Prefixed { prefix, extra, .. }, Names::IgnoreAsciiCase) => {
                key.get(..prefix.len())
                    .is_some_and(|head| head.eq_ignore_ascii_case(prefix))
                    && charset(&key[prefix.len()..], extra)
            }
            (Self::Charset { extra, .. }, _) => charset(key, extra),
            (Self::Custom { matches, .. }, _) => matches(key),
        }
    }

    /// The pattern as vanilla prints it in "Required child %s not found".
    pub fn text(self) -> &'static str {
        match self {
            Self::Name(name) => name,
            Self::Prefixed { regex, .. }
            | Self::Charset { regex, .. }
            | Self::Custom { regex, .. } => regex,
        }
    }
}

/// `[a-zA-Z0-9<extra>]+`, in full.
fn charset(key: &str, extra: &[u8]) -> bool {
    !key.is_empty()
        && key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || extra.contains(&b))
}

/// One name group of a schema object: its pattern and whether a member must match it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Group {
    /// The pattern a member name is matched against.
    pub pattern: Pattern,
    /// A member must match the pattern.
    pub required: bool,
}

impl Group {
    /// An optional group of one fixed name.
    pub const fn name(name: &'static str) -> Self {
        Self {
            pattern: Pattern::Name(name),
            required: false,
        }
    }

    /// A required group of one fixed name.
    pub const fn required(name: &'static str) -> Self {
        Self {
            pattern: Pattern::Name(name),
            required: true,
        }
    }

    /// A group of any pattern.
    pub const fn pattern(pattern: Pattern, required: bool) -> Self {
        Self { pattern, required }
    }
}

/// What reading one member gave.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hit {
    /// The member was read.
    Ok,
    /// The member failed; under [`Failure::MemberFailed`] its finding was pushed, under
    /// [`Failure::ErrorRaised`] the raised error counts instead.
    Failed,
    /// The member's only option refused it by its predicate: vanilla's search moves on and
    /// ends with "child '%s' not valid here.".
    NoOption,
}

impl<T> From<Option<T>> for Hit {
    fn from(value: Option<T>) -> Self {
        if value.is_some() {
            Self::Ok
        } else {
            Self::Failed
        }
    }
}

impl From<bool> for Hit {
    fn from(ok: bool) -> Self {
        if ok { Self::Ok } else { Self::Failed }
    }
}

// ---- the rules ---------------------------------------------------------------------------------

/// How a literal member name and a prefix compare with a key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Names {
    /// As written (render controllers, fog, geometry).
    Exact,
    /// Without regard to ASCII case (both animation kinds): a member spelled in another case
    /// than declared is noted in [`Walk::non_canonical`], and of two members that differ in
    /// case alone the earlier is told as dropped unless both hold one value.
    IgnoreAsciiCase,
}

/// The order an object's members are visited in, which is the order of their findings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Order {
    /// The sorted order of the keys' bytes, the order of jsoncpp's `std::map` (render
    /// controllers, fog below 1.26.20).
    Sorted,
    /// The file's order; of a key written twice, the first position and the last value
    /// (geometry, both animation kinds).
    Document,
}

/// What an undeclared or wrong-typed member is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Validation {
    /// An error: "child '%s' not valid here." or the type error.
    Strict,
    /// An information and the member's removal, the game saying nothing (geometry below
    /// 1.19.30): the undeclared member is told with `removed`, the crate's own text of one
    /// argument (the key), and the wrong type with the type error's text.
    Lenient {
        /// The text of an undeclared member.
        removed: &'static str,
    },
}

/// When an object or an array fails.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Failure {
    /// When a member's read failed or a member was undeclared, whatever was raised; the
    /// required check is then not made (render controllers, fog, both animation kinds).
    MemberFailed,
    /// When an error was raised below it; a member whose read failed without one was removed,
    /// and the required check is made unless an error was raised (geometry).
    ErrorRaised,
}

/// Whether a number a float does not hold is told.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Lossy {
    /// With a `LossyConversion` information, the crate's own (render controllers, fog,
    /// geometry).
    Told,
    /// Silently (both animation kinds).
    Silent,
}

/// The rules of one loader's node trees.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rules {
    /// How names compare.
    pub names: Names,
    /// The order members are visited in.
    pub order: Order,
    /// What an undeclared or wrong-typed member is.
    pub validation: Validation,
    /// When an object or array fails.
    pub failure: Failure,
    /// Whether a number a float does not hold is told.
    pub lossy: Lossy,
}

// ---- the walk ---------------------------------------------------------------------------------

/// The walk's state: its rules, the findings so far and the breadcrumb of the value being read.
#[derive(Debug)]
pub struct Walk {
    /// The findings of the walk so far.
    pub diagnostics: Diagnostics,
    /// The members a reader marked for a later check, by path and span.
    pub marked: Vec<(JsonPath, Span)>,
    /// Member names spelled in another case than declared ([`Names::IgnoreAsciiCase`]).
    pub non_canonical: Vec<(JsonPath, Span)>,
    path: JsonPath,
    rules: Rules,
    /// How many errors [`push`](Self::push) raised.
    errors: usize,
}

impl Walk {
    /// A walk under `rules` whose findings are reported below `at`.
    pub fn new(rules: Rules, at: &JsonPath) -> Self {
        Self {
            diagnostics: Diagnostics::default(),
            marked: Vec::new(),
            non_canonical: Vec::new(),
            path: at.clone(),
            rules,
            errors: 0,
        }
    }

    /// Marks the member at the current path.
    pub fn mark(&mut self, span: Span) {
        self.marked.push((self.path.clone(), span));
    }

    /// Descend into the member `key`.
    pub fn enter(&mut self, key: &str) {
        self.path.push(key);
    }

    /// Descend into the element `index`.
    pub fn enter_index(&mut self, index: usize) {
        self.path.push_index(index);
    }

    /// Go back up one level.
    pub fn leave(&mut self) {
        self.path.pop();
    }

    /// A finding at the current path.
    pub fn push(&mut self, code: Code, severity: Severity, span: impl Into<Option<Span>>) {
        if severity == Severity::Error {
            self.errors += 1;
        }
        let mut finding = Diagnostic::error(code)
            .severity(severity)
            .at(self.path.clone());
        finding.location.span = span.into();
        self.diagnostics.push(finding);
    }

    /// A kept error at the current path: the game logs an error line and goes on loading
    /// the document. It fails no validation.
    pub fn push_kept(&mut self, code: Code, span: impl Into<Option<Span>>) {
        let mut finding = Diagnostic::error(code).at(self.path.clone());
        finding.location.span = span.into();
        self.diagnostics.push_kept_error(finding);
    }

    /// A schema error at the current path.
    pub fn error(&mut self, code: Code, span: Span) {
        self.push(code, Severity::Error, span);
    }

    /// "unknown child schema option type.  Allowed types: …" at the current path: an error,
    /// or under [`Validation::Lenient`] an information (the member is removed).
    pub fn bad_type(&mut self, allowed: &str, node: Node<'_>) {
        let severity = match self.rules.validation {
            Validation::Strict => Severity::Error,
            Validation::Lenient { .. } => Severity::Info,
        };
        self.push(cereal::unknown_child_option(allowed), severity, node.span());
    }

    /// `child '<key>' not valid here.` at the member, or under [`Validation::Lenient`] its
    /// removal as an information.
    fn unknown(&mut self, key: &str, node: Node<'_>) {
        self.enter(key);
        match self.rules.validation {
            Validation::Strict => {
                self.error(core_codes::CHILD_NOT_VALID.with([key]), node.span());
            }
            // The crate's own text of the family, given whole (`vanilla` gives it, as the
            // legacy text did).
            Validation::Lenient { removed } => self.push(
                core_codes::UNRECOGNIZED_FIELD_WORDED.with([removed.replacen("{}", key, 1)]),
                Severity::Info,
                node.span(),
            ),
        }
        self.leave();
    }

    /// `Required child <name> not found` at the current path.
    pub fn missing(&mut self, name: &str, node: Node<'_>) {
        self.error(core_codes::REQUIRED_CHILD.with([name]), node.span());
    }

    /// An error with one of vanilla's lines at the current path: an `InvalidValue` of the
    /// schema tree ([`codes::OUTSIDE_RANGE`](crate::diagnostic::codes::OUTSIDE_RANGE),
    /// [`codes::NOT_ALLOWED_REGEX`](crate::diagnostic::codes::NOT_ALLOWED_REGEX), ...).
    pub fn invalid(&mut self, code: Code, node: Node<'_>) {
        self.error(code, node.span());
    }

    /// `"Value '<value>' not allowed"` (a bool outside its set) at the current path.
    pub fn not_allowed(&mut self, value: &str, node: Node<'_>) {
        self.invalid(core_codes::VALUE_NOT_ALLOWED.with([value]), node);
    }

    /// The members of an object node in the file's order (of a key written twice the first
    /// position and the last value, as jsoncpp keeps the last); `None` when the node is not an
    /// object.
    #[allow(clippy::unused_self, reason = "read like the other node accessors")]
    pub fn members<'a>(&self, node: Node<'a>) -> Option<Vec<(String, Node<'a>)>> {
        if node.kind() != Kind::Object {
            return None;
        }
        let map: IndexMap<String, RawJson<'a>> = parse_at(node.slice).ok()?;
        map.iter()
            .map(|(key, raw)| Some((key.clone(), node.nested(raw)?)))
            .collect()
    }

    /// The elements of an array node; `None` when the node is not an array.
    #[allow(clippy::unused_self, reason = "read like the other node accessors")]
    pub fn elements<'a>(&self, node: Node<'a>) -> Option<Vec<Node<'a>>> {
        if node.kind() != Kind::Array {
            return None;
        }
        let raw: Vec<RawJson<'a>> = parse_at(node.slice).ok()?;
        raw.iter().map(|raw| node.nested(raw)).collect()
    }

    /// Walks an object node against its name groups, the members in the walk's [`Order`]: each
    /// member is handed to `on` with the index of the first group it matches; an unmatched
    /// member is undeclared (an unknown-member finding), and so is one `on` gives
    /// [`Hit::NoOption`]. Unless the object failed by the walk's [`Failure`], each required
    /// group without a member is "Required child %s not found". `false` when the object failed.
    pub fn object<'a>(
        &mut self,
        node: Node<'a>,
        groups: &[Group],
        mut on: impl FnMut(&mut Self, usize, &str, Node<'a>) -> Hit,
    ) -> bool {
        let Some(mut members) = self.members(node) else {
            self.bad_type("object", node);
            return false;
        };
        if self.rules.order == Order::Sorted {
            members.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
        }
        let case = self.rules.names;
        let before = self.errors;
        let mut failed = false;
        let mut found = vec![false; groups.len()];
        // The member last read for each group of one literal name.
        let mut named: Vec<Option<(String, Span, Node<'a>)>> = vec![None; groups.len()];
        for (key, value) in members {
            let Some(index) = groups.iter().position(|g| g.pattern.matches(&key, case)) else {
                self.unknown(&key, value);
                failed = true;
                continue;
            };
            if case == Names::IgnoreAsciiCase {
                self.case_duplicate(&mut named[index], groups[index].pattern, &key, value);
            }
            self.enter(&key);
            let hit = on(self, index, &key, value);
            self.leave();
            match hit {
                Hit::Ok => found[index] = true,
                Hit::Failed => failed = true,
                Hit::NoOption => {
                    self.unknown(&key, value);
                    failed = true;
                }
            }
        }
        let ok = |walk: &Self, failed: bool| match walk.rules.failure {
            Failure::MemberFailed => !failed,
            Failure::ErrorRaised => walk.errors == before,
        };
        if ok(self, failed) {
            for (group, found) in groups.iter().zip(found) {
                if group.required && !found {
                    self.missing(group.pattern.text(), node);
                    failed = true;
                }
            }
        }
        ok(self, failed)
    }

    /// Under [`Names::IgnoreAsciiCase`]: notes a member of a literal name or prefix spelled in
    /// another case than declared, and tells the earlier of two members of one literal name
    /// whose values differ as dropped (this one's value replaces it).
    fn case_duplicate<'a>(
        &mut self,
        last: &mut Option<(String, Span, Node<'a>)>,
        pattern: Pattern,
        key: &str,
        value: Node<'a>,
    ) {
        if matches!(pattern, Pattern::Name(_))
            && let Some((earlier, span, earlier_value)) =
                last.replace((key.to_owned(), value.span(), value))
            && !same_value(earlier_value, value)
        {
            // Two members that differ in case alone are one member: this one's value
            // replaces the earlier's, which is told where the file has it unless the
            // value is the same (a `DroppedField` information of the crate's own).
            self.enter(&earlier);
            self.push(Code::from(&core_codes::DROPPED_FIELD), Severity::Info, span);
            self.leave();
        }
        let spelled_otherwise = match pattern {
            Pattern::Name(name) => key != name,
            Pattern::Prefixed { prefix, .. } => !key.starts_with(prefix),
            Pattern::Charset { .. } | Pattern::Custom { .. } => false,
        };
        if spelled_otherwise {
            let at = self.path.child(key);
            self.non_canonical.push((at, value.span()));
        }
    }

    /// The member count of an object node, for the object nodes with a child-count bound:
    /// exactly `count` members, else "Node has too few/many children". `true` for a node that is
    /// no object.
    pub fn child_count(&mut self, node: Node<'_>, count: usize) -> bool {
        let Some(members) = self.members(node) else {
            return true;
        };
        let n = members.len();
        let code = match n.cmp(&count) {
            std::cmp::Ordering::Less => cereal::too_few_children(n, count),
            std::cmp::Ordering::Greater => cereal::too_many_children(n, count),
            std::cmp::Ordering::Equal => return true,
        };
        self.error(code, node.span());
        false
    }

    /// Walks an array node: each element handed to `on` at its index. `false` when the node is
    /// not an array, or the array failed by the walk's [`Failure`].
    pub fn array<'a>(
        &mut self,
        node: Node<'a>,
        mut on: impl FnMut(&mut Self, usize, Node<'a>) -> Hit,
    ) -> bool {
        let Some(elements) = self.elements(node) else {
            self.bad_type("array", node);
            return false;
        };
        let before = self.errors;
        let mut ok = true;
        for (index, element) in elements.into_iter().enumerate() {
            self.enter_index(index);
            ok &= on(self, index, element) == Hit::Ok;
            self.leave();
        }
        match self.rules.failure {
            Failure::MemberFailed => ok,
            Failure::ErrorRaised => self.errors == before,
        }
    }

    /// A list of values read by `read`, when the array did not fail.
    pub fn list<'a, T>(
        &mut self,
        node: Node<'a>,
        mut read: impl FnMut(&mut Self, Node<'a>) -> Option<T>,
    ) -> Option<Vec<T>> {
        let mut out = Vec::new();
        let ok = self.array(node, |walk, _, element| {
            read(walk, element).map(|value| out.push(value)).into()
        });
        ok.then_some(out)
    }

    /// The kinds of the elements of an array node (none when the node is no array), for
    /// predicates over the whole array.
    pub fn kinds(&mut self, node: Node<'_>) -> Vec<Kind> {
        self.elements(node)
            .map(|e| e.iter().map(Node::kind).collect())
            .unwrap_or_default()
    }

    // ---- typed leaves ----

    /// A string node, decoded.
    pub fn string(&mut self, node: Node<'_>) -> Option<String> {
        if node.kind() != Kind::String {
            self.bad_type("string", node);
            return None;
        }
        parse_at::<String>(node.slice).ok()
    }

    /// A number node.
    pub fn number(&mut self, node: Node<'_>) -> Option<f64> {
        if node.kind() != Kind::Number {
            self.bad_type("float", node);
            return None;
        }
        parse_at::<f64>(node.slice).ok()
    }

    /// A number node as `f32`: the nearest float, told as lossy by the walk's [`Lossy`] when it
    /// is written back as another number, or the largest finite one of its sign for a number
    /// beyond a float (with an `InvalidValue` information at the number).
    #[allow(
        clippy::cast_possible_truncation,
        reason = "vanilla reads these members as float"
    )]
    pub fn float(&mut self, node: Node<'_>) -> Option<f32> {
        let number = self.number(node)?;
        let nearest = number as f32;
        if self.rules.lossy == Lossy::Told && super::lossy::narrows(number) {
            // The game holds the member as a float, and the document is written with the
            // float: digits of the file a float does not hold are noted by the crate's own
            // information.
            self.push(
                Code::from(&core_codes::LOSSY_CONVERSION),
                Severity::Info,
                node.span(),
            );
        }
        if nearest.is_infinite() {
            // A number beyond a float is the largest finite float of its sign, with the
            // information of such a number on its token; the file loads, and the written text
            // reads back to the same value without a finding.
            self.push(
                Code::from(&core_codes::INVALID_VALUE),
                Severity::Info,
                node.span(),
            );
            return Some(f32::MAX.copysign(nearest));
        }
        Some(nearest)
    }

    /// A bool node.
    pub fn bool(&mut self, node: Node<'_>) -> Option<bool> {
        match (node.kind(), node.slice.text) {
            (Kind::Bool, "true") => Some(true),
            (Kind::Bool, "false") => Some(false),
            _ => {
                self.bad_type("bool", node);
                None
            }
        }
    }

    /// An `ExpressionNode` member: a number, bool or string (`ReaderKind::SchemaValidated`);
    /// anything else is the type error. The context version is set by pass 1.
    pub fn molang(&mut self, node: Node<'_>) -> Option<Molang> {
        let scalar = match node.kind() {
            Kind::Number => JsonScalar::Number(parse_at::<f64>(node.slice).ok()?),
            Kind::Bool => JsonScalar::Bool(node.slice.text == "true"),
            Kind::String => JsonScalar::String(Arc::from(parse_at::<String>(node.slice).ok()?)),
            Kind::Null => JsonScalar::Other(OtherJson::Null),
            Kind::Array => JsonScalar::Other(OtherJson::Array),
            Kind::Object => JsonScalar::Other(OtherJson::Object),
        };
        // A number beyond `f32` is the largest finite float, with core's one warning at the
        // number.
        let read = crate::molang::read_finite(scalar, ReaderKind::SchemaValidated, RawVersion(-1));
        if let Ok((ReadField::Value(value), clamped)) = read {
            if clamped {
                self.diagnostics.push(crate::molang::beyond_f32_warning(
                    self.path.clone(),
                    Some(node.span()),
                ));
            }
            Some(value)
        } else {
            self.bad_type("ExpressionNode", node);
            None
        }
    }

    /// A schema array of exactly three `ExpressionNode`s (`addChildArray(name, 3, …)`): told of
    /// by its length first, then by its elements.
    pub fn molang3(&mut self, node: Node<'_>) -> Option<[Molang; 3]> {
        if node.kind() != Kind::Array {
            self.bad_type("array", node);
            return None;
        }
        // The array is read once, for its length and for its elements.
        let elements = self.elements(node).unwrap_or_default();
        let n = elements.len();
        if n != 3 {
            let def = if n < 3 {
                &core_codes::ARRAY_TOO_SMALL
            } else {
                &core_codes::ARRAY_TOO_LARGE
            };
            self.invalid(def.with([n.to_string(), "3".to_owned()]), node);
            return None;
        }
        let mut values = Vec::with_capacity(3);
        for (index, element) in elements.into_iter().enumerate() {
            self.enter_index(index);
            values.extend(self.molang(element));
            self.leave();
        }
        values.try_into().ok()
    }

    /// An array of strings.
    pub fn strings(&mut self, node: Node<'_>) -> Option<Vec<String>> {
        let mut out = Vec::new();
        let ok = self.array(node, |walk, _, element| {
            walk.string(element).map(|s| out.push(s)).into()
        });
        ok.then_some(out)
    }

    /// An object of `"<key>": float` members under one pattern group.
    pub fn float_map(
        &mut self,
        node: Node<'_>,
        pattern: Pattern,
        required: bool,
    ) -> Option<IndexMap<String, f32>> {
        let mut map = IndexMap::new();
        let ok = self.object(
            node,
            &[Group::pattern(pattern, required)],
            |walk, _, key, v| walk.float(v).map(|x| map.insert(key.to_owned(), x)).into(),
        );
        ok.then_some(map)
    }
}

#[cfg(test)]
mod tests;
