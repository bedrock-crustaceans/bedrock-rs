//! The second table of `#[derive(Components)]`: per component row, the facts the version
//! chain's `ComponentMeta` does not carry (its typed version, its scalar shorthand, goal-ness
//! and the trigger members of an untyped row).

use crate::version::FormatVersion;

/// What a component written as a scalar is read as: `shorthand = scalar` on the row.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Shorthand {
    /// A bare number, bool or string stands for the component's one value.
    Scalar,
}

/// One row of a component enum's `SCHEMA_ROWS`, sorted by name like its `TABLE`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ComponentRow {
    /// The component's JSON key.
    pub name: &'static str,
    /// `typed = ".."`: the format version from which the game reads it with its typed struct.
    pub typed: Option<FormatVersion>,
    /// `shorthand = scalar`.
    pub shorthand: Option<Shorthand>,
    /// `goal`: the row is an AI goal.
    pub goal: bool,
    /// `triggers = [..]`: trigger members of a row whose payload is no typed struct.
    pub triggers: &'static [&'static str],
}

/// The row named `name` of `rows` (sorted by name), for the derived `schema_row`.
// hidden: emitted by the schema derives
#[doc(hidden)]
pub fn find_row(rows: &'static [ComponentRow], name: &str) -> Option<&'static ComponentRow> {
    rows.binary_search_by(|row| row.name.cmp(name))
        .ok()
        .map(|index| &rows[index])
}

/// The payloads of a component enum's `shorthand = scalar` rows, for
/// `versioning::parse_component`: implemented by `#[derive(Components)]` for every component
/// enum (a `from_only` one has no rows, and so none). A family's shape test reads it through
/// [`misshapen_shorthand_rows`].
pub trait ShorthandRows {
    /// `(key, payload shape)` of every typed row with `shorthand = scalar`, in declaration
    /// order.
    const SHORTHAND_PAYLOADS: &'static [(&'static str, &'static facet::Shape)] = &[];

    /// The key of the `shorthand = scalar` row whose payload is `payload`.
    fn shorthand_key(payload: &'static facet::Shape) -> Option<&'static str> {
        Self::SHORTHAND_PAYLOADS
            .iter()
            .find(|(_, shape)| shape.id == payload.id)
            .map(|(key, _)| *key)
    }
}

/// The shape of `T`, in a const: what `#[derive(Components)]` lists a payload by.
// hidden: emitted by the schema derives
#[doc(hidden)]
#[must_use]
pub const fn shape_of<T: facet::Facet<'static>>() -> &'static facet::Shape {
    T::SHAPE
}

/// How a `shorthand = scalar` row's payload reads a bare scalar.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ShorthandShape {
    /// A struct with one named member (`skip` fields aside): the bare scalar is that member's
    /// value, read by the door as the member (`versioning::parse_component`).
    OneField {
        /// The member's JSON key.
        key: &'static str,
    },
    /// An untagged enum: one of its arms reads the scalar itself.
    Untagged,
    /// A `#[facet(transparent)]` newtype: it reads as its one field, the scalar itself.
    Transparent,
}

/// What a bare scalar of a payload of `shape` is read as; `None` for a shape no bare scalar
/// stands for (a struct of several members, an enum with a tag).
#[must_use]
pub fn shorthand_shape(shape: &'static facet::Shape) -> Option<ShorthandShape> {
    use facet::{StructKind, Type, UserType};
    if shape.is_untagged() {
        return Some(ShorthandShape::Untagged);
    }
    let Type::User(UserType::Struct(st)) = shape.ty else {
        return None;
    };
    if shape.is_transparent() {
        return Some(ShorthandShape::Transparent);
    }
    if st.kind != StructKind::Struct {
        return None;
    }
    let mut read = st.fields.iter().filter(|f| !f.should_skip_deserializing());
    match (read.next(), read.next()) {
        (Some(field), None) if !field.is_flattened() => Some(ShorthandShape::OneField {
            key: field.effective_name(),
        }),
        _ => None,
    }
}

/// The keys of `C`'s `shorthand = scalar` rows whose payload no bare scalar stands for
/// ([`shorthand_shape`] is `None`): what a family's shape test holds empty.
#[must_use]
pub fn misshapen_shorthand_rows<C: ShorthandRows>() -> Vec<&'static str> {
    C::SHORTHAND_PAYLOADS
        .iter()
        .filter(|(_, shape)| shorthand_shape(shape).is_none())
        .map(|(key, _)| *key)
        .collect()
}
