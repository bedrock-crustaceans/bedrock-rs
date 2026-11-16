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
//!   every profile as facet-json read it); it tells by a [`Profile`] (every failure or
//!   the first, members in file or declaration order, unknown members first, where met or
//!   last, the findings that fail nothing where met or after the read by [`Pass`]), and a
//!   [`fork`](Read::fork) of it is an attempt to [commit](Read::commit) or drop;
//! - [`Members`] and [`Elements`] are an object's members and an array's elements, each a
//!   [`Node`] one step down;
//! - [`Written`] is the slot a family that keeps its root as the author wrote it holds it in,
//!   left out of equality.
//!
//! What the reading decides:
//!
//! | axis | what the kit does with it |
//! |---|---|
//! | [`Reading::null`] | [`NullHandling::Absent`] (and `AbsentSilently`, which tells nothing): [`opt`](Read::opt) reads a `null` member as absent, [`required`](Read::required) as missing; otherwise the member is there and its read takes the `null` |
//! | [`Reading::scalars`] | every profile in [`scalar`](Read::scalar), through `json::judge` (the `ConvertAndTell` conversions as the plain typed read makes them; [`Scalars::Fork`] by the sound files' coercions of `jsoncpp::fork`, [`Scalars::Lenient`] by the texture files' of `jsoncpp::lenient`, [`Scalars::Accessors`] by the spawn files' accessors, told at its severity). In the hand reads every profile reads alike: a token of another kind is a wrong type, and [`int`](Read::int) takes what `isInt` takes. [`bool`](Read::bool) and [`string`](Read::string) coerce under none |
//! | [`Reading::beyond_width`] | in [`scalar`](Read::scalar): a number beyond its float reads as the infinity it is, told so the load clamps it and reads again ([`BeyondWidth::ClampAndTell`](crate::json::BeyondWidth::ClampAndTell)), or is a wrong type ([`BeyondWidth::Error`](crate::json::BeyondWidth::Error)) |
//! | [`Reading::unknown`] | the severity of [`unknown`](Read::unknown) |
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
//! use bedrock_addon_core::json::kit::{Node, Read};
//! use bedrock_addon_core::schema::UnknownRule;
//! use bedrock_addon_core::json::parse::{Options, parse};
//! use bedrock_addon_core::versioning::Reading;
//!
//! let text = r#"{"name": "ruby", "size": "big", "glow": 1}"#;
//! let parsed = parse(text, Options::default()).unwrap();
//! let payload = Node::parsed(&parsed, JsonPath::of_keys(["minecraft:gem"]));
//! let mut found = Diagnostics::default();
//! let mut read = Read::new(Reading::DOCUMENT, &mut found);
//!
//! let members = read.object(&payload).unwrap();
//! read.unknown(&members, &["name", "size"], UnknownRule::Keep);
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

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::OnceLock;

use facet_value::{VArray, VObject};

use super::parse::{Parsed, SpanNode};
use super::{Coerce, Coercing, JsonSlice, Value};
use crate::diagnostic::codes as core_codes;
use crate::diagnostic::{
    Class, Code, CodeDef, Diagnostic, Diagnostics, JsonPath, JsonPathSegment, Phase, Severity, Span,
};
use crate::schema::UnknownRule;
use crate::versioning::{NullHandling, RangeMembers, Reading, Scalars};

// hidden: API not yet reviewed
#[doc(hidden)]
pub mod fixed_path;
// hidden: API not yet reviewed
mod misread;
mod scalar;
#[cfg(test)]
mod tests;
#[doc(hidden)]
pub mod write;
mod written;

pub use misread::{BEYOND_FLOAT_MEMBER, Misread, Misreading};
pub use scalar::Scalar;
pub use written::{Present, Written};

/// A value being read: where it is in the document, and its text when the caller has it.
///
/// The text is the span source: with it, a finding at the node or below it carries the byte
/// range of the file it is about. It is the tree of spans [`json::parse`](super::parse) made
/// with the value ([`parsed`](Self::parsed): every span is a lookup).
#[derive(Clone, Debug)]
pub struct Node<'v> {
    value: &'v Value,
    path: JsonPath,
    text: Text<'v>,
}

/// Where a node's text is.
#[derive(Clone, Copy)]
enum Text<'v> {
    /// No text: no span.
    None,
    /// A node of a parse's span tree. The root of a read of a whole text has that text as
    /// `whole`, white space and comments round the value included, as a slice has it.
    Tree {
        node: SpanNode<'v, 'v>,
        whole: Option<JsonSlice<'v>>,
    },
}

impl std::fmt::Debug for Text<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => f.write_str("None"),
            Self::Tree { node, .. } => f.debug_tuple("Tree").field(&node.span()).finish(),
        }
    }
}

impl<'v> Text<'v> {
    /// A node below the root: its own token.
    fn tree(node: SpanNode<'v, 'v>) -> Self {
        Self::Tree { node, whole: None }
    }

    fn slice(self) -> Option<JsonSlice<'v>> {
        match self {
            Self::None => None,
            Self::Tree { node, whole } => Some(whole.unwrap_or_else(|| node.text())),
        }
    }

    fn span(self) -> Option<Span> {
        match self {
            Self::None => None,
            Self::Tree { node, whole } => Some(whole.map_or_else(|| node.span(), span_of)),
        }
    }
}

impl<'v> Node<'v> {
    /// The value at `path`, without a text: findings at it carry no span.
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Node, Read};
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
            text: Text::None,
        }
    }

    /// The root of a [parsed](super::parse) text, at `path`: findings at it and below it carry
    /// spans, each a lookup in the tree the parse made beside the value (nothing reads the text
    /// again). The root's own span is the text as given, white space and comments round the
    /// value included.
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Node, Read};
    /// # use bedrock_addon_core::json::parse::{Options, parse};
    /// # use bedrock_addon_core::versioning::Reading;
    /// let text = r#"{"size": "big"}"#;
    /// let parsed = parse(text, Options::default()).unwrap();
    /// let root = Node::parsed(&parsed, JsonPath::new());
    /// let mut found = Diagnostics::default();
    /// let mut read = Read::new(Reading::DOCUMENT, &mut found);
    /// let members = read.object(&root).unwrap();
    /// let size = read.required(&members, "size").unwrap();
    /// let span = size.span().unwrap();
    /// assert_eq!(&text[span.offset as usize..][..span.len as usize], r#""big""#);
    /// ```
    pub fn parsed<'t: 'v>(parsed: &'v Parsed<'t>, path: JsonPath) -> Self {
        Self {
            value: &parsed.value,
            path,
            text: Text::Tree {
                node: parsed.spans.root(),
                whole: Some(parsed.spans.text()),
            },
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
        self.text.slice()
    }

    /// The byte range of the value in the file, when the node has its text.
    pub fn span(&self) -> Option<Span> {
        self.text.span()
    }
}

/// The members of an object being read, each a [`Node`] one key down. From
/// [`Read::object`].
#[derive(Clone, Debug)]
pub struct Members<'v> {
    node: Node<'v>,
    object: &'v VObject,
    texts: Option<SpanNode<'v, 'v>>,
    /// The members of a tree level that has many, by key: made by the first lookup.
    by_key: OnceLock<HashMap<Cow<'v, str>, usize>>,
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
    /// # use bedrock_addon_core::json::kit::{Node, Read};
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
    /// # use bedrock_addon_core::json::kit::{Node, Read};
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
        // A tree that lists as many members as the object has has no repeated key, so its
        // members are the object's in the same order: no lookup by key per member.
        let aligned = matches!(&self.texts, Some(tree) if tree.len() == self.object.len());
        self.object
            .iter()
            .enumerate()
            .map(move |(index, (key, value))| {
                let key = key.as_str();
                let node = match &self.texts {
                    Some(tree) if aligned => match tree.member_at(index) {
                        Some(member) if member.key == key => {
                            self.child_of(Text::tree(member.value), key, value)
                        }
                        _ => self.child(key, value),
                    },
                    _ => self.child(key, value),
                };
                (key, node)
            })
    }

    /// The last member of the tree level written `key`: a scan for a few members, a table of
    /// them by key (made once) for many, so that a lookup per key costs the key, not the object.
    fn tree_member(&self, tree: &SpanNode<'v, 'v>, key: &str) -> Option<SpanNode<'v, 'v>> {
        if tree.len() <= SCAN_MEMBERS {
            return tree.member(key);
        }
        let by_key = self.by_key.get_or_init(|| {
            // Later copies replace earlier ones: the last stands.
            (0..tree.len())
                .filter_map(|index| tree.member_at(index).map(|member| (member.key, index)))
                .collect()
        });
        by_key
            .get(key)
            .and_then(|index| tree.member_at(*index))
            .map(|member| member.value)
    }

    fn child_of(&self, text: Text<'v>, key: &str, value: &'v Value) -> Node<'v> {
        Node {
            value,
            path: self.node.path.child(key),
            text,
        }
    }

    fn child(&self, key: &str, value: &'v Value) -> Node<'v> {
        let text = match &self.texts {
            Some(tree) => self.tree_member(tree, key).map_or(Text::None, Text::tree),
            None => Text::None,
        };
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
    texts: Option<SpanNode<'v, 'v>>,
}

/// The most members of an object whose lookup by key scans them.
const SCAN_MEMBERS: usize = 16;

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
    /// # use bedrock_addon_core::json::kit::{Node, Read};
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
        let text = match &self.texts {
            Some(tree) => tree.element(index).map_or(Text::None, Text::tree),
            None => Text::None,
        };
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

impl UnknownRule {
    /// The code of an unknown member told under this rule: the rule's own line, or the
    /// family's text given whole. A member told as an unknown field (`Keep`, `Report`) takes the
    /// family's code for one ([`Texts::unknown_named`], [`Texts::unknown_code`]).
    fn code(self, texts: Texts, key: &str) -> Code {
        let unrecognized = matches!(self, Self::Keep | Self::Report | Self::Silent);
        if let (true, Some(code)) = (unrecognized, texts.unknown_named) {
            return code.with([key.to_owned()]);
        }
        if let (true, Some(code)) = (unrecognized, texts.unknown_code) {
            return code.into();
        }
        // A text's `{}` is the member's key.
        let text = texts.unknown.map(|text| text.replacen("{}", key, 1));
        let told = |plain: &'static CodeDef<0>, worded: &'static CodeDef<1>| {
            text.map_or_else(|| plain.into(), |text| worded.with([text]))
        };
        match self {
            Self::Keep | Self::Report | Self::Silent => told(
                &core_codes::UNRECOGNIZED_FIELD,
                &core_codes::UNRECOGNIZED_FIELD_WORDED,
            ),
            Self::NotKept => told(
                &core_codes::MEMBER_NOT_KEPT,
                &core_codes::MEMBER_NOT_KEPT_WORDED,
            ),
            Self::Reject => told(
                &core_codes::DROPPED_FIELD,
                &core_codes::DROPPED_FIELD_WORDED,
            ),
            Self::Ignore => told(
                &core_codes::IGNORED_MEMBER,
                &core_codes::IGNORED_MEMBER_WORDED,
            ),
            // The game's own line, which the text of an unknown field does not reword.
            Self::Component => Code::from(&core_codes::UNRECOGNIZED_COMPONENT),
        }
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

/// A family's wording of a wrong type by where it is told ([`Texts::wrong_at`]): the code to
/// tell at the path of the value, from the path.
#[derive(Clone, Copy, Debug)]
pub struct WrongAt(pub fn(&JsonPath) -> Code);

/// By the function: two wordings are the same when they are the same function.
impl PartialEq for WrongAt {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::fn_addr_eq(self.0, other.0)
    }
}

impl Eq for WrongAt {}

impl std::hash::Hash for WrongAt {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.0 as usize).hash(state);
    }
}

/// The texts of the kit's findings. `None` is the code's own wording, which is what most of the
/// hand-walked families print: `wrong JSON type`, `missing required field`, `unrecognized
/// field`. A family whose game prints another line for one of them names it here once, for
/// every read of the [`Read`].
///
/// How a wrong type is worded, in the order the read asks: the member's own `schema::wrong_type`,
/// [`wrong_kind`](Self::wrong_kind) (what was expected, by a function), [`wrong_at`](Self::wrong_at)
/// (where it is told, by a function), [`accepts`](Self::accepts) (what the value's type takes,
/// in a template), the code's own wording.
///
/// The game's JSON schema node tree words three findings its own way ([`TREE`](Self::TREE):
/// the animation, render controller, geometry and fog loaders, the feature validator, which
/// words an enum name it does not know as an invalid value, and the older client's
/// attachable and client entity descriptions, which word a wrong type by its place). The
/// camera and aim-assist files word a missing and an unknown member in cereal's lines
/// (`versioning::chain_file`), `voxel_shape` an unknown member, `crafting_catalog` and
/// `deferred_settings` a wrong type by what was expected, and entity's lenient conversions
/// every value they give up ([`defaulted`](Self::defaulted)).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Texts {
    /// The text of a missing required member
    /// ([`codes::MISSING_REQUIRED_FIELD`](core_codes::MISSING_REQUIRED_FIELD)).
    pub missing: Option<&'static str>,
    /// The text of an unknown member, whatever its [`UnknownRule`]; a `{}` in it is the
    /// member's key.
    pub unknown: Option<&'static str>,
    /// The code an unknown member told as an `UnrecognizedField` ([`UnknownRule::Keep`] and
    /// [`UnknownRule::Report`], which `schema::unknown` words other than `reject`, `ignore` and
    /// `component` tell) is told under, in
    /// place of [`UNRECOGNIZED_FIELD`](core_codes::UNRECOGNIZED_FIELD) and [`unknown`](Self::unknown):
    /// a family whose game prints cereal's line for it
    /// ([`MEMBER_NOT_IN_SCHEMA`](core_codes::MEMBER_NOT_IN_SCHEMA)) names that once.
    pub unknown_code: Option<&'static CodeDef<0>>,
    /// The code of a wrong type where the read knows what it expected ([`Read::object`],
    /// [`Read::array`], the scalar reads), from what it expected and what it found, in place
    /// of [`INVALID_TYPE`](core_codes::INVALID_TYPE): `crafting_catalog` words it as vanilla's
    /// `MalformedInput` naming both.
    pub wrong_kind: Option<WrongKind>,
    /// The code of every wrong type, made from where it is told, in place of
    /// [`INVALID_TYPE`](core_codes::INVALID_TYPE) and [`wrong_kind`](Self::wrong_kind): a family whose
    /// game words a wrong type by the member it is of (the older client's "unknown child schema
    /// option type.  Allowed types: …"), whichever type the member's reader expected.
    pub wrong_at: Option<WrongAt>,
    /// The code of a missing required member, made from the member's key (`"Required child %s
    /// not found"`), in place of [`missing`](Self::missing): the client's older schema tree words
    /// it with the name.
    pub missing_named: Option<&'static CodeDef<1>>,
    /// The code an unknown member told as an `UnrecognizedField` ([`UnknownRule::Keep`]) is told
    /// under, made from the member's key (`"child '%s' not valid here."`), in place of
    /// [`unknown_code`](Self::unknown_code) and [`unknown`](Self::unknown). Its severity is the
    /// read's ([`Read::unknown_severity`]), which the family sets to an error where its game
    /// rejects whatever the unknown mask says.
    pub unknown_named: Option<&'static CodeDef<1>>,
    /// Whether a missing required member is told at the object that lacks it, not at the member:
    /// the older client's schema tree words it "Required child %s not found" at the parent.
    pub missing_in_object: bool,
    /// The code of an enum name that is none of its values, in place of
    /// [`INVALID_ENUM_VALUE`](core_codes::INVALID_ENUM_VALUE).
    pub invalid_enum: Option<&'static CodeDef<0>>,
    /// The code every failure is told under where the reading defaults what failed
    /// (`DefaultAndTell`), in place of its own: a lenient conversion that tells each value it
    /// gives up with one line of its own. A missing member is then told at the object that
    /// lacks it, the innermost value the file has there.
    pub defaulted: Option<&'static CodeDef<0>>,
    /// The code of a wrong type made from what the value's type accepts, in the schema's words
    /// (`"object"`, `"bool, string"`: [`Read::wrong_accepting`], and [`Read::wrong_kind`] from
    /// the [`Expected`] kind's word), in place of [`INVALID_TYPE`](core_codes::INVALID_TYPE): the game's
    /// schema node tree's `"unknown child schema option type.  Allowed types: %s"`. That is the
    /// line [`wrong_at`](Self::wrong_at) words by place, which comes first where both are set: a
    /// family whose types know what they accept says `accepts`, one that knows it by the member
    /// says `wrong_at`.
    pub accepts: Option<&'static CodeDef<1>>,
}

impl Texts {
    /// Every finding in its code's own wording.
    pub const CODES: Self = Self {
        missing: None,
        unknown: None,
        unknown_code: None,
        wrong_kind: None,
        wrong_at: None,
        missing_named: None,
        unknown_named: None,
        missing_in_object: false,
        invalid_enum: None,
        defaulted: None,
        accepts: None,
    };

    /// The game's JSON schema node tree's lines: `"Required child %s not found"`, `"child '%s'
    /// not valid here."` and `"unknown child schema option type.  Allowed types: %s"`.
    pub const TREE: Self = Self {
        missing_named: Some(&core_codes::REQUIRED_CHILD),
        unknown_named: Some(&core_codes::CHILD_NOT_VALID),
        accepts: Some(&core_codes::UNKNOWN_CHILD_OPTION),
        ..Self::CODES
    };
}

impl Expected {
    /// The kind in the words of the game's schema node tree (a number is its float node's).
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Object => "object",
            Self::Array => "array",
            Self::String => "string",
            Self::Number => "float",
            Self::Bool => "bool",
        }
    }
}

/// How a [`Read`] tells: how many failures, in which order it reads an object's members, where
/// it tells their unknown members, and whether the findings that are no failure wait for the
/// end of the read. Four choices, each on its own ([`failures`](Self::failures),
/// [`members`](Self::members), [`unknown`](Self::unknown), [`passes`](Self::passes)); a family
/// takes a ready-made profile and changes an axis with a `with_*` builder where its game's
/// reader differs.
///
/// The profile is the read's, set where the family reads ([`Read::profile`],
/// `versioning::read_payload_with`), not the slice's [`Reading`]: it is the shape of the game's
/// reader of a document, the same in every slice of the families read so far, where a
/// [`Reading`] says what differs between slices (the null rule, the scalars, the unknown mask);
/// and the old typed read, which read by the same [`Reading`], had no profile to honour.
///
/// | profile | failures | members | unknown | passes | for |
/// |---|---|---|---|---|---|
/// | [`FIRST_FAILURE`](Self::FIRST_FAILURE) | first | file | where met | after the read | a family that facet-json read (the default of `read_payload`) |
/// | [`EVERY_FAILURE`](Self::EVERY_FAILURE) | every | file | where met | where met | a game reader that binds every member and tells each failure (poi, camera, aim assist, the atomic client entity); the default of [`Read::new`] |
/// | [`UNKNOWN_FIRST`](Self::UNKNOWN_FIRST) | every | file | first | where met | a reader that tells an object's unknown members before it reads it (`crafting_catalog`, the simplified spawn rules, block culling) |
/// | [`DECLARED_ORDER`](Self::DECLARED_ORDER) | every | declaration | first | where met | a reader that asks for its members one by one in the order it declares them, after telling the unknown ones (the kit's hand walks: `ddui`'s styles, loot, deferred settings) |
/// | [`TREE`](Self::TREE) | every | file | where met | where met | the game's JSON schema node tree ([`Objects::Tree`]; its loaders name their [`Names`], [`Lossy`] and order) |
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct Profile {
    /// How many failures the read tells.
    pub failures: Failures,
    /// The order the members of an object are read in.
    pub members: MemberOrder,
    /// Where the unknown members of an object are told.
    pub unknown: UnknownAt,
    /// When the findings that fail nothing are told.
    pub passes: Passes,
    /// What takes its default when a value fails under `DefaultAndTell`.
    pub recovery: Recovery,
    /// When an object fails, and where and whether its missing members are told.
    pub objects: Objects,
    /// How a member's name is compared with the names a struct declares.
    pub names: Names,
    /// Whether a number a float member does not hold exactly is told.
    pub lossy: Lossy,
    /// Whether a finding of the read spans the bytes it is about.
    pub spans: SpanRule,
}

/// When an object fails, and where and whether its missing members are told
/// ([`Profile::objects`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Objects {
    /// An object fails when a member it must have fails; an unknown member fails nothing, and
    /// a missing member is told at the member, whatever else failed.
    Members,
    /// The game's JSON schema node tree (the animation, render controller, geometry and older
    /// fog loaders): an object (a map's too) fails when an error was raised while its members
    /// were read, an unknown member's included; its missing members are then not told, and
    /// otherwise each is told at the object. A required member whose read failed without an
    /// error (a lenient reading's information) is missing. Every failure tells a finding
    /// ([`FromJson::from_json`](crate::schema::FromJson::from_json)): a leaf that returns `None`
    /// silently is a defect, and a debug build panics on it.
    Tree,
}

/// How a member's name is compared with the names a struct declares and the prefix of a key
/// rule ([`Profile::names`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Names {
    /// As written.
    Exact,
    /// Without regard to ASCII case (both animation loaders): a member spelled in another case
    /// than declared is read as the declared one and noted ([`Read::take_respelled`]); of two
    /// members that differ in case alone the later is read, and the earlier is told as dropped
    /// (a `DROPPED_FIELD` information) unless both hold one value.
    IgnoreAsciiCase,
}

/// Whether a number a float member does not hold exactly is told ([`Profile::lossy`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Lossy {
    /// It reads as the nearest float, silently.
    Silent,
    /// It reads as the nearest float, and a `LOSSY_CONVERSION` information at the number says
    /// so (the crate's own: the document is written back with the float).
    Told,
}

/// Whether a finding of the read spans the bytes of the value it is about ([`Profile::spans`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SpanRule {
    /// Every finding spans its value, as the one rule of the reader says.
    Told,
    /// No finding has a span: a file whose bytes no finding may address (the server's
    /// `secrets.json`, whose findings are printed where the file is not), or a reader that
    /// names a place by its path alone. What a finding says is unchanged.
    Withheld,
}

/// How many failures a read tells ([`Profile::failures`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Failures {
    /// The read stops at its first failure, which is then all it tells, and the failure is an
    /// error whatever the reading's [`on_member_failure`](Reading::on_member_failure): facet's
    /// typed read, which stops at its first error. A failed list element fails the list.
    First,
    /// Every failure, each graded by the reading's
    /// [`on_member_failure`](Reading::on_member_failure): the read goes on past a failed member
    /// and tells the next one.
    Every,
    /// The first failure alone, as under [`First`](Self::First) (an error whatever the reading
    /// says, a failed list element fails the list), but the read goes on past it: every later
    /// member is read, and what fails nothing is told as if none had failed: the unknown members
    /// and the nulls read as absent, by [`Passes::AfterRead`]; a scalar read as another value
    /// ([`Pass::Misread`]) only of a read that did not fail. A schema validator that stops at its
    /// first error and still checks the document's members against it: the older client's.
    FirstOnly,
}

/// The order a struct reads the members of its object in ([`Profile::members`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MemberOrder {
    /// As the file writes them: facet's order, and the game's readers that walk the object.
    File,
    /// As the struct declares its fields: the readers that ask for each member by name. A
    /// missing member is told in declaration order under either.
    Declaration,
    /// In the sorted order of the keys' bytes, jsoncpp's `std::map` (the render controller and
    /// older fog loaders); a map's entries too, which it then holds in that order.
    Sorted,
    /// Read and told as [`Sorted`](Self::Sorted), a map holding its entries in the file's
    /// order: a reader that walks jsoncpp's `std::map` (the server's `secrets.json`, whose
    /// warnings come in the names' byte order) of a document the crate writes back as written.
    SortedInPlace,
}

/// Where a struct tells the unknown members of its object ([`Profile::unknown`]), each in file
/// order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum UnknownAt {
    /// Before it reads any member.
    First,
    /// Each where it stands among the members (read in [`MemberOrder::File`]); after them
    /// where the members are read in declaration order, which meets no unknown one.
    WhereMet,
    /// After every member is read.
    Last,
}

/// When the findings that fail nothing are told ([`Profile::passes`]): the members written
/// `null` that the reading refuses, those read as absent for their `null`, the unknown members
/// and the scalars read as another value than written.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Passes {
    /// Where the read meets each; a converted scalar or a refused `null` is a wrong type in the
    /// family's wording, and fails its member as one.
    WhereMet,
    /// After the read ([`Read::finish`]), by [`Pass`] (in its order, each pass in the order the
    /// read met its findings), and only when the read did not stop; the refused `null`s, when
    /// there are any, are then all that is told: the old typed read's passes. A converted scalar
    /// is its misread wrong type, which leaves its member read; under `DefaultAndTell` a
    /// scalar member its profile refuses is read as absent and told in the last pass, as the
    /// old scalar pre-pass blanked it.
    AfterRead,
}

impl Profile {
    /// The read of facet-json's typed read (the old `parse_payload`): the first failure, in file
    /// order, and the rest by pass after the read. What every family that read through it
    /// reads by, and `versioning::read_payload`'s default.
    pub const FIRST_FAILURE: Self = Self {
        failures: Failures::First,
        members: MemberOrder::File,
        unknown: UnknownAt::WhereMet,
        passes: Passes::AfterRead,
        recovery: Recovery::Member,
        objects: Objects::Members,
        names: Names::Exact,
        lossy: Lossy::Silent,
        spans: SpanRule::Told,
    };
    /// Every failure and every finding where the read meets it, in file order: a game reader
    /// that binds every member and tells each failure. [`Read::new`]'s.
    pub const EVERY_FAILURE: Self = Self {
        failures: Failures::Every,
        members: MemberOrder::File,
        unknown: UnknownAt::WhereMet,
        passes: Passes::WhereMet,
        recovery: Recovery::Member,
        objects: Objects::Members,
        names: Names::Exact,
        lossy: Lossy::Silent,
        spans: SpanRule::Told,
    };
    /// As [`EVERY_FAILURE`](Self::EVERY_FAILURE), an object's unknown members told before its
    /// members.
    pub const UNKNOWN_FIRST: Self = Self::EVERY_FAILURE.with_unknown(UnknownAt::First);
    /// Every failure, an object's unknown members first and then its members in the order the
    /// struct declares them: the hand walks that ask for each member by name.
    pub const DECLARED_ORDER: Self = Self::UNKNOWN_FIRST.with_members(MemberOrder::Declaration);

    /// The profile telling `failures`.
    #[must_use]
    pub const fn with_failures(self, failures: Failures) -> Self {
        Self { failures, ..self }
    }

    /// The profile reading an object's members in `members` order.
    #[must_use]
    pub const fn with_members(self, members: MemberOrder) -> Self {
        Self { members, ..self }
    }

    /// The profile telling an object's unknown members `unknown`.
    #[must_use]
    pub const fn with_unknown(self, unknown: UnknownAt) -> Self {
        Self { unknown, ..self }
    }

    /// The profile telling what fails nothing `passes`.
    #[must_use]
    pub const fn with_passes(self, passes: Passes) -> Self {
        Self { passes, ..self }
    }

    /// The profile giving up `recovery` when a value fails under `DefaultAndTell`.
    #[must_use]
    pub const fn with_recovery(self, recovery: Recovery) -> Self {
        Self { recovery, ..self }
    }

    /// The profile failing its objects as `objects` says.
    #[must_use]
    pub const fn with_objects(self, objects: Objects) -> Self {
        Self { objects, ..self }
    }

    /// The profile comparing member names as `names` says.
    #[must_use]
    pub const fn with_names(self, names: Names) -> Self {
        Self { names, ..self }
    }

    /// The profile telling a float member's lossy number as `lossy` says.
    #[must_use]
    pub const fn with_lossy(self, lossy: Lossy) -> Self {
        Self { lossy, ..self }
    }

    /// The profile spanning its findings as `spans` says.
    #[must_use]
    pub const fn with_spans(self, spans: SpanRule) -> Self {
        Self { spans, ..self }
    }

    /// The game's JSON schema node tree: every failure where it is met, in file order, an
    /// object failed by any error below it ([`Objects::Tree`]); a loader names its tree's
    /// order, names and floats with the builders.
    pub const TREE: Self = Self::EVERY_FAILURE.with_objects(Objects::Tree);

    /// A schema validator's read: the first failure alone, then the nulls read as absent, the
    /// unknown members told beside it and the rest of what fails nothing
    /// ([`Failures::FirstOnly`], [`Passes::AfterRead`]).
    pub const FIRST_THEN_PASSES: Self = Self {
        failures: Failures::FirstOnly,
        members: MemberOrder::File,
        unknown: UnknownAt::WhereMet,
        passes: Passes::AfterRead,
        recovery: Recovery::Member,
        objects: Objects::Members,
        names: Names::Exact,
        lossy: Lossy::Silent,
        spans: SpanRule::Told,
    };

    /// Whether the read stops at its first failure ([`Failures::First`]).
    #[must_use]
    pub const fn stops(self) -> bool {
        matches!(self.failures, Failures::First)
    }

    /// Whether the first failure is the only one told ([`Failures::First`] and
    /// [`Failures::FirstOnly`]): it is an error whatever the reading says, and a failed list
    /// element fails the list.
    #[must_use]
    pub const fn first_alone(self) -> bool {
        matches!(self.failures, Failures::First | Failures::FirstOnly)
    }

    /// Whether the findings that fail nothing wait for the end of the read
    /// ([`Passes::AfterRead`]).
    #[must_use]
    pub const fn defers(self) -> bool {
        matches!(self.passes, Passes::AfterRead)
    }
}

/// What a read under `DefaultAndTell` gives up when a value fails ([`Profile::recovery`]), as
/// the game's lenient reader of the document does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Recovery {
    /// The failed member takes its default; a failed list element or map value fails its list
    /// or map, whose member then takes its default.
    Member,
    /// The innermost value that fails: a member takes its default, a list's element or a
    /// map's entry is left out. A legacy reader that reads member by member at every level.
    Value,
    /// A member of the root, or an element of a list the root holds, whole: what fails below
    /// one fails it, and it takes its default or is left out of its list. The first failure
    /// below one is the only one told of it, and one told below it is followed by the
    /// member or element told dropped ([`DROPPED_FIELD`](core_codes::DROPPED_FIELD), an
    /// information). A legacy reader that reads the root's members one by one, each whole.
    TopLevel,
}

/// The pass a finding told after the read belongs to ([`Passes::AfterRead`]), in the order the
/// passes run.
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
    fn len(&self) -> usize {
        match self {
            Self::Borrowed(findings) => findings.len(),
            Self::Owned(findings) => findings.len(),
        }
    }

    fn get(&mut self) -> &mut Diagnostics {
        match self {
            Self::Borrowed(findings) => findings,
            Self::Owned(findings) => findings,
        }
    }
}

/// What a [`Read`] had told at a moment ([`Read::mark`]).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Mark {
    errors: usize,
    told: usize,
}

/// One read of a hand-walked document: the slice's [`Reading`], the [`Texts`] and where the
/// findings go.
#[derive(Debug)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "the read's state: failed, stopped, a member read as absent, a map's value"
)]
pub struct Read<'d> {
    reading: Reading,
    texts: Texts,
    unknown_severity: Severity,
    findings: Sink<'d>,
    /// The note of the information a scalar read as another JSON type than written is told with
    /// ([`non_strict`](Self::non_strict)).
    rewrite_note: Option<&'static str>,
    /// The phase every finding of the read is told in, where it is not the default
    /// ([`phase`](Self::phase)).
    phase: Option<Phase>,
    /// What the document is read for, fed to a field's code of one argument
    /// ([`subject`](Self::subject)).
    subject: Option<&'d str>,
    /// The format version the document is written at, for the members a struct reads from a
    /// version on ([`format`](Self::format)).
    format: Option<crate::version::FormatVersion>,
    failed: bool,
    profile: Profile,
    /// The read met its first failure under [`Failures::First`].
    stopped: bool,
    /// The read told its first failure under [`Failures::FirstOnly`], which is the only one.
    told_first: bool,
    /// The findings of the passes after the read, under [`Passes::AfterRead`].
    deferred: Vec<(Pass, Diagnostic)>,
    /// The last scalar read was a member read as absent ([`Passes::AfterRead`] under
    /// `DefaultAndTell`), for the owner to take as absent.
    defaulted: bool,
    /// The `schema::unknown` of the field being read, for the object it holds.
    field_unknown: Option<crate::schema::UnknownRule>,
    /// How many map values or boxes the read is inside: an unknown member there is told and
    /// never kept, as the old walk could not reach it.
    unkeepable: u32,
    /// The value read is a map's value, not a struct's member: a `null` there its profile
    /// refuses is not read as absent ([`Passes::AfterRead`] under `DefaultAndTell`).
    map_value: bool,
    /// The rules of the field whose member is being read, and the address of that member's
    /// value: they hold at that value only, not at one inside it.
    member: Option<(usize, &'static crate::schema::derive_support::MemberRules)>,
    /// The load the document is read in, for a read that hands a part of it to another reader
    /// (a component map's rows): [`load`](Self::load).
    load: Option<&'d crate::load::LoadContext>,
    /// How many steps the path of the root read has ([`Recovery::TopLevel`]'s members and
    /// elements are one below it).
    root: usize,
    /// The member or element of the root the last failure told was in, under
    /// [`Recovery::TopLevel`].
    told_unit: Option<JsonPath>,
    /// A Molang number beyond `f32` reads as the largest finite one and is told
    /// ([`clamps_molang`](Self::clamps_molang)).
    clamp_molang: bool,
    /// A number of a `json::Value` kept whole that no double holds reads as the largest finite
    /// one and is told ([`clamps_kept`](Self::clamps_kept)).
    clamp_kept: bool,
    /// How many findings that reject the document the read told: an object of a tree fails by
    /// those told below it ([`Objects::Tree`]).
    errors: usize,
    /// The members read as a declared name spelled in another case ([`Names::IgnoreAsciiCase`]).
    respelled: Vec<(JsonPath, Option<Span>)>,
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
            rewrite_note: None,
            phase: None,
            subject: None,
            format: None,
            failed: false,
            profile: Profile::EVERY_FAILURE,
            stopped: false,
            told_first: false,
            deferred: Vec::new(),
            defaulted: false,
            field_unknown: None,
            unkeepable: 0,
            map_value: false,
            member: None,
            load: None,
            root: 0,
            told_unit: None,
            clamp_molang: false,
            clamp_kept: false,
            errors: 0,
            respelled: Vec::new(),
        }
    }

    /// The read telling by `profile` (a [`Read::new`] tells by [`Profile::EVERY_FAILURE`]).
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Node, Read};
    /// # use bedrock_addon_core::json::{JsonSlice, Value, from_str_jsonc};
    /// # use bedrock_addon_core::versioning::Reading;
    /// # fn told(d: &Diagnostics) -> Vec<String> {
    /// #     d.iter().map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code)).collect()
    /// # }
    /// use bedrock_addon_core::json::kit::Profile;
    /// let json: Value = from_str_jsonc(r#"{"a": 1, "b": 2}"#).unwrap();
    /// let mut found = Diagnostics::default();
    /// let mut read = Read::new(Reading::DOCUMENT, &mut found).profile(Profile::FIRST_FAILURE);
    /// let members = read.object(&Node::new(&json, JsonPath::new())).unwrap();
    /// for key in ["a", "b"] {
    ///     let node = read.required(&members, key).unwrap();
    ///     read.string(&node);
    /// }
    /// // A read of the first failure stops there.
    /// assert!(read.stopped());
    /// assert_eq!(told(&found), ["Error a wrong JSON type"]);
    /// ```
    #[must_use]
    pub fn profile(mut self, profile: Profile) -> Self {
        self.profile = profile;
        self
    }

    /// The read telling every finding in `phase`, where the default ([`Phase::Parse`]) is not
    /// the document's: a merged document is read after the stack's merge, in
    /// [`Phase::Validate`].
    #[must_use]
    pub fn phase(mut self, phase: Phase) -> Self {
        self.phase = Some(phase);
        self
    }

    /// The read naming `subject` (the block an entry is read for, the file a document is in) in
    /// every code of one argument a field states for its member: `schema::wrong_type`,
    /// `missing`, `invalid` or `negative` given a `CodeDef<1>`. A read without a subject tells
    /// its own code where a field states one of those (a code of no argument is told either
    /// way).
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Node, Read};
    /// # use bedrock_addon_core::json::{Value, from_str_jsonc};
    /// # use bedrock_addon_core::versioning::Reading;
    /// let json: Value = from_str_jsonc("{}").unwrap();
    /// let mut found = Diagnostics::default();
    /// let read = Read::new(Reading::DOCUMENT, &mut found).subject("minecraft:stone");
    /// assert_eq!(read.subject_text(), Some("minecraft:stone"));
    /// ```
    #[must_use]
    pub fn subject(mut self, subject: &'d str) -> Self {
        self.subject = Some(subject);
        self
    }

    /// What the document is read for ([`subject`](Self::subject)), for a leaf that words its
    /// own finding with it.
    pub fn subject_text(&self) -> Option<&'d str> {
        self.subject
    }

    /// The code `code` tells under this read: fed the subject when it takes one, `None` when it
    /// does and the read names none.
    pub(crate) fn member_code(
        &self,
        code: crate::schema::derive_support::MemberCode,
    ) -> Option<Code> {
        code.code(self.subject)
    }

    /// The code the field whose member is `node` states for a value its type refuses
    /// (`schema::invalid`), fed the read's subject; `None` where it states none, or one of one
    /// argument and the read has no subject. For a leaf that tells its refusal at the code's own
    /// severity, as the game logs it, where [`refuse`](Self::refuse) grades it by the reading.
    pub fn invalid_code(&self, node: &Node<'_>) -> Option<Code> {
        self.member_rules(node)
            .and_then(|rules| rules.invalid)
            .and_then(|code| self.member_code(code))
    }

    /// The profile the read tells by.
    pub fn telling(&self) -> Profile {
        self.profile
    }

    /// Whether the read stopped at its first failure ([`Failures::First`]): nothing more is
    /// read or told, and every read returns `None`.
    pub fn stopped(&self) -> bool {
        self.stopped
    }

    /// Stops the read whatever its profile: nothing more is read or told, and every read
    /// returns `None`, as after the first failure of a [`Failures::First`] read. For a game
    /// reader that returns at a message of its own and gives up the document there (a trade
    /// table's legacy parser returns an empty table), after it has told that message.
    pub fn stop(&mut self) {
        self.stopped = true;
    }

    /// A read with this read's reading, texts, severities and profile that tells into a list of
    /// its own, for an attempt the caller may [`commit`](Self::commit) or drop: an arm of a
    /// one-of that is tried and may be given up for another.
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Node, Read};
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
    pub fn fork(&self) -> Read<'d> {
        Read {
            reading: self.reading,
            texts: self.texts,
            unknown_severity: self.unknown_severity,
            findings: Sink::Owned(Diagnostics::default()),
            rewrite_note: self.rewrite_note,
            phase: self.phase,
            subject: self.subject,
            format: self.format,
            failed: false,
            profile: self.profile,
            // A fork of a read that has stopped reads nothing either.
            stopped: self.stopped,
            told_first: self.told_first,
            deferred: Vec::new(),
            defaulted: false,
            field_unknown: self.field_unknown,
            unkeepable: self.unkeepable,
            map_value: self.map_value,
            member: self.member,
            load: self.load,
            root: self.root,
            told_unit: self.told_unit.clone(),
            clamp_molang: self.clamp_molang,
            clamp_kept: self.clamp_kept,
            errors: 0,
            respelled: Vec::new(),
        }
    }

    /// Takes `fork`'s findings (with the moves of the lists it shortened, so a member below
    /// one is still named as the file has it), its kept unknown members, its failure and a
    /// member it read as absent as this read's own.
    pub fn commit(&mut self, fork: Read<'_>) {
        let Read {
            findings,
            failed,
            stopped,
            told_first,
            deferred,
            defaulted,
            told_unit,
            errors,
            respelled,
            ..
        } = fork;
        let findings = match findings {
            Sink::Owned(findings) => findings,
            // A borrowed sink is not a fork's: its findings are already where they go.
            Sink::Borrowed(_) => Diagnostics::default(),
        };
        self.findings.get().append(findings);
        self.deferred.extend(deferred);
        self.failed |= failed;
        self.told_first |= told_first;
        self.stopped |= stopped;
        self.defaulted |= defaulted;
        if told_unit.is_some() {
            self.told_unit = told_unit;
        }
        self.errors += errors;
        self.respelled.extend(respelled);
    }

    /// Takes `fork`'s findings and what it defers as this read's own, and nothing else: an
    /// attempt whose value is given up and kept as written (`schema::or_kept`), whose failure
    /// fails nothing and whose kept members go with it.
    pub(crate) fn commit_findings(&mut self, fork: Read<'_>) {
        let Read {
            findings, deferred, ..
        } = fork;
        if let Sink::Owned(mut findings) = findings {
            drop(findings.take_unknown());
            self.findings.get().append(findings);
        }
        self.deferred.extend(deferred);
    }

    /// Tells `finding` in `pass`: now under [`Passes::WhereMet`], after the read under
    /// [`Passes::AfterRead`] ([`finish`](Self::finish)). A finding that rejects the document
    /// fails the read when it is told.
    pub fn defer(&mut self, pass: Pass, finding: Diagnostic) {
        if self.stopped {
            return;
        }
        if self.profile.defers() {
            self.deferred.push((pass, finding));
        } else {
            self.push(finding);
        }
    }

    /// Ends the read: under [`Passes::AfterRead`], tells what the passes after the read found,
    /// unless the read stopped (its failure is then all it tells), and only the [`Pass::Null`]
    /// errors when there are any. Nothing under [`Passes::WhereMet`], which has told everything
    /// already.
    pub fn finish(&mut self) {
        let deferred = std::mem::take(&mut self.deferred);
        if self.stopped || !self.profile.defers() {
            return;
        }
        // A read that went on past its first failure tells a refused `null` and a converted
        // scalar of a document that read alone: the old passes ran on what read.
        let past_failure = self.profile.failures == Failures::FirstOnly && self.failed;
        let nulls = !past_failure && deferred.iter().any(|(pass, _)| *pass == Pass::Null);
        let mut deferred = deferred;
        // Stable: within a pass, in the order the read met them.
        deferred.sort_by_key(|(pass, _)| *pass);
        for (pass, finding) in deferred {
            if nulls && pass != Pass::Null {
                break;
            }
            if past_failure && matches!(pass, Pass::Null | Pass::Misread) {
                continue;
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
    /// under [`Failures::Every`], an error that fails the owner under [`Failures::First`].
    pub fn member_failure(&self) -> crate::schema::Grading {
        if self.profile.first_alone() {
            crate::schema::grade(crate::versioning::MemberFailure::Refuse)
        } else {
            crate::schema::grade(self.reading.on_member_failure)
        }
    }

    /// The code of an enum name that is none of the enum's values: the family's
    /// ([`Texts::invalid_enum`]), else [`INVALID_ENUM_VALUE`](core_codes::INVALID_ENUM_VALUE).
    // the derived reads' protocol
    pub(crate) fn invalid_enum_code(&self) -> Code {
        self.texts
            .invalid_enum
            .map_or_else(|| Code::from(&core_codes::INVALID_ENUM_VALUE), Code::from)
    }

    /// Whether the last scalar read was a member read as absent ([`Passes::AfterRead`] under
    /// `DefaultAndTell`), clearing it: its owner takes it as absent.
    // the derived reads' protocol with the scalar reads
    pub(crate) fn take_defaulted(&mut self) -> bool {
        std::mem::take(&mut self.defaulted)
    }

    /// The `schema::unknown` of the field being read, set by its owner around the read of
    /// its value.
    // the derived reads' protocol
    pub(crate) fn field_unknown(&self) -> Option<crate::schema::UnknownRule> {
        self.field_unknown
    }

    /// Sets [`field_unknown`](Self::field_unknown), returning the one it replaces.
    // the derived reads' protocol
    pub(crate) fn set_field_unknown(
        &mut self,
        rule: Option<crate::schema::UnknownRule>,
    ) -> Option<crate::schema::UnknownRule> {
        std::mem::replace(&mut self.field_unknown, rule)
    }

    /// Runs `read` inside a map value or a box, where an unknown member is told and never
    /// kept (the old walk could not reach it to keep it).
    // the derived reads' protocol
    pub(crate) fn unkeepable<T>(&mut self, read: impl FnOnce(&mut Self) -> T) -> T {
        self.unkeepable += 1;
        let out = read(self);
        self.unkeepable -= 1;
        out
    }

    /// The rules of the field whose member `node` is read (`schema::wrong_type` ..) for that
    /// member, returning the ones they replace ([`restore_member_rules`](Self::restore_member_rules)).
    pub(crate) fn set_member_rules(
        &mut self,
        node: &Node<'_>,
        rules: &'static crate::schema::derive_support::MemberRules,
    ) -> Option<(usize, &'static crate::schema::derive_support::MemberRules)> {
        self.member.replace((address(node.value), rules))
    }

    /// Puts back the rules [`set_member_rules`](Self::set_member_rules) replaced.
    pub(crate) fn restore_member_rules(
        &mut self,
        saved: Option<(usize, &'static crate::schema::derive_support::MemberRules)>,
    ) {
        self.member = saved;
    }

    /// The rules of the field whose member `node` is, when the read is at that member.
    pub(crate) fn member_rules(
        &self,
        node: &Node<'_>,
    ) -> Option<&'static crate::schema::derive_support::MemberRules> {
        self.member
            .filter(|(at, _)| *at == address(node.value))
            .map(|(_, rules)| rules)
    }

    /// Runs `read` on a map's value (`true`) or a struct's member (`false`), restoring what
    /// was read before after it.
    // the derived reads' protocol
    pub(crate) fn in_map_value<T>(
        &mut self,
        map_value: bool,
        read: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let saved = std::mem::replace(&mut self.map_value, map_value);
        let out = read(self);
        self.map_value = saved;
        out
    }

    /// Whether an unknown member read here can be kept ([`unkeepable`](Self::unkeepable)).
    // the derived reads' protocol
    pub(crate) fn keepable(&self) -> bool {
        self.unkeepable == 0
    }

    /// The read taking a Molang number beyond `f32` as the largest finite one of its sign, told
    /// with an information at the number: a game whose schema takes any JSON number for an
    /// expression and holds it as a float.
    #[must_use]
    pub fn clamp_molang(mut self) -> Self {
        self.clamp_molang = true;
        self
    }

    /// Whether the read clamps a Molang number beyond `f32` ([`clamp_molang`](Self::clamp_molang)).
    pub fn clamps_molang(&self) -> bool {
        self.clamp_molang
    }

    /// The read taking a number that no double holds, in a value kept whole (`json::Value`), as
    /// the largest finite double of its sign, told with an information at the number: a game
    /// whose value is written back and must be finite.
    #[must_use]
    pub fn clamp_kept(mut self) -> Self {
        self.clamp_kept = true;
        self
    }

    /// Whether the read clamps the numbers of the values it keeps ([`clamp_kept`](Self::clamp_kept)).
    pub fn clamps_kept(&self) -> bool {
        self.clamp_kept
    }

    /// Marks `node` as the root of the read: [`Recovery::TopLevel`]'s members and elements
    /// are one step below it.
    // the root read's protocol
    pub(crate) fn set_root(&mut self, node: &Node<'_>) {
        self.root = node.path.len();
    }

    /// Whether a failed member at a path of `depth` steps takes its default: the reading's
    /// grading ([`member_failure`](Self::member_failure)), and under [`Recovery::TopLevel`]
    /// only a member of the root.
    // the derived reads' protocol
    pub(crate) fn resets_at(&self, depth: usize) -> bool {
        self.member_failure().resets
            && (self.profile.recovery != Recovery::TopLevel || depth == self.root + 1)
    }

    /// Whether the reading defaults a failed value (`DefaultAndTell`), which the read does not
    /// stop at.
    fn defaults(&self) -> bool {
        self.member_failure().resets
    }

    /// Whether a failed element of the list or a failed value of the map at `node` is left out
    /// under `DefaultAndTell`: under [`Recovery::Value`], and under [`Recovery::TopLevel`] for
    /// a list that is a member of the root.
    // the containers' protocol
    pub(crate) fn defaults_drop(&self, node: &Node<'_>, list: bool) -> bool {
        self.defaults()
            && match self.profile.recovery {
                Recovery::Member => false,
                Recovery::Value => true,
                Recovery::TopLevel => list && node.path.len() == self.root + 1,
            }
    }

    /// The member or element of the root `path` is in ([`Recovery::TopLevel`]): the root's
    /// member, or the element of a list there; `None` for the root itself.
    fn unit_of(&self, path: &JsonPath) -> Option<JsonPath> {
        let steps: Vec<JsonPathSegment<'_>> = path.segments().collect();
        let member = steps.get(self.root)?;
        let mut unit = JsonPath::new();
        for step in &steps[..self.root] {
            push_step(&mut unit, *step);
        }
        push_step(&mut unit, *member);
        if let Some(JsonPathSegment::Index(index)) = steps.get(self.root + 1) {
            unit.push_index(*index);
        }
        Some(unit)
    }

    /// The read with `texts` for its findings.
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Node, Read};
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

    /// The read of a document written at the format `format`: a member a struct reads from a
    /// later format on (`schema::since_format`) is an unknown member; a read that knows none
    /// reads every member. `versioning::read_payload` gives the document's.
    #[must_use]
    pub fn format(mut self, format: Option<crate::version::FormatVersion>) -> Self {
        self.format = format;
        self
    }

    /// The format version the read knows the document is written at ([`format`](Self::format)).
    pub fn known_format(&self) -> Option<crate::version::FormatVersion> {
        self.format
    }

    /// The read of a document loaded in `load`, which a part read by another reader (a
    /// component map's rows, `versioning::read_component_map_by`) is read in too.
    /// `versioning::read_payload` gives the slice's.
    #[must_use]
    pub fn load(mut self, load: &'d crate::load::LoadContext) -> Self {
        self.load = Some(load);
        self
    }

    /// The load the document is read in, when the read knows it ([`load`](Self::load)).
    pub fn load_context(&self) -> Option<&'d crate::load::LoadContext> {
        self.load
    }

    /// The read with `null` for its null handling instead of the reading's: a reader the game
    /// runs on one reading for part of a slice (a legacy parser below a version) gives it here.
    #[must_use]
    pub fn null(mut self, null: NullHandling) -> Self {
        self.reading = self.reading.with_null(null);
        self
    }

    /// The read with `scalars` for its scalar rule instead of the reading's: a part of a slice
    /// the game reads by another rule (members of a legacy block document that were always read
    /// strictly, beside its components, which its non-strict reader reads).
    #[must_use]
    pub fn scalars(mut self, scalars: crate::versioning::Scalars) -> Self {
        self.reading = self.reading.with_scalars(scalars);
        self
    }

    /// The read with `keeps_unknown` for whether a struct that says nothing keeps its unknown
    /// members instead of the reading's: a family's typed reads keep them for write-back where
    /// its slices' reading (the one its reading table prints) keeps none.
    #[must_use]
    pub fn keeps_unknown(mut self, keeps_unknown: bool) -> Self {
        self.reading = self.reading.with_keeps_unknown(keeps_unknown);
        self
    }

    /// The read telling unknown members at `severity` instead of the reading's: a reader the
    /// game runs without a word about them is told as information.
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Node, Read};
    /// # use bedrock_addon_core::schema::UnknownRule;
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
    /// read.unknown(&members, &[], UnknownRule::Keep);
    /// assert_eq!(told(&found), ["Info zz unrecognized field"]);
    /// ```
    #[must_use]
    pub fn unknown_severity(mut self, severity: Severity) -> Self {
        self.unknown_severity = severity;
        self
    }

    /// The read taking its scalars as the game's non-strict reader does
    /// ([`Scalars::NonStrict`]), whatever the reading says, through [`scalar`](Self::scalar) (the
    /// typed reads): a number where a bool is bound is read as `value != 0`, a bool where a
    /// number is bound as 1 or 0. Each such scalar, a token
    /// read as the other JSON type, is told as the information `ValueRewritten` with `note` (a
    /// write-back writes the member's own type there, so nothing is changed in silence).
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Node, Read};
    /// # use bedrock_addon_core::json::{JsonSlice, Value, from_str_jsonc};
    /// # use bedrock_addon_core::versioning::Reading;
    /// # fn told(d: &Diagnostics) -> Vec<String> {
    /// #     d.iter().map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code)).collect()
    /// # }
    /// use bedrock_addon_core::json::Declared;
    /// use bedrock_addon_core::json::kit::Scalar;
    /// let json: Value = from_str_jsonc(r#"{"lit": 2, "level": true}"#).unwrap();
    /// let mut found = Diagnostics::default();
    /// let mut read = Read::new(Reading::DOCUMENT, &mut found).non_strict("read as the bound type");
    /// let members = read.object(&Node::new(&json, JsonPath::new())).unwrap();
    /// let lit = read.required(&members, "lit").unwrap();
    /// assert_eq!(read.scalar(&lit, Declared::Bool), Some(Scalar::Bool(true)));
    /// let level = read.required(&members, "level").unwrap();
    /// assert_eq!(read.scalar(&level, Declared::Float { bits: 32 }), Some(Scalar::Real(1.0)));
    /// assert_eq!(told(&found).len(), 2);
    /// ```
    #[must_use]
    pub fn non_strict(mut self, note: &'static str) -> Self {
        self.reading = self.reading.with_scalars(Scalars::NonStrict);
        self.rewrite_note = Some(note);
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

    /// Sets whether what is read has failed: a list that leaves a failed element out is as
    /// failed as it was before the element.
    pub(crate) fn set_failed(&mut self, failed: bool) {
        self.failed = failed;
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
    /// # use bedrock_addon_core::json::kit::{Node, Read};
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
    pub fn push(&mut self, mut finding: Diagnostic) {
        if let Some(phase) = self.phase {
            finding.phase = phase;
        }
        if self.profile.spans == SpanRule::Withheld {
            finding.location.span = None;
        }
        let rejects = finding.rejects();
        self.failed |= rejects;
        self.errors += usize::from(rejects);
        self.findings.get().push(finding);
    }

    /// What the read has told so far, to ask afterwards what was told since
    /// ([`raised_since`](Self::raised_since), [`left_out_since`](Self::left_out_since)).
    pub(crate) fn mark(&self) -> Mark {
        Mark {
            errors: self.errors,
            told: self.findings.len() + self.deferred.len(),
        }
    }

    /// Whether a finding that rejects the document was told since `mark`.
    pub(crate) fn raised_since(&self, mark: Mark) -> bool {
        self.errors > mark.errors
    }

    /// Whether a value that failed, read since `mark`, is left out of its list or map rather
    /// than failing it: under the game's schema tree ([`Objects::Tree`]), a value that failed
    /// without raising an error (a lenient reading's information).
    ///
    /// A value that fails tells why, so a failure that told nothing is a reader that returned
    /// `None` silently ([`FromJson::from_json`](crate::schema::FromJson::from_json) says it
    /// must tell): the tree would leave the member out and read on. That is a defect of the
    /// reader, caught here in a debug build.
    pub(crate) fn left_out_since(&self, mark: Mark) -> bool {
        if self.profile.objects != Objects::Tree || self.raised_since(mark) {
            return false;
        }
        debug_assert!(
            self.findings.len() + self.deferred.len() > mark.told,
            "a value failed under Objects::Tree and told nothing: a reader returns None only \
             after it has told a finding"
        );
        true
    }

    /// The members read so far as a declared name spelled in another case
    /// ([`Names::IgnoreAsciiCase`]), each at its path and spanning its value, in the order read;
    /// the read forgets them. The game reads them silently: a family that tells them does so
    /// with its own finding.
    pub fn take_respelled(&mut self) -> Vec<(JsonPath, Option<Span>)> {
        std::mem::take(&mut self.respelled)
    }

    /// Notes the member `node` as read under a declared name spelled in another case.
    pub(crate) fn respelled(&mut self, node: &Node<'_>) {
        self.respelled.push((node.path.clone(), node.span()));
    }

    /// The members of `members` in the order the profile reads them: the file's, or the sorted
    /// order of their keys ([`MemberOrder::Sorted`], [`MemberOrder::SortedInPlace`]).
    pub fn ordered<'m, 'v>(&self, members: &'m Members<'v>) -> Vec<(&'v str, Node<'v>)> {
        let mut out: Vec<(&'v str, Node<'v>)> = members.iter().collect();
        if matches!(
            self.profile.members,
            MemberOrder::Sorted | MemberOrder::SortedInPlace
        ) {
            out.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
        }
        out
    }

    /// Tells `code` at `path` and `span`, graded by the reading's
    /// [`on_member_failure`](Reading::on_member_failure) ([`member_failure`](Self::member_failure)):
    /// a failed member. Under [`Failures::First`] the read stops there; a read that has stopped
    /// tells nothing more.
    fn fail(&mut self, code: Code, path: JsonPath, span: Option<Span>) {
        if self.stopped {
            return;
        }
        let grading = self.member_failure();
        self.failed |= grading.fails_owner;
        if self.silent_failure() {
            return;
        }
        let code = match self.texts.defaulted {
            Some(defaulted) if grading.resets => Code::from(defaulted),
            _ => code,
        };
        let dropped = if grading.resets && self.profile.recovery == Recovery::TopLevel {
            let unit = self.unit_of(&path);
            if unit.is_some() && unit == self.told_unit {
                // A failure of the member or element whose first failure is told.
                return;
            }
            self.told_unit.clone_from(&unit);
            unit.filter(|unit| unit.len() < path.len())
        } else {
            None
        };
        let mut finding = Diagnostic::error(code).severity(grading.severity).at(path);
        finding.location.span = span;
        self.push(finding);
        if let Some(unit) = dropped {
            // The member or element goes whole for the value inside it: told at it too.
            self.push(Diagnostic::of(&core_codes::DROPPED_FIELD).at(unit));
        }
        self.stopped = self.profile.stops();
    }

    /// Whether a failure is the one a [`Failures::FirstOnly`] read already told: it fails its
    /// owner and is not told. Marks the first.
    fn silent_failure(&mut self) -> bool {
        if self.profile.failures != Failures::FirstOnly {
            return false;
        }
        std::mem::replace(&mut self.told_first, true)
    }

    /// Tells that `node` holds a value its member refuses, with `code` (`INVALID_VALUE`, an
    /// enum's `INVALID_ENUM_VALUE`, a leaf's own), graded as every failed member is
    /// ([`member_failure`](Self::member_failure)).
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Node, Read};
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
        let code = self.invalid_code(node).unwrap_or(code);
        self.fail(code, node.path.clone(), node.span());
    }

    /// Tells that `node` is a value its enum has no variant for, under `code` (the enum's
    /// `schema::invalid`), graded as every failed member is: as facet-json told a variant
    /// lookup, at the member the value belongs to (a list's elements at the list's path), over
    /// the value.
    // the derived reads' protocol
    pub(crate) fn refuse_variant(&mut self, node: &Node<'_>, code: &'static CodeDef<0>) {
        let mut at = node.path.clone();
        while at
            .segments()
            .last()
            .is_some_and(|step| matches!(step, JsonPathSegment::Index(_)))
        {
            at.pop();
        }
        self.fail(Code::from(code), at, node.span());
    }

    /// Tells `finding` as a failed member, at its own path, span and code, graded as
    /// [`refuse`](Self::refuse) grades: for a reader that builds the finding itself (a leaf's
    /// own detail).
    pub fn refuse_with(&mut self, finding: Diagnostic) {
        if self.stopped {
            return;
        }
        if self.defaults() {
            let span = finding.location.span;
            return self.fail(finding.code, finding.location.json_path, span);
        }
        let grading = self.member_failure();
        let finding = finding.severity(grading.severity);
        self.failed |= grading.fails_owner;
        if self.silent_failure() {
            return;
        }
        self.push(finding);
        self.stopped = self.profile.stops();
    }

    /// The code a wrong type at `node` is told under where a rule words it: the field's own
    /// `schema::wrong_type` (at the member itself), else the family's wording by where it is
    /// told ([`Texts::wrong_at`]). One word for every wrong type of the member, a refused `null`
    /// included.
    pub(crate) fn worded_wrong_type(&self, node: &Node<'_>) -> Option<Code> {
        if let Some(code) = self
            .member_rules(node)
            .and_then(|rules| rules.wrong_type)
            .and_then(|code| self.member_code(code))
        {
            return Some(code);
        }
        let WrongAt(word) = self.texts.wrong_at?;
        Some(word(&node.path))
    }

    /// Tells that `node` holds a value of a JSON type its member does not take.
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Node, Read};
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
        let code = self
            .worded_wrong_type(node)
            .unwrap_or_else(|| Code::from(&core_codes::INVALID_TYPE));
        self.fail(code, node.path.clone(), node.span());
    }

    /// Tells that `node` holds a value of a JSON type its member does not take, where the read
    /// knows what it `expected`: in the family's [`Texts::wrong_kind`] wording when it has one,
    /// else as [`wrong_type`](Self::wrong_type) words it.
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Node, Read};
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
        if self
            .member_rules(node)
            .and_then(|rules| rules.wrong_type)
            .and_then(|code| self.member_code(code))
            .is_some()
        {
            return self.wrong_type(node);
        }
        match self.texts.wrong_kind {
            Some(WrongKind(word)) => {
                let code = word(expected, node.value);
                self.fail(code, node.path.clone(), node.span());
            }
            None => self.wrong_accepting(node, expected.word()),
        }
    }

    /// Tells that `node` holds a value of a JSON type its member does not take, where the
    /// member's type says what it `accepts` in the schema's words (`"bool, string"`): in the
    /// family's [`Texts::accepts`] wording when it has one, else as
    /// [`wrong_type`](Self::wrong_type) words it.
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Node, Read, Texts};
    /// # use bedrock_addon_core::json::{Value, from_str_jsonc};
    /// # use bedrock_addon_core::versioning::Reading;
    /// let json: Value = from_str_jsonc("[]").unwrap();
    /// let mut found = Diagnostics::default();
    /// let mut read = Read::new(Reading::DOCUMENT, &mut found).texts(Texts::TREE);
    /// read.wrong_accepting(&Node::new(&json, JsonPath::of_keys(["loop"])), "bool, string");
    /// let told = found.iter().next().unwrap();
    /// assert_eq!(told.code.id(), "core/unknown_child_option");
    /// ```
    pub fn wrong_accepting(&mut self, node: &Node<'_>, accepts: &str) {
        match self.texts.accepts {
            Some(code) if self.worded_wrong_type(node).is_none() => {
                self.fail(code.with([accepts]), node.path.clone(), node.span());
            }
            _ => self.wrong_type(node),
        }
    }

    /// The members of `node`; a [wrong type](Self::wrong_type) when it is no object.
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Node, Read};
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
        let texts = match node.text {
            Text::None => None,
            Text::Tree { node, .. } => Some(node),
        };
        Some(Members {
            node: node.clone(),
            object,
            texts,
            by_key: OnceLock::new(),
        })
    }

    /// The elements of `node`; a [wrong type](Self::wrong_type) when it is no array.
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Node, Read};
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
        let texts = match node.text {
            Text::None => None,
            Text::Tree { node, .. } => Some(node),
        };
        Some(Elements {
            node: node.clone(),
            array,
            texts,
        })
    }

    /// The member `key` of `members`, when it is written; nothing is told. A `null` member is
    /// absent under [`NullHandling::Absent`] and [`NullHandling::AbsentSilently`].
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Node, Read};
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
        let absent = node.value.is_null()
            && matches!(
                self.reading.null,
                NullHandling::Absent | NullHandling::AbsentSilently
            );
        (!absent).then_some(node)
    }

    /// The member `key` of `members`; when it is not written (or is `null` under
    /// [`NullHandling::Absent`]), a missing required member at it, spanning the object.
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Node, Read};
    /// # use bedrock_addon_core::json::{JsonSlice, Value, from_str_jsonc};
    /// # use bedrock_addon_core::versioning::Reading;
    /// # fn told(d: &Diagnostics) -> Vec<String> {
    /// #     d.iter().map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code)).collect()
    /// # }
    /// let text = r#"{"pack_id": "x"}"#;
    /// let parsed = bedrock_addon_core::json::parse::parse(text, Default::default()).unwrap();
    /// let root = Node::parsed(&parsed, JsonPath::of_keys(["entry"]));
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

    /// Tells that the member `key` of `members` is required and missing, with `code` (a
    /// field's `schema::missing`).
    pub(crate) fn missing_member_as(&mut self, members: &Members<'_>, key: &str, code: Code) {
        self.tell_missing(code, self.missing_at(members, key), members.node.span());
    }

    /// Tells a missing member: a failed member, or under the game's schema tree an error
    /// whatever the reading grades (its required check always fails the object).
    fn tell_missing(&mut self, code: Code, path: JsonPath, span: Option<Span>) {
        if self.profile.objects == Objects::Tree {
            self.reject(&path, span, code);
        } else {
            self.fail(code, path, span);
        }
    }

    /// Tells `code` at `node` as an error, whatever the reading grades a failed member: a
    /// constraint the game's schema tree fails on in every reading, its lenient one included
    /// (which removes a member of the wrong type, and tells that as an information).
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath, Severity, codes};
    /// # use bedrock_addon_core::json::kit::{Node, Read};
    /// # use bedrock_addon_core::json::{Value, from_str_jsonc};
    /// # use bedrock_addon_core::versioning::{MemberFailure, Reading};
    /// let json: Value = from_str_jsonc("-1").unwrap();
    /// let mut found = Diagnostics::default();
    /// let lenient = Reading::OWN.with_on_member_failure(MemberFailure::DefaultAndTell);
    /// let mut read = Read::new(lenient, &mut found);
    /// read.error(&Node::new(&json, JsonPath::of_keys(["id"])), (&codes::INVALID_VALUE).into());
    /// assert_eq!(found.iter().next().unwrap().severity, Severity::Error);
    /// ```
    pub fn error(&mut self, node: &Node<'_>, code: Code) {
        self.reject(&node.path, node.span(), code);
    }

    fn reject(&mut self, path: &JsonPath, span: Option<Span>, code: Code) {
        if self.stopped {
            return;
        }
        let mut finding = Diagnostic::error(code).at(path.clone());
        finding.location.span = span;
        self.push(finding);
        self.stopped = self.profile.stops();
    }

    /// Where a missing member `key` of `members` is told: at it, or at the object that lacks it
    /// where the read words a defaulted failure as one ([`Texts::defaulted`]) or is a tree's
    /// ([`Objects::Tree`]).
    fn missing_at(&self, members: &Members<'_>, key: &str) -> JsonPath {
        if self.texts.missing_in_object
            || self.profile.objects == Objects::Tree
            || (self.texts.defaulted.is_some() && self.defaults())
        {
            members.node.path.clone()
        } else {
            members.node.path.child(key)
        }
    }

    /// Tells that the member `key` of `members` is required and missing.
    fn missing(&mut self, members: &Members<'_>, key: &str) {
        let code = match (self.texts.missing_named, self.texts.missing) {
            (Some(named), _) => named.with([key.to_owned()]),
            (None, Some(text)) => core_codes::MISSING_REQUIRED_FIELD_WORDED.with([text]),
            (None, None) => Code::from(&core_codes::MISSING_REQUIRED_FIELD),
        };
        self.tell_missing(code, self.missing_at(members, key), members.node.span());
    }

    /// Tells every member of `members` that `known` does not name, in file order, under
    /// `rule` at the unknown severity, spanning its value. Where the rule keeps
    /// ([`UnknownRule::Keep`], [`UnknownRule::Silent`]) the members come back, for the caller to
    /// keep; otherwise none does.
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Node, Read};
    /// # use bedrock_addon_core::schema::UnknownRule;
    /// # use bedrock_addon_core::json::{JsonSlice, Value, from_str_jsonc};
    /// # use bedrock_addon_core::versioning::Reading;
    /// # fn told(d: &Diagnostics) -> Vec<String> {
    /// #     d.iter().map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code)).collect()
    /// # }
    /// let json: Value = from_str_jsonc(r#"{"name": "a", "colour": 1, "note": "x"}"#).unwrap();
    /// let mut found = Diagnostics::default();
    /// let mut read = Read::new(Reading::DOCUMENT, &mut found);
    /// let members = read.object(&Node::new(&json, JsonPath::new())).unwrap();
    /// let kept = read.unknown(&members, &["name"], UnknownRule::Keep);
    /// let kept: Vec<&str> = kept.iter().map(|(key, _)| *key).collect();
    /// assert_eq!(kept, ["colour", "note"]);
    /// assert_eq!(told(&found), ["Error colour unrecognized field", "Error note unrecognized field"]);
    /// ```
    pub fn unknown<'v>(
        &mut self,
        members: &Members<'v>,
        known: &[&str],
        rule: UnknownRule,
    ) -> Vec<(&'v str, &'v Value)> {
        let mut kept = Vec::new();
        for (key, node) in members.iter() {
            if known.contains(&key) {
                continue;
            }
            self.unknown_member(&node, rule);
            if matches!(rule, UnknownRule::Keep | UnknownRule::Silent) {
                kept.push((key, node.value));
            }
        }
        kept
    }

    /// Tells the one member `node` as unknown under `rule`, at the unknown severity:
    /// for a reader that tells its unknown members in file order between the reads of the
    /// known ones. [`unknown`](Self::unknown) is this for every member `known` does not name.
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Node, Read};
    /// # use bedrock_addon_core::schema::UnknownRule;
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
    ///         _ => read.unknown_member(&node, UnknownRule::NotKept),
    ///     }
    /// }
    /// assert_eq!(told(&found), ["Error x member read and not kept", "Error identifier wrong JSON type"]);
    /// ```
    pub fn unknown_member(&mut self, node: &Node<'_>, rule: UnknownRule) {
        if rule == UnknownRule::Silent {
            return;
        }
        let code = rule.code(self.texts, node.path.last_key().unwrap_or_default());
        let mut finding = Diagnostic::of(code)
            .severity(self.unknown_severity)
            .at(node.path.clone());
        finding.location.span = node.span();
        self.push(finding);
    }

    /// Tells the one member `node` as unknown by its struct's word (`schema::unknown`), as
    /// facet-json told it: `keep` and `report` an `UnrecognizedField` (kept or not, its
    /// one line), `reject` a `DroppedField`, `ignore` an `IgnoredMember`, `silent` nothing; at
    /// the unknown severity, spanning its value, in [`Pass::Unknown`].
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Node, Read};
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
    pub fn unknown_by_rule(&mut self, node: &Node<'_>, rule: UnknownRule) {
        if rule == UnknownRule::Silent {
            return;
        }
        let code = rule.code(self.texts, node.path.last_key().unwrap_or_default());
        let mut finding = Diagnostic::of(code)
            .severity(self.unknown_severity)
            .at(node.path.clone());
        finding.location.span = node.span();
        self.defer(Pass::Unknown, finding);
    }

    /// Tells the member `node` of a range's object form, or its element past the two ends, that
    /// the range does not read, as [`Reading::range_members`] says: an unknown member of the
    /// word `report` (told, not kept), or a [`MEMBER_NOT_KEPT`](core_codes::MEMBER_NOT_KEPT) at
    /// the unknown severity in the last pass ([`Pass::Misread`]), spanning its value.
    pub(crate) fn unread_range_member(&mut self, node: &Node<'_>) {
        match self.reading.range_members {
            RangeMembers::Silent => {}
            RangeMembers::Unrecognized => {
                self.unknown_by_rule(node, UnknownRule::Report);
            }
            RangeMembers::NotKept => {
                let code =
                    UnknownRule::NotKept.code(self.texts, node.path.last_key().unwrap_or_default());
                let mut finding = Diagnostic::of(code)
                    .severity(self.unknown_severity)
                    .at(node.path.clone());
                finding.location.span = node.span();
                self.defer(Pass::Misread, finding);
            }
        }
    }

    /// The bool `node` holds; a [wrong type](Self::wrong_type) for any other value.
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Node, Read};
    /// # use bedrock_addon_core::json::{JsonSlice, Value, from_str_jsonc};
    /// # use bedrock_addon_core::versioning::Reading;
    /// # fn told(d: &Diagnostics) -> Vec<String> {
    /// #     d.iter().map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code)).collect()
    /// # }
    /// let json: Value = from_str_jsonc("[true, 1]").unwrap();
    /// let mut found = Diagnostics::default();
    /// let mut read = Read::new(Reading::OWN, &mut found);
    /// let elements = read.array(&Node::new(&json, JsonPath::new())).unwrap();
    /// let read: Vec<Option<bool>> = elements.iter().map(|n| read.bool(&n)).collect();
    /// // A number is no bool.
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
    /// # use bedrock_addon_core::json::kit::{Node, Read};
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

    /// The number `node` holds, as the double it spells. A [wrong type](Self::wrong_type) for
    /// any other value, a bool or a string included.
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Node, Read};
    /// # use bedrock_addon_core::json::{JsonSlice, Value, from_str_jsonc};
    /// # use bedrock_addon_core::versioning::Reading;
    /// # fn told(d: &Diagnostics) -> Vec<String> {
    /// #     d.iter().map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code)).collect()
    /// # }
    /// let json: Value = from_str_jsonc("[0.5, 2, true]").unwrap();
    /// let root = Node::new(&json, JsonPath::new());
    /// let mut found = Diagnostics::default();
    /// let mut read = Read::new(Reading::DOCUMENT, &mut found);
    /// let numbers: Vec<Option<f64>> = read.array(&root).unwrap().iter().map(|n| read.number(&n)).collect();
    /// assert_eq!(numbers, [Some(0.5), Some(2.0), None]);
    /// ```
    pub fn number(&mut self, node: &Node<'_>) -> Option<f64> {
        let read = node
            .value
            .as_number()
            .map(facet_value::VNumber::to_f64_lossy);
        if read.is_none() {
            self.wrong_kind(node, Expected::Number);
        }
        read
    }

    /// The `i32` `node` holds, under `ints` (jsoncpp's `isInt`, which the families disagree on,
    /// so each names its own): the number `isInt` takes. A [wrong type](Self::wrong_type) for
    /// any other value, a bool or a string included.
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Node, Read};
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
    /// ```
    pub fn int(&mut self, node: &Node<'_>, ints: Coercing) -> Option<i32> {
        let read = node.value.is_int(ints);
        if read.is_none() {
            self.wrong_kind(node, Expected::Number);
        }
        read
    }
}

/// `path` with `step` after it.
fn push_step(path: &mut JsonPath, step: JsonPathSegment<'_>) {
    match step {
        JsonPathSegment::Key(key) => path.push(key),
        JsonPathSegment::Index(index) => path.push_index(index),
    }
}

/// The address of a value, which names one node of a read (a transparent newtype or an
/// `Option` reads the same node; an element or a member is another).
fn address(value: &Value) -> usize {
    std::ptr::from_ref(value).addr()
}

/// The byte range of `slice` in its file.
fn span_of(slice: JsonSlice<'_>) -> Span {
    let narrow = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
    Span {
        offset: narrow(slice.offset),
        len: narrow(slice.text.len()),
    }
}
