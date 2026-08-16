//! Content-mapping hook.
//!
//! This crate parses the LevelDB world format only; it has no knowledge of what a
//! particular block, item, or biome *is*. Anything that requires that knowledge —
//! translating a legacy numeric or named id, applying a block-state schema upgrade,
//! resolving neighbour-dependent state, naming a biome — is delegated to a
//! [`ContentProvider`] supplied by the caller.
//!
//! ## The no-provider path
//!
//! A content-dependent upgrade step in the level layer takes `Option<&dyn
//! ContentProvider>` at its call site, not a `ContentProvider` by value — `None` is
//! how a caller opens a world with no game-content knowledge at all (architecture
//! decision 9). `None` and "a provider that answers every lookup with `NoEntry`/
//! `None`" reach the same on-disk outcome: the step is skipped and the affected data
//! keeps its prior version rather than being relabelled as current. They are not the
//! same *fact*, though — one is the caller declining to supply content knowledge, the
//! other is a provider that was supplied and does not cover this particular id, name,
//! or version — and a caller that wants to tell them apart (to warn "no provider
//! configured" once, say, rather than per record) does it by matching on the
//! `Option<&dyn ContentProvider>` itself before ever calling into this trait, not by
//! looking for a distinct outcome from these methods. Nothing on this trait needs to
//! re-encode that distinction; it already exists one level up.

use indexmap::IndexMap;

use crate::subchunk::BlockDef;

/// A block's position relative to another, in whole blocks — the unit
/// [`ContentProvider::resolve_with_neighbours`]'s callback reads at.
///
/// Not bounded to a chunk, a subchunk, or even a single block step: the callback the
/// level layer supplies is free to walk across a chunk boundary to satisfy one, the
/// same way a door's hinge state (stored on the lower half, computed from the upper
/// half at `(0, 1, 0)`) or a wall's centre post (also `(0, 1, 0)`) reads a neighbour
/// that always exists inside the same chunk, while a fence or redstone-wire connection
/// at a chunk edge reads one that may not.
///
/// Named fields rather than the sibling position types' bare tuple struct
/// (`BlockPosition`, `ChunkPosition`): those name a *location*, where `.0`/`.1`/`.2`
/// reads fine once the axis order is known, but a *delta* is read as much for its sign
/// as its axis, and `dy: -1` stays legible at a call site in a way `.1 == -1` does not.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RelativeOffset {
    /// Offset along X, in blocks.
    pub dx: i32,
    /// Offset along Y, in blocks.
    pub dy: i32,
    /// Offset along Z, in blocks.
    pub dz: i32,
}

impl RelativeOffset {
    /// Builds an offset from its three components.
    #[inline]
    pub const fn new(dx: i32, dy: i32, dz: i32) -> Self {
        Self { dx, dy, dz }
    }
}

impl From<(i32, i32, i32)> for RelativeOffset {
    fn from((dx, dy, dz): (i32, i32, i32)) -> Self {
        Self::new(dx, dy, dz)
    }
}

impl From<[i32; 3]> for RelativeOffset {
    fn from([dx, dy, dz]: [i32; 3]) -> Self {
        Self::new(dx, dy, dz)
    }
}

/// A block-state compound a provider was asked to interpret that does not match what
/// its own version stamp, or its own name, promises — decision 11's "malformed", not
/// "unmodelled". The provider recognized enough to know something is wrong (an
/// expected property is missing, a value falls outside the range its schema
/// enumerates for that version); it did not simply lack a rule, which is
/// [`StateUpgrade::NoEntry`]/[`ContextResolution::NoEntry`] instead.
///
/// The message names what was wrong with the compound the provider was handed; it
/// carries no chunk, position, or palette-index context, because the provider does not
/// have any — the caller attaches that when it turns this into a crate-level error
/// naming the record.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct MalformedState(pub String);

/// A block state paired with the schema version it is guaranteed to be expressed at.
///
/// This is the shape [`ContentProvider::legacy_block_state`],
/// [`ContentProvider::numeric_block_state`], and a successful
/// [`ContentProvider::upgrade_block_state`] hand back. The version lives beside
/// `block`, not inside it: [`BlockDef::version`] stays `Option<i32>` because a *stored*
/// entry may genuinely carry no stamp, but a value a provider resolves or upgrades
/// always has a definite one, and putting it in its own non-optional field makes that
/// the type's contract instead of a doc comment a careless implementation could ignore
/// by leaving `block.version` as `None`.
#[derive(Debug, Clone, PartialEq)]
pub struct VersionedBlockDef {
    /// The resolved or upgraded block state.
    pub block: BlockDef,
    /// The schema version `block` is expressed at.
    pub version: i32,
}

/// Outcome of [`ContentProvider::upgrade_block_state`].
///
/// Three-way rather than `Result<Option<_>>` so each branch carries exactly what the
/// caller needs from it. [`Upgraded`](Self::Upgraded) is decision 6's per-record
/// achieved-version report given a shape: the carried
/// [`VersionedBlockDef::version`] is the version the provider actually reached, which
/// may fall short of the requested target when its schema data runs out partway
/// there, so the caller stamps the entry with *that*, never with the target it asked
/// for. [`NoEntry`](Self::NoEntry) is decision 9's carve-out — the provider has no
/// schema rule starting from this entry's source version at all — and hands the
/// original entry straight back unchanged, version stamp included, so nothing is lost
/// and nothing is relabelled. [`Malformed`](Self::Malformed) is decision 11's other
/// kind of problem: also hands the original entry back (a provider that rejects a
/// compound has generally not modified it), paired with what was wrong.
#[derive(Debug, Clone, PartialEq)]
#[must_use]
pub enum StateUpgrade {
    /// The provider brought the entry forward; see this type's doc comment for when
    /// [`VersionedBlockDef::version`] can fall short of the requested target.
    Upgraded(VersionedBlockDef),
    /// The provider has no schema rule that applies starting from this entry's source
    /// version. The entry is returned unchanged.
    NoEntry(BlockDef),
    /// The entry does not match what its version stamp promises. The entry is
    /// returned unchanged, alongside what was wrong with it.
    Malformed(BlockDef, MalformedState),
}

impl StateUpgrade {
    /// The carried block state, regardless of which outcome this is: the upgraded
    /// entry, or the original handed back unchanged.
    pub fn into_block(self) -> BlockDef {
        match self {
            Self::Upgraded(versioned) => versioned.block,
            Self::NoEntry(block) | Self::Malformed(block, _) => block,
        }
    }

    /// The reason this was [`Malformed`](Self::Malformed), or `None` for the other two
    /// outcomes — a way to read it without matching out the full variant.
    pub fn malformed_reason(&self) -> Option<&str> {
        match self {
            Self::Malformed(_, reason) => Some(&reason.0),
            Self::Upgraded(_) | Self::NoEntry(_) => None,
        }
    }
}

/// Outcome of [`ContentProvider::resolve_with_neighbours`].
///
/// Deliberately not [`StateUpgrade`]: this pass runs after schema upgrading has
/// already brought an entry as far forward as versioning can take it (see
/// [`ContentProvider::needs_neighbours`]), so there is no target version to fall short
/// of and no achieved version to report — only whether the neighbour-dependent
/// properties (stair shape, fence/wall connections, and so on) could be computed.
#[derive(Debug, Clone, PartialEq)]
#[must_use]
pub enum ContextResolution {
    /// The provider computed the neighbour-dependent properties and returns the
    /// resolved entry for this one position.
    Resolved(BlockDef),
    /// The provider flagged this entry via [`ContentProvider::needs_neighbours`] but,
    /// asked to actually resolve one, has no rule for it after all. The entry is
    /// returned unchanged.
    NoEntry(BlockDef),
    /// The entry does not match what its name promises. The entry is returned
    /// unchanged, alongside what was wrong with it.
    Malformed(BlockDef, MalformedState),
}

impl ContextResolution {
    /// The carried block state, regardless of which outcome this is.
    pub fn into_block(self) -> BlockDef {
        match self {
            Self::Resolved(block) | Self::NoEntry(block) | Self::Malformed(block, _) => block,
        }
    }

    /// The reason this was [`Malformed`](Self::Malformed), or `None` for the other two
    /// outcomes — a way to read it without matching out the full variant.
    pub fn malformed_reason(&self) -> Option<&str> {
        match self {
            Self::Malformed(_, reason) => Some(&reason.0),
            Self::Resolved(_) | Self::NoEntry(_) => None,
        }
    }
}

/// Supplies the game-content knowledge that reading and upgrading a world can require,
/// but that this crate does not itself contain.
///
/// A world written before the flattening update stores blocks as small integers (a
/// numeric id, or a name plus an opaque `val`) rather than the modern name-plus-
/// properties shape; even a modern world's palette entries can be stamped at an older
/// schema version than the one this crate targets. Resolving those, and reshaping a
/// block's persisted state to match a newer schema — including the state that cannot
/// be resolved without looking at neighbouring blocks — requires version-specific
/// content tables that live outside this crate. An implementor owns those tables; this
/// crate only calls through the trait when a caller asks it to resolve or upgrade
/// something. See the module docs for what a missing `&dyn ContentProvider` means
/// versus a provider that simply has no entry for a given lookup.
pub trait ContentProvider {
    /// Resolves a pre-flattening palette entry — a block name plus its opaque `val`
    /// short, [`PaletteEntry::Legacy`](crate::subchunk::PaletteEntry::Legacy)'s on-disk
    /// shape — to the modern name-plus-properties form.
    ///
    /// The data value participates in the key because one legacy `(name, val)` pair
    /// can split into several modern ids (the wood-type and colour families are the
    /// obvious cases), with properties that differ per branch.
    ///
    /// Returns `None` when the pair is not recognized. On a hit, the returned
    /// [`VersionedBlockDef::version`] is the schema version the mapping corresponds to
    /// — pass it as [`upgrade_block_state`](Self::upgrade_block_state)'s `source` to
    /// continue upgrading toward the crate's current schema. Callers that need a value
    /// when this returns `None` (rather than treating it as absent) are expected to
    /// fall back to a synthetic namespaced id of their own choosing — see architecture
    /// decision 11 on why this crate never substitutes air instead.
    fn legacy_block_state(&self, name: &str, val: i16) -> Option<VersionedBlockDef>;

    /// Resolves a pre-palette numeric block id plus its 4-bit metadata nibble —
    /// [`PaletteEntry::Numeric`](crate::subchunk::PaletteEntry::Numeric)'s on-disk
    /// shape, also the shape `0x30` LegacyTerrain and `0x34` LegacyBlockExtraData
    /// store — to the modern name-plus-properties form.
    ///
    /// Kept as a distinct lookup from [`legacy_block_state`](Self::legacy_block_state)
    /// rather than folded into it: the key is a numeric id, not a string, and the two
    /// eras never share a table. As with the legacy lookup, the data value is part of
    /// the key because one numeric id can split into several modern ids by data value.
    ///
    /// Returns `None` when the pair is not recognized, with the same fallback
    /// expectation as [`legacy_block_state`](Self::legacy_block_state).
    fn numeric_block_state(&self, id: u8, data: u8) -> Option<VersionedBlockDef>;

    /// Applies rename, property-rename, property-value-remap, add-with-default,
    /// removal, and one-to-many flattening rules to a block-state entry, advancing it
    /// from `source` toward `target`.
    ///
    /// `state` is the entry as read (or as already resolved by
    /// [`legacy_block_state`](Self::legacy_block_state)/
    /// [`numeric_block_state`](Self::numeric_block_state)); its own
    /// [`version`](BlockDef::version) field is not consulted — `source` is the
    /// authoritative stamp for it, so a caller that has resolved an absent on-disk
    /// version stamp to a concrete earliest-possible version (as architecture decision
    /// 12's block-state versioning task requires) does not also need to write that
    /// resolution into `state` before calling. `source` and `target` are the packed
    /// `(major << 24) | (minor << 16) | (patch << 8) | revision` word
    /// [`BlockDef::pack_version`] produces.
    ///
    /// See [`StateUpgrade`] for what each outcome means and carries.
    fn upgrade_block_state(&self, state: BlockDef, source: i32, target: i32) -> StateUpgrade;

    /// Reports whether `state` — an entry already brought forward by
    /// [`upgrade_block_state`](Self::upgrade_block_state) — needs the neighbour-aware
    /// pass to reach its final on-disk properties: stair shape, fence/wall
    /// connections, redstone wire routing, double-chest pairing, note-block
    /// instrument, and pre-1.16 stem attachment are the known cases.
    ///
    /// This is the flag half of a two-pass design, not a per-position answer: a
    /// palette entry is shared by every block using it, but two blocks sharing one
    /// entry can need different resolved state depending on what is next to each of
    /// them, so *whether* neighbour access is needed can be decided once per entry
    /// even though the resolution itself cannot. A caller that sees `true` here
    /// revisits every position using this entry individually through
    /// [`resolve_with_neighbours`](Self::resolve_with_neighbours); a caller that sees
    /// `false` uses `state` as-is with no per-position work at all.
    ///
    /// Defaults to `false` — a provider with no neighbour-dependent rules of its own
    /// (or one that has not implemented this pass yet) opts out entirely rather than
    /// having to implement [`resolve_with_neighbours`](Self::resolve_with_neighbours)
    /// only to always return [`ContextResolution::NoEntry`] from it.
    fn needs_neighbours(&self, state: &BlockDef) -> bool {
        let _ = state;
        false
    }

    /// Resolves the neighbour-dependent portion of `state`'s properties for one block
    /// position, given a way to read the already-resolved block at a position offset
    /// from it.
    ///
    /// `neighbour` answers a [`RelativeOffset`] with the block there, already carried
    /// through both the schema-upgrade pass and this same neighbour pass wherever that
    /// neighbour did not itself depend on state further out — it returns `None` when
    /// there is no data at that position (outside the world's built range, or a
    /// neighbour chunk that is not present or not loaded), never a synthetic air, per
    /// architecture decision 11. The lookup may cross a chunk boundary; the pass that
    /// drives it, including cross-chunk access, lives in the level crate, not here —
    /// this trait only defines the shape of the question a provider gets asked.
    ///
    /// Only meaningful to call for an entry [`needs_neighbours`](Self::needs_neighbours)
    /// flagged `true`; the default implementation matches the default `needs_neighbours`
    /// (which never flags anything) by always answering [`ContextResolution::NoEntry`].
    ///
    /// A forward-looking note for whoever builds the revisit pass this trait only
    /// defines the question for: taking `state` by value and returning the resolution
    /// by value, while `neighbour` only ever hands out shared borrows, is what commits
    /// that pass to a collect-then-apply shape — every flagged position's resolution
    /// computed under the neighbours' shared borrow first, none of them written back
    /// until the whole pass is done — rather than resolving and mutating one position
    /// at a time, which a signature that let this call write through `neighbour`
    /// directly could not do while also handing out `&BlockDef` borrows into that same
    /// storage. One consequence of that shape worth knowing going in rather than
    /// rediscovering: a neighbour read during this pass sees the state the
    /// schema-upgrade pass produced, never a same-pass neighbour's already-computed
    /// resolution, since nothing is written back until every flagged position has run.
    fn resolve_with_neighbours<'n>(
        &self,
        state: BlockDef,
        neighbour: &mut dyn FnMut(RelativeOffset) -> Option<&'n BlockDef>,
    ) -> ContextResolution {
        let _ = neighbour;
        ContextResolution::NoEntry(state)
    }

    /// Resolves a numeric biome id to its name.
    ///
    /// Biome ids are numeric on disk in every format this crate reads — one byte per
    /// column in the 2D layout, `i32` palette entries in the 3D one — so this is
    /// purely a presentation-layer lookup, never a format change.
    ///
    /// Returns `None` when the id is not recognized — an addon world's custom biome
    /// registry (`BiomeIdsTable`/`DimensionNameIdTable`, not yet modelled by this
    /// crate) is one source of ids no vanilla table has ever heard of, and a provider
    /// with only vanilla data reports those as unknown rather than guessing.
    fn biome_name(&self, id: i32) -> Option<&str>;
}

/// A `states` map in the shape [`BlockDef::states`] and [`ContentProvider`]'s
/// signatures use — re-exported here so an implementor needs only this module to build
/// one, without reaching into `crate::subchunk` for a type alias.
pub type States = IndexMap<String, nbtx::Value>;

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal in-test provider covering every tri-state outcome this trait can
    /// produce, keyed on fixed inputs rather than a real content table.
    struct TestProvider;

    fn block(name: &str, version: Option<i32>) -> BlockDef {
        BlockDef {
            name: name.to_string(),
            version,
            states: States::new(),
        }
    }

    fn versioned(name: &str, version: i32) -> VersionedBlockDef {
        VersionedBlockDef {
            block: block(name, None),
            version,
        }
    }

    impl ContentProvider for TestProvider {
        fn legacy_block_state(&self, name: &str, val: i16) -> Option<VersionedBlockDef> {
            match (name, val) {
                ("legacy:wood", 0) => Some(versioned("minecraft:oak_planks", 1)),
                _ => None,
            }
        }

        fn numeric_block_state(&self, id: u8, data: u8) -> Option<VersionedBlockDef> {
            match (id, data) {
                (5, 0) => Some(versioned("minecraft:oak_planks", 1)),
                _ => None,
            }
        }

        fn upgrade_block_state(&self, state: BlockDef, source: i32, target: i32) -> StateUpgrade {
            if state.name == "malformed:entry" {
                return StateUpgrade::Malformed(
                    state,
                    MalformedState("missing required property `variant`".to_string()),
                );
            }
            match (source, target) {
                // No rule at all starting from this source version.
                (99, _) => StateUpgrade::NoEntry(state),
                // A rule exists but only reaches an intermediate version.
                (1, 3) => StateUpgrade::Upgraded(versioned(&state.name, 2)),
                // A rule exists and reaches the target exactly.
                (1, 2) => StateUpgrade::Upgraded(versioned(&state.name, 2)),
                _ => StateUpgrade::NoEntry(state),
            }
        }

        fn needs_neighbours(&self, state: &BlockDef) -> bool {
            state.name == "minecraft:oak_stairs"
        }

        fn resolve_with_neighbours<'n>(
            &self,
            state: BlockDef,
            neighbour: &mut dyn FnMut(RelativeOffset) -> Option<&'n BlockDef>,
        ) -> ContextResolution {
            if state.name == "malformed:stairs" {
                return ContextResolution::Malformed(
                    state,
                    MalformedState("facing property out of range".to_string()),
                );
            }
            // The door-hinge/wall-centre-post case this type's doc comment describes:
            // the property this block needs lives one block above it.
            match neighbour(RelativeOffset::new(0, 1, 0)) {
                Some(above) if above.name == "minecraft:air" => {
                    let mut resolved = state;
                    resolved.states.insert(
                        "shape".to_string(),
                        nbtx::Value::String("outer_left".into()),
                    );
                    ContextResolution::Resolved(resolved)
                }
                Some(_) => ContextResolution::NoEntry(state),
                None => ContextResolution::NoEntry(state),
            }
        }

        fn biome_name(&self, id: i32) -> Option<&str> {
            match id {
                1 => Some("minecraft:plains"),
                _ => None,
            }
        }
    }

    #[test]
    fn legacy_lookup_found_and_unknown() {
        let provider = TestProvider;
        let found = provider.legacy_block_state("legacy:wood", 0).unwrap();
        assert_eq!(found.block.name, "minecraft:oak_planks");
        assert_eq!(found.version, 1);

        assert!(provider.legacy_block_state("legacy:wood", 1).is_none());
        assert!(provider.legacy_block_state("legacy:nothing", 0).is_none());
    }

    #[test]
    fn numeric_lookup_found_and_unknown() {
        let provider = TestProvider;
        let found = provider.numeric_block_state(5, 0).unwrap();
        assert_eq!(found.block.name, "minecraft:oak_planks");
        assert_eq!(found.version, 1);

        assert!(provider.numeric_block_state(5, 1).is_none());
        assert!(provider.numeric_block_state(255, 255).is_none());
    }

    #[test]
    fn state_upgrade_reaches_target() {
        let provider = TestProvider;
        let state = block("minecraft:oak_planks", None);
        match provider.upgrade_block_state(state, 1, 2) {
            StateUpgrade::Upgraded(upgraded) => assert_eq!(upgraded.version, 2),
            other => panic!("expected Upgraded, got {other:?}"),
        }
    }

    #[test]
    fn state_upgrade_falls_short_of_target_but_still_reports_progress() {
        let provider = TestProvider;
        let state = block("minecraft:oak_planks", None);
        match provider.upgrade_block_state(state, 1, 3) {
            // The provider only reached version 2, not the requested 3 -- still
            // `Upgraded`, and the achieved version is what the caller stamps the
            // entry with, per `StateUpgrade`'s doc comment.
            StateUpgrade::Upgraded(upgraded) => assert_eq!(upgraded.version, 2),
            other => panic!("expected Upgraded, got {other:?}"),
        }
    }

    #[test]
    fn state_upgrade_no_entry_preserves_original() {
        let provider = TestProvider;
        let state = block("minecraft:oak_planks", Some(7));
        match provider.upgrade_block_state(state.clone(), 99, 5) {
            StateUpgrade::NoEntry(returned) => assert_eq!(returned, state),
            other => panic!("expected NoEntry, got {other:?}"),
        }
    }

    #[test]
    fn state_upgrade_malformed_reports_reason_and_preserves_original() {
        let provider = TestProvider;
        let state = block("malformed:entry", None);
        match provider.upgrade_block_state(state.clone(), 1, 2) {
            StateUpgrade::Malformed(returned, reason) => {
                assert_eq!(returned, state);
                assert!(reason.0.contains("variant"));
            }
            other => panic!("expected Malformed, got {other:?}"),
        }
    }

    #[test]
    fn state_upgrade_into_block_unwraps_every_outcome() {
        let provider = TestProvider;

        let upgraded = provider.upgrade_block_state(block("minecraft:oak_planks", None), 1, 2);
        assert_eq!(upgraded.into_block().name, "minecraft:oak_planks");

        let no_entry = provider.upgrade_block_state(block("minecraft:stone", None), 99, 5);
        assert_eq!(no_entry.into_block().name, "minecraft:stone");

        let malformed = provider.upgrade_block_state(block("malformed:entry", None), 1, 2);
        assert_eq!(malformed.into_block().name, "malformed:entry");
    }

    #[test]
    fn state_upgrade_malformed_reason_is_readable_without_a_full_match() {
        let provider = TestProvider;

        let malformed = provider.upgrade_block_state(block("malformed:entry", None), 1, 2);
        assert!(malformed.malformed_reason().unwrap().contains("variant"));

        let upgraded = provider.upgrade_block_state(block("minecraft:oak_planks", None), 1, 2);
        assert_eq!(upgraded.malformed_reason(), None);
    }

    #[test]
    fn needs_neighbours_flags_only_the_context_dependent_entry() {
        let provider = TestProvider;
        assert!(provider.needs_neighbours(&block("minecraft:oak_stairs", None)));
        assert!(!provider.needs_neighbours(&block("minecraft:stone", None)));
    }

    #[test]
    fn resolve_with_neighbours_reads_the_offset_it_is_given() {
        let provider = TestProvider;
        let air = block("minecraft:air", None);
        let state = block("minecraft:oak_stairs", Some(1));

        let resolved = provider.resolve_with_neighbours(state, &mut |offset| {
            assert_eq!(offset, RelativeOffset::new(0, 1, 0));
            Some(&air)
        });

        match resolved {
            ContextResolution::Resolved(state) => {
                assert_eq!(state.states.get("shape").unwrap(), "outer_left");
            }
            other => panic!("expected Resolved, got {other:?}"),
        }
    }

    #[test]
    fn resolve_with_neighbours_no_entry_when_neighbour_is_absent() {
        let provider = TestProvider;
        let state = block("minecraft:oak_stairs", Some(1));

        let resolved = provider.resolve_with_neighbours(state.clone(), &mut |_offset| None);

        match resolved {
            ContextResolution::NoEntry(returned) => assert_eq!(returned, state),
            other => panic!("expected NoEntry, got {other:?}"),
        }
    }

    #[test]
    fn resolve_with_neighbours_malformed_preserves_original() {
        let provider = TestProvider;
        let state = block("malformed:stairs", Some(1));

        let resolved = provider.resolve_with_neighbours(state.clone(), &mut |_offset| None);

        match resolved {
            ContextResolution::Malformed(returned, reason) => {
                assert_eq!(returned, state);
                assert!(reason.0.contains("facing"));
            }
            other => panic!("expected Malformed, got {other:?}"),
        }
    }

    #[test]
    fn context_resolution_into_block_unwraps_every_outcome() {
        let provider = TestProvider;
        let air = block("minecraft:air", None);
        let stairs = block("minecraft:oak_stairs", Some(1));
        let malformed_stairs = block("malformed:stairs", Some(1));

        let resolved = provider.resolve_with_neighbours(stairs.clone(), &mut |_| Some(&air));
        assert_eq!(resolved.into_block().name, "minecraft:oak_stairs");

        let no_entry = provider.resolve_with_neighbours(stairs, &mut |_| None);
        assert_eq!(no_entry.into_block().name, "minecraft:oak_stairs");

        let malformed = provider.resolve_with_neighbours(malformed_stairs, &mut |_| None);
        assert_eq!(malformed.into_block().name, "malformed:stairs");
    }

    #[test]
    fn context_resolution_malformed_reason_is_readable_without_a_full_match() {
        let provider = TestProvider;
        let stairs = block("minecraft:oak_stairs", Some(1));
        let malformed_stairs = block("malformed:stairs", Some(1));

        let malformed = provider.resolve_with_neighbours(malformed_stairs, &mut |_| None);
        assert!(malformed.malformed_reason().unwrap().contains("facing"));

        let no_entry = provider.resolve_with_neighbours(stairs, &mut |_| None);
        assert_eq!(no_entry.malformed_reason(), None);
    }

    #[test]
    fn default_neighbour_hooks_opt_out() {
        /// A provider that implements only the non-defaulted methods.
        struct Minimal;

        impl ContentProvider for Minimal {
            fn legacy_block_state(&self, _name: &str, _val: i16) -> Option<VersionedBlockDef> {
                None
            }

            fn numeric_block_state(&self, _id: u8, _data: u8) -> Option<VersionedBlockDef> {
                None
            }

            fn upgrade_block_state(
                &self,
                state: BlockDef,
                _source: i32,
                _target: i32,
            ) -> StateUpgrade {
                StateUpgrade::NoEntry(state)
            }

            fn biome_name(&self, _id: i32) -> Option<&str> {
                None
            }
        }

        let provider = Minimal;
        let state = block("minecraft:oak_stairs", Some(1));
        assert!(!provider.needs_neighbours(&state));
        match provider.resolve_with_neighbours(state.clone(), &mut |_offset| None) {
            ContextResolution::NoEntry(returned) => assert_eq!(returned, state),
            other => panic!("expected NoEntry, got {other:?}"),
        }
    }

    #[test]
    fn biome_name_reports_unknown_as_none() {
        let provider = TestProvider;
        assert_eq!(provider.biome_name(1), Some("minecraft:plains"));
        assert_eq!(provider.biome_name(9999), None);
    }
}
