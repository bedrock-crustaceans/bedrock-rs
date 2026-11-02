//! The iterator types of [`Files`](super::Files).

use std::iter::FusedIterator;

use super::{Content, Document, PackPath};

/// The copies of one path: the effective one, if any, and the shadowed ones, top-first.
pub(crate) struct FileSlot<T> {
    pub(crate) effective: Option<Document<T>>,
    pub(crate) shadowed: Vec<Document<T>>,
}

impl<T> Default for FileSlot<T> {
    fn default() -> Self {
        Self {
            effective: None,
            shadowed: Vec::new(),
        }
    }
}

/// `(path, effective document)`.
pub struct Iter<'a, T: Content> {
    inner: indexmap::map::Iter<'a, PackPath, FileSlot<T>>,
}

impl<'a, T: Content> Iter<'a, T> {
    pub(crate) fn new(inner: indexmap::map::Iter<'a, PackPath, FileSlot<T>>) -> Self {
        Self { inner }
    }
}

impl<'a, T: Content> Iterator for Iter<'a, T> {
    type Item = (&'a PackPath, &'a Document<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .by_ref()
            .find_map(|(path, slot)| Some((path, slot.effective.as_ref()?)))
    }
}

impl<T: Content> FusedIterator for Iter<'_, T> {}

impl<T: Content> Clone for Iter<'_, T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

/// Every copy of one path, the effective one first
/// ([`Files::all_layers`](super::Files::all_layers)).
pub struct Layers<'a, T: Content> {
    effective: Option<&'a Document<T>>,
    shadowed: std::slice::Iter<'a, Document<T>>,
}

impl<'a, T: Content> Layers<'a, T> {
    pub(crate) fn new(slot: Option<&'a FileSlot<T>>) -> Self {
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

impl<'a, T: Content> Iterator for Layers<'a, T> {
    type Item = &'a Document<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.effective.take().or_else(|| self.shadowed.next())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = usize::from(self.effective.is_some()) + self.shadowed.len();
        (len, Some(len))
    }
}

impl<T: Content> ExactSizeIterator for Layers<'_, T> {}

impl<T: Content> FusedIterator for Layers<'_, T> {}

impl<T: Content> Clone for Layers<'_, T> {
    fn clone(&self) -> Self {
        Self {
            effective: self.effective,
            shadowed: self.shadowed.clone(),
        }
    }
}

impl<T: Content> std::fmt::Debug for Iter<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Iter")
            .field("remaining_slots", &self.inner.len())
            .finish_non_exhaustive()
    }
}

impl<T: Content> std::fmt::Debug for Layers<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Layers")
            .field("remaining", &self.size_hint().0)
            .finish_non_exhaustive()
    }
}
