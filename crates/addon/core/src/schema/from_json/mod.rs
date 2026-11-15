//! [`FromJson`]: the one reader of a typed document, member by member on the reader kit
//! ([`json::kit`](crate::json::kit)), and its impls for the scalars and containers.
//!
//! A document type is `#[derive(Facet, Schema)]`; the `Schema` derive emits its `FromJson`,
//! which reads every field through its type's `FromJson` under the read's
//! [`Reading`](crate::versioning::Reading), so a failed member is told and the rest is read. A
//! leaf type with its own syntax implements it by hand, next to the type (`leaves`); a facet type
//! not converted yet reads through the bridge to the old facet read
//! ([`from_json_via_facet!`](crate::from_json_via_facet)).
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

mod bridge;
// hidden: emitted by the schema derive
#[doc(hidden)]
pub mod derived;
mod impls;
mod leaves;

#[cfg(test)]
mod tests;

pub use bridge::read_via_facet;
pub use impls::MapKey;
pub use leaves::from_value;

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
pub trait FromJson: Sized {
    /// Reads the value at `node`, telling every finding to `read`; `None` when the value could
    /// not be read (already told).
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self>;

    /// What JSON the type takes, in the words of a wrong type ([`Texts::wrong_kind`]): where
    /// an `Option` of it refuses a `null`, it names what it expected. `None` for a type of
    /// several JSON kinds; the derive says `Object` for a struct.
    ///
    /// [`Texts::wrong_kind`]: crate::json::kit::Texts::wrong_kind
    const EXPECTED: Option<Expected> = None;

    /// Whether the type holds a `null` as written (`json::Value`): an `Option` of it reads a
    /// `null` member as `None` without a word under every null rule, as the door does
    /// (vanilla's dynamic value holds a `null`).
    #[doc(hidden)]
    const HOLDS_NULL: bool = false;
}
