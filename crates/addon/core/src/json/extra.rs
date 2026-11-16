//! `Extra`, the marker of a struct that keeps its unknown members, and `UnknownMembers`, the
//! members a hand reader keeps itself; defined here beside the walk and the writer. Their one
//! public path is `document::{Extra, UnknownMembers}`.

use facet::Facet;
use indexmap::IndexMap;

use super::Value;

/// The mark of a struct whose unknown members are kept for write-back: a struct that keeps
/// them has `#[facet(skip)] pub extra: Extra`. It holds nothing (zero
/// bytes): the reader puts each unknown member of such a struct into the document's
/// [`Unknown`](crate::document::Unknown) table by the path of its object, and the writer
/// writes them from there after the object's known members. A struct without the field
/// reports its unknown members and does not keep them.
///
/// Braced and without a field, so that `Extra::default()` is how a struct gets one.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Facet)]
pub struct Extra {}

/// Unknown members a hand reader keeps itself, at their level, for its own writer: a newtype
/// over `IndexMap<String, json::Value>` in document order. The reader never fills it (a
/// struct it reads marks itself with [`Extra`] and its members go to the document's
/// table); a reader that walks a `Value` by hand does, and its writer emits them after the
/// known members. `#[facet(transparent)]`: as a facet value it is the map.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(transparent)]
pub struct UnknownMembers(IndexMap<String, Value>);

impl UnknownMembers {
    /// The unknown member `key`.
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
    #[allow(clippy::iter_without_into_iter)] // read through `iter`, not by `IntoIterator`
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Value)> + '_ {
        self.0.iter().map(|(key, value)| (key.as_str(), value))
    }

    /// Puts the members in the order of `place`, the place of a member's name.
    pub(crate) fn sort_by_place(&mut self, mut place: impl FnMut(&str) -> usize) {
        self.0.sort_by_cached_key(|key, _| place(key));
    }

    /// How many unknown members.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// No unknown members.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl FromIterator<(String, Value)> for UnknownMembers {
    fn from_iter<I: IntoIterator<Item = (String, Value)>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl Extend<(String, Value)> for UnknownMembers {
    fn extend<I: IntoIterator<Item = (String, Value)>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}
