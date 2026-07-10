//! Content-mapping hook.
//!
//! This crate parses the LevelDB world format only; it has no knowledge of what a
//! particular block, item, or biome *is*. Anything that requires that knowledge —
//! translating a legacy numeric id, applying a block-state schema upgrade, naming a
//! biome — is delegated to a [`ContentProvider`] supplied by the caller.

/// Supplies the game-content knowledge that reading and upgrading a world can require,
/// but that this crate does not itself contain.
///
/// A world written before the flattening update stores blocks and biomes as small
/// integers rather than string ids. Resolving those integers, and reshaping a block's
/// persisted state to match a newer schema, both require version-specific content
/// tables that live outside this crate. An implementor owns those tables; this crate
/// only calls through the trait when a caller asks it to resolve or upgrade something.
pub trait ContentProvider {
    /// Resolves a legacy numeric block id plus its data value to a string id.
    ///
    /// The data value participates in the lookup because a single numeric id can map
    /// to several string ids depending on it (color and wood variants, for example).
    ///
    /// Returns `None` when the pair is not recognized. Callers that need a value in
    /// that case (rather than treating it as absent) are expected to fall back to a
    /// synthetic namespaced id of their own choosing.
    fn block_name(&self, id: i32, data: i16) -> Option<&str>;

    /// Applies flatten/rename/value-remap rules to a persisted block-state compound,
    /// bringing it from the given on-disk version forward to the newest schema the
    /// provider knows.
    ///
    /// `state` is the raw state compound as read from disk; `version` is the packed
    /// per-block version it was stored under: one byte each of major, minor, patch,
    /// revision, most significant first. The returned compound is the same state
    /// expressed under the provider's newest schema. A provider with no rule for
    /// `version` should return `state` unchanged.
    ///
    /// This is the infallible forward-to-latest subset of state upgrading;
    /// target-version and fallible variants are planned but not yet part of the
    /// contract.
    fn upgrade_block_state(&self, state: nbtx::Value, version: u32) -> nbtx::Value;

    /// Resolves a numeric biome id to its name.
    ///
    /// Returns `None` when the id is not recognized.
    fn biome_name(&self, id: i32) -> Option<&str>;
}
