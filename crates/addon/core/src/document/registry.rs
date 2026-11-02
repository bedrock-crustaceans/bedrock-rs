//! The iterator types of [`Registry`](super::Registry).

use std::iter::FusedIterator;

use super::{Definition, Document};
use crate::id::Id;

/// The documents of one identifier: the effective one, if any, and the shadowed ones,
/// top-first.
pub(crate) struct Slot<T> {
    pub(crate) effective: Option<Document<T>>,
    pub(crate) shadowed: Vec<Document<T>>,
}

impl<T> Default for Slot<T> {
    fn default() -> Self {
        Self {
            effective: None,
            shadowed: Vec::new(),
        }
    }
}

/// `(id, effective document)` in load order.
pub struct Iter<'a, T: Definition> {
    inner: indexmap::map::Iter<'a, Id<T::Kind>, Slot<T>>,
}

impl<'a, T: Definition> Iter<'a, T> {
    pub(crate) fn new(inner: indexmap::map::Iter<'a, Id<T::Kind>, Slot<T>>) -> Self {
        Self { inner }
    }
}

impl<'a, T: Definition> Iterator for Iter<'a, T> {
    type Item = (&'a Id<T::Kind>, &'a Document<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .by_ref()
            .find_map(|(id, slot)| Some((id, slot.effective.as_ref()?)))
    }
}

impl<T: Definition> FusedIterator for Iter<'_, T> {}

impl<T: Definition> Clone for Iter<'_, T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

/// Every definition of one id, effective first.
pub struct Layers<'a, T: Definition> {
    effective: Option<&'a Document<T>>,
    shadowed: std::slice::Iter<'a, Document<T>>,
}

impl<'a, T: Definition> Layers<'a, T> {
    pub(crate) fn new(slot: Option<&'a Slot<T>>) -> Self {
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

impl<'a, T: Definition> Iterator for Layers<'a, T> {
    type Item = &'a Document<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.effective.take().or_else(|| self.shadowed.next())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = usize::from(self.effective.is_some()) + self.shadowed.len();
        (len, Some(len))
    }
}

impl<T: Definition> ExactSizeIterator for Layers<'_, T> {}

impl<T: Definition> FusedIterator for Layers<'_, T> {}

impl<T: Definition> Clone for Layers<'_, T> {
    fn clone(&self) -> Self {
        Self {
            effective: self.effective,
            shadowed: self.shadowed.clone(),
        }
    }
}

impl<T: Definition> std::fmt::Debug for Iter<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Iter")
            .field("remaining_slots", &self.inner.len())
            .finish_non_exhaustive()
    }
}

impl<T: Definition> std::fmt::Debug for Layers<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Layers")
            .field("remaining", &self.size_hint().0)
            .finish_non_exhaustive()
    }
}
