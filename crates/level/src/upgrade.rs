//! The content-upgrade pass: reporting types and version-floor constants for
//! [`crate::level_chunk::Chunk::upgrade_content`], the engine that walks a decoded chunk's
//! palette entries and asks an attached [`crate::provider::ContentProvider`] to bring each
//! one forward.
//!
//! This module holds the *shape* of what the pass reports, not the walk itself -- the walk
//! needs `Chunk`'s private position/subchunk arithmetic ([`crate::level_chunk::
//! ChunkBlockPosition::split`]), so it lives in `level_chunk.rs` alongside the type it
//! mutates, and calls back into the constants and report types defined here.
//!
//! # Two version floors, for two different reasons a stamp can be missing
//!
//! A palette entry with no honest version to report falls into one of two cases, and this
//! crate does not conflate them:
//!
//! - A [`crate::subchunk::PaletteEntry::Modern`] entry with [`BlockDef::version`] `None` is
//!   still in the post-flattening, NBT-`states` shape -- it is simply missing its stamp.
//!   [`EARLIEST_BLOCK_STATE_VERSION`] is the honest floor for it: the packed version of
//!   1.13.0, the release the References record as where Bedrock's `nbt-blockstate` palette
//!   format began (see `TODO.md`'s References section). No entry in this format could have
//!   been written by anything older, so treating an absent stamp as "at least this old, quite
//!   possibly newer" -- never as current -- is the honest source to upgrade *from*.
//! - A [`crate::subchunk::PaletteEntry::Legacy`]/[`crate::subchunk::PaletteEntry::Numeric`]
//!   entry that a provider could not resolve to a modern form at all is a different, strictly
//!   older situation: it predates the `nbt-blockstate` format itself, so it is not merely
//!   "unstamped inside the packed scheme," it is outside the packed scheme entirely.
//!   [`PRE_NBT_BLOCKSTATE_VERSION`] (the packed representation of version 0.0.0.0, which
//!   sorts below every real stamp a `Modern` entry can carry) is what such an entry
//!   contributes to [`UpgradeReport::achieved_block_state_version`]'s minimum -- a value low
//!   enough that it never overstates an entry this crate could not even begin to place on the
//!   modern version axis.

use crate::level_chunk::ChunkBlockPosition;
use crate::subchunk::BlockDef;

/// The packed block-state version of 1.13.0 -- the floor a stampless
/// [`crate::subchunk::PaletteEntry::Modern`] entry is upgraded *from*. See the module docs.
///
/// Corpus-untested: across all thirteen imported fixtures there are zero stampless
/// `Modern` entries and no `Modern` stamp observed below 1.13 -- every real palette this
/// crate has ever decoded either has a stamp already, or is `Legacy`/`Numeric` (which does
/// not go through this floor at all; see [`PRE_NBT_BLOCKSTATE_VERSION`]). This constant is
/// therefore exercised only by the synthetic unit test that constructs a stampless entry
/// by hand, not by anything a real fixture has been observed to need.
pub const EARLIEST_BLOCK_STATE_VERSION: i32 = BlockDef::pack_version([1, 13, 0, 0]);

/// The packed representation of version 0.0.0.0 -- what an unresolved
/// [`crate::subchunk::PaletteEntry::Legacy`]/[`crate::subchunk::PaletteEntry::Numeric`] entry
/// contributes to an achieved-version minimum. See the module docs.
pub const PRE_NBT_BLOCKSTATE_VERSION: i32 = BlockDef::pack_version([0, 0, 0, 0]);

/// Why a palette entry the upgrade pass visited was left unconverted.
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub enum UnconvertedReason {
    /// The provider has no rule that applies to this entry at all --
    /// [`crate::provider::StateUpgrade::NoEntry`]/`None` from a legacy or numeric lookup.
    NoEntry,
    /// The provider recognized the entry but rejected it as not matching what its own
    /// version stamp (or name) promises -- [`crate::provider::StateUpgrade::Malformed`].
    /// Carries the provider's own reason string.
    Malformed(String),
}

/// The upgrade pass's outcome for one palette entry.
#[derive(Debug, Clone, PartialEq)]
#[must_use]
pub enum EntryOutcome {
    /// The entry was brought to (or already stood at) this packed block-state version --
    /// the achieved version, per architecture decision 6, which the pass also stamps onto
    /// the entry's own [`BlockDef::version`]. May fall short of the pass's target; see
    /// [`crate::provider::StateUpgrade`]'s doc comment for when that happens.
    Converted(i32),
    /// The entry is unchanged from how it was read. `achieved` is the value this outcome
    /// contributes to [`UpgradeReport::achieved_block_state_version`]'s minimum -- see the
    /// module docs for which floor applies to which case. It is not written back onto the
    /// entry itself: an unconverted entry keeps its own prior stamp (`None` included),
    /// never a target relabelled over state that was never actually moved.
    Unconverted {
        achieved: i32,
        reason: UnconvertedReason,
    },
}

impl EntryOutcome {
    /// Builds the [`Unconverted`](Self::Unconverted) outcome.
    pub(crate) fn unconverted(achieved: i32, reason: UnconvertedReason) -> Self {
        Self::Unconverted { achieved, reason }
    }

    /// The value this outcome contributes to an achieved-version minimum: the achieved
    /// version for [`Converted`](Self::Converted), or the documented floor for
    /// [`Unconverted`](Self::Unconverted).
    pub fn achieved(&self) -> i32 {
        match self {
            Self::Converted(version) => *version,
            Self::Unconverted { achieved, .. } => *achieved,
        }
    }

    /// Whether the provider actually moved this entry forward (or confirmed it was already
    /// there), as opposed to leaving it untouched.
    pub fn was_converted(&self) -> bool {
        matches!(self, Self::Converted(_))
    }
}

/// Where one reported [`EntryOutcome`] came from: a specific palette slot, not a position --
/// the schema-upgrade pass runs once per palette entry, per architecture decision 12's
/// per-palette-entry versioning task, never once per block position.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct EntryLocation {
    /// The subchunk's absolute vertical index.
    pub subchunk: i8,
    /// The layer within that subchunk (0 = blocks, 1 = liquid where present).
    pub layer: usize,
    /// The index into that layer's palette.
    pub palette_index: u16,
}

/// The neighbour-pass outcome for one block position -- see
/// [`crate::provider::ContextResolution`], which this maps from, dropping the resolved
/// [`BlockDef`] itself (already written back into the chunk's terrain by the time a caller
/// sees this) and keeping only what a caller needs to know happened.
#[derive(Debug, Clone, PartialEq)]
#[must_use]
pub enum NeighbourOutcome {
    /// The provider computed this position's neighbour-dependent properties and they were
    /// written back.
    Resolved,
    /// The provider had no rule for this position after all, despite flagging its entry via
    /// [`crate::provider::ContentProvider::needs_neighbours`]. Unchanged.
    NoEntry,
    /// The entry did not match what its name promises. Unchanged. Carries the provider's
    /// reason string.
    Malformed(String),
}

/// The full report [`crate::level_chunk::Chunk::upgrade_content`] produces: every palette
/// entry the schema-upgrade pass visited, plus every position the neighbour pass revisited.
///
/// Kept on the [`crate::level_chunk::Chunk`] itself (see
/// [`crate::level_chunk::Chunk::content_upgrade`]) rather than only returned from
/// `upgrade_content`, because [`crate::level::Level::chunk`] runs the pass automatically
/// (architecture decision 4) with no separate call a caller could catch the return value
/// of -- the report has to be reachable from the `Chunk` alone.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct UpgradeReport {
    entries: Vec<(EntryLocation, EntryOutcome)>,
    neighbours: Vec<(ChunkBlockPosition, NeighbourOutcome)>,
}

impl UpgradeReport {
    pub(crate) fn push_entry(&mut self, location: EntryLocation, outcome: EntryOutcome) {
        self.entries.push((location, outcome));
    }

    pub(crate) fn push_neighbour(
        &mut self,
        position: ChunkBlockPosition,
        outcome: NeighbourOutcome,
    ) {
        self.neighbours.push((position, outcome));
    }

    /// Every palette entry the schema-upgrade pass visited, with where it lives and what
    /// happened to it.
    pub fn entries(&self) -> impl Iterator<Item = (EntryLocation, &EntryOutcome)> {
        self.entries.iter().map(|(loc, outcome)| (*loc, outcome))
    }

    /// Every position the neighbour pass revisited, with what happened.
    pub fn neighbour_resolutions(
        &self,
    ) -> impl Iterator<Item = (ChunkBlockPosition, &NeighbourOutcome)> {
        self.neighbours.iter().map(|(pos, outcome)| (*pos, outcome))
    }

    /// The outcome recorded for the entry at `location`, if the pass visited it.
    pub fn entry(&self, location: EntryLocation) -> Option<&EntryOutcome> {
        self.entries
            .iter()
            .find(|(loc, _)| *loc == location)
            .map(|(_, outcome)| outcome)
    }

    /// This chunk's honestly achieved block-state version: the minimum of every visited
    /// entry's [`EntryOutcome::achieved`] value, per architecture decision 6 -- the chunk is
    /// only as upgraded as its least-upgraded entry. `None` when the pass visited no
    /// entries at all (an empty chunk, or one with no paletted subchunks), which is
    /// different from "achieved the floor": there is nothing to report a version for.
    pub fn achieved_block_state_version(&self) -> Option<i32> {
        self.entries
            .iter()
            .map(|(_, outcome)| outcome.achieved())
            .min()
    }
}
