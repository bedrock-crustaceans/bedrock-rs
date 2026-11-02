//! Structures (`.mcstructure`): `Structure`.
//!
//! Structures: `structures/**/*.mcstructure` of a behaviour pack, little-endian NBT read by
//! the game's structure loader. One format (1), so there is no version chain and no `history`.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod load;
mod new;
mod parse;
mod text;
mod validate;
mod write;

#[doc(hidden)]
pub use load::load_files;

use std::collections::BTreeMap;

use indexmap::IndexMap;

/// A structure template (`.mcstructure`): a box of block indices into named palettes, plus the
/// entities saved with it.
///
/// NBT (little-endian, root name empty): `{format_version: 1, size: [x, y, z],
/// structure_world_origin: [x, y, z], structure: {block_indices, entities, palette},
/// neighbor_aware_block_upgrade_version?}`. Every rule of the format fails the whole file;
/// vanilla then tries the same structure in the next pack of the stack.
#[derive(Clone, Debug, PartialEq)]
#[doc(alias = "mcstructure")]
pub struct Structure {
    /// The format (NBT `format_version`, Int): always [`Structure::FORMAT_VERSION`].
    pub format_version: i32,
    /// Which neighbour-aware block upgrades the blocks already had (NBT
    /// `neighbor_aware_block_upgrade_version`, Int, 0 or 1). Absent reads as 0: walls are
    /// re-connected to their neighbours when the structure is placed; 1 places them as stored.
    pub neighbor_aware_block_upgrade_version: Option<i32>,
    /// The size of the box in blocks, x, y, z (NBT `size`, three Ints, none negative).
    pub size: [i32; 3],
    /// Where the box's minimum corner was in the world it was saved from (NBT
    /// `structure_world_origin`, three Ints).
    pub structure_world_origin: [i32; 3],
    /// The blocks, palettes and entities (NBT `structure`).
    pub structure: Contents,
    /// Root tags the format does not define, kept for write-back.
    pub extra: nbtx::Compound,
    /// How the file was laid out (tag order, integer widths, empty-list tags), so an unchanged
    /// structure is written back byte for byte; not part of equality. Public only so that
    /// struct-update syntax (`..Structure::new(size)`) works outside the crate; `Default` is
    /// vanilla's layout.
    #[doc(hidden)]
    pub layout: Layout,
}

/// The blocks, palettes and entities of a [`Structure`] (NBT `structure`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Contents {
    /// Two layers of palette indices, one per block position of the box (NBT
    /// `block_indices`, a list of exactly two lists of Ints). Layer 0 is the block, layer 1
    /// the extra block in the same position (water in a waterlogged block). Positions run z
    /// fastest, then y, then x ([`Structure::index`]); a negative index leaves the world's
    /// block in place, an index past the palette places air. Both layers have the same
    /// length, the box's volume, or are both empty.
    pub block_indices: [Vec<i32>; 2],
    /// The entities, as raw actor NBT (NBT `entities`, a list of compounds; not validated).
    pub entities: Vec<nbtx::Compound>,
    /// The palettes by name (NBT `palette`); [`Structure::DEFAULT_PALETTE`] is the one placed.
    pub palette: IndexMap<String, Palette>,
    /// Tags the format does not define, kept for write-back.
    pub extra: nbtx::Compound,
}

/// One palette of a [`Structure`]: the blocks its indices name and per-position data.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Palette {
    /// The blocks, as raw block NBT `{name, states, version}` (NBT `block_palette`, a list of
    /// compounds). Kept as written: vanilla resolves an entry, upgrading it by its `version`,
    /// only when the structure is placed.
    pub block_palette: Vec<nbtx::Compound>,
    /// Block entities and scheduled ticks by position index (NBT `block_position_data`, a
    /// compound keyed by the index in decimal).
    pub block_position_data: BTreeMap<i32, BlockPositionData>,
    /// Tags the format does not define, kept for write-back.
    pub extra: nbtx::Compound,
}

/// What one position of a [`Palette`] carries besides its block.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlockPositionData {
    /// The block entity, as raw NBT (NBT `block_entity_data`, a compound).
    pub block_entity_data: Option<nbtx::Compound>,
    /// Scheduled block ticks (NBT `tick_queue_data`, a list of compounds).
    pub tick_queue_data: Option<Vec<TickQueueEntry>>,
    /// Tags the format does not define, kept for write-back.
    pub extra: nbtx::Compound,
}

/// One scheduled tick of a block position.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TickQueueEntry {
    /// Ticks until the block ticks (NBT `tick_delay`, Int); absent, or not an Int, reads as 0
    /// ([`TickQueueEntry::tick_delay`]).
    pub tick_delay: Option<i32>,
    /// Tags the format does not define, kept for write-back.
    pub extra: nbtx::Compound,
}

/// The on-disk layout of a loaded [`Structure`], kept so that write-back reproduces the file.
///
/// Empty for a file already in vanilla's layout and for a structure built in code.
#[doc(hidden)]
#[derive(Clone, Default)]
pub struct Layout {
    /// The file's tree, root name included, when it differs from what the writer produces
    /// from the typed value.
    pub(crate) source: Option<std::sync::Arc<nbtx::Named<nbtx::Value>>>,
    /// Bytes after the root tag, which vanilla does not read.
    pub(crate) trailing: Box<[u8]>,
}

/// Layouts never decide equality: two structures with the same content are equal however their
/// files were laid out.
impl PartialEq for Layout {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

/// Only whether a layout is kept.
impl std::fmt::Debug for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Layout")
            .field("kept", &self.source.is_some())
            .finish_non_exhaustive()
    }
}

impl Structure {
    /// The only format vanilla reads.
    pub const FORMAT_VERSION: i32 = 1;
    /// The palette vanilla places.
    pub const DEFAULT_PALETTE: &'static str = "default";
    /// The namespace of a structure file directly under `structures/`.
    pub const DEFAULT_NAMESPACE: &'static str = "mystructure";

    /// [`neighbor_aware_block_upgrade_version`](Self::neighbor_aware_block_upgrade_version),
    /// 0 when absent.
    #[must_use]
    pub fn neighbor_aware_block_upgrade_version(&self) -> i32 {
        self.neighbor_aware_block_upgrade_version.unwrap_or(0)
    }
}

impl TickQueueEntry {
    /// [`tick_delay`](Self::tick_delay), 0 when absent.
    #[must_use]
    pub fn tick_delay(&self) -> i32 {
        self.tick_delay.unwrap_or(0)
    }
}

#[cfg(test)]
mod tests;
