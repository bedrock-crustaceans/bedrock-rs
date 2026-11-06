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
//!   [`unknown`](Read::unknown), [`wrong_type`](Read::wrong_type) and the scalars
//!   ([`bool`](Read::bool), [`string`](Read::string), [`number`](Read::number),
//!   [`int`](Read::int));
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
//! | [`Reading::scalars`] | [`Scalars::JsonCpp`] only, in two reads: [`number`](Read::number) also takes a bool, as 1 or 0, and [`int`](Read::int) any number, cut as `asInt` cuts it under the caller's [`Coercing`] (a string or `null` is still a wrong type to both, and a bool to `int`). [`Scalars::Strict`], [`Scalars::NonStrict`] and [`Scalars::ConvertAndTell`] read alike here: a token of another kind is a wrong type, and `int` takes what `isInt` takes. [`bool`](Read::bool) and [`string`](Read::string) coerce under none |
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
use crate::diagnostic::{
    Class, Code, Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Severity, Span,
};
use crate::versioning::{NullHandling, Reading, Scalars};

// hidden: API not yet reviewed
#[doc(hidden)]
pub mod fixed_path;
// hidden: API not yet reviewed
#[doc(hidden)]
pub mod sound;
#[cfg(test)]
mod tests;
mod written;

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
}

impl Texts {
    /// Every finding in its code's own wording.
    pub const CODES: Self = Self {
        wrong_type: None,
        missing: None,
        unknown: None,
    };
}

/// One read of a hand-walked document: the slice's [`Reading`], the [`Texts`] and where the
/// findings go.
#[derive(Debug)]
pub struct Read<'d> {
    reading: Reading,
    texts: Texts,
    unknown_severity: Severity,
    findings: &'d mut Diagnostics,
    failed: bool,
}

impl<'d> Read<'d> {
    /// A read under `reading`, telling into `findings`.
    pub fn new(reading: Reading, findings: &'d mut Diagnostics) -> Self {
        let unknown_severity = reading.grade(Class::Unknown);
        Self {
            reading,
            texts: Texts::CODES,
            unknown_severity,
            findings,
            failed: false,
        }
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
        self.findings.push(finding);
    }

    /// Tells `code` at `path` and `span`, graded by the reading's
    /// [`on_member_failure`](Reading::on_member_failure).
    fn fail(&mut self, code: DiagnosticCode, path: JsonPath, span: Option<Span>) {
        let grading = crate::schema::grade(self.reading.on_member_failure);
        let mut finding = Diagnostic::error(code).severity(grading.severity).at(path);
        finding.location.span = span;
        self.failed |= grading.fails_owner;
        self.push(finding);
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
        self.fail(code.into(), node.path.clone(), node.span());
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
            self.wrong_type(node);
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
            self.wrong_type(node);
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

    /// Tells that the member `key` of `members` is required and missing.
    fn missing(&mut self, members: &Members<'_>, key: &str) {
        let code = self.texts.missing.map_or_else(
            || Code::from(&core_codes::MISSING_REQUIRED_FIELD),
            |text| core_codes::MISSING_REQUIRED_FIELD_WORDED.with([text]),
        );
        self.fail(
            code.into(),
            members.node.path.child(key),
            members.node.span(),
        );
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
            self.wrong_type(node);
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
            self.wrong_type(node);
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
            self.wrong_type(node);
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
            self.wrong_type(node);
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
