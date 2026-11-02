//! `Merged<T, L>`: one value merged from every pack of the stack, and the fold
//! that builds it with the family's merge strategy.

use std::fmt;
use std::ops::Deref;

use indexmap::IndexMap;

use super::combine::rank;
use super::{Document, Merge, Origin, Rejected};
use crate::diagnostic::{Diagnostic, Diagnostics};
use crate::json;

/// A value merged from every pack of the stack, plus what each pack contributed.
///
/// `L` is the per-pack layer: a parsed `Document<T>` where vanilla parses each copy before
/// merging (texts, catalogue, tick functions), or a [`RawLayer`] where vanilla merges the JSON
/// first and parses the result (atlases, flipbooks, sound files, UI definitions, material files,
/// legacy client biomes).
// `T: Merge` is a bound on the struct itself, since the origins are keyed by `T::Key`.
pub struct Merged<T: Merge, L = Document<T>> {
    value: T,
    origins: IndexMap<T::Key, Origin>,
    layers: Vec<L>,
    diagnostics: Diagnostics,
}

impl<T: Merge, L> Merged<T, L> {
    /// The merged result.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Which pack supplied the entry `key` of the merged value (the winning one).
    pub fn origin(&self, key: &T::Key) -> Option<&Origin> {
        self.origins.get(key)
    }

    /// Per-pack contributions, top-first.
    pub fn layers(&self) -> &[L] {
        &self.layers
    }

    /// Every finding of the merge.
    ///
    /// The layers' and the failed copies' findings in walk order, with the strategy's own
    /// findings where it raised them.
    pub fn diagnostics(&self) -> impl Iterator<Item = &Diagnostic> + '_ {
        self.diagnostics.iter()
    }

    /// A merged value from its parts: the value, the winning origin per key, the layers
    /// top-first and the findings (the fields are private, so `bedrock_addon` builds
    /// it here).
    #[doc(hidden)]
    pub fn new(
        value: T,
        origins: IndexMap<T::Key, Origin>,
        layers: Vec<L>,
        diagnostics: Diagnostics,
    ) -> Self {
        Self {
            value,
            origins,
            layers,
            diagnostics,
        }
    }
}

impl<T: Merge, L: Layer> Merged<T, L> {
    /// Folds the copies of a family's walk, in walk order, into `initial` with the family's
    /// merge strategy `step`, which records the winning origin of each entry it writes
    /// ([`MergeCx::claim`]). A failed copy contributes its findings only. The layers are kept
    /// top-first (`bedrock_addon` builds the value here).
    #[doc(hidden)]
    pub fn merge(
        walk: Vec<Result<L, Rejected>>,
        initial: T,
        mut step: impl FnMut(&mut T, &L, &mut MergeCx<'_, T::Key>),
    ) -> Self {
        let mut value = initial;
        let mut origins = IndexMap::new();
        let mut diagnostics = Diagnostics::default();
        let mut layers = Vec::new();
        for copy in walk {
            match copy {
                Ok(layer) => {
                    if let Some(findings) = layer.layer_diagnostics() {
                        diagnostics.extend(findings.iter().cloned());
                    }
                    let mut cx = MergeCx {
                        origin: layer.layer_origin(),
                        origins: &mut origins,
                        diagnostics: &mut diagnostics,
                    };
                    step(&mut value, &layer, &mut cx);
                    layers.push(layer);
                }
                Err(rejected) => diagnostics.extend(rejected.diagnostics),
            }
        }
        // Top-first; within one pack the later copy first.
        layers.reverse();
        layers.sort_by_key(|layer| rank(&layer.layer_origin().pack));
        Self {
            value,
            origins,
            layers,
            diagnostics,
        }
    }
}

impl<T: Merge, L> Deref for Merged<T, L> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T, L> fmt::Debug for Merged<T, L>
where
    T: Merge + fmt::Debug,
    T::Key: fmt::Debug,
    L: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Merged")
            .field("value", &self.value)
            .field("origins", &self.origins)
            .field("layers", &self.layers)
            .field("diagnostics", &self.diagnostics)
            .finish()
    }
}

/// What a merge step sees of the copy it merges and records about it.
pub struct MergeCx<'a, K> {
    origin: &'a Origin,
    origins: &'a mut IndexMap<K, Origin>,
    diagnostics: &'a mut Diagnostics,
}

impl<K: Eq + std::hash::Hash> MergeCx<'_, K> {
    /// Where the copy being merged came from.
    #[must_use]
    pub fn origin(&self) -> &Origin {
        self.origin
    }

    /// The copy being merged wrote entry `key`: it is now the entry's origin.
    pub fn claim(&mut self, key: K) {
        self.origins.insert(key, self.origin.clone());
    }

    /// The copy being merged removed entry `key`.
    pub fn release(&mut self, key: &K) {
        self.origins.shift_remove(key);
    }

    /// A finding of the strategy, after the ones raised so far.
    pub fn push_diagnostic(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }
}

impl<K> std::fmt::Debug for MergeCx<'_, K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MergeCx")
            .field("origin", &self.origin)
            .field("claimed", &self.origins.len())
            .finish_non_exhaustive()
    }
}

/// One pack's contribution to a [`Merged`] value: a parsed document or a raw layer.
pub trait Layer {
    /// Where the copy came from.
    fn layer_origin(&self) -> &Origin;
    /// The copy's own findings, if it was parsed.
    fn layer_diagnostics(&self) -> Option<&Diagnostics>;
}

impl<T> Layer for Document<T> {
    fn layer_origin(&self) -> &Origin {
        self.origin()
    }

    fn layer_diagnostics(&self) -> Option<&Diagnostics> {
        Some(self.diagnostics())
    }
}

impl Layer for RawLayer {
    fn layer_origin(&self) -> &Origin {
        &self.origin
    }

    fn layer_diagnostics(&self) -> Option<&Diagnostics> {
        None
    }
}

/// One pack's copy of a merge-before-parse file: where it came from and its raw JSON.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct RawLayer {
    /// Where the copy came from.
    pub origin: Origin,
    /// The copy's JSON, as read.
    pub json: json::Value,
}

impl RawLayer {
    /// A layer from its parts (the struct is `#[non_exhaustive]`, so `bedrock_addon`
    /// builds it here).
    #[doc(hidden)]
    #[must_use]
    pub fn new(origin: Origin, json: json::Value) -> Self {
        Self { origin, json }
    }
}
