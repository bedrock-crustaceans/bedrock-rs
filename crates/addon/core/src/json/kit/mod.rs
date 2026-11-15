//! The reader kit: the vocabulary a family whose game reader is hand-written walks its JSON
//! with, in place of a private one per family.
//!
//! A hand-walked family reads a [`Value`] member by member. Every such family today has its own
//! `wrong_type`, `required`, `unknown` and `object`, each with its own wording, severity and
//! idea of where the finding goes. The kit is those operations once, graded by the slice's
//! [`Reading`]:
//!
//! - a [`Node`] is a value with its path, and with its text when the caller has the file, so
//!   every finding at it carries the span of the text it is about;
//! - [`Read`] holds the reading and the findings, and does the reading: [`object`](Read::object),
//!   [`array`](Read::array), [`opt`](Read::opt), [`required`](Read::required),
//!   [`unknown`](Read::unknown), [`wrong_type`](Read::wrong_type), [`refuse`](Read::refuse) and
//!   the scalars ([`bool`](Read::bool), [`string`](Read::string), [`number`](Read::number),
//!   [`int`](Read::int), and [`scalar`](Read::scalar), a member of a declared type read under
//!   every profile as the typed door reads it); it tells in an [`Order`] (as read, every
//!   failure; or the typed door's, which stops at the first and tells the rest after the read,
//!   by [`Pass`]), and a [`fork`](Read::fork) of it is an attempt to [commit](Read::commit) or
//!   drop;
//! - [`Members`] and [`Elements`] are an object's members and an array's elements, each a
//!   [`Node`] one step down;
//! - [`Written`] is the slot a family that keeps its root as the author wrote it holds it in,
//!   left out of equality.
//!
//! What the reading decides:
//!
//! | axis | what the kit does with it |
//! |---|---|
//! | [`Reading::null`] | [`NullHandling::Absent`]: [`opt`](Read::opt) reads a `null` member as absent, [`required`](Read::required) as missing; otherwise the member is there and its read takes the `null` |
//! | [`Reading::scalars`] | every profile in [`scalar`](Read::scalar), through `json::judge` (the `ConvertAndTell` conversions as the plain typed read makes them). In the hand reads [`Scalars::JsonCpp`] only, in two reads: [`number`](Read::number) also takes a bool, as 1 or 0, and [`int`](Read::int) any number, cut as `asInt` cuts it under the caller's [`Coercing`] (a string or `null` is still a wrong type to both, and a bool to `int`). [`Scalars::Strict`], [`Scalars::NonStrict`] and [`Scalars::ConvertAndTell`] read alike here: a token of another kind is a wrong type, and `int` takes what `isInt` takes. [`bool`](Read::bool) and [`string`](Read::string) coerce under none |
//! | [`Reading::beyond_width`] | in [`scalar`](Read::scalar): a number beyond its float reads as the infinity it is, told so the load clamps it and reads again ([`BeyondWidth::ClampAndTell`](crate::json::BeyondWidth::ClampAndTell)), or is a wrong type ([`BeyondWidth::Error`](crate::json::BeyondWidth::Error)) |
//! | [`Reading::unknown`] | the severity of [`unknown`](Read::unknown): [`UnknownMask::Error`](crate::versioning::UnknownMask::Error) an error, [`UnknownMask::Warn`](crate::versioning::UnknownMask::Warn) a warning |
//! | [`Reading::on_member_failure`] | a failed read (a wrong type, a missing member), graded by [`schema::grade`](crate::schema::grade): [`MemberFailure::DefaultAndTell`](crate::versioning::MemberFailure::DefaultAndTell) an information, since the member then takes its default (the game says nothing); any other an error that fails what is read ([`Read::failed`]), the caller dropping what the rule drops |
//!
//! A finding the kit does not word as a family's game does is the family's to tell, with
//! [`Read::push`] at the node's [`path`](Node::path) and [`span`](Node::span); a text of the
//! family's own for the kit's wording is [`Texts`].
//!
//! A payload read with its text, so every finding carries its span:
//!
//! ```
//! use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
//! use bedrock_addon_core::json::kit::{Disposition, Node, Read};
//! use bedrock_addon_core::json::{JsonSlice, Value, from_str_jsonc};
//! use bedrock_addon_core::versioning::Reading;
//!
//! let text = r#"{"name": "ruby", "size": "big", "glow": 1}"#;
//! let json: Value = from_str_jsonc(text).unwrap();
//! let payload = Node::with_text(&json, JsonPath::of_keys(["minecraft:gem"]), JsonSlice::new(text));
//! let mut found = Diagnostics::default();
//! let mut read = Read::new(Reading::DOCUMENT, &mut found);
//!
//! let members = read.object(&payload).unwrap();
//! read.unknown(&members, &["name", "size"], Disposition::Keep);
//! let name = read.required(&members, "name").and_then(|node| read.string(&node));
//! let size = read.opt(&members, "size").and_then(|node| read.number(&node));
//! assert_eq!((name, size), (Some("ruby"), None));
//! assert!(read.failed());
//!
//! let told: Vec<String> = found
//!     .iter()
//!     .map(|f| format!("{} {} {:?}", f.location.json_path, f.code, f.location.span))
//!     .collect();
//! assert_eq!(told, [
//!     "minecraft:gem/glow unrecognized field Some(Span { offset: 40, len: 1 })",
//!     "minecraft:gem/size wrong JSON type Some(Span { offset: 25, len: 5 })",
//! ]);
//! ```

use facet_json::RawJson;
use facet_value::{VArray, VObject};
use indexmap::IndexMap;

use super::{Coerce, Coercing, JsonSlice, Value};
use crate::diagnostic::codes as core_codes;
use crate::diagnostic::{Class, Code, Diagnostic, Diagnostics, JsonPath, Severity, Span};
use crate::versioning::{NullHandling, Reading, Scalars};

// hidden: API not yet reviewed
#[doc(hidden)]
pub mod fixed_path;
// hidden: API not yet reviewed
mod scalar;
#[doc(hidden)]
pub mod sound;
#[cfg(test)]
mod tests;
mod written;

pub use scalar::Scalar;
pub use written::Written;

/// A value being read: where it is in the document, and its text when the caller has it.
///
/// The text is the span source: with it, a finding at the node or below it carries the byte
/// range of the file it is about. It must be the text the value was read from (a
/// [`JsonSlice`] at its file offset).
#[derive(Clone, Debug)]
pub struct Node<'v> {
    value: &'v Value,
    path: JsonPath,
    text: Option<JsonSlice<'v>>,
}

impl<'v> Node<'v> {
    /// The value at `path`, without a text: findings at it carry no span.
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Disposition, Node, Read};
    /// # use bedrock_addon_core::json::{JsonSlice, Value, from_str_jsonc};
    /// # use bedrock_addon_core::versioning::Reading;
    /// # fn told(d: &Diagnostics) -> Vec<String> {
    /// #     d.iter().map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code)).collect()
    /// # }
    /// let json: Value = from_str_jsonc("[1]").unwrap();
    /// let root = Node::new(&json, JsonPath::of_keys(["list"]));
    /// assert_eq!(root.path().to_string(), "list");
    /// assert!(root.value().is_array());
    /// assert_eq!(root.span(), None);
    /// ```
    pub fn new(value: &'v Value, path: JsonPath) -> Self {
        Self {
            value,
            path,
            text: None,
        }
    }

    /// The value at `path` read from `text`: findings at it and below it carry spans.
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Disposition, Node, Read};
    /// # use bedrock_addon_core::json::{JsonSlice, Value, from_str_jsonc};
    /// # use bedrock_addon_core::versioning::Reading;
    /// # fn told(d: &Diagnostics) -> Vec<String> {
    /// #     d.iter().map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code)).collect()
    /// # }
    /// let text = r#"{"size": "big"}"#;
    /// let json: Value = from_str_jsonc(text).unwrap();
    /// let root = Node::with_text(&json, JsonPath::new(), JsonSlice::new(text));
    /// let mut found = Diagnostics::default();
    /// let mut read = Read::new(Reading::DOCUMENT, &mut found);
    /// let members = read.object(&root).unwrap();
    /// let size = read.required(&members, "size").unwrap();
    /// // The span is the member's value as the file has it.
    /// let span = size.span().unwrap();
    /// assert_eq!(&text[span.offset as usize..][..span.len as usize], r#""big""#);
    /// ```
    pub fn with_text(value: &'v Value, path: JsonPath, text: JsonSlice<'v>) -> Self {
        Self {
            value,
            path,
            text: Some(text),
        }
    }

    /// The value.
    pub fn value(&self) -> &'v Value {
        self.value
    }

    /// Where the value is in the document.
    pub fn path(&self) -> &JsonPath {
        &self.path
    }

    /// The value's text, when the node has one.
    pub fn text(&self) -> Option<JsonSlice<'v>> {
        self.text
    }

    /// The byte range of the value in the file, when the node has its text.
    pub fn span(&self) -> Option<Span> {
        self.text.map(span_of)
    }
}

/// The members of an object being read, each a [`Node`] one key down. From
/// [`Read::object`].
#[derive(Clone, Debug)]
pub struct Members<'v> {
    node: Node<'v>,
    object: &'v VObject,
    texts: Option<IndexMap<String, RawJson<'v>>>,
}

impl<'v> Members<'v> {
    /// The object itself.
    pub fn node(&self) -> &Node<'v> {
        &self.node
    }

    /// How many members the object has.
    pub fn len(&self) -> usize {
        self.object.len()
    }

    /// Whether the object has no member.
    pub fn is_empty(&self) -> bool {
        self.object.is_empty()
    }

    /// Whether the object has the member `key`, whatever its value.
    pub fn contains(&self, key: &str) -> bool {
        self.object.get(key).is_some()
    }

    /// The member `key` as written, `null` included, without a word: the raw access below
    /// [`Read::opt`].
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Disposition, Node, Read};
    /// # use bedrock_addon_core::json::{JsonSlice, Value, from_str_jsonc};
    /// # use bedrock_addon_core::versioning::Reading;
    /// # fn told(d: &Diagnostics) -> Vec<String> {
    /// #     d.iter().map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code)).collect()
    /// # }
    /// let json: Value = from_str_jsonc(r#"{"a": null}"#).unwrap();
    /// let mut found = Diagnostics::default();
    /// let mut read = Read::new(Reading::NULL_ABSENT, &mut found);
    /// let members = read.object(&Node::new(&json, JsonPath::new())).unwrap();
    /// // As written, `null` included; `Read::opt` applies the null rule.
    /// assert!(members.get("a").unwrap().value().is_null());
    /// assert!(read.opt(&members, "a").is_none());
    /// ```
    pub fn get(&self, key: &str) -> Option<Node<'v>> {
        let value = self.object.get(key)?;
        Some(self.child(key, value))
    }

    /// Every member, in file order.
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Disposition, Node, Read};
    /// # use bedrock_addon_core::json::{JsonSlice, Value, from_str_jsonc};
    /// # use bedrock_addon_core::versioning::Reading;
    /// # fn told(d: &Diagnostics) -> Vec<String> {
    /// #     d.iter().map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code)).collect()
    /// # }
    /// let json: Value = from_str_jsonc(r#"{"b": 1, "a": 2}"#).unwrap();
    /// let mut found = Diagnostics::default();
    /// let mut read = Read::new(Reading::DOCUMENT, &mut found);
    /// let members = read.object(&Node::new(&json, JsonPath::of_keys(["root"]))).unwrap();
    /// let paths: Vec<String> = members.iter().map(|(_, node)| node.path().to_string()).collect();
    /// assert_eq!(paths, ["root/b", "root/a"]);
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = (&'v str, Node<'v>)> + '_ {
        self.object
            .iter()
            .map(|(key, value)| (key.as_str(), self.child(key.as_str(), value)))
    }

    fn child(&self, key: &str, value: &'v Value) -> Node<'v> {
        let text = self
            .node
            .text
            .zip(self.texts.as_ref())
            .and_then(|(text, texts)| texts.get(key).and_then(|raw| text.nested(raw)));
        Node {
            value,
            path: self.node.path.child(key),
            text,
        }
    }
}

/// The elements of an array being read, each a [`Node`] one index down. From
/// [`Read::array`].
#[derive(Clone, Debug)]
pub struct Elements<'v> {
    node: Node<'v>,
    array: &'v VArray,
    texts: Option<Vec<RawJson<'v>>>,
}

impl<'v> Elements<'v> {
    /// The array itself.
    pub fn node(&self) -> &Node<'v> {
        &self.node
    }

    /// How many elements the array has.
    pub fn len(&self) -> usize {
        self.array.len()
    }

    /// Whether the array has no element.
    pub fn is_empty(&self) -> bool {
        self.array.is_empty()
    }

    /// The element at `index`.
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Disposition, Node, Read};
    /// # use bedrock_addon_core::json::{JsonSlice, Value, from_str_jsonc};
    /// # use bedrock_addon_core::versioning::Reading;
    /// # fn told(d: &Diagnostics) -> Vec<String> {
    /// #     d.iter().map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code)).collect()
    /// # }
    /// let json: Value = from_str_jsonc(r#"["x", 2]"#).unwrap();
    /// let mut found = Diagnostics::default();
    /// let mut read = Read::new(Reading::DOCUMENT, &mut found);
    /// let elements = read.array(&Node::new(&json, JsonPath::of_keys(["names"]))).unwrap();
    /// let second = elements.element(1).unwrap();
    /// assert_eq!(read.string(&second), None);
    /// assert!(elements.element(2).is_none());
    /// assert_eq!(told(&found), ["Error names/1 wrong JSON type"]);
    /// ```
    pub fn element(&self, index: usize) -> Option<Node<'v>> {
        let value = self.array.get(index)?;
        let text = self
            .node
            .text
            .zip(self.texts.as_ref())
            .and_then(|(text, texts)| texts.get(index).and_then(|raw| text.nested(raw)));
        Some(Node {
            value,
            path: self.node.path.index(index),
            text,
        })
    }

    /// Every element, in order.
    pub fn iter(&self) -> impl Iterator<Item = Node<'v>> + '_ {
        (0..self.len()).filter_map(|index| self.element(index))
    }
}

/// What an unknown member is, and the code it is told under. The four are the four the game's
/// readers have, and [`Read::unknown`] tells each at the reading's unknown severity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Disposition {
    /// The member is kept beside the typed value and written back
    /// ([`codes::UNRECOGNIZED_FIELD`](core_codes::UNRECOGNIZED_FIELD)); [`Read::unknown`] hands
    /// it back to be kept.
    Keep,
    /// The member is told and not kept: nothing reads it and a written document lacks it
    /// ([`codes::MEMBER_NOT_KEPT`](core_codes::MEMBER_NOT_KEPT)).
    Report,
    /// The member is dropped by a rule ([`codes::DROPPED_FIELD`](core_codes::DROPPED_FIELD)).
    Reject,
    /// The member is ignored, as the game ignores it
    /// ([`codes::IGNORED_MEMBER`](core_codes::IGNORED_MEMBER)).
    Ignore,
}

impl Disposition {
    /// The code of an unknown member told under this disposition: the disposition's own line,
    /// or the family's text given whole.
    fn code(self, text: Option<&'static str>) -> Code {
        let (plain, worded) = match self {
            Self::Keep => (
                &core_codes::UNRECOGNIZED_FIELD,
                &core_codes::UNRECOGNIZED_FIELD_WORDED,
            ),
            Self::Report => (
                &core_codes::MEMBER_NOT_KEPT,
                &core_codes::MEMBER_NOT_KEPT_WORDED,
            ),
            Self::Reject => (
                &core_codes::DROPPED_FIELD,
                &core_codes::DROPPED_FIELD_WORDED,
            ),
            Self::Ignore => (
                &core_codes::IGNORED_MEMBER,
                &core_codes::IGNORED_MEMBER_WORDED,
            ),
        };
        text.map_or_else(|| plain.into(), |text| worded.with([text]))
    }
}

/// What JSON a read expected where it found another, for a family that words a wrong type by
/// both ([`Texts::wrong_kind`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Expected {
    /// An object.
    Object,
    /// An array.
    Array,
    /// A string.
    String,
    /// A number.
    Number,
    /// A bool.
    Bool,
}

/// A family's wording of a wrong type that names what was expected and what was found
/// ([`Texts::wrong_kind`]): the code to tell, from the two.
#[derive(Clone, Copy, Debug)]
pub struct WrongKind(pub fn(Expected, &Value) -> Code);

/// By the function: two wordings are the same when they are the same function.
impl PartialEq for WrongKind {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::fn_addr_eq(self.0, other.0)
    }
}

impl Eq for WrongKind {}

impl std::hash::Hash for WrongKind {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.0 as usize).hash(state);
    }
}

/// The texts of the kit's findings. `None` is the code's own wording, which is what most of the
/// hand-walked families print: `wrong JSON type`, `missing required field`, `unrecognized
/// field`. A family whose game prints another line for one of them names it here once, for
/// every read of the [`Read`].
///
/// The families whose wording differs today (an operation not listed reads `None`):
/// `wrong_type` font and `json_ui` (a text per member), attachable and `client_entity`
/// ("unknown child schema option type.  Allowed types: …", formatted per member, so through
/// [`Read::push`]), `crafting_catalog` (a `MalformedInput` naming both types, through
/// [`Read::push`]); `missing` particle's legacy era, loot, font and `json_ui` (a text per
/// member); `unknown` script ([`cereal::MEMBER_NOT_IN_SCHEMA_TEXT`]). `block_culling` words its
/// `missing` and `unknown` findings with the member's path as a breadcrumb before the game's
/// text ([`cereal::missing_required_field`], [`cereal::member_not_in_schema`]), a text per
/// path, so it tells them through [`Read::push`], not here.
///
/// [`cereal::missing_required_field`]: crate::diagnostic::cereal::missing_required_field
/// [`cereal::member_not_in_schema`]: crate::diagnostic::cereal::member_not_in_schema
/// [`cereal::MEMBER_NOT_IN_SCHEMA_TEXT`]: crate::diagnostic::cereal::MEMBER_NOT_IN_SCHEMA_TEXT
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Texts {
    /// The text of a value of the wrong JSON type
    /// ([`codes::INVALID_TYPE`](core_codes::INVALID_TYPE)).
    pub wrong_type: Option<&'static str>,
    /// The text of a missing required member
    /// ([`codes::MISSING_REQUIRED_FIELD`](core_codes::MISSING_REQUIRED_FIELD)).
    pub missing: Option<&'static str>,
    /// The text of an unknown member, whatever its [`Disposition`].
    pub unknown: Option<&'static str>,
    /// The code of a wrong type where the read knows what it expected ([`Read::object`],
    /// [`Read::array`], the scalar reads), from what it expected and what it found, in place
    /// of [`wrong_type`](Self::wrong_type): `crafting_catalog` words it as vanilla's
    /// `MalformedInput` naming both.
    pub wrong_kind: Option<WrongKind>,
}

impl Texts {
    /// Every finding in its code's own wording.
    pub const CODES: Self = Self {
        wrong_type: None,
        missing: None,
        unknown: None,
        wrong_kind: None,
    };
}

/// The order a [`Read`] tells its findings in, and how many failures it tells.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Order {
    /// Every finding where the read meets it, and every failure: the reader goes on past a
    /// failed member and tells the next one. The hand-walked families' order.
    #[default]
    AsRead,
    /// As [`AsRead`](Self::AsRead), but a struct tells the unknown members of its object, in
    /// file order, before it reads any of its members: `crafting_catalog`'s reader.
    UnknownFirst,
    /// The typed door's order (`versioning::parse_payload`, whose facet read stops at its first
    /// error): the read stops at its first failure, which is then all it tells, and a failure is
    /// an error whatever the reading's [`on_member_failure`](Reading::on_member_failure) (but
    /// for a scalar member under `DefaultAndTell`, below). What it finds besides is told after
    /// the read ([`Read::finish`]), by [`Pass`], and only when nothing failed: the members
    /// written `null` that the reading refuses (their errors then are all that is told), the
    /// members read as absent for their `null`, the unknown members, and the scalars read as
    /// another value. A scalar member its profile refuses under `DefaultAndTell` is read as
    /// absent and told in the last pass, as the door's scalar pre-pass does.
    Door,
}

/// The pass of the door ([`Order::Door`]) a finding told after the read belongs to, in the
/// order the passes run.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Pass {
    /// A member written `null` that its type refuses ([`NullHandling::Rejected`]): an error.
    Null,
    /// A member written `null` read as absent ([`NullHandling::Absent`]).
    Absent,
    /// An unknown member.
    Unknown,
    /// A scalar read as another value than it is written: converted, beyond its float, or
    /// defaulted.
    Misread,
}

/// Where a read's findings go: the caller's list, or the read's own (a [fork](Read::fork)).
#[derive(Debug)]
enum Sink<'d> {
    Borrowed(&'d mut Diagnostics),
    Owned(Diagnostics),
}

impl Sink<'_> {
    fn get(&mut self) -> &mut Diagnostics {
        match self {
            Self::Borrowed(findings) => findings,
            Self::Owned(findings) => findings,
        }
    }
}

/// One read of a hand-walked document: the slice's [`Reading`], the [`Texts`] and where the
/// findings go.
#[derive(Debug)]
pub struct Read<'d> {
    reading: Reading,
    texts: Texts,
    unknown_severity: Severity,
    findings: Sink<'d>,
    failed: bool,
    order: Order,
    /// The read met its first failure under [`Order::Door`].
    stopped: bool,
    /// The findings of the passes after the read, under [`Order::Door`].
    deferred: Vec<(Pass, Diagnostic)>,
    /// The last scalar read was a member read as absent ([`Order::Door`] under
    /// `DefaultAndTell`), for the owner to take as absent.
    defaulted: bool,
    /// The `schema::unknown` of the field being read, for the object it holds.
    field_unknown: Option<crate::schema::UnknownRule>,
    /// How many map values or boxes the read is inside: an unknown member there is told and
    /// never kept, as the door's walk cannot reach it.
    unkeepable: u32,
}

impl<'d> Read<'d> {
    /// A read under `reading`, telling into `findings`.
    pub fn new(reading: Reading, findings: &'d mut Diagnostics) -> Self {
        let unknown_severity = reading.grade(Class::Unknown);
        Self {
            reading,
            texts: Texts::CODES,
            unknown_severity,
            findings: Sink::Borrowed(findings),
            failed: false,
            order: Order::AsRead,
            stopped: false,
            deferred: Vec::new(),
            defaulted: false,
            field_unknown: None,
            unkeepable: 0,
        }
    }

    /// The read telling in `order`.
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Disposition, Node, Read};
    /// # use bedrock_addon_core::json::{JsonSlice, Value, from_str_jsonc};
    /// # use bedrock_addon_core::versioning::Reading;
    /// # fn told(d: &Diagnostics) -> Vec<String> {
    /// #     d.iter().map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code)).collect()
    /// # }
    /// use bedrock_addon_core::json::kit::Order;
    /// let json: Value = from_str_jsonc(r#"{"a": 1, "b": 2}"#).unwrap();
    /// let mut found = Diagnostics::default();
    /// let mut read = Read::new(Reading::DOCUMENT, &mut found).order(Order::Door);
    /// let members = read.object(&Node::new(&json, JsonPath::new())).unwrap();
    /// for key in ["a", "b"] {
    ///     let node = read.required(&members, key).unwrap();
    ///     read.string(&node);
    /// }
    /// // The door's read stops at its first failure.
    /// assert!(read.stopped());
    /// assert_eq!(told(&found), ["Error a wrong JSON type"]);
    /// ```
    #[must_use]
    pub fn order(mut self, order: Order) -> Self {
        self.order = order;
        self
    }

    /// The order the read tells in.
    pub fn telling(&self) -> Order {
        self.order
    }

    /// Whether the read stopped at its first failure ([`Order::Door`]): nothing more is read
    /// or told, and every read returns `None`.
    pub fn stopped(&self) -> bool {
        self.stopped
    }

    /// A read with this read's reading, texts, severities and order that tells into a list of
    /// its own, for an attempt the caller may [`commit`](Self::commit) or drop: an arm of a
    /// one-of that is tried and may be given up for another.
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Disposition, Node, Read};
    /// # use bedrock_addon_core::json::{JsonSlice, Value, from_str_jsonc};
    /// # use bedrock_addon_core::versioning::Reading;
    /// # fn told(d: &Diagnostics) -> Vec<String> {
    /// #     d.iter().map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code)).collect()
    /// # }
    /// let json: Value = from_str_jsonc("5").unwrap();
    /// let node = Node::new(&json, JsonPath::of_keys(["v"]));
    /// let mut found = Diagnostics::default();
    /// let mut read = Read::new(Reading::DOCUMENT, &mut found);
    /// // Try the value as a string; it is none, so the attempt is dropped.
    /// let mut attempt = read.fork();
    /// assert!(attempt.string(&node).is_none());
    /// drop(attempt);
    /// // As a number it reads, and that attempt is kept.
    /// let mut attempt = read.fork();
    /// assert_eq!(attempt.number(&node), Some(5.0));
    /// read.commit(attempt);
    /// assert!(!read.failed());
    /// assert!(told(&found).is_empty());
    /// ```
    #[must_use]
    pub fn fork(&self) -> Read<'static> {
        Read {
            reading: self.reading,
            texts: self.texts,
            unknown_severity: self.unknown_severity,
            findings: Sink::Owned(Diagnostics::default()),
            failed: false,
            order: self.order,
            stopped: false,
            deferred: Vec::new(),
            defaulted: false,
            field_unknown: self.field_unknown,
            unkeepable: self.unkeepable,
        }
    }

    /// Takes `fork`'s findings, its kept unknown members and its failure as this read's own.
    pub fn commit(&mut self, fork: Read<'_>) {
        let Read {
            findings,
            failed,
            stopped,
            deferred,
            ..
        } = fork;
        let findings = match findings {
            Sink::Owned(findings) => findings,
            // A borrowed sink is not a fork's: its findings are already where they go.
            Sink::Borrowed(_) => Diagnostics::default(),
        };
        self.findings.get().extend_with_unknown(findings);
        self.deferred.extend(deferred);
        self.failed |= failed;
        self.stopped |= stopped;
    }

    /// Tells `finding` in `pass`: now under [`Order::AsRead`], after the read under
    /// [`Order::Door`] ([`finish`](Self::finish)). A finding that rejects the document fails the
    /// read when it is told.
    pub fn defer(&mut self, pass: Pass, finding: Diagnostic) {
        if self.stopped {
            return;
        }
        match self.order {
            Order::Door => self.deferred.push((pass, finding)),
            Order::AsRead | Order::UnknownFirst => self.push(finding),
        }
    }

    /// Ends the read: under [`Order::Door`], tells what the passes after the read found, unless
    /// the read failed (its failure is then all it tells), and only the [`Pass::Null`] errors
    /// when there are any. Nothing under [`Order::AsRead`], which has told everything already.
    pub fn finish(&mut self) {
        let deferred = std::mem::take(&mut self.deferred);
        if self.stopped || self.order != Order::Door {
            return;
        }
        let nulls = deferred.iter().any(|(pass, _)| *pass == Pass::Null);
        let mut deferred = deferred;
        // Stable: within a pass, in the order the read met them.
        deferred.sort_by_key(|(pass, _)| *pass);
        for (pass, finding) in deferred {
            if nulls && pass != Pass::Null {
                break;
            }
            self.push(finding);
        }
    }

    /// The findings of the read so far, the document's table of kept unknown members with them.
    pub fn findings_mut(&mut self) -> &mut Diagnostics {
        self.findings.get()
    }

    /// What a failed member does under the read: the reading's
    /// [`on_member_failure`](Reading::on_member_failure) graded ([`schema::grade`](crate::schema::grade))
    /// under [`Order::AsRead`] and [`Order::UnknownFirst`], an error that fails the owner
    /// under [`Order::Door`].
    pub fn member_failure(&self) -> crate::schema::Grading {
        match self.order {
            Order::Door => crate::schema::grade(crate::versioning::MemberFailure::Refuse),
            Order::AsRead | Order::UnknownFirst => {
                crate::schema::grade(self.reading.on_member_failure)
            }
        }
    }

    /// Whether the last scalar read was a member read as absent ([`Order::Door`] under
    /// `DefaultAndTell`), clearing it: its owner takes it as absent.
    // hidden: the derived reads' protocol with the scalar reads
    #[doc(hidden)]
    pub fn take_defaulted(&mut self) -> bool {
        std::mem::take(&mut self.defaulted)
    }

    /// The `schema::unknown` of the field being read, set by its owner around the read of
    /// its value.
    // hidden: the derived reads' protocol
    #[doc(hidden)]
    pub fn field_unknown(&self) -> Option<crate::schema::UnknownRule> {
        self.field_unknown
    }

    /// Sets [`field_unknown`](Self::field_unknown), returning the one it replaces.
    // hidden: the derived reads' protocol
    #[doc(hidden)]
    pub fn set_field_unknown(
        &mut self,
        rule: Option<crate::schema::UnknownRule>,
    ) -> Option<crate::schema::UnknownRule> {
        std::mem::replace(&mut self.field_unknown, rule)
    }

    /// Runs `read` inside a map value or a box, where an unknown member is told and never
    /// kept (the door's walk cannot reach it to keep it).
    // hidden: the derived reads' protocol
    #[doc(hidden)]
    pub fn unkeepable<T>(&mut self, read: impl FnOnce(&mut Self) -> T) -> T {
        self.unkeepable += 1;
        let out = read(self);
        self.unkeepable -= 1;
        out
    }

    /// Whether an unknown member read here can be kept ([`unkeepable`](Self::unkeepable)).
    // hidden: the derived reads' protocol
    #[doc(hidden)]
    pub fn keepable(&self) -> bool {
        self.unkeepable == 0
    }

    /// The read with `texts` for its findings.
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Disposition, Node, Read};
    /// # use bedrock_addon_core::json::{JsonSlice, Value, from_str_jsonc};
    /// # use bedrock_addon_core::versioning::Reading;
    /// # fn told(d: &Diagnostics) -> Vec<String> {
    /// #     d.iter().map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code)).collect()
    /// # }
    /// use bedrock_addon_core::json::kit::Texts;
    /// let json: Value = from_str_jsonc("{}").unwrap();
    /// let mut found = Diagnostics::default();
    /// let mut read = Read::new(Reading::DOCUMENT, &mut found).texts(Texts {
    ///     missing: Some("particle has no material!"),
    ///     ..Texts::CODES
    /// });
    /// let members = read.object(&Node::new(&json, JsonPath::new())).unwrap();
    /// read.required(&members, "material");
    /// assert_eq!(told(&found), ["Error material particle has no material!"]);
    /// ```
    #[must_use]
    pub fn texts(mut self, texts: Texts) -> Self {
        self.texts = texts;
        self
    }

    /// The read telling unknown members at `severity` instead of the reading's: a reader the
    /// game runs without a word about them is told as information.
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Disposition, Node, Read};
    /// # use bedrock_addon_core::json::{JsonSlice, Value, from_str_jsonc};
    /// # use bedrock_addon_core::versioning::Reading;
    /// # fn told(d: &Diagnostics) -> Vec<String> {
    /// #     d.iter().map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code)).collect()
    /// # }
    /// use bedrock_addon_core::diagnostic::Severity;
    /// let json: Value = from_str_jsonc(r#"{"zz": 1}"#).unwrap();
    /// let mut found = Diagnostics::default();
    /// let mut read = Read::new(Reading::OWN, &mut found).unknown_severity(Severity::Info);
    /// let members = read.object(&Node::new(&json, JsonPath::new())).unwrap();
    /// read.unknown(&members, &[], Disposition::Keep);
    /// assert_eq!(told(&found), ["Info zz unrecognized field"]);
    /// ```
    #[must_use]
    pub fn unknown_severity(mut self, severity: Severity) -> Self {
        self.unknown_severity = severity;
        self
    }

    /// The severity the read tells an unknown member at.
    pub fn unknown_grade(&self) -> Severity {
        self.unknown_severity
    }

    /// The reading the read is under.
    pub fn reading(&self) -> Reading {
        self.reading
    }

    /// Whether what is read has failed: a finding that rejects the document has been told, or a
    /// member failed under a rule that fails its owner (every [`MemberFailure`](crate::versioning::MemberFailure) but
    /// `DefaultAndTell`; the caller drops the document, the owner or the element).
    pub fn failed(&self) -> bool {
        self.failed
    }

    /// Tells `finding`, as it is.
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Disposition, Node, Read};
    /// # use bedrock_addon_core::json::{JsonSlice, Value, from_str_jsonc};
    /// # use bedrock_addon_core::versioning::Reading;
    /// # fn told(d: &Diagnostics) -> Vec<String> {
    /// #     d.iter().map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code)).collect()
    /// # }
    /// use bedrock_addon_core::diagnostic::{Diagnostic, codes};
    /// let json: Value = from_str_jsonc(r#"{"name": "no_namespace"}"#).unwrap();
    /// let mut found = Diagnostics::default();
    /// let mut read = Read::new(Reading::DOCUMENT, &mut found);
    /// let members = read.object(&Node::new(&json, JsonPath::new())).unwrap();
    /// let name = read.required(&members, "name").unwrap();
    /// if !read.string(&name).is_some_and(|text| text.contains(':')) {
    ///     // A rule of the family's own, at the node.
    ///     read.push(Diagnostic::of(&codes::INVALID_VALUE).at(name.path().clone()));
    /// }
    /// assert!(read.failed());
    /// assert_eq!(told(&found), ["Error name invalid value"]);
    /// ```
    pub fn push(&mut self, finding: Diagnostic) {
        self.failed |= finding.rejects();
        self.findings.get().push(finding);
    }

    /// Tells `code` at `path` and `span`, graded by the reading's
    /// [`on_member_failure`](Reading::on_member_failure) ([`member_failure`](Self::member_failure)):
    /// a failed member. Under [`Order::Door`] the read stops there; a read that has stopped
    /// tells nothing more.
    fn fail(&mut self, code: Code, path: JsonPath, span: Option<Span>) {
        if self.stopped {
            return;
        }
        let grading = self.member_failure();
        let mut finding = Diagnostic::error(code).severity(grading.severity).at(path);
        finding.location.span = span;
        self.failed |= grading.fails_owner;
        self.push(finding);
        self.stopped = self.order == Order::Door;
    }

    /// Tells that `node` holds a value its member refuses, with `code` (`INVALID_VALUE`, an
    /// enum's `INVALID_ENUM_VALUE`, a leaf's own), graded as every failed member is
    /// ([`member_failure`](Self::member_failure)).
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Disposition, Node, Read};
    /// # use bedrock_addon_core::json::{JsonSlice, Value, from_str_jsonc};
    /// # use bedrock_addon_core::versioning::Reading;
    /// # fn told(d: &Diagnostics) -> Vec<String> {
    /// #     d.iter().map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code)).collect()
    /// # }
    /// use bedrock_addon_core::diagnostic::codes;
    /// let json: Value = from_str_jsonc(r#""huge""#).unwrap();
    /// let mut found = Diagnostics::default();
    /// let mut read = Read::new(Reading::DOCUMENT, &mut found);
    /// read.refuse(&Node::new(&json, JsonPath::of_keys(["size"])), (&codes::INVALID_VALUE).into());
    /// assert!(read.failed());
    /// assert_eq!(told(&found), ["Error size invalid value"]);
    /// ```
    pub fn refuse(&mut self, node: &Node<'_>, code: Code) {
        self.fail(code, node.path.clone(), node.span());
    }

    /// Tells `finding` as a failed member, at its own path, span and code, graded as
    /// [`refuse`](Self::refuse) grades: for a reader that builds the finding itself (the bridge
    /// to facet's read, a leaf's own detail).
    pub fn refuse_with(&mut self, finding: Diagnostic) {
        if self.stopped {
            return;
        }
        let grading = self.member_failure();
        let finding = finding.severity(grading.severity);
        self.failed |= grading.fails_owner;
        self.push(finding);
        self.stopped = self.order == Order::Door;
    }

    /// Tells that `node` holds a value of a JSON type its member does not take.
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Disposition, Node, Read};
    /// # use bedrock_addon_core::json::{JsonSlice, Value, from_str_jsonc};
    /// # use bedrock_addon_core::versioning::Reading;
    /// # fn told(d: &Diagnostics) -> Vec<String> {
    /// #     d.iter().map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code)).collect()
    /// # }
    /// let json: Value = from_str_jsonc(r#"{"count": "3"}"#).unwrap();
    /// let mut found = Diagnostics::default();
    /// let mut read = Read::new(Reading::DOCUMENT, &mut found);
    /// let members = read.object(&Node::new(&json, JsonPath::new())).unwrap();
    /// let count = read.required(&members, "count").unwrap();
    /// // A reader of the family's own that takes a digit string only from a number.
    /// if count.value().as_number().is_none() {
    ///     read.wrong_type(&count);
    /// }
    /// assert_eq!(told(&found), ["Error count wrong JSON type"]);
    /// ```
    pub fn wrong_type(&mut self, node: &Node<'_>) {
        let code = self.texts.wrong_type.map_or_else(
            || Code::from(&core_codes::INVALID_TYPE),
            |text| core_codes::INVALID_TYPE_WORDED.with([text]),
        );
        self.fail(code, node.path.clone(), node.span());
    }

    /// Tells that `node` holds a value of a JSON type its member does not take, where the read
    /// knows what it `expected`: in the family's [`Texts::wrong_kind`] wording when it has one,
    /// else as [`wrong_type`](Self::wrong_type) words it.
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Disposition, Node, Read};
    /// # use bedrock_addon_core::json::{JsonSlice, Value, from_str_jsonc};
    /// # use bedrock_addon_core::versioning::Reading;
    /// # fn told(d: &Diagnostics) -> Vec<String> {
    /// #     d.iter().map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code)).collect()
    /// # }
    /// use bedrock_addon_core::diagnostic::{Code, builders};
    /// use bedrock_addon_core::json::kit::{Expected, Texts, WrongKind};
    /// fn named(expected: Expected, found: &Value) -> Code {
    ///     builders::syntax(format!("expected {expected:?}, found {:?}", found.is_array()))
    /// }
    /// let json: Value = from_str_jsonc("[]").unwrap();
    /// let mut found = Diagnostics::default();
    /// let mut read = Read::new(Reading::DOCUMENT, &mut found)
    ///     .texts(Texts { wrong_kind: Some(WrongKind(named)), ..Texts::CODES });
    /// assert!(read.object(&Node::new(&json, JsonPath::of_keys(["root"]))).is_none());
    /// assert_eq!(told(&found), ["Error root expected Object, found true"]);
    /// ```
    pub fn wrong_kind(&mut self, node: &Node<'_>, expected: Expected) {
        match self.texts.wrong_kind {
            Some(WrongKind(word)) => {
                let code = word(expected, node.value);
                self.fail(code, node.path.clone(), node.span());
            }
            None => self.wrong_type(node),
        }
    }

    /// The members of `node`; a [wrong type](Self::wrong_type) when it is no object.
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Disposition, Node, Read};
    /// # use bedrock_addon_core::json::{JsonSlice, Value, from_str_jsonc};
    /// # use bedrock_addon_core::versioning::Reading;
    /// # fn told(d: &Diagnostics) -> Vec<String> {
    /// #     d.iter().map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code)).collect()
    /// # }
    /// let json: Value = from_str_jsonc(r#"{"description": [], "components": {}}"#).unwrap();
    /// let mut found = Diagnostics::default();
    /// let mut read = Read::new(Reading::DOCUMENT, &mut found);
    /// let members = read.object(&Node::new(&json, JsonPath::of_keys(["minecraft:block"]))).unwrap();
    /// let description = read.required(&members, "description").unwrap();
    /// assert!(read.object(&description).is_none());
    /// assert_eq!(told(&found), ["Error minecraft:block/description wrong JSON type"]);
    /// ```
    pub fn object<'v>(&mut self, node: &Node<'v>) -> Option<Members<'v>> {
        let Some(object) = node.value.as_object() else {
            self.wrong_kind(node, Expected::Object);
            return None;
        };
        let texts = node
            .text
            .and_then(|text| super::from_str_borrowed_jsonc(text.text).ok());
        Some(Members {
            node: node.clone(),
            object,
            texts,
        })
    }

    /// The elements of `node`; a [wrong type](Self::wrong_type) when it is no array.
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Disposition, Node, Read};
    /// # use bedrock_addon_core::json::{JsonSlice, Value, from_str_jsonc};
    /// # use bedrock_addon_core::versioning::Reading;
    /// # fn told(d: &Diagnostics) -> Vec<String> {
    /// #     d.iter().map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code)).collect()
    /// # }
    /// let json: Value = from_str_jsonc(r#"{"items": ["a:b", "c:d"], "groups": {}}"#).unwrap();
    /// let mut found = Diagnostics::default();
    /// let mut read = Read::new(Reading::DOCUMENT, &mut found);
    /// let members = read.object(&Node::new(&json, JsonPath::new())).unwrap();
    /// let items = read.required(&members, "items").unwrap();
    /// let items: Vec<&str> = read.array(&items).unwrap().iter().filter_map(|n| n.value().as_string().map(|s| s.as_str())).collect();
    /// assert_eq!(items, ["a:b", "c:d"]);
    /// let groups = read.required(&members, "groups").unwrap();
    /// assert!(read.array(&groups).is_none());
    /// assert_eq!(told(&found), ["Error groups wrong JSON type"]);
    /// ```
    pub fn array<'v>(&mut self, node: &Node<'v>) -> Option<Elements<'v>> {
        let Some(array) = node.value.as_array() else {
            self.wrong_kind(node, Expected::Array);
            return None;
        };
        let texts = node
            .text
            .and_then(|text| super::from_str_borrowed_jsonc(text.text).ok());
        Some(Elements {
            node: node.clone(),
            array,
            texts,
        })
    }

    /// The member `key` of `members`, when it is written; nothing is told. A `null` member is
    /// absent under [`NullHandling::Absent`].
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Disposition, Node, Read};
    /// # use bedrock_addon_core::json::{JsonSlice, Value, from_str_jsonc};
    /// # use bedrock_addon_core::versioning::Reading;
    /// # fn told(d: &Diagnostics) -> Vec<String> {
    /// #     d.iter().map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code)).collect()
    /// # }
    /// let json: Value = from_str_jsonc(r#"{"speed": null}"#).unwrap();
    /// let root = Node::new(&json, JsonPath::new());
    /// let mut found = Diagnostics::default();
    /// // The jsoncpp families read a `null` member as absent ...
    /// let mut read = Read::new(Reading::NULL_ABSENT, &mut found);
    /// let members = read.object(&root).unwrap();
    /// assert!(read.opt(&members, "speed").is_none());
    /// // ... the strict document as a value its read refuses.
    /// let mut read = Read::new(Reading::DOCUMENT, &mut found);
    /// let members = read.object(&root).unwrap();
    /// let speed = read.opt(&members, "speed").unwrap();
    /// assert_eq!(read.number(&speed), None);
    /// assert!(read.opt(&members, "range").is_none());
    /// assert_eq!(told(&found), ["Error speed wrong JSON type"]);
    /// ```
    pub fn opt<'v>(&self, members: &Members<'v>, key: &str) -> Option<Node<'v>> {
        let node = members.get(key)?;
        let absent = node.value.is_null() && self.reading.null == NullHandling::Absent;
        (!absent).then_some(node)
    }

    /// The member `key` of `members`; when it is not written (or is `null` under
    /// [`NullHandling::Absent`]), a missing required member at it, spanning the object.
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Disposition, Node, Read};
    /// # use bedrock_addon_core::json::{JsonSlice, Value, from_str_jsonc};
    /// # use bedrock_addon_core::versioning::Reading;
    /// # fn told(d: &Diagnostics) -> Vec<String> {
    /// #     d.iter().map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code)).collect()
    /// # }
    /// let text = r#"{"pack_id": "x"}"#;
    /// let json: Value = from_str_jsonc(text).unwrap();
    /// let root = Node::with_text(&json, JsonPath::of_keys(["entry"]), JsonSlice::new(text));
    /// let mut found = Diagnostics::default();
    /// let mut read = Read::new(Reading::DOCUMENT, &mut found);
    /// let members = read.object(&root).unwrap();
    /// assert!(read.required(&members, "pack_id").is_some());
    /// assert!(read.required(&members, "values").is_none());
    /// assert_eq!(told(&found), ["Error entry/values missing required field"]);
    /// // Spanning the object that lacks it.
    /// assert_eq!(found.iter().next().unwrap().location.span.map(|s| s.len as usize), Some(text.len()));
    /// ```
    pub fn required<'v>(&mut self, members: &Members<'v>, key: &str) -> Option<Node<'v>> {
        let node = self.opt(members, key);
        if node.is_none() {
            self.missing(members, key);
        }
        node
    }

    /// Tells that the member `key` of `members` is required and missing, spanning the object,
    /// as [`required`](Self::required) does: for a reader that found the member absent itself.
    pub fn missing_member(&mut self, members: &Members<'_>, key: &str) {
        self.missing(members, key);
    }

    /// Tells that the member `key` of `members` is required and missing.
    fn missing(&mut self, members: &Members<'_>, key: &str) {
        let code = self.texts.missing.map_or_else(
            || Code::from(&core_codes::MISSING_REQUIRED_FIELD),
            |text| core_codes::MISSING_REQUIRED_FIELD_WORDED.with([text]),
        );
        self.fail(code, members.node.path.child(key), members.node.span());
    }

    /// Tells every member of `members` that `known` does not name, in file order, under
    /// `disposition` at the unknown severity, spanning its value. With [`Disposition::Keep`]
    /// the members told come back, for the caller to keep; otherwise none does.
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Disposition, Node, Read};
    /// # use bedrock_addon_core::json::{JsonSlice, Value, from_str_jsonc};
    /// # use bedrock_addon_core::versioning::Reading;
    /// # fn told(d: &Diagnostics) -> Vec<String> {
    /// #     d.iter().map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code)).collect()
    /// # }
    /// let json: Value = from_str_jsonc(r#"{"name": "a", "colour": 1, "note": "x"}"#).unwrap();
    /// let mut found = Diagnostics::default();
    /// let mut read = Read::new(Reading::DOCUMENT, &mut found);
    /// let members = read.object(&Node::new(&json, JsonPath::new())).unwrap();
    /// let kept = read.unknown(&members, &["name"], Disposition::Keep);
    /// let kept: Vec<&str> = kept.iter().map(|(key, _)| *key).collect();
    /// assert_eq!(kept, ["colour", "note"]);
    /// assert_eq!(told(&found), ["Error colour unrecognized field", "Error note unrecognized field"]);
    /// ```
    pub fn unknown<'v>(
        &mut self,
        members: &Members<'v>,
        known: &[&str],
        disposition: Disposition,
    ) -> Vec<(&'v str, &'v Value)> {
        let mut kept = Vec::new();
        for (key, node) in members.iter() {
            if known.contains(&key) {
                continue;
            }
            self.unknown_member(&node, disposition);
            if disposition == Disposition::Keep {
                kept.push((key, node.value));
            }
        }
        kept
    }

    /// Tells the one member `node` as unknown under `disposition`, at the unknown severity:
    /// for a reader that tells its unknown members in file order between the reads of the
    /// known ones. [`unknown`](Self::unknown) is this for every member `known` does not name.
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Disposition, Node, Read};
    /// # use bedrock_addon_core::json::{JsonSlice, Value, from_str_jsonc};
    /// # use bedrock_addon_core::versioning::Reading;
    /// # fn told(d: &Diagnostics) -> Vec<String> {
    /// #     d.iter().map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code)).collect()
    /// # }
    /// let json: Value = from_str_jsonc(r#"{"x": 1, "identifier": 5}"#).unwrap();
    /// let mut found = Diagnostics::default();
    /// let mut read = Read::new(Reading::DOCUMENT, &mut found);
    /// let members = read.object(&Node::new(&json, JsonPath::new())).unwrap();
    /// // One walk in file order: each member read or told where it stands.
    /// for (key, node) in members.iter() {
    ///     match key {
    ///         "identifier" => {
    ///             read.string(&node);
    ///         }
    ///         _ => read.unknown_member(&node, Disposition::Report),
    ///     }
    /// }
    /// assert_eq!(told(&found), ["Error x member read and not kept", "Error identifier wrong JSON type"]);
    /// ```
    pub fn unknown_member(&mut self, node: &Node<'_>, disposition: Disposition) {
        let code = disposition.code(self.texts.unknown);
        let mut finding = Diagnostic::of(code)
            .severity(self.unknown_severity)
            .at(node.path.clone());
        finding.location.span = node.span();
        self.push(finding);
    }

    /// Tells the one member `node` as unknown by its struct's word (`schema::unknown`), as the
    /// typed door tells it: `keep` and `report` an `UnrecognizedField` (kept or not, the door's
    /// one line), `reject` a `DroppedField`, `ignore` an `IgnoredMember`; at the unknown
    /// severity, spanning its value, in [`Pass::Unknown`].
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Disposition, Node, Read};
    /// # use bedrock_addon_core::json::{JsonSlice, Value, from_str_jsonc};
    /// # use bedrock_addon_core::versioning::Reading;
    /// # fn told(d: &Diagnostics) -> Vec<String> {
    /// #     d.iter().map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code)).collect()
    /// # }
    /// use bedrock_addon_core::schema::UnknownRule;
    /// let json: Value = from_str_jsonc(r#"{"a": 1, "b": 2}"#).unwrap();
    /// let mut found = Diagnostics::default();
    /// let mut read = Read::new(Reading::DOCUMENT, &mut found);
    /// let members = read.object(&Node::new(&json, JsonPath::new())).unwrap();
    /// read.unknown_by_rule(&members.get("a").unwrap(), UnknownRule::Report);
    /// read.unknown_by_rule(&members.get("b").unwrap(), UnknownRule::Ignore);
    /// assert_eq!(told(&found), ["Error a unrecognized field", "Error b member ignored"]);
    /// ```
    pub fn unknown_by_rule(&mut self, node: &Node<'_>, rule: crate::schema::UnknownRule) {
        use crate::schema::UnknownRule;
        let disposition = match rule {
            UnknownRule::Keep | UnknownRule::Report => Disposition::Keep,
            UnknownRule::Reject => Disposition::Reject,
            UnknownRule::Ignore => Disposition::Ignore,
        };
        let code = disposition.code(self.texts.unknown);
        let mut finding = Diagnostic::of(code)
            .severity(self.unknown_severity)
            .at(node.path.clone());
        finding.location.span = node.span();
        self.defer(Pass::Unknown, finding);
    }

    /// The bool `node` holds; a [wrong type](Self::wrong_type) for any other value.
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Disposition, Node, Read};
    /// # use bedrock_addon_core::json::{JsonSlice, Value, from_str_jsonc};
    /// # use bedrock_addon_core::versioning::Reading;
    /// # fn told(d: &Diagnostics) -> Vec<String> {
    /// #     d.iter().map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code)).collect()
    /// # }
    /// let json: Value = from_str_jsonc("[true, 1]").unwrap();
    /// let mut found = Diagnostics::default();
    /// use bedrock_addon_core::versioning::Scalars;
    /// let mut read = Read::new(Reading::OWN.with_scalars(Scalars::JsonCpp), &mut found);
    /// let elements = read.array(&Node::new(&json, JsonPath::new())).unwrap();
    /// let read: Vec<Option<bool>> = elements.iter().map(|n| read.bool(&n)).collect();
    /// // Not even the `JsonCpp` reading takes a number for a bool.
    /// assert_eq!(read, [Some(true), None]);
    /// ```
    pub fn bool(&mut self, node: &Node<'_>) -> Option<bool> {
        let read = node.value.as_bool();
        if read.is_none() {
            self.wrong_kind(node, Expected::Bool);
        }
        read
    }

    /// The string `node` holds; a [wrong type](Self::wrong_type) for any other value.
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Disposition, Node, Read};
    /// # use bedrock_addon_core::json::{JsonSlice, Value, from_str_jsonc};
    /// # use bedrock_addon_core::versioning::Reading;
    /// # fn told(d: &Diagnostics) -> Vec<String> {
    /// #     d.iter().map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code)).collect()
    /// # }
    /// let json: Value = from_str_jsonc(r#"{"texture": "textures/ruby", "material": 7}"#).unwrap();
    /// let mut found = Diagnostics::default();
    /// let mut read = Read::new(Reading::DOCUMENT, &mut found);
    /// let members = read.object(&Node::new(&json, JsonPath::new())).unwrap();
    /// let texture = read.required(&members, "texture").unwrap();
    /// let material = read.required(&members, "material").unwrap();
    /// assert_eq!(read.string(&texture), Some("textures/ruby"));
    /// assert_eq!(read.string(&material), None);
    /// assert_eq!(told(&found), ["Error material wrong JSON type"]);
    /// ```
    pub fn string<'v>(&mut self, node: &Node<'v>) -> Option<&'v str> {
        let read = node.value.as_string().map(facet_value::VString::as_str);
        if read.is_none() {
            self.wrong_kind(node, Expected::String);
        }
        read
    }

    /// The number `node` holds, as the double it spells; under [`Scalars::JsonCpp`] a bool
    /// too, as 1 or 0. A [wrong type](Self::wrong_type) for any other value, a string included
    /// under either.
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Disposition, Node, Read};
    /// # use bedrock_addon_core::json::{JsonSlice, Value, from_str_jsonc};
    /// # use bedrock_addon_core::versioning::Reading;
    /// # fn told(d: &Diagnostics) -> Vec<String> {
    /// #     d.iter().map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code)).collect()
    /// # }
    /// let json: Value = from_str_jsonc("[0.5, 2, true]").unwrap();
    /// let root = Node::new(&json, JsonPath::new());
    /// let mut found = Diagnostics::default();
    /// let mut read = Read::new(Reading::DOCUMENT, &mut found);
    /// let strict: Vec<Option<f64>> = read.array(&root).unwrap().iter().map(|n| read.number(&n)).collect();
    /// use bedrock_addon_core::versioning::Scalars;
    /// let mut read = Read::new(Reading::OWN.with_scalars(Scalars::JsonCpp), &mut found);
    /// let coercing: Vec<Option<f64>> = read.array(&root).unwrap().iter().map(|n| read.number(&n)).collect();
    /// assert_eq!(strict, [Some(0.5), Some(2.0), None]);
    /// assert_eq!(coercing, [Some(0.5), Some(2.0), Some(1.0)]);
    /// ```
    pub fn number(&mut self, node: &Node<'_>) -> Option<f64> {
        let read = match (node.value.as_number(), self.reading.scalars) {
            (Some(number), _) => Some(number.to_f64_lossy()),
            (None, Scalars::JsonCpp) => node.value.as_bool().map(|flag| f64::from(u8::from(flag))),
            (None, _) => None,
        };
        if read.is_none() {
            self.wrong_kind(node, Expected::Number);
        }
        read
    }

    /// The `i32` `node` holds, under `ints` (jsoncpp's `isInt` and `asInt`, which the families
    /// disagree on, so each names its own): under every profile but `JsonCpp` the number `isInt`
    /// takes; under [`Scalars::JsonCpp`] any number, as `asInt` cuts it. A
    /// [wrong type](Self::wrong_type) for any other value, a bool or a string included under
    /// either.
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Disposition, Node, Read};
    /// # use bedrock_addon_core::json::{JsonSlice, Value, from_str_jsonc};
    /// # use bedrock_addon_core::versioning::Reading;
    /// # fn told(d: &Diagnostics) -> Vec<String> {
    /// #     d.iter().map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code)).collect()
    /// # }
    /// use bedrock_addon_core::json::{Coercing, IsIntRule, WideInt};
    /// const INTEGRAL: Coercing = Coercing::new(IsIntRule::Integral, WideInt::LowWord);
    /// const TOKEN: Coercing = Coercing::new(IsIntRule::Token, WideInt::LowWord);
    /// let json: Value = from_str_jsonc("[3, 2.0, 2.5]").unwrap();
    /// let root = Node::new(&json, JsonPath::new());
    /// let mut found = Diagnostics::default();
    /// let mut read = Read::new(Reading::DOCUMENT, &mut found);
    /// let ints = |read: &mut Read<'_>, rule| -> Vec<Option<i32>> {
    ///     read.array(&root).unwrap().iter().map(|n| read.int(&n, rule)).collect()
    /// };
    /// assert_eq!(ints(&mut read, TOKEN), [Some(3), None, None]);
    /// assert_eq!(ints(&mut read, INTEGRAL), [Some(3), Some(2), None]);
    /// use bedrock_addon_core::versioning::Scalars;
    /// let mut read = Read::new(Reading::OWN.with_scalars(Scalars::JsonCpp), &mut found);
    /// // `asInt` cuts a real toward zero.
    /// assert_eq!(ints(&mut read, TOKEN), [Some(3), Some(2), Some(2)]);
    /// ```
    pub fn int(&mut self, node: &Node<'_>, ints: Coercing) -> Option<i32> {
        let read = match self.reading.scalars {
            Scalars::JsonCpp => node.value.as_number().map(|_| node.value.as_int(0, ints)),
            _ => node.value.is_int(ints),
        };
        if read.is_none() {
            self.wrong_kind(node, Expected::Number);
        }
        read
    }
}

/// The byte range of `slice` in its file.
fn span_of(slice: JsonSlice<'_>) -> Span {
    let narrow = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
    Span {
        offset: narrow(slice.offset),
        len: narrow(slice.text.len()),
    }
}
