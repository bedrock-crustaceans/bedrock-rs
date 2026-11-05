//! [`OneOrMany`]: a member that takes one value or an array of them, kept in the form it was
//! written.

use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::schema::{SchemaCx, SchemaFields};

/// One value, or an array of them, as written: a member that takes either keeps the form, so
/// writing reproduces it.
///
/// JSON (facet's untagged read, for a family that reads it through facet): the value itself,
/// or an array of values. A family with its own reader keeps it: what a one form may be (an
/// atlas takes any value but an array), whether an empty array is accepted and the findings of
/// a bad element are the family's.
#[derive(Clone, Debug, PartialEq, Eq, Hash, facet::Facet)]
#[facet(untagged)]
#[repr(u8)]
pub enum OneOrMany<T> {
    /// A single value, not in an array.
    One(T),
    /// An array.
    Many(Vec<T>),
}

impl<T> OneOrMany<T> {
    /// The values, in document order: one for [`One`](Self::One), all for
    /// [`Many`](Self::Many).
    pub fn as_slice(&self) -> &[T] {
        match self {
            Self::One(value) => std::slice::from_ref(value),
            Self::Many(values) => values,
        }
    }

    /// The values, mutably.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        match self {
            Self::One(value) => std::slice::from_mut(value),
            Self::Many(values) => values,
        }
    }

    /// The values, in document order.
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.as_slice().iter()
    }

    /// How many values.
    pub fn len(&self) -> usize {
        self.as_slice().len()
    }

    /// No value (an empty array).
    pub fn is_empty(&self) -> bool {
        self.as_slice().is_empty()
    }

    /// Whether the values were written as an array (one value in an array included).
    pub fn is_many(&self) -> bool {
        matches!(self, Self::Many(_))
    }

    /// The values, in document order.
    pub fn into_vec(self) -> Vec<T> {
        match self {
            Self::One(value) => vec![value],
            Self::Many(values) => values,
        }
    }
}

impl<T> From<Vec<T>> for OneOrMany<T> {
    fn from(many: Vec<T>) -> Self {
        Self::Many(many)
    }
}

/// An empty array: the form a member that is a list by default writes.
impl<T> Default for OneOrMany<T> {
    fn default() -> Self {
        Self::Many(Vec::new())
    }
}

impl<T> IntoIterator for OneOrMany<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_vec().into_iter()
    }
}

impl<'a, T> IntoIterator for &'a OneOrMany<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// The one form is walked at the member itself, the array form element by element at its
/// index, as a `Vec` is.
impl<T: MolangFields> MolangFields for OneOrMany<T> {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        match self {
            Self::One(value) => value.visit_molang(visitor),
            Self::Many(values) => values.visit_molang(visitor),
        }
    }
}

/// As the Molang walk: the one form at the member, the array form at each index. A trigger
/// is found at the member (`on_x`) and in each element (`on_x[]`).
impl<T: SchemaFields> SchemaFields for OneOrMany<T> {
    fn check_fields(&mut self, cx: &mut SchemaCx<'_>) {
        match self {
            Self::One(value) => value.check_fields(cx),
            Self::Many(values) => values.check_fields(cx),
        }
    }

    fn trigger_members(prefix: &str, out: &mut Vec<String>) {
        T::trigger_members(prefix, out);
        <Vec<T>>::trigger_members(prefix, out);
    }
}
