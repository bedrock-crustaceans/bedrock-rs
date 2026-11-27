//! `Layered<K, V>`: values keyed in load order, each key with its effective value and the
//! ones it shadowed: the storage of [`Registry`](super::Registry) (keyed by identifier) and
//! [`Files`](super::Files) (keyed by path), with one [`Iter`] and one [`Layers`], the two
//! public types of the module: the collections return them.

use std::hash::Hash;
use std::iter::FusedIterator;

use indexmap::IndexMap;

/// The values of one key: the effective one, if any, and the shadowed ones, top-first.
pub(crate) struct Slot<V> {
    pub(crate) effective: Option<V>,
    pub(crate) shadowed: Vec<V>,
}

impl<V> Default for Slot<V> {
    fn default() -> Self {
        Self {
            effective: None,
            shadowed: Vec::new(),
        }
    }
}

/// Values keyed in load order, each key with its effective value and the values it
/// shadowed. The count of effective values is kept, so [`len`](Self::len) is O(1).
pub(crate) struct Layered<K, V> {
    entries: IndexMap<K, Slot<V>>,
    /// How many slots have an effective value.
    len: usize,
}

impl<K: Hash + Eq, V> Layered<K, V> {
    /// The finished slots, in load order.
    pub(crate) fn new(entries: IndexMap<K, Slot<V>>) -> Self {
        let len = entries
            .values()
            .filter(|slot| slot.effective.is_some())
            .count();
        Self { entries, len }
    }

    /// The effective value of `key`.
    pub(crate) fn get(&self, key: &K) -> Option<&V> {
        self.entries.get(key)?.effective.as_ref()
    }

    /// Whether `key` has an effective value.
    pub(crate) fn contains(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    /// How many keys have an effective value.
    pub(crate) fn len(&self) -> usize {
        self.len
    }

    /// No key has an effective value.
    pub(crate) fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// `(key, effective value)` in load order.
    pub(crate) fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            inner: self.entries.iter(),
        }
    }

    /// Every value of `key`, the effective one first, then the shadowed ones (top-first).
    pub(crate) fn layers(&self, key: &K) -> Layers<'_, V> {
        Layers::new(self.entries.get(key))
    }

    /// Every key a value is kept of, with all of them, in load order; a key with no value
    /// kept is not listed.
    pub(crate) fn all_layers(&self) -> impl Iterator<Item = (&K, Layers<'_, V>)> + '_ {
        self.entries
            .iter()
            .map(|(key, slot)| (key, Layers::new(Some(slot))))
            .filter(|(_, layers)| layers.len() > 0)
    }
}

impl<'a, K: Hash + Eq, V> IntoIterator for &'a Layered<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// `(key, effective value)` in load order.
pub struct Iter<'a, K, V> {
    inner: indexmap::map::Iter<'a, K, Slot<V>>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .by_ref()
            .find_map(|(key, slot)| Some((key, slot.effective.as_ref()?)))
    }
}

impl<K, V> FusedIterator for Iter<'_, K, V> {}

impl<K, V> Clone for Iter<'_, K, V> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<K, V> std::fmt::Debug for Iter<'_, K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Iter")
            .field("remaining_slots", &self.inner.len())
            .finish_non_exhaustive()
    }
}

/// Every value of one key, the effective one first.
pub struct Layers<'a, V> {
    effective: Option<&'a V>,
    shadowed: std::slice::Iter<'a, V>,
}

impl<'a, V> Layers<'a, V> {
    fn new(slot: Option<&'a Slot<V>>) -> Self {
        match slot {
            Some(slot) => Self {
                effective: slot.effective.as_ref(),
                shadowed: slot.shadowed.iter(),
            },
            None => Self {
                effective: None,
                shadowed: [].iter(),
            },
        }
    }
}

impl<'a, V> Iterator for Layers<'a, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.effective.take().or_else(|| self.shadowed.next())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = usize::from(self.effective.is_some()) + self.shadowed.len();
        (len, Some(len))
    }
}

impl<V> ExactSizeIterator for Layers<'_, V> {}

impl<V> FusedIterator for Layers<'_, V> {}

impl<V> Clone for Layers<'_, V> {
    fn clone(&self) -> Self {
        Self {
            effective: self.effective,
            shadowed: self.shadowed.clone(),
        }
    }
}

impl<V> std::fmt::Debug for Layers<'_, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Layers")
            .field("remaining", &self.size_hint().0)
            .finish_non_exhaustive()
    }
}
