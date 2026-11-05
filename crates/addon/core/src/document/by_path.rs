//! `Files<T>`: whole files keyed by path, and the builder that picks the
//! effective copy of each path.

use std::fmt;

use indexmap::IndexMap;

use super::combine::{FilesRule, rank};
use super::files;
use super::layered::{Layered, Slot};
use super::{Content, Document, Rejected};
use crate::diagnostic::{Diagnostic, Diagnostics};
use crate::load::LoadOptions;
use crate::pack_identity::PackPath;

/// Whole files addressed by path; the top pack's copy wins.
///
/// Iterated by the first copy of each path in the family's walk.
pub struct Files<T: Content> {
    layered: Layered<PackPath, Document<T>>,
    rejected: Vec<Rejected>,
    /// Per-document, rejected and combine-time findings, in walk order.
    diagnostics: Diagnostics,
}

impl<T: Content> Files<T> {
    /// The effective file at `path`, or `None` if no pack has it.
    pub fn get(&self, path: &PackPath) -> Option<&Document<T>> {
        self.layered.get(path)
    }

    /// How many paths have an effective copy.
    pub fn len(&self) -> usize {
        self.layered.len()
    }

    /// No path has an effective copy.
    pub fn is_empty(&self) -> bool {
        self.layered.is_empty()
    }

    /// Whether a pack has `path`.
    pub fn contains(&self, path: &PackPath) -> bool {
        self.layered.contains(path)
    }

    /// `(path, effective document)`; named ([`files::Iter`]) like [`Registry::iter`].
    ///
    /// [`Registry::iter`]: super::Registry::iter
    pub fn iter(&self) -> files::Iter<'_, T> {
        self.layered.iter()
    }

    /// Every pack's copy of `path`, effective first. Only the effective one unless
    /// `LoadOptions::keep_shadowed`.
    pub fn layers(&self, path: &PackPath) -> files::Layers<'_, T> {
        self.layered.layers(path)
    }

    /// Every path a copy is kept of, with all of them: `(path, layers)` in the order of
    /// [`iter`](Self::iter), the layers as [`layers`](Self::layers) gives them.
    ///
    /// Unlike [`iter`](Self::iter) this lists a path without an effective copy: one a failed
    /// higher copy claimed, whose lower copies are kept with `LoadOptions::keep_shadowed`.
    /// A path nothing is kept of is not listed. Whether the first layer is the effective one
    /// is [`get`](Self::get)`(path).is_some()`.
    pub fn all_layers(&self) -> impl Iterator<Item = (&PackPath, files::Layers<'_, T>)> + '_ {
        self.layered.all_layers()
    }

    /// Files that failed to load, with their diagnostics.
    pub fn rejected(&self) -> &[Rejected] {
        &self.rejected
    }

    /// Every finding of this family.
    pub fn diagnostics(&self) -> impl Iterator<Item = &Diagnostic> + '_ {
        self.diagnostics.iter()
    }

    /// The combine's own collection of the findings, moves and places included.
    #[cfg(test)]
    pub(crate) fn held_diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }
}

impl<'a, T: Content> IntoIterator for &'a Files<T> {
    type Item = (&'a PackPath, &'a Document<T>);
    type IntoIter = files::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// The effective documents by path, the rejected files and the findings.
impl<T: Content + fmt::Debug> fmt::Debug for Files<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Files")
            .field("entries", &DebugEntries(self))
            .field("rejected", &self.rejected)
            .field("diagnostics", &self.diagnostics)
            .finish_non_exhaustive()
    }
}

struct DebugEntries<'a, T: Content>(&'a Files<T>);

impl<T: Content + fmt::Debug> fmt::Debug for DebugEntries<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.0.iter()).finish()
    }
}

/// Builds a [`Files`] from every copy of a family's files, in walk order (`Files` has
/// private fields, so `bedrock_addon` builds it here).
pub struct FilesBuilder<T: Content> {
    rule: FilesRule,
    keep_shadowed: bool,
    /// Every copy per path, in walk order.
    copies: IndexMap<PackPath, Vec<Result<Document<T>, Rejected>>>,
    rejected: Vec<Rejected>,
    diagnostics: Diagnostics,
}

impl<T: Content> fmt::Debug for FilesBuilder<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FilesBuilder")
            .field("paths", &self.copies.len())
            .field("keep_shadowed", &self.keep_shadowed)
            .field("rejected", &self.rejected.len())
            .finish_non_exhaustive()
    }
}

impl<T: Content> FilesBuilder<T> {
    /// A builder applying `rule`, with the stack's `keep_shadowed`.
    pub fn new(rule: FilesRule, options: &LoadOptions) -> Self {
        Self {
            rule,
            keep_shadowed: options.keep_shadowed,
            copies: IndexMap::new(),
            rejected: Vec::new(),
            diagnostics: Diagnostics::default(),
        }
    }

    /// Adds the next copy of the walk under `path`, the path the collection is keyed by
    /// (relative to the pack's content root, without a subpack folder).
    pub fn push(&mut self, path: PackPath, copy: Result<Document<T>, Rejected>) {
        match &copy {
            Ok(document) => self
                .diagnostics
                .extend(document.diagnostics().iter().cloned()),
            Err(rejected) => {
                self.diagnostics
                    .extend(rejected.diagnostics.iter().cloned());
                self.rejected.push(rejected.clone());
            }
        }
        self.copies.entry(path).or_default().push(copy);
    }

    /// Adds a family's own combine-time finding, after the ones raised so far.
    pub fn push_diagnostic(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// The collection of a walk's copies, in walk order, each under the path it is keyed by:
    /// [`new`](Self::new), a [`push`](Self::push) of each and [`finish`](Self::finish).
    pub fn combine(
        parsed: impl IntoIterator<Item = (PackPath, Result<Document<T>, Rejected>)>,
        rule: FilesRule,
        options: &LoadOptions,
    ) -> Files<T> {
        let mut builder = Self::new(rule, options);
        for (path, copy) in parsed {
            builder.push(path, copy);
        }
        builder.finish()
    }

    /// The collection.
    pub fn finish(self) -> Files<T> {
        let Self {
            rule,
            keep_shadowed,
            copies,
            rejected,
            mut diagnostics,
        } = self;
        let entries = copies
            .into_iter()
            .map(|(path, mut copies)| {
                // Top-first; within one pack the later copy first.
                copies.reverse();
                copies.sort_by_key(|copy| match copy {
                    Ok(document) => rank(&document.origin().pack),
                    Err(rejected) => rank(&rejected.origin.pack),
                });
                let slot = pick(&rule, copies, keep_shadowed, &mut diagnostics);
                (path, slot)
            })
            .collect();
        Files {
            layered: Layered::new(entries),
            rejected,
            diagnostics,
        }
    }
}

/// The effective copy of one path from its copies, top-first.
fn pick<T>(
    rule: &FilesRule,
    copies: Vec<Result<Document<T>, Rejected>>,
    keep_shadowed: bool,
    diagnostics: &mut Diagnostics,
) -> Slot<Document<T>> {
    let mut slot = Slot::default();
    let mut failed_above = None;
    for (position, copy) in copies.into_iter().enumerate() {
        match copy {
            Ok(document) => {
                let claimed = slot.effective.is_some()
                    || (position > 0 && matches!(rule, FilesRule::TopWins));
                if claimed {
                    if keep_shadowed {
                        slot.shadowed.push(document);
                    }
                    continue;
                }
                if let (
                    FilesRule::TopWinsFallback {
                        fallback: Some(notice),
                    },
                    Some(origin),
                ) = (rule, failed_above.take())
                {
                    diagnostics.push(notice.at(&origin));
                }
                slot.effective = Some(document);
            }
            Err(rejected) => {
                if slot.effective.is_none() && failed_above.is_none() {
                    failed_above = Some(rejected.origin);
                }
            }
        }
    }
    slot
}
