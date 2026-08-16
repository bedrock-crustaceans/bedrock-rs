//! Vanilla game-content knowledge for Bedrock worlds: concrete block, item, and biome
//! data that `bedrock_level` deliberately does not contain.
//!
//! `bedrock_level` parses the on-disk format only. Anything that requires knowing what
//! a specific block, item, or biome *is* — the block-entity payload shapes, numeric-id
//! tables, block-state schemas — lives here instead, and is handed back to
//! `bedrock_level` through the `bedrock_level::provider::ContentProvider` trait.

pub mod block_entities;

use bedrock_level::provider::{ContentProvider, StateUpgrade, VersionedBlockDef};
use bedrock_level::subchunk::BlockDef;

/// A `ContentProvider` backed by this crate's vanilla content tables.
///
/// The generated tables (architecture decision 12) do not exist yet — that is the
/// `xtask` track of Phase 3 — so every lookup here honestly reports "unknown" rather
/// than guessing, and [`upgrade_block_state`](ContentProvider::upgrade_block_state)
/// always answers [`StateUpgrade::NoEntry`]. This is not a stub standing in for real
/// behaviour: "no provider entry" is architecture decision 9's carve-out, a caller
/// with this provider gets exactly the same honest "kept its prior version" outcome a
/// world read with no provider at all gets, which is the correct behaviour until the
/// tables land, not an approximation of it. As the generator produces tables, the
/// lookups below are where they get wired into the trait — the method bodies change,
/// the signatures and the honest-default shape do not.
#[derive(Debug, Default, Clone, Copy)]
pub struct VanillaContent;

impl ContentProvider for VanillaContent {
    fn legacy_block_state(&self, _name: &str, _val: i16) -> Option<VersionedBlockDef> {
        None
    }

    fn numeric_block_state(&self, _id: u8, _data: u8) -> Option<VersionedBlockDef> {
        None
    }

    fn upgrade_block_state(&self, state: BlockDef, _source: i32, _target: i32) -> StateUpgrade {
        StateUpgrade::NoEntry(state)
    }

    fn biome_name(&self, _id: i32) -> Option<&str> {
        None
    }
}
