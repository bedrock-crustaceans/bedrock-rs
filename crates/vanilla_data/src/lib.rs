//! Vanilla game-content knowledge for Bedrock worlds: concrete block, item, and biome
//! data that `bedrock_level` deliberately does not contain.
//!
//! `bedrock_level` parses the on-disk format only. Anything that requires knowing what
//! a specific block, item, or biome *is* — the block-entity payload shapes, numeric-id
//! tables, block-state schemas — lives here instead, and is handed back to
//! `bedrock_level` through the `bedrock_level::provider::ContentProvider` trait.

pub mod block_entities;

use bedrock_level::provider::ContentProvider;

/// A `ContentProvider` backed by this crate's vanilla content tables.
///
/// The tables are not yet populated; every lookup currently reports "unknown" and
/// state upgrades are the identity. As numeric-id and block-state-schema tables are
/// added to this crate, this type is where they get wired into the trait.
#[derive(Debug, Default, Clone, Copy)]
pub struct VanillaContent;

impl ContentProvider for VanillaContent {
    fn block_name(&self, _id: i32, _data: i16) -> Option<&str> {
        None
    }

    fn upgrade_block_state(&self, state: nbtx::Value, _version: u32) -> nbtx::Value {
        state
    }

    fn biome_name(&self, _id: i32) -> Option<&str> {
        None
    }
}
