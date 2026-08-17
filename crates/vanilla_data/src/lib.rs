//! Vanilla game-content knowledge for Bedrock worlds: concrete block, item, and biome
//! data that `bedrock_level` deliberately does not contain.
//!
//! `bedrock_level` parses the on-disk format only. Anything that requires knowing what
//! a specific block, item, or biome *is* — the block-entity payload shapes, numeric-id
//! tables, block-state schemas — lives here instead, and is handed back to
//! `bedrock_level` through the `bedrock_level::provider::ContentProvider` trait.

pub mod block_entities;
pub mod palette;

#[cfg(feature = "generated-palettes")]
pub mod generated;

use bedrock_level::provider::{ContentProvider, StateUpgrade, VersionedBlockDef};
#[cfg(feature = "generated-palettes")]
use bedrock_level::provider::MalformedState;
use bedrock_level::subchunk::BlockDef;

/// A `ContentProvider` backed by this crate's vanilla content tables.
///
/// [`legacy_block_state`](ContentProvider::legacy_block_state) and
/// [`numeric_block_state`](ContentProvider::numeric_block_state) always answer `None`:
/// the generated tables (architecture decision 12) are built from the game's own
/// published block-palette dumps, and those dumps only cover the post-flattening,
/// name-plus-properties era (the earliest upstream release that publishes one is well
/// after Bedrock's `nbt-blockstate` format began) — there is no official dump of a
/// pre-flattening `(name, val)` or `(numeric id, data)` table at all, at any version,
/// so this is not a gap the generator can close by running again. Closing it needs a
/// hand-authored supplement layered over the generated data, per architecture decision
/// 12's fallback clause; nothing here provides one yet.
///
/// [`upgrade_block_state`](ContentProvider::upgrade_block_state) answers only the
/// degenerate case the generated data directly supports: `source == target` and a
/// generated palette exists for that exact version. That is not a schema upgrade (no
/// rename, property-remap, or flattening rule is applied — the entry comes back
/// unchanged on success), it is a confirmation that the block name is one the game
/// actually shipped at that version, *and* that every property the entry actually
/// carries is one that block declares there, with a value inside that property's
/// declared domain. An undeclared property key, or a declared key whose value falls
/// outside the domain, is exactly what
/// [`MalformedState`](bedrock_level::provider::MalformedState)'s doc comment describes —
/// a compound that does not match what its own version stamp promises — so it comes back
/// [`StateUpgrade::Malformed`], not
/// [`StateUpgrade::NoEntry`]: the provider recognized enough to know something is
/// wrong, it did not simply lack a rule. This is deliberately not a completeness check:
/// a state that *omits* a property the block declares is still accepted, because the
/// dump does not say a property is mandatory on every instance, only that it is one the
/// block recognizes — see `palette::PropertyKind::accepts` for exactly which NBT tags a
/// value is allowed to arrive under. Cross-version upgrading needs a diff between two
/// versions' generated tables and rules for applying it, which is a later deliverable;
/// asking for one here answers [`StateUpgrade::NoEntry`] rather than guessing.
///
/// Without the `generated-palettes` feature, every lookup here honestly reports
/// "unknown" — the same "no provider entry" outcome architecture decision 9 describes,
/// which is the correct behaviour with no tables compiled in, not an approximation of
/// it.
#[derive(Debug, Default, Clone, Copy)]
pub struct VanillaContent;

impl ContentProvider for VanillaContent {
    fn legacy_block_state(&self, _name: &str, _val: i16) -> Option<VersionedBlockDef> {
        None
    }

    fn numeric_block_state(&self, _id: u8, _data: u8) -> Option<VersionedBlockDef> {
        None
    }

    #[cfg(feature = "generated-palettes")]
    fn upgrade_block_state(&self, state: BlockDef, source: i32, target: i32) -> StateUpgrade {
        if source != target {
            // Cross-version schema diffing is not implemented yet -- see this impl's
            // doc comment.
            return StateUpgrade::NoEntry(state);
        }

        let [major, minor, patch, _revision] = target.to_be_bytes();
        let Some(known) = palette::palette_for_version(major, minor, patch) else {
            return StateUpgrade::NoEntry(state);
        };
        let Some(block) = known.block(&state.name) else {
            return StateUpgrade::NoEntry(state);
        };

        for (key, value) in &state.states {
            if !block.properties.contains(&key.as_str()) {
                let reason = MalformedState(format!(
                    "`{}` does not declare a `{key}` property at version {major}.{minor}.{patch}",
                    state.name
                ));
                return StateUpgrade::Malformed(state, reason);
            }
            // `key` is declared, so this lookup always hits -- `known.property` and
            // `block.properties` are generated from the same dump's two tables, and the
            // generator rejects (`UnknownPropertyRef`) any block that references a
            // property name the property table does not also declare.
            let spec = known.property(key).expect("a block-declared property is always in the version's property table");
            if !spec.kind.accepts(value) {
                let reason = MalformedState(format!(
                    "`{}`'s `{key}` value does not match its declared domain at version \
                     {major}.{minor}.{patch}",
                    state.name
                ));
                return StateUpgrade::Malformed(state, reason);
            }
        }

        StateUpgrade::Upgraded(VersionedBlockDef { block: state, version: target })
    }

    #[cfg(not(feature = "generated-palettes"))]
    fn upgrade_block_state(&self, state: BlockDef, _source: i32, _target: i32) -> StateUpgrade {
        StateUpgrade::NoEntry(state)
    }

    fn biome_name(&self, _id: i32) -> Option<&str> {
        None
    }
}
