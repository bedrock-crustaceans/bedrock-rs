//! The collections a stack load fills: the set a `StackOptions` selects, and the type-erased
//! view of one accessor line the `StackLoader` drives (walk, parse per file, combine, install
//! into the accessor's `OnceLock`). The accessor macro implements [`ErasedCollection`] once per
//! line (`pack/stack/accessors.rs`).

use std::any::Any;
use std::sync::OnceLock;

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics};
use bedrock_addon_core::load::{
    CollectionLoader, LoadContext, Registered, StackCx, Walk, WalkFile,
};
use bedrock_addon_core::pack_identity::{PackPath, PackSide};

pub use crate::pack::stack::Collection;

use super::job::Erased;

/// The most collections a [`CollectionSet`] holds.
const CAPACITY: usize = 128;

const _: () = assert!(
    Collection::ALL.len() <= CAPACITY,
    "CollectionSet holds at most 128 collections"
);

/// A set of [`Collection`]s: the collections a stack load parses now (`StackOptions`).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CollectionSet {
    bits: u128,
}

impl CollectionSet {
    /// Every collection whose feature is compiled in.
    pub const ALL: CollectionSet = CollectionSet {
        bits: mask(Collection::ALL.len()),
    };

    /// No collection.
    pub const NONE: CollectionSet = CollectionSet { bits: 0 };

    /// Only these collections.
    pub fn only(collections: impl IntoIterator<Item = Collection>) -> CollectionSet {
        let bits = collections
            .into_iter()
            .filter_map(bit)
            .fold(0, |bits, bit| bits | bit);
        CollectionSet { bits }
    }

    /// Whether `collection` is in the set.
    #[must_use]
    pub fn contains(&self, collection: Collection) -> bool {
        bit(collection).is_some_and(|bit| self.bits & bit != 0)
    }
}

/// The collections, in list order.
impl std::fmt::Debug for CollectionSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set()
            .entries(Collection::ALL.iter().filter(|c| self.contains(**c)))
            .finish()
    }
}

/// The low `n` bits.
const fn mask(n: usize) -> u128 {
    if n >= CAPACITY {
        u128::MAX
    } else {
        (1 << n) - 1
    }
}

/// The bit of `collection`: its position in the list.
fn bit(collection: Collection) -> Option<u128> {
    Collection::ALL
        .iter()
        .position(|c| *c == collection)
        .map(|position| 1 << position)
}

/// A collection loaded into its accessor's cache, with what loading it found beyond the
/// collection's own diagnostics (`FileNotLoaded`, `ReadError`, `UnsupportedFileFormat`,
/// `InternalError`).
#[allow(
    dead_code,
    reason = "the accessor lines of the enabled families use it; with no family feature on it is unused"
)]
pub(crate) struct Loaded<C> {
    pub(crate) value: C,
    pub(crate) findings: Diagnostics,
}

/// One accessor line, type-erased for the `StackLoader`. `stack` arguments are the
/// `PackStack<S>` of the line's side.
pub(crate) trait ErasedCollection: Sync {
    /// The line's `Collection` variant (`None` for test lines).
    fn collection(&self) -> Option<Collection>;
    /// The stack side the accessor exists on.
    fn side(&self) -> PackSide;
    /// Which files the collection reads.
    fn walk(&self) -> Walk;
    /// The identifier kinds whose registrations the collection's `combine` asks the stack for
    /// (`CollectionLoader::needs`).
    fn needs(&self) -> &'static [&'static str] {
        &[]
    }
    /// The identifiers the cached collection registered, for the `combine` of a collection
    /// that needs its kind; `None` when it is not loaded or registers none.
    fn registered(&self, _stack: &dyn Any) -> Option<Registered> {
        None
    }
    /// Whether the collection has a second walk stage (`CollectionLoader::then`).
    fn has_then(&self) -> bool {
        false
    }
    /// The keys of the second stage's files, from the first stage's parsed files in walk
    /// order; none for a collection without the stage.
    fn then(&self, _parsed: &[Erased]) -> Vec<PackPath> {
        Vec::new()
    }
    /// Parses one file (a `Parse` job).
    fn parse(&self, file: &WalkFile, cx: &LoadContext) -> Erased;
    /// Combines the parsed files in walk order (the `Combine` job).
    fn combine(&self, parsed: Vec<Erased>, cx: &StackCx<'_>) -> Erased;
    /// Installs a combined collection and its loader findings into the accessor's cache;
    /// nothing if it is filled already.
    fn install(&self, stack: &dyn Any, value: Erased, findings: Diagnostics);
    /// The accessor's cache is filled.
    fn is_loaded(&self, stack: &dyn Any) -> bool;
    /// Calls the accessor (the lazy load on the calling thread).
    fn load(&self, stack: &dyn Any);
    /// Appends the cached collection's loader findings and diagnostics, if loaded.
    fn report<'a>(&self, stack: &'a dyn Any, out: &mut Vec<&'a Diagnostic>);
    /// The cached collection's `Debug`, if loaded.
    fn debug(&self, stack: &dyn Any) -> Option<String>;
}

/// Parses one file with `loader`'s `parse`.
#[allow(
    dead_code,
    reason = "the accessor lines of the enabled families use it; with no family feature on it is unused"
)]
pub(crate) fn erased_parse<C, P: Send + Sync + 'static>(
    loader: CollectionLoader<C, P>,
    file: &WalkFile,
    cx: &LoadContext,
) -> Erased {
    Box::new((loader.parse)(file, cx))
}

/// The keys `loader`'s second stage asks for, given the first stage's parsed files; a value of
/// another type (never produced) is left out.
#[allow(
    dead_code,
    reason = "the accessor lines of the enabled families use it; with no family feature on it is unused"
)]
pub(crate) fn erased_then<C, P: 'static>(
    loader: CollectionLoader<C, P>,
    parsed: &[Erased],
) -> Vec<PackPath> {
    let Some(then) = loader.then else {
        return Vec::new();
    };
    let parsed: Vec<&P> = parsed
        .iter()
        .filter_map(|value| value.downcast_ref::<P>())
        .collect();
    then(&parsed)
}

/// Combines parsed files with `loader`'s `combine`; a value of another type (never produced)
/// is left out.
#[allow(
    dead_code,
    reason = "the accessor lines of the enabled families use it; with no family feature on it is unused"
)]
pub(crate) fn erased_combine<C: Send + Sync + 'static, P: 'static>(
    loader: CollectionLoader<C, P>,
    parsed: Vec<Erased>,
    cx: &StackCx<'_>,
) -> Erased {
    let parsed = parsed
        .into_iter()
        .filter_map(|value| value.downcast::<P>().ok().map(|value| *value))
        .collect();
    Box::new((loader.combine)(parsed, cx))
}

/// Fills `cache` with a combined `C`, unless it is filled already.
#[allow(
    dead_code,
    reason = "the accessor lines of the enabled families use it; with no family feature on it is unused"
)]
pub(crate) fn install<C: 'static>(
    cache: &OnceLock<Loaded<C>>,
    value: Erased,
    findings: Diagnostics,
) {
    if let Ok(value) = value.downcast::<C>() {
        let _ = cache.set(Loaded {
            value: *value,
            findings,
        });
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn the_set_is_queryable_copyable_and_hashable() {
        let Some(&first) = Collection::ALL.first() else {
            return;
        };
        let one = CollectionSet::only([first]);
        let copy = one;
        assert!(one.contains(first) && copy.contains(first));
        assert!(!CollectionSet::NONE.contains(first));
        assert!(CollectionSet::ALL.contains(first));
        assert_eq!(CollectionSet::only([]), CollectionSet::NONE);
        let distinct: HashSet<CollectionSet> =
            [one, copy, CollectionSet::NONE, CollectionSet::ALL].into();
        assert_eq!(
            distinct.len(),
            if Collection::ALL.len() == 1 { 2 } else { 3 }
        );
    }
}
