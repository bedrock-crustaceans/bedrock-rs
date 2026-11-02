//! `Extra`, defined here beside the walk that fills it and the writer that emits it; its one
//! public path is `document::Extra`.

use facet::Facet;
use indexmap::IndexMap;

use super::Value;

/// Unknown members of an object, kept at their level for write-back: a newtype over
/// `IndexMap<String, json::Value>` in document order, filled by the unknown-member walk, never by
/// `#[facet(flatten)]`. A struct that keeps them has
/// `#[facet(skip, rename = "\u{0}extra")] pub extra: Extra`; the writer emits them after the
/// struct's known members.
///
/// # Skipped fields
///
/// facet (rc.7) matches a JSON member against every field's name, a `#[facet(skip)]` field's
/// included: a member written `"extra": 1` fails the typed parse, and `"extra": {..}` is read
/// into the side-car and written back as root members. So every skipped field, `extra` or a
/// side-car the parser fills itself, is renamed to its own name behind a NUL,
/// `#[facet(skip, rename = "\u{0}<name>")]`, a key no document writes. A member named like the
/// field is then an unknown member like any other: reported (`UnrecognizedField` in the
/// families), kept here and written back. The unknown-member walk and the writer find the field
/// by its type, never by its name, so the rename changes nothing else. A test in `bedrock_addon`
/// holds every skipped field of that crate to the convention.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(transparent)]
pub struct Extra(IndexMap<String, Value>);

impl Extra {
    /// The unknown member `key`.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.get(key)
    }

    /// Inserts an unknown member, or replaces it in place; returns the replaced value.
    pub fn insert(&mut self, key: String, value: Value) -> Option<Value> {
        self.0.insert(key, value)
    }

    /// Removes an unknown member, keeping the order of the others.
    pub fn remove(&mut self, key: &str) -> Option<Value> {
        self.0.shift_remove(key)
    }

    /// The unknown members in document order.
    #[allow(clippy::iter_without_into_iter)] // `Extra` is read through `iter`, not by `IntoIterator`
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Value)> + '_ {
        self.0.iter().map(|(key, value)| (key.as_str(), value))
    }

    /// How many unknown members.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// No unknown members.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl FromIterator<(String, Value)> for Extra {
    fn from_iter<I: IntoIterator<Item = (String, Value)>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl Extend<(String, Value)> for Extra {
    fn extend<I: IntoIterator<Item = (String, Value)>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}
