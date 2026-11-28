//! Per-version biome id->name lookup, backed by the generated tables under
//! [`crate::generated`] (architecture decision 12) -- the biome half of Phase 4's
//! biome-resolution task, mirroring `palette.rs`'s block-side lookup.
//!
//! Hand-written: this module defines the shapes the generator emits data into. No biome
//! data lives here -- that is entirely what `crate::generated` holds, and it exists only
//! when the `generated-palettes` feature is enabled.
//!
//! Coverage is narrower than the block palette's: the pinned dumps only carry a numeric
//! biome id from `v1.21.130.3` onward (`xtask/dumps/bedrock-samples/COVERAGE.md`'s biome
//! section), so [`biomes_for_version`] answers `None` for any version outside that
//! narrower set even where a block palette exists for it.

/// One biome id and the name it resolves to, at one game version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BiomeEntry {
    pub id: i32,
    pub name: &'static str,
}

/// One version's full biome id->name table, as generated from one pinned dump.
#[derive(Debug, Clone, Copy)]
pub struct VersionBiomes {
    /// The upstream release tag this was generated from, e.g. `"v1.21.130.3"`.
    pub tag: &'static str,
    /// `(major, minor, patch)` -- same meaning as
    /// [`crate::palette::VersionPalette::game_version`].
    pub game_version: (u8, u8, u8),
    /// Sorted by [`BiomeEntry::id`]; generated that way, so lookup is a binary search.
    pub biomes: &'static [BiomeEntry],
}

impl VersionBiomes {
    /// Looks up a biome's name by id.
    pub fn name(&self, id: i32) -> Option<&'static str> {
        self.biomes
            .binary_search_by_key(&id, |b| b.id)
            .ok()
            .map(|i| self.biomes[i].name)
    }
}

/// Finds the generated biome table whose [`VersionBiomes::game_version`] matches
/// `(major, minor, patch)` exactly.
///
/// `None` when no generated table covers that exact version -- including every version
/// this crate's block palette does cover but whose biome dump carried no numeric id (see
/// this module's doc comment), a distinction a caller that also has a block palette for
/// that version should not paper over by falling back to a neighbouring biome table.
///
/// Only compiled with the `generated-palettes` feature, since [`crate::generated`] does
/// not exist without it.
#[cfg(feature = "generated-palettes")]
pub fn biomes_for_version(major: u8, minor: u8, patch: u8) -> Option<&'static VersionBiomes> {
    crate::generated::BIOMES
        .iter()
        .find(|b| b.game_version == (major, minor, patch))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TABLE: VersionBiomes = VersionBiomes {
        tag: "vTest",
        game_version: (1, 0, 0),
        biomes: &[
            BiomeEntry {
                id: 0,
                name: "minecraft:ocean",
            },
            BiomeEntry {
                id: 1,
                name: "minecraft:plains",
            },
        ],
    };

    #[test]
    fn name_finds_a_known_id() {
        assert_eq!(TABLE.name(1), Some("minecraft:plains"));
    }

    #[test]
    fn name_reports_unknown_as_none() {
        assert_eq!(TABLE.name(9999), None);
    }
}
