//! [`FromJson`]: the one reader of a typed document, member by member on the reader kit
//! ([`json::kit`](crate::json::kit)), and its impls for the scalars and containers.
//!
//! A document type is `#[derive(Facet, Schema)]`; the `Schema` derive emits its `FromJson`,
//! which reads every field through its type's `FromJson` under the read's
//! [`Reading`](crate::versioning::Reading), so a failed member is told and the rest is read. A
//! leaf type with its own syntax implements it by hand, next to the type (`leaves`).
//!
//! ```
//! use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
//! use bedrock_addon_core::json::kit::{Node, Read};
//! use bedrock_addon_core::json::{Value, from_str_jsonc};
//! use bedrock_addon_core::schema::FromJson;
//! use bedrock_addon_core::versioning::Reading;
//!
//! let json: Value = from_str_jsonc(r#"[1, "two", 3]"#).unwrap();
//! let mut found = Diagnostics::default();
//! let mut read = Read::new(Reading::DOCUMENT, &mut found);
//! let list = Vec::<u8>::from_json(&Node::new(&json, JsonPath::of_keys(["counts"])), &mut read);
//! // The element that is no number is told; the list, which needs every element, fails.
//! assert_eq!(list, None);
//! let told: Vec<String> = found.iter().map(|f| format!("{} {}", f.location.json_path, f.code)).collect();
//! assert_eq!(told, ["counts/1 wrong JSON type"]);
//! ```

// hidden: emitted by the schema derive
#[doc(hidden)]
pub mod derived;
mod impls;
mod keys;
mod leaves;
mod members;

#[cfg(test)]
mod tests;

pub use impls::{
    ElementFailure, MapKey, read_each, read_entries, read_iterated, read_iterated_silently,
    read_list,
};
pub use keys::{KeyPattern, Keys, read_keyed};
pub use leaves::{from_value, from_value_ref, read_later};
pub use members::FromMembers;

use crate::json::kit::{Expected, Node, Read};

/// A type read from JSON member by member: `#[derive(Schema)]` implements it for a struct, a
/// transparent newtype and an enum; core for the scalars, `json::Value` and the containers; a
/// leaf type with a syntax of its own by hand, next to the type.
///
/// `from_json` reads the value at `node`, telling every finding to `read` (at the node's path,
/// spanning its text when the node has one), and is `None` when the value could not be read,
/// which it has told. It tells under the read's reading and order: what a failed member does
/// is [`Read::member_failure`]; it runs no bound (`Schema::check` stays the bounds pass, after
/// the read).
///
/// A hand impl reads with the kit ([`Read::object`], [`Read::array`], [`Read::scalar`], ...),
/// tells a value its type refuses with [`Read::refuse`] or [`Read::wrong_type`], and starts
/// with nothing when the read has [stopped](Read::stopped):
///
/// ```
/// use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath, codes};
/// use bedrock_addon_core::json::kit::{Node, Read};
/// use bedrock_addon_core::json::{Value, from_str_jsonc};
/// use bedrock_addon_core::schema::FromJson;
/// use bedrock_addon_core::versioning::Reading;
///
/// /// A colour written `"#rrggbb"`.
/// struct Rgb(u32);
///
/// impl FromJson for Rgb {
///     fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
///         let text = read.string(node)?;
///         let parsed = text.strip_prefix('#').and_then(|hex| u32::from_str_radix(hex, 16).ok());
///         if parsed.is_none() {
///             read.refuse(node, (&codes::INVALID_VALUE).into());
///         }
///         parsed.map(Rgb)
///     }
/// }
///
/// let json: Value = from_str_jsonc(r#""teal""#).unwrap();
/// let mut found = Diagnostics::default();
/// let mut read = Read::new(Reading::DOCUMENT, &mut found);
/// assert!(Rgb::from_json(&Node::new(&json, JsonPath::of_keys(["color"])), &mut read).is_none());
/// assert_eq!(found.iter().next().unwrap().code.to_string(), "invalid value");
/// ```
#[diagnostic::on_unimplemented(
    message = "`{Self}` has no JSON read (`FromJson`)",
    label = "a field holds `{Self}`, which has no read",
    note = "derive `Schema` on it, write `impl FromJson` beside it, or read the field with `schema::read_with`"
)]
pub trait FromJson: Sized {
    /// Reads the value at `node`, telling every finding to `read`; `None` when the value could
    /// not be read (already told).
    ///
    /// A `None` tells a finding first, always. Under [`Objects::Tree`] a value that failed
    /// without an error is left out of its object and the read goes on, so a leaf that fails
    /// silently would lose a member without a word: a debug build panics on it.
    ///
    /// [`Objects::Tree`]: crate::json::kit::Objects::Tree
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self>;

    /// What JSON the type takes, in the words of a wrong type ([`Texts::wrong_kind`]): where
    /// an `Option` of it refuses a `null`, it names what it expected. `None` for a type of
    /// several JSON kinds; the derive says `Object` for a struct.
    ///
    /// [`Texts::wrong_kind`]: crate::json::kit::Texts::wrong_kind
    const EXPECTED: Option<Expected> = None;

    /// Whether the type holds a `null` as written (`json::Value`): an `Option` of it reads a
    /// `null` member as `None` without a word under every null rule, as facet-json did
    /// (vanilla's dynamic value holds a `null`).
    #[doc(hidden)]
    const HOLDS_NULL: bool = false;
}

/// Reads a whole document (or a component) at `node` as `T`, and ends the read
/// ([`Read::finish`]): the entry of every read of a root, where [`FromJson::from_json`] is the
/// read of one value inside it. The unknown members the read keeps go to the table of the
/// read's findings only when the value is read: a document that fails keeps none, as the
/// old typed read kept them only after it worked.
///
/// The read goes a few calls deeper for each level of the text, so it runs on the stack the
/// text's depth can take, as the old typed read did (`json::stack`): on the thread's own
/// where enough is left, else on one made for it. A text nests at most
/// [`MAX_JSON_DEPTH`](crate::json::MAX_JSON_DEPTH) deep; a deeper one is read on the stack
/// that depth takes.
///
/// ```
/// use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
/// use bedrock_addon_core::json::kit::{Node, Read};
/// use bedrock_addon_core::json::{Value, from_str_jsonc};
/// use bedrock_addon_core::schema::read_document;
/// use bedrock_addon_core::versioning::Reading;
///
/// let json: Value = from_str_jsonc("[[1], [2, 3]]").unwrap();
/// let mut found = Diagnostics::default();
/// let mut read = Read::new(Reading::DOCUMENT, &mut found);
/// let lists: Option<Vec<Vec<u8>>> = read_document(&Node::new(&json, JsonPath::new()), &mut read);
/// assert_eq!(lists, Some(vec![vec![1], vec![2, 3]]));
/// ```
pub fn read_document<T: FromJson>(node: &Node<'_>, read: &mut Read<'_>) -> Option<T> {
    let levels = crate::json::levels_of(node.value()).unwrap_or(crate::json::MAX_JSON_DEPTH);
    // The table the read keeps into is its own until the value is read.
    let before = read.findings_mut().take_unknown();
    read.set_root(node);
    let value = crate::json::with_stack_for(levels, || T::from_json(node, read));
    read.finish();
    let kept = std::mem::replace(read.findings_mut().unknown_mut(), before);
    if value.is_some() && !kept.is_empty() {
        read.findings_mut()
            .unknown_mut()
            .absorb(&crate::diagnostic::JsonPath::new(), kept);
    }
    value
}

/// A value held in memory read as `T`, in silence: what an upgrade step does with a `Value` it
/// built or kept as written, and wrote out as text to read the text (`to_json` and
/// `from_str`). `None` when it does not read: a member of the wrong type or a
/// missing required one, a number its type does not hold, a value nested deeper than the text
/// of a document may be ([`MAX_JSON_DEPTH`](crate::json::MAX_JSON_DEPTH)). Nothing is told, and
/// nothing is converted: scalars read strictly, `null` as the type's own rule, a member `T` does
/// not know is ignored.
///
/// ```
/// use bedrock_addon_core::json::{Value, from_str_jsonc};
/// use bedrock_addon_core::schema::read_value;
///
/// let json: Value = from_str_jsonc(r#"[1, 2, 3]"#).unwrap();
/// assert_eq!(read_value::<Vec<u8>>(&json), Some(vec![1, 2, 3]));
/// assert_eq!(read_value::<Vec<bool>>(&json), None);
/// ```
pub fn read_value<T: FromJson>(value: &crate::json::Value) -> Option<T> {
    use crate::diagnostic::{Diagnostics, JsonPath};
    use crate::versioning::Reading;
    crate::json::levels_of(value).ok()?;
    let mut told = Diagnostics::default();
    let reading = Reading::OWN.strict();
    let mut read = Read::new(reading, &mut told).profile(crate::json::kit::Profile::FIRST_FAILURE);
    read_document(&Node::new(value, JsonPath::new()), &mut read)
}
