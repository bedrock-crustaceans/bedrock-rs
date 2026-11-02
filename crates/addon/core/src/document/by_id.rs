//! `Registry<T>`: definitions keyed by identifier, and the builder that applies a
//! combine rule to them in walk order.

use std::any::Any;
use std::collections::HashSet;
use std::fmt;
use std::sync::Arc;

use indexmap::IndexMap;

use super::combine::{CombineRule, Combined, Contribution, rank};
use super::registry::{self, Slot};
use super::{Definition, Document, Origin, Rejected};
use crate::diagnostic::{Diagnostic, Diagnostics};
use crate::id::{Id, Kind};
use crate::load::{BuiltIns, LoadOptions, Registered};

/// Definitions of one kind, as the stack resolves them: one effective document per
/// identifier, in load order.
///
/// Load order: by the first definition of each identifier in the family's walk, an override
/// taking the position of the definition it replaces.
pub struct Registry<T: Definition> {
    entries: IndexMap<Id<T::Kind>, Slot<T>>,
    /// How many slots have an effective document.
    len: usize,
    rejected: Vec<Rejected>,
    /// Per-document, rejected and combine-time findings, in walk order.
    diagnostics: Diagnostics,
    /// The `Registry<T::Beta>`, type-erased: a `Registry<T>` holding a `Registry<T::Beta>`
    /// by type overflows the drop check.
    beta: Option<Box<dyn Any + Send + Sync>>,
    built_ins: Option<Arc<dyn BuiltIns>>,
}

impl<T: Definition> Registry<T> {
    /// The effective definition of `id`, or `None` if no pack defines it.
    pub fn get(&self, id: &Id<T::Kind>) -> Option<&Document<T>> {
        self.entries.get(id)?.effective.as_ref()
    }

    /// [`get`](Self::get), plus whether a missing id is a known built-in.
    pub fn resolve(&self, id: &Id<T::Kind>) -> Resolution<'_, T> {
        match self.get(id) {
            Some(document) => Resolution::Defined(document),
            None if self
                .built_ins
                .as_ref()
                .is_some_and(|built_ins| built_ins.contains(T::Kind::NAME, id.as_str())) =>
            {
                Resolution::BuiltIn
            }
            None => Resolution::Missing,
        }
    }

    /// Whether a pack defines `id`.
    pub fn contains(&self, id: &Id<T::Kind>) -> bool {
        self.get(id).is_some()
    }

    /// How many effective definitions.
    pub fn len(&self) -> usize {
        self.len
    }

    /// No effective definitions.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// `(id, effective document)` in load order. A named type ([`registry::Iter`]) because
    /// `IntoIterator for &Registry` needs one; `std::collections::hash_map` style.
    pub fn iter(&self) -> registry::Iter<'_, T> {
        registry::Iter::new(self.entries.iter())
    }

    /// Every definition of `id`, effective first, then the ones it shadowed (top-first).
    /// Only the effective one unless `LoadOptions::keep_shadowed`.
    ///
    /// A stable definition that a higher-priority beta definition of the same identifier
    /// replaces is listed here whatever `keep_shadowed` says, with no effective one before it:
    /// the beta definition is effective in [`beta`](Self::beta).
    pub fn layers(&self, id: &Id<T::Kind>) -> registry::Layers<'_, T> {
        registry::Layers::new(self.entries.get(id))
    }

    /// Every identifier a definition is kept of, with all of them: `(id, layers)` in load
    /// order, the layers as [`layers`](Self::layers) gives them (the effective one first, if
    /// there is one, then the others, top-first).
    ///
    /// [`iter`](Self::iter) lists the effective definitions. This also lists an identifier
    /// that has none but has a definition kept:
    ///
    /// - whatever `LoadOptions::keep_shadowed` says: a stable definition a beta one replaced
    ///   and, in [`beta`](Self::beta), a beta definition a stable one replaced;
    /// - with `keep_shadowed` only: the lower definitions of an identifier a failed higher
    ///   definition claimed, and a first definition its family ignores.
    ///
    /// The copies that are merely shadowed are kept with `keep_shadowed` too. An identifier
    /// nothing is kept of is not listed. Whether the first layer is the effective one is
    /// [`get`](Self::get)`(id).is_some()`.
    pub fn all_layers(&self) -> impl Iterator<Item = (&Id<T::Kind>, registry::Layers<'_, T>)> + '_ {
        self.entries
            .iter()
            .map(|(id, slot)| (id, registry::Layers::new(Some(slot))))
            .filter(|(_, layers)| layers.len() > 0)
    }

    /// Files that failed to load, with their diagnostics.
    pub fn rejected(&self) -> &[Rejected] {
        &self.rejected
    }

    /// Every finding of this family: per-document, rejected files and combine-time ones.
    ///
    /// In walk order, then the beta documents' findings.
    pub fn diagnostics(&self) -> impl Iterator<Item = &Diagnostic> + '_ {
        let beta = self
            .beta()
            .into_iter()
            .flat_map(|beta| beta.diagnostics.iter());
        self.diagnostics.iter().chain(beta)
    }

    /// Beta documents (`"format_version": "beta"`), when `LoadOptions::beta_apis` allows them.
    /// Stored boxed (`Registry<T::Beta>` names `Registry<NoBeta>` one level down).
    ///
    /// `Some` when Beta APIs are on or a base-game pack provided beta documents.
    pub fn beta(&self) -> Option<&Registry<T::Beta>> {
        self.beta.as_deref()?.downcast_ref()
    }

    /// Every identifier with an effective definition, stable or beta, with the pack of that
    /// definition, in load order (stable first): what the `combine` of another collection sees
    /// of this one through `StackCx::registered`. A rejected file registers nothing.
    pub fn registered(&self) -> Registered {
        let stable = self.entries.iter().filter_map(|(id, slot)| {
            let document = slot.effective.as_ref()?;
            Some((id.clone(), document.origin().pack.clone()))
        });
        let beta = self.beta().into_iter().flat_map(|beta| {
            beta.entries.iter().filter_map(|(id, slot)| {
                let document = slot.effective.as_ref()?;
                Some((id.clone(), document.origin().pack.clone()))
            })
        });
        Registered::new::<T::Kind>(stable.chain(beta))
    }
}

impl<'a, T: Definition> IntoIterator for &'a Registry<T> {
    type Item = (&'a Id<T::Kind>, &'a Document<T>);
    type IntoIter = registry::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// The effective documents in load order, the rejected files, the findings and the beta
/// registry's identifiers.
impl<T: Definition + fmt::Debug> fmt::Debug for Registry<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Registry")
            .field("entries", &DebugEntries(self))
            .field("rejected", &self.rejected)
            .field("diagnostics", &self.diagnostics)
            .field(
                "beta",
                &self
                    .beta()
                    .map(|beta| beta.iter().map(|(id, _)| id).collect::<Vec<_>>()),
            )
            .finish_non_exhaustive()
    }
}

struct DebugEntries<'a, T: Definition>(&'a Registry<T>);

impl<T: Definition + fmt::Debug> fmt::Debug for DebugEntries<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.0.iter()).finish()
    }
}

/// Whether and how an identifier resolves ([`Registry::resolve`]).
pub enum Resolution<'a, T: Definition> {
    /// A pack on the stack defines it.
    Defined(&'a Document<T>),
    /// No pack defines it, but `LoadOptions::built_ins` knows it (a hard-coded vanilla block,
    /// an engine-defined entity).
    BuiltIn,
    /// Nothing defines it.
    Missing,
}

impl<T: Definition> Clone for Resolution<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Definition> Copy for Resolution<'_, T> {}

impl<T: Definition + fmt::Debug> fmt::Debug for Resolution<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Defined(document) => f.debug_tuple("Defined").field(document).finish(),
            Self::BuiltIn => f.write_str("BuiltIn"),
            Self::Missing => f.write_str("Missing"),
        }
    }
}

/// Builds a [`Registry`] from the contributions of a family's walk, in walk order, applying
/// the family's combine rule (`Registry` has private fields, so `bedrock_addon` builds
/// it here).
pub struct RegistryBuilder<T: Definition> {
    stable: Combiner<T>,
    beta: Combiner<T::Beta>,
    any_beta: bool,
    beta_apis: bool,
    keep_shadowed: bool,
    built_ins: Option<Arc<dyn BuiltIns>>,
    rejected: Vec<Rejected>,
    /// Failed documents that named their identifier: the highest-priority one per identifier.
    claims: IndexMap<Id<T::Kind>, Origin>,
}

/// One registry's slots while it is built.
struct Combiner<T: Definition> {
    rule: CombineRule<T>,
    entries: IndexMap<Id<T::Kind>, Slot<T>>,
    diagnostics: Diagnostics,
}

impl<T: Definition> Combiner<T> {
    fn new(rule: CombineRule<T>) -> Self {
        Self {
            rule,
            entries: IndexMap::new(),
            diagnostics: Diagnostics::default(),
        }
    }

    fn push(&mut self, id: Id<T::Kind>, document: Document<T>) {
        self.diagnostics
            .extend(document.diagnostics().iter().cloned());
        let slot = self.entries.entry(id).or_default();
        let Combined { effective, dropped } =
            self.rule
                .combine(slot.effective.take(), document, &mut self.diagnostics);
        slot.effective = effective;
        slot.shadowed.extend(dropped);
    }
}

impl<T: Definition> fmt::Debug for RegistryBuilder<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RegistryBuilder")
            .field("stable", &self.stable.entries.len())
            .field("beta", &self.beta.entries.len())
            .field("beta_apis", &self.beta_apis)
            .field("keep_shadowed", &self.keep_shadowed)
            .field("rejected", &self.rejected.len())
            .finish_non_exhaustive()
    }
}

impl<T: Definition> RegistryBuilder<T> {
    /// A builder applying `rule` to stable definitions and `beta_rule` to beta ones, with the
    /// stack's `keep_shadowed`, `beta_apis` and `built_ins`.
    pub fn new(
        rule: CombineRule<T>,
        beta_rule: CombineRule<T::Beta>,
        options: &LoadOptions,
    ) -> Self {
        Self {
            stable: Combiner::new(rule),
            beta: Combiner::new(beta_rule),
            any_beta: false,
            beta_apis: options.beta_apis,
            keep_shadowed: options.keep_shadowed,
            built_ins: options.built_ins.clone(),
            rejected: Vec::new(),
            claims: IndexMap::new(),
        }
    }

    /// Adds the next contribution of the walk.
    pub fn push(&mut self, contribution: Contribution<T>) {
        match contribution {
            Contribution::Stable(id, document) => self.stable.push(id, document),
            Contribution::Beta(id, document) => {
                self.any_beta = true;
                self.beta.push(id, document);
            }
            Contribution::Rejected(id, rejected) => {
                self.stable
                    .diagnostics
                    .extend(rejected.diagnostics.iter().cloned());
                if let Some(id) = id {
                    let claim = self
                        .claims
                        .entry(id)
                        .or_insert_with(|| rejected.origin.clone());
                    if rank(&rejected.origin.pack) < rank(&claim.pack) {
                        *claim = rejected.origin.clone();
                    }
                }
                self.rejected.push(rejected);
            }
        }
    }

    /// Adds a family's own combine-time finding, after the ones raised so far.
    pub fn push_diagnostic(&mut self, diagnostic: Diagnostic) {
        self.stable.diagnostics.push(diagnostic);
    }

    /// The registry.
    pub fn finish(self) -> Registry<T> {
        let Self {
            mut stable,
            mut beta,
            any_beta,
            beta_apis,
            keep_shadowed,
            built_ins,
            rejected,
            claims,
        } = self;
        let reported = apply_claims(&mut stable, &claims);
        claim_beta(&mut stable, &mut beta, &claims, &reported);
        if !keep_shadowed {
            for slot in stable.entries.values_mut() {
                slot.shadowed.clear();
            }
            for slot in beta.entries.values_mut() {
                slot.shadowed.clear();
            }
        }
        cross_beta(&mut stable, &mut beta);
        let beta = (beta_apis || any_beta).then(|| {
            Box::new(finish(beta, Vec::new(), None, built_ins.clone()))
                as Box<dyn Any + Send + Sync>
        });
        finish(stable, rejected, beta, built_ins)
    }
}

/// Failed claims: `TopWinsFallback` reports the fallback, `ClaimBeforeValidate` drops the lower
/// definition. Returns the identifiers whose claim's notice was raised.
fn apply_claims<T: Definition>(
    combiner: &mut Combiner<T>,
    claims: &IndexMap<Id<T::Kind>, Origin>,
) -> HashSet<Id<T::Kind>> {
    let mut reported = HashSet::new();
    for (id, claim) in claims {
        let Some(slot) = combiner.entries.get_mut(id) else {
            continue;
        };
        let Some(effective) = &slot.effective else {
            continue;
        };
        if rank(&claim.pack) >= rank(&effective.origin().pack) {
            continue;
        }
        match &combiner.rule {
            CombineRule::TopWinsFallback {
                fallback: Some(notice),
            } => {
                combiner.diagnostics.push(notice.at(claim));
                reported.insert(id.clone());
            }
            CombineRule::ClaimBeforeValidate { claimed } => {
                if let Some(notice) = claimed {
                    combiner.diagnostics.push(notice.at(claim));
                    reported.insert(id.clone());
                }
                slot.shadowed.extend(slot.effective.take());
            }
            _ => {}
        }
    }
    reported
}

/// Under `ClaimBeforeValidate` a failed claim holds its identifier against every lower
/// document, the beta ones too: a lower beta definition is dropped, and the claim's notice is
/// raised here unless `reported` (what [`apply_claims`] returned) says a stable definition
/// below the claim raised it already.
// `reported` carries what the stable pass raised, instead of reading the stable slot's state
// back.
fn claim_beta<T: Definition>(
    stable: &mut Combiner<T>,
    beta: &mut Combiner<T::Beta>,
    claims: &IndexMap<Id<T::Kind>, Origin>,
    reported: &HashSet<Id<T::Kind>>,
) {
    let CombineRule::ClaimBeforeValidate { claimed } = &stable.rule else {
        return;
    };
    for (id, claim) in claims {
        let Some(slot) = beta.entries.get_mut(id) else {
            continue;
        };
        let Some(effective) = &slot.effective else {
            continue;
        };
        if rank(&claim.pack) >= rank(&effective.origin().pack) {
            continue;
        }
        slot.shadowed.extend(slot.effective.take());
        if let Some(notice) = claimed
            && !reported.contains(id)
        {
            stable.diagnostics.push(notice.at(claim));
        }
    }
}

/// An identifier with a stable and a beta definition: the higher-priority one stays effective
/// in its registry, the other is listed in its registry's layers.
fn cross_beta<T: Definition>(stable: &mut Combiner<T>, beta: &mut Combiner<T::Beta>) {
    for (id, beta_slot) in &mut beta.entries {
        let Some(stable_slot) = stable.entries.get_mut(id) else {
            continue;
        };
        let (Some(stable_document), Some(beta_document)) =
            (&stable_slot.effective, &beta_slot.effective)
        else {
            continue;
        };
        if rank(&beta_document.origin().pack) < rank(&stable_document.origin().pack) {
            stable_slot.shadowed.extend(stable_slot.effective.take());
        } else {
            beta_slot.shadowed.extend(beta_slot.effective.take());
        }
    }
}

fn finish<T: Definition>(
    combiner: Combiner<T>,
    rejected: Vec<Rejected>,
    beta: Option<Box<dyn Any + Send + Sync>>,
    built_ins: Option<Arc<dyn BuiltIns>>,
) -> Registry<T> {
    let Combiner {
        mut entries,
        diagnostics,
        ..
    } = combiner;
    for slot in entries.values_mut() {
        // Top-first; a stable sort keeps walk order within a pack.
        slot.shadowed
            .sort_by_key(|document| rank(&document.origin().pack));
    }
    let len = entries
        .values()
        .filter(|slot| slot.effective.is_some())
        .count();
    Registry {
        entries,
        len,
        rejected,
        diagnostics,
        beta,
        built_ins,
    }
}
