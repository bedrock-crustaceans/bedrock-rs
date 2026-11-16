//! [`FromMembers`]: a field fed by several members of its struct's object
//! (`#[facet(schema::members = ["a", "b"])]`), where the game reads one value from more than
//! one key: a literal or the name of where to find it (`block_pos` or `block_pos_id`), one list
//! under three spellings (`states`, `+states`, `-states`), a struct built from members spread
//! over its owner.

use crate::json::kit::{Node, Read};

/// A value read from `N` members of the object of the struct that holds it, named by the field's
/// `schema::members = [..]` in that order.
///
/// The derived read calls it once, at the field's place, with the struct's object: in a read in declaration order where
/// the field is declared, in file order where the file has the first of its members (after the
/// members when it has none). It is called whether or not any of them is written, with each
/// member as written (`None` for one the object lacks, a `null` as a value) and `default`, the
/// field's value when the document writes none of them (its `default`, `None` for an `Option`
/// field); none of them is an unknown member. The value says what it read (which spelling,
/// which key) and how two members written together merge; `None` fails the struct (told).
///
/// A field whose value has no such impl, or whose struct says it its own way, reads with
/// `schema::read_with = path`, `fn(&Node<'_>, &[Option<Node<'_>>; N], T, &mut Read<'_>) ->
/// Option<T>`.
///
/// ```
/// use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
/// use bedrock_addon_core::json::kit::{Node, Read};
/// use bedrock_addon_core::json::{Value, from_str_jsonc};
/// use bedrock_addon_core::schema::{self, FromJson, FromMembers, Schema};
/// use bedrock_addon_core::versioning::Reading;
///
/// /// A count, or the name of the entry that holds it.
/// #[derive(Debug, Default, PartialEq, facet::Facet)]
/// struct Count {
///     count: u32,
///     entry: String,
/// }
/// bedrock_addon_core::impl_schema_leaf!(Count);
/// bedrock_addon_core::impl_no_molang!(Count);
///
/// impl FromMembers<2> for Count {
///     fn from_members(
///         _: &Node<'_>,
///         members: &[Option<Node<'_>>; 2],
///         default: Self,
///         read: &mut Read<'_>,
///     ) -> Option<Self> {
///         let [count, entry] = members;
///         // The count wins over the entry.
///         if let Some(count) = count {
///             return Some(Count { count: u32::from_json(count, read)?, entry: String::new() });
///         }
///         let Some(entry) = entry else { return Some(default) };
///         Some(Count { entry: read.string(entry)?.to_owned(), ..default })
///     }
/// }
///
/// #[derive(Debug, PartialEq, facet::Facet, Schema)]
/// struct Repeat {
///     #[facet(schema::members = ["times", "times_id"], default = Count { count: 1, entry: "times".into() })]
///     times: Count,
/// }
///
/// let json: Value = from_str_jsonc(r#"{"times_id": "laps"}"#).unwrap();
/// let mut found = Diagnostics::default();
/// let mut read = Read::new(Reading::DOCUMENT, &mut found);
/// let repeat = Repeat::from_json(&Node::new(&json, JsonPath::new()), &mut read).unwrap();
/// assert_eq!(repeat.times, Count { count: 1, entry: "laps".to_owned() });
/// assert!(found.iter().next().is_none());
/// ```
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not read from {N} members (`FromMembers<{N}>`)",
    label = "fed by {N} members here",
    note = "implement `FromMembers<{N}>` beside the type, or read the field with `schema::read_with`"
)]
pub trait FromMembers<const N: usize>: Sized {
    /// Reads the value from `members` of `object` (in the order the field names them, `None`
    /// for one not written), `default` the field's value when none is; `None` when it cannot be
    /// read (told).
    fn from_members(
        object: &Node<'_>,
        members: &[Option<Node<'_>>; N],
        default: Self,
        read: &mut Read<'_>,
    ) -> Option<Self>;
}
