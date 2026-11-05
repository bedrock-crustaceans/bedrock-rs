//! The combine rules: how the definitions of one identifier from several packs
//! become the effective one. Hidden: each family selects its rule in its loader; the public
//! effect is only `Registry::get`, `Origin::contributors` and `Registry::layers`.

use std::cmp::Ordering;
use std::fmt;

use super::{Definition, Document, Origin, Rejected, Unknown};
use crate::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, Phase, Severity};
use crate::id::Id;
use crate::pack_identity::PackRef;

/// A finding a combine rule raises about one document, with the code and severity the family
/// chose (`ShadowedDefinition`, `DuplicateIdentifier`, `RenderControllerFallback`, …).
#[derive(Clone, Debug, PartialEq)]
pub struct Notice {
    /// The code.
    pub code: DiagnosticCode,
    /// The severity.
    pub severity: Severity,
}

impl Notice {
    /// `ShadowedDefinition` at info: a definition replaced, or one ignored.
    pub fn shadowed() -> Self {
        Self {
            code: DiagnosticCode::ShadowedDefinition { text: None },
            severity: Severity::Info,
        }
    }

    /// `DuplicateIdentifier` at info: one of two definitions dropped by a rule other than
    /// pack priority. This is the crate's own information: the game's registries take the second
    /// definition of one pack without a content-log line (the block registry's "block name '%s'
    /// already registered" is only an assertion); a family whose game logs the duplicate
    /// raises its own finding (a recipe's "JSON: %s has duplicate identifier in same pack: %s"
    /// or a feature's "already exists").
    pub fn duplicate() -> Self {
        Self {
            code: DiagnosticCode::DuplicateIdentifier { text: None },
            severity: Severity::Info,
        }
    }

    /// The finding at `origin`'s pack and file. Combine-time findings have phase `Validate`.
    pub(crate) fn at(&self, origin: &Origin) -> Diagnostic {
        Diagnostic::new(
            self.code.clone(),
            self.severity,
            Phase::Validate,
            origin.location(),
        )
    }
}

/// What an `EngineVersionMerge` family does with a definition of an identifier it has seen
/// (client entities and attachables by `description.min_engine_version`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EngineVersionDecision {
    /// Drop the incoming definition (it is listed in `layers`).
    Ignore,
    /// Merge the incoming definition into the existing one (the family's `merge`).
    Merge,
    /// The incoming definition replaces the existing one.
    Replace,
}

/// The `EngineVersionMerge` rule's choice: the existing effective definition (`None` for the first
/// of an identifier), the incoming one, and where the family's own findings go. A closure, so
/// a family can carry stack-wide input (the stack's highest `min_engine_version`).
pub type DecideFn<T> = Box<
    dyn Fn(Option<&Document<T>>, &Document<T>, &mut Diagnostics) -> EngineVersionDecision
        + Send
        + Sync,
>;

/// How the definitions of one identifier from several packs combine. The
/// registry builder applies it to the definitions in walk order.
///
/// Pack priority is the stack index (0 is the top; the app-data layer is below every pack).
/// Two definitions of one identifier in one pack are a `DuplicateIdentifier` warning under
/// every rule that decides by priority; the later one in walk order wins, except under
/// `BottomWins`, where the earlier one stays.
pub enum CombineRule<T> {
    /// The highest-priority definition replaces lower ones entirely. A failed document does
    /// not reserve its identifier.
    TopWins {
        /// Raised at each definition a higher one replaces or ignores.
        shadowed: Option<Notice>,
    },
    /// The lowest-priority definition is kept; higher ones are shadowed.
    BottomWins {
        /// Raised at each higher definition ([`Notice::shadowed`] in vanilla's families).
        shadowed: Option<Notice>,
    },
    /// The later definition in walk order wins, whatever the pack priority.
    LastLoadedWins {
        /// Raised at each replaced definition ([`Notice::duplicate`]).
        replaced: Option<Notice>,
    },
    /// `TopWins`, and a failing higher definition keeps the lower valid one.
    TopWinsFallback {
        /// Raised at the failing higher document when a lower one is effective
        /// (`RenderControllerFallback` for render controllers).
        fallback: Option<Notice>,
    },
    /// Ignore, merge or replace, as the family's closure chooses.
    EngineVersionMerge {
        /// The closure that chooses.
        decide: DecideFn<T>,
        /// Merges the incoming value into the existing one.
        merge: fn(&mut T, T),
    },
    /// Per field, lower then higher: the family's `merge` writes the higher value over the
    /// lower one.
    FieldMerge {
        /// `merge(lower, higher)`.
        merge: fn(&mut T, T),
    },
    /// The effective definition is top-wins, but a side list accumulates: the family's
    /// `accumulate` adds the other definition's list to the winner's.
    Accumulate {
        /// `accumulate(winner, other)`.
        accumulate: fn(&mut T, &T),
    },
    /// The top pack claims the identifier even when its document fails: a failed claim leaves
    /// the identifier undefined and the lower definitions shadowed.
    ClaimBeforeValidate {
        /// Raised at a failing claim that shadows a lower definition
        /// (`ClientBiomeShadowsLowerPacks` for client biomes).
        claimed: Option<Notice>,
    },
    /// `rule` between definitions of different packs; between two definitions of one pack,
    /// `within_pack` decides instead, and nothing is merged or accumulated
    /// ([`within_pack`](Self::within_pack)).
    Tied {
        /// The rule across packs.
        rule: Box<CombineRule<T>>,
        /// The tie within one pack.
        within_pack: PackTie,
    },
}

/// Which of two definitions of one identifier in one pack wins.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TieWinner {
    /// The first in walk order stays; the later one is dropped.
    First,
    /// The later in walk order replaces the earlier one.
    Last,
}

/// The tie between two definitions of one identifier in one pack
/// ([`CombineRule::within_pack`]): the winner, and the notice raised at the loser.
///
/// Without a tie each rule keeps its own: the later definition with a
/// [`Notice::duplicate`] at the earlier one, and under `BottomWins` the earlier one with the
/// notice at the later one.
///
/// The tie compares the incoming definition with the current effective one only, not with
/// every definition the pack gave before it. The loser lands in the slot's shadowed list, so
/// it is seen in `Registry::layers` under `LoadOptions::keep_shadowed`. Fog and `feature_rules`,
/// whose own combines this tie stands for, drop the loser today; their migration onto it pins
/// `layers()` before and after.
#[derive(Clone, Debug, PartialEq)]
pub struct PackTie {
    /// Which definition wins.
    pub winner: TieWinner,
    /// Raised at the definition that loses.
    pub notice: Option<Notice>,
}

impl PackTie {
    /// The first definition of the pack wins; `notice` at each later one.
    pub fn first(notice: Option<Notice>) -> Self {
        Self {
            winner: TieWinner::First,
            notice,
        }
    }

    /// The later definition of the pack wins; `notice` at the one it replaces.
    pub fn last(notice: Option<Notice>) -> Self {
        Self {
            winner: TieWinner::Last,
            notice,
        }
    }

    /// The winner of `existing` and `incoming`, both of one pack, and the loser.
    fn decide<T>(
        &self,
        existing: Document<T>,
        incoming: Document<T>,
        diagnostics: &mut Diagnostics,
    ) -> Combined<T> {
        match self.winner {
            TieWinner::First => {
                notify(self.notice.as_ref(), &incoming, diagnostics);
                keep(existing, incoming)
            }
            TieWinner::Last => {
                notify(self.notice.as_ref(), &existing, diagnostics);
                keep(incoming, existing)
            }
        }
    }
}

impl<T> CombineRule<T> {
    /// `TopWins` without a notice: the rule of most families.
    pub fn top_wins() -> Self {
        Self::TopWins { shadowed: None }
    }

    /// `BottomWins` with [`Notice::shadowed`].
    pub fn bottom_wins() -> Self {
        Self::BottomWins {
            shadowed: Some(Notice::shadowed()),
        }
    }

    /// `LastLoadedWins` with [`Notice::duplicate`].
    pub fn last_loaded_wins() -> Self {
        Self::LastLoadedWins {
            replaced: Some(Notice::duplicate()),
        }
    }

    /// The rule with `tie` deciding between two definitions of one pack (a tied rule's tie
    /// replaced): fog keeps the first file of a pack, `PackTie::first(Some(Notice::shadowed()))`.
    #[must_use]
    pub fn within_pack(self, tie: PackTie) -> Self {
        let rule = match self {
            Self::Tied { rule, .. } => rule,
            rule => Box::new(rule),
        };
        Self::Tied {
            rule,
            within_pack: tie,
        }
    }

    /// The rule across packs: the rule itself, or a tied rule's own.
    pub(crate) fn base(&self) -> &Self {
        match self {
            Self::Tied { rule, .. } => rule.base(),
            rule => rule,
        }
    }
}

/// Prints the rule's name only (a tied rule as `Tied(<rule>)`).
impl<T> fmt::Debug for CombineRule<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Tied { rule, .. } => return f.debug_tuple("Tied").field(rule).finish(),
            Self::TopWins { .. } => "TopWins",
            Self::BottomWins { .. } => "BottomWins",
            Self::LastLoadedWins { .. } => "LastLoadedWins",
            Self::TopWinsFallback { .. } => "TopWinsFallback",
            Self::EngineVersionMerge { .. } => "EngineVersionMerge",
            Self::FieldMerge { .. } => "FieldMerge",
            Self::Accumulate { .. } => "Accumulate",
            Self::ClaimBeforeValidate { .. } => "ClaimBeforeValidate",
        })
    }
}

/// One loaded file's contribution to a registry, handed to the builder in walk order.
pub enum Contribution<T: Definition> {
    /// A definition of the stable struct.
    Stable(Id<T::Kind>, Document<T>),
    /// A beta definition (`"format_version": "beta"`), which the load allowed (Beta APIs on or
    /// a base-game pack).
    Beta(Id<T::Kind>, Document<T::Beta>),
    /// A document that failed, with its identifier when the loader read one before failing
    /// (`TopWinsFallback` and `ClaimBeforeValidate` use it).
    Rejected(Option<Id<T::Kind>>, Rejected),
}

/// Prints the variant, the identifier and the document's origin, without a `T: Debug` bound.
impl<T: Definition> fmt::Debug for Contribution<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stable(id, document) => f
                .debug_tuple("Stable")
                .field(id)
                .field(document.origin())
                .finish(),
            Self::Beta(id, document) => f
                .debug_tuple("Beta")
                .field(id)
                .field(document.origin())
                .finish(),
            Self::Rejected(id, rejected) => {
                f.debug_tuple("Rejected").field(id).field(rejected).finish()
            }
        }
    }
}

/// The priority rank of a pack: 0 is the top of the stack; the app-data layer is below every
/// pack.
pub(crate) fn rank(pack: &PackRef) -> usize {
    pack.index().unwrap_or(usize::MAX)
}

/// The outcome of combining an incoming definition with the effective one: the new effective
/// one, and the one it drops (listed in `layers`), if any.
pub(crate) struct Combined<T> {
    pub(crate) effective: Option<Document<T>>,
    pub(crate) dropped: Option<Document<T>>,
}

impl<T> CombineRule<T> {
    /// Combines `incoming` with the effective definition of its identifier (`None` for the
    /// first one), raising the rule's findings into `diagnostics`.
    pub(crate) fn combine(
        &self,
        existing: Option<Document<T>>,
        incoming: Document<T>,
        diagnostics: &mut Diagnostics,
    ) -> Combined<T> {
        let Some(existing) = existing else {
            return self.first(incoming, diagnostics);
        };
        let higher = rank(&incoming.origin().pack) < rank(&existing.origin().pack);
        let same_pack = rank(&incoming.origin().pack) == rank(&existing.origin().pack);
        match self {
            Self::TopWins { shadowed: notice } => {
                let (winner, other) = top_wins(existing, incoming, notice.as_ref(), diagnostics);
                keep(winner, other)
            }
            Self::TopWinsFallback { .. } | Self::ClaimBeforeValidate { .. } => {
                let (winner, other) = top_wins(existing, incoming, None, diagnostics);
                keep(winner, other)
            }
            Self::BottomWins { shadowed } => {
                if same_pack {
                    diagnostics.push(Notice::duplicate().at(incoming.origin()));
                    keep(existing, incoming)
                } else if higher {
                    notify(shadowed.as_ref(), &incoming, diagnostics);
                    keep(existing, incoming)
                } else {
                    notify(shadowed.as_ref(), &existing, diagnostics);
                    keep(incoming, existing)
                }
            }
            Self::LastLoadedWins { replaced } => {
                notify(replaced.as_ref(), &existing, diagnostics);
                keep(incoming, existing)
            }
            Self::EngineVersionMerge { decide, merge } => {
                match decide(Some(&existing), &incoming, diagnostics) {
                    EngineVersionDecision::Ignore => keep(existing, incoming),
                    EngineVersionDecision::Replace => keep(incoming, existing),
                    EngineVersionDecision::Merge => Combined {
                        effective: Some(merge_into(existing, incoming, *merge)),
                        dropped: None,
                    },
                }
            }
            Self::FieldMerge { merge } => {
                let merged = if higher || same_pack {
                    // The incoming definition is the higher one: written over the existing.
                    merge_into(existing, incoming, *merge)
                } else {
                    // The incoming definition is lower: it is the base the existing one is
                    // written over; the provenance stays the higher one's.
                    let (base, lower_origin, _, lower_diagnostics, unknown) = incoming.into_parts();
                    let (upper, origin, version, diagnostics, upper_unknown) =
                        existing.into_parts();
                    let mut document = Document::new(base, origin, version, lower_diagnostics)
                        .with_unknown_members(merged_unknown(unknown, upper_unknown));
                    merge(document.value_mut(), upper);
                    document.diagnostics_mut().extend(diagnostics);
                    document.origin_mut().add_contributor(&lower_origin.pack);
                    for pack in &lower_origin.contributors {
                        document.origin_mut().add_contributor(pack);
                    }
                    document
                };
                Combined {
                    effective: Some(merged),
                    dropped: None,
                }
            }
            Self::Accumulate { accumulate } => {
                let (mut winner, other) = top_wins(existing, incoming, None, diagnostics);
                accumulate(winner.value_mut(), other.value());
                winner.origin_mut().add_contributor(&other.origin().pack);
                for pack in &other.origin().contributors {
                    winner.origin_mut().add_contributor(pack);
                }
                keep(winner, other)
            }
            Self::Tied { rule, within_pack } => {
                if same_pack {
                    within_pack.decide(existing, incoming, diagnostics)
                } else {
                    rule.combine(Some(existing), incoming, diagnostics)
                }
            }
        }
    }

    /// The first definition of an identifier: effective, unless an `EngineVersionMerge` family
    /// ignores it.
    fn first(&self, incoming: Document<T>, diagnostics: &mut Diagnostics) -> Combined<T> {
        if let Self::EngineVersionMerge { decide, .. } = self.base()
            && decide(None, &incoming, diagnostics) == EngineVersionDecision::Ignore
        {
            return Combined {
                effective: None,
                dropped: Some(incoming),
            };
        }
        Combined {
            effective: Some(incoming),
            dropped: None,
        }
    }
}

/// `(winner, other)`: the higher-priority definition wins; within one pack the later one, with
/// a `DuplicateIdentifier` information ([`Notice::duplicate`]).
fn top_wins<T>(
    existing: Document<T>,
    incoming: Document<T>,
    notice: Option<&Notice>,
    diagnostics: &mut Diagnostics,
) -> (Document<T>, Document<T>) {
    match rank(&incoming.origin().pack).cmp(&rank(&existing.origin().pack)) {
        Ordering::Equal => {
            diagnostics.push(Notice::duplicate().at(existing.origin()));
            (incoming, existing)
        }
        Ordering::Less => {
            notify(notice, &existing, diagnostics);
            (incoming, existing)
        }
        Ordering::Greater => {
            notify(notice, &incoming, diagnostics);
            (existing, incoming)
        }
    }
}

fn keep<T>(effective: Document<T>, dropped: Document<T>) -> Combined<T> {
    Combined {
        effective: Some(effective),
        dropped: Some(dropped),
    }
}

fn notify<T>(notice: Option<&Notice>, at: &Document<T>, diagnostics: &mut Diagnostics) {
    if let Some(notice) = notice {
        diagnostics.push(notice.at(at.origin()));
    }
}

/// `incoming` merged into `existing`: the value by `merge`, the provenance the higher one's
/// (the incoming one's), with the existing document's pack as a contributor.
fn merge_into<T>(
    existing: Document<T>,
    incoming: Document<T>,
    merge: fn(&mut T, T),
) -> Document<T> {
    let (mut value, existing_origin, _, mut diagnostics, unknown) = existing.into_parts();
    let (incoming_value, mut origin, version, incoming_diagnostics, incoming_unknown) =
        incoming.into_parts();
    merge(&mut value, incoming_value);
    diagnostics.extend(incoming_diagnostics);
    origin.add_contributor(&existing_origin.pack);
    for pack in &existing_origin.contributors {
        origin.add_contributor(pack);
    }
    Document::new(value, origin, version, diagnostics)
        .with_unknown_members(merged_unknown(unknown, incoming_unknown))
}

/// The unknown members of a merged document: those of the base, and over them those of the
/// document merged into it, each by its path ([`Unknown::absorb`]'s rule).
fn merged_unknown(mut base: Unknown, upper: Unknown) -> Unknown {
    base.absorb(&crate::diagnostic::JsonPath::new(), upper);
    base
}

/// How the copies of one path from several packs combine in a `Files` collection.
#[derive(Clone, Debug, PartialEq)]
pub enum FilesRule {
    /// The top pack's copy wins; when it fails the path has no effective file (loot tables,
    /// trade tables: vanilla looks the path up once).
    TopWins,
    /// The top valid copy wins: a copy that fails falls through to the next pack
    /// (structures).
    TopWinsFallback {
        /// Raised at a failing higher copy when a lower one is effective.
        fallback: Option<Notice>,
    },
}
