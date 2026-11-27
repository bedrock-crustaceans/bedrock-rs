//! The legacy aggregate block names (`wool`, `log`, `stone`, …) a `blocks.json` key may be:
//! below a name's gate version the client splits the entry onto the blocks the name was split
//! into, at or above it the name is an error (unless it is still a block of its own).

use std::collections::HashMap;
use std::sync::LazyLock;

use super::tables::LEGACY_ALIASES;
use bedrock_addon_core::version::FormatVersion;

/// One legacy name, as the client registers it: the block names it was split into, and the
/// `blocks.json` version from which the name counts as flattened.
#[derive(Debug)]
pub(super) struct LegacyAlias {
    /// The legacy name, without `minecraft:`.
    pub(super) name: &'static str,
    /// The legacy numeric block id.
    #[allow(
        dead_code,
        reason = "kept with the table for the reader; nothing compares it"
    )]
    pub(super) legacy_id: u16,
    /// The `blocks.json` version from which the name counts as flattened; `None` for the name
    /// that never does (`lava_cauldron`).
    pub(super) gate: Option<FormatVersion>,
    /// The variant of the first post-split block.
    pub(super) start_variant: u32,
    /// The block that carries the pre-flattening name: the first name the client lists. The name is exempt from the old-name error when this is the name
    /// itself.
    pub(super) default_name: &'static str,
    /// The block of data value 0: what a namespaced key (`minecraft:wool`) resolves to.
    pub(super) aux0: &'static str,
    /// The post-split blocks, each with its position in the list the client names them in: a
    /// block takes the textures `name:<start_variant + position>`. The order is the client's
    /// listing order (`wood`: oak, stripped oak, spruce, …), not the order of the data values.
    /// No block is listed twice: where the client names one again, its first position is kept.
    pub(super) post_split: &'static [(u32, &'static str)],
    /// The block was removed from the game: its entry is dropped instead of split.
    pub(super) removed: bool,
}

impl LegacyAlias {
    /// Whether the client expects the flattened names at `version`: the name has a gate and the
    /// version is not below it.
    pub(super) fn is_flattened_at(&self, version: &super::version::Effective) -> bool {
        self.gate.is_some_and(|gate| !version.is_below(gate))
    }

    /// Whether the legacy name is still the name of its own block, which the client then
    /// resolves directly.
    pub(super) fn is_still_a_block(&self) -> bool {
        self.default_name == self.name
    }
}

/// The alias called `name` (lower case, without `minecraft:`).
pub(super) fn alias(name: &str) -> Option<&'static LegacyAlias> {
    static BY_NAME: LazyLock<HashMap<&'static str, &'static LegacyAlias>> =
        LazyLock::new(|| LEGACY_ALIASES.iter().map(|a| (a.name, a)).collect());
    BY_NAME.get(name).copied()
}

/// Every legacy name.
#[cfg(test)]
pub(super) fn aliases() -> &'static [LegacyAlias] {
    LEGACY_ALIASES
}
