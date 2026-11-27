//! Chunk-version dispatch: what a chunk's on-disk record layout is, given
//! its stored version byte.
//!
//! A chunk carries its format version under `0x2c` (`ChunkVersion`), and on
//! some worlds also under the older `0x76` (`LegacyVersion`) location for
//! the same value. [`DecodeVersion::select`] resolves the two raw bytes to
//! the one version a read path should decode with, and [`FormatProfile::
//! for_version`] turns a resolved version into the handful of facts that
//! govern which decoder runs: where terrain lives, how biomes are stored,
//! and whether a subchunk key's index byte carries the Caves-and-Cliffs
//! preview offset. This module is pure data and lookup -- it decodes
//! nothing itself.

use std::num::NonZeroUsize;
use std::ops::RangeInclusive;

use bedrock_shared::world::dimension::Dimension;

use crate::key::subchunk_key_index_is_offset;
use crate::version::GameVersion;

/// Where a chunk's terrain lives on disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TerrainLocation {
    /// `0x30` `LegacyTerrain`: one 16x16x128 column blob covering the whole
    /// chunk, split into 8 vertical sections of 16 blocks each. Versions
    /// 0-2.
    LegacyTerrain,
    /// `0x2f` subchunks, each a version byte followed by 4096 raw block ids
    /// and a 2048-nibble metadata array -- no palette. Versions 3-7.
    NonPalettedSubChunks,
    /// `0x2f` subchunks, each a version byte, a layer count, and one or
    /// more paletted block-index layers. Version 8 and every version since.
    PalettedSubChunks,
}

/// How a chunk's biomes are stored on disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BiomeForm {
    /// `0x2d` `HeightMap`: one biome id per column, packed alongside the
    /// 16x16 heightmap.
    Columns2d,
    /// `0x2b` Biome3d: a paletted volume per subchunk section, one section
    /// per vertical 16-block slice of the chunk.
    Volumes3d,
}

/// The newest chunk version this table has a confirmed entry for. A queried
/// version above this is extrapolated: see [`FormatProfile::extrapolated`].
pub const KNOWN_CHUNK_VERSION_CEILING: u8 = 42;

/// The newest chunk version [`crate::level_chunk::Chunk::encode_as`] has a confirmed writer
/// for, and therefore [`crate::level::Level`]'s write-target default (architecture decision
/// 6 in `TODO.md`: "the newest version this crate implements a writer for," moving as
/// writers land rather than a value declared independently of them).
///
/// This equals [`KNOWN_CHUNK_VERSION_CEILING`] deliberately, not a second number a reviewer
/// has to notice must be kept in sync with it by hand: encoding itself is structurally
/// generic over any `u8` (every version resolves to *some* [`FormatProfile`], including a
/// version past the ceiling, via nearest-below inheritance), so "has a writer" cannot mean
/// merely "does not panic" -- that would be true of every `u8` and give a vacuous default.
/// The meaningful cut is "confirmed, not extrapolated" ([`FormatProfile::extrapolated`]),
/// which is exactly what [`KNOWN_CHUNK_VERSION_CEILING`] already marks, and the
/// `const _: () = assert!(...)` beside [`INTRODUCED_IN`] ties that constant's value to the
/// table's own length, so growing [`INTRODUCED_IN`] with a matching version bump is what
/// moves this default forward -- not an independent edit here.
pub const DEFAULT_WRITE_VERSION: u8 = KNOWN_CHUNK_VERSION_CEILING;

/// A fact-boundary: the format facts below hold from `from_version` up to
/// (but not including) the next entry's `from_version`, or forever past the
/// last entry. Resolving a queried version against this table is a
/// nearest-below lookup, which is what gives an in-range version with no
/// exact entry -- and a version past the ceiling -- defined behavior
/// instead of an error.
struct FactBoundary {
    from_version: u8,
    terrain: TerrainLocation,
    biomes: BiomeForm,
}

/// Sorted ascending by `from_version`; the first entry starts at 0 so a
/// nearest-below lookup always finds a match.
///
/// Chunk versions 25 and up store biomes as paletted 3D volumes rather than
/// the flat 2D column form; this crate's fixture corpus confirms 3D biomes
/// are already in use at version 25, the same Caves-and-Cliffs preview
/// window in which a subchunk key's index byte carries the offset [`crate::
/// key::subchunk_index_from_key`] undoes (versions 25-28). The window's own
/// end at 29 changes nothing about biome storage -- 3D biomes stay in use --
/// so it is not a separate fact-boundary entry here.
const FACT_BOUNDARIES: &[FactBoundary] = &[
    FactBoundary {
        from_version: 0,
        terrain: TerrainLocation::LegacyTerrain,
        biomes: BiomeForm::Columns2d,
    },
    FactBoundary {
        from_version: 3,
        terrain: TerrainLocation::NonPalettedSubChunks,
        biomes: BiomeForm::Columns2d,
    },
    FactBoundary {
        from_version: 8,
        terrain: TerrainLocation::PalettedSubChunks,
        biomes: BiomeForm::Columns2d,
    },
    FactBoundary {
        from_version: 25,
        terrain: TerrainLocation::PalettedSubChunks,
        biomes: BiomeForm::Volumes3d,
    },
];

/// The game version each chunk version `0..=42` first appeared in, indexed
/// by chunk version. `None` marks a version this table cannot attach a real
/// release to: 16 and 17 were never observed being written by any release,
/// only ever placeholder-dated in the source this table was checked
/// against, so no game version is recorded for them at all rather than
/// carrying a fabricated one.
///
/// A run of versions in the 1.17 Caves-and-Cliffs preview cycle (23, 24, 26,
/// 27, 28, 30, and 32-38) share a single recorded introducing version
/// (1.17.0) rather than each having its own confirmed release -- every
/// preview build in that cycle bumped the chunk version without a
/// corresponding public release to attach the bump to, so the shared value
/// is the earliest point any of them could have first appeared, not a claim
/// that all of them debuted simultaneously.
/// Ties [`KNOWN_CHUNK_VERSION_CEILING`] to [`INTRODUCED_IN`]'s own length rather than
/// letting the two drift: the array's type below is sized off the constant, so shrinking
/// or growing the table without updating the constant to match fails to compile, and
/// growing the table *with* a matching bump is exactly the "moves as writers land"
/// mechanism [`crate::level::DEFAULT_WRITE_VERSION`] depends on for its own derivation --
/// see that constant's doc comment.
const _: () = assert!(INTRODUCED_IN.len() == KNOWN_CHUNK_VERSION_CEILING as usize + 1);

const INTRODUCED_IN: [Option<(i32, i32, i32, i32)>; KNOWN_CHUNK_VERSION_CEILING as usize + 1] = [
    /* 0  */ Some((0, 9, 0, 0)),
    /* 1  */ Some((0, 9, 2, 0)),
    /* 2  */ Some((0, 9, 5, 0)),
    /* 3  */ Some((0, 17, 0, 0)),
    /* 4  */ Some((0, 18, 0, 0)),
    /* 5  */ Some((0, 18, 0, 0)),
    /* 6  */ Some((1, 2, 0, 0)),
    /* 7  */ Some((1, 2, 0, 0)),
    /* 8  */ Some((1, 3, 0, 0)),
    /* 9  */ Some((1, 8, 0, 0)),
    /* 10 */ Some((1, 9, 0, 0)),
    /* 11 */ Some((1, 10, 0, 0)),
    /* 12 */ Some((1, 11, 0, 0)),
    /* 13 */ Some((1, 11, 1, 0)),
    /* 14 */ Some((1, 11, 2, 0)),
    /* 15 */ Some((1, 12, 0, 0)),
    /* 16 */ None,
    /* 17 */ None,
    /* 18 */ Some((1, 16, 0, 0)),
    /* 19 */ Some((1, 16, 0, 0)),
    /* 20 */ Some((1, 16, 100, 56)),
    /* 21 */ Some((1, 16, 100, 58)),
    /* 22 */ Some((1, 16, 210, 0)),
    /* 23 */ Some((1, 17, 0, 0)),
    /* 24 */ Some((1, 17, 0, 0)),
    /* 25 */ Some((1, 17, 0, 0)),
    /* 26 */ Some((1, 17, 0, 0)),
    /* 27 */ Some((1, 17, 0, 0)),
    /* 28 */ Some((1, 17, 0, 0)),
    /* 29 */ Some((1, 17, 30, 0)),
    /* 30 */ Some((1, 17, 0, 0)),
    /* 31 */ Some((1, 17, 40, 0)),
    /* 32 */ Some((1, 17, 0, 0)),
    /* 33 */ Some((1, 17, 0, 0)),
    /* 34 */ Some((1, 17, 0, 0)),
    /* 35 */ Some((1, 17, 0, 0)),
    /* 36 */ Some((1, 17, 0, 0)),
    /* 37 */ Some((1, 17, 0, 0)),
    /* 38 */ Some((1, 17, 0, 0)),
    /* 39 */ Some((1, 18, 0, 0)),
    /* 40 */ Some((1, 18, 30, 0)),
    /* 41 */ Some((1, 21, 40, 0)),
    /* 42 */ Some((1, 21, 120, 0)),
];

/// The decoded facts a chunk-version byte resolves to: everything the read
/// path needs to pick a terrain decoder, a biome decoder, and a subchunk-key
/// convention, plus provenance for the version itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormatProfile {
    /// The chunk version this profile describes.
    pub chunk_version: u8,
    /// Where this version's terrain lives and in what layout.
    pub terrain: TerrainLocation,
    /// How this version's biomes are stored.
    pub biomes: BiomeForm,
    /// Whether an Overworld subchunk key's index byte carries the offset
    /// [`crate::key::subchunk_index_from_key`] undoes for this version.
    /// Always `false` outside the Overworld regardless of this flag -- call
    /// [`crate::key::subchunk_index_from_key`] (or [`crate::key::Key::
    /// subchunk_absolute_index`]) for the actual per-key conversion, which
    /// also accounts for dimension.
    pub subchunk_key_offset_window: bool,
    /// The game version that first wrote this chunk version, where known.
    /// `None` both for the two chunk versions no release can be attached to
    /// (see [`INTRODUCED_IN`]) and for any version past
    /// [`KNOWN_CHUNK_VERSION_CEILING`], where no release is known at all.
    pub introduced_in: Option<GameVersion>,
    /// `true` if `chunk_version` is past [`KNOWN_CHUNK_VERSION_CEILING`].
    /// `terrain`, `biomes`, and `subchunk_key_offset_window` are still
    /// populated in this case -- with the newest known facts, nearest-below
    /// the same as any other lookup -- but they describe an extrapolation
    /// past what this table actually confirms, not a verified fact about
    /// `chunk_version` itself.
    pub extrapolated: bool,
}

impl FormatProfile {
    /// Resolves `chunk_version` to its format facts.
    ///
    /// Every `u8` value produces a profile: a version inside the known
    /// range with no exact fact-boundary entry inherits the facts of the
    /// nearest lower version that changed them, and a version past
    /// [`KNOWN_CHUNK_VERSION_CEILING`] inherits the newest known facts with
    /// [`FormatProfile::extrapolated`] set, rather than either case being an
    /// error.
    #[expect(
        clippy::missing_panics_doc,
        reason = "the expect is unreachable: FACT_BOUNDARIES has an entry starting at version 0"
    )]
    pub fn for_version(chunk_version: u8) -> Self {
        // `FACT_BOUNDARIES` starts at 0, so this always matches.
        let boundary = FACT_BOUNDARIES
            .iter()
            .rev()
            .find(|boundary| boundary.from_version <= chunk_version)
            .expect("FACT_BOUNDARIES has an entry starting at version 0");

        let extrapolated = chunk_version > KNOWN_CHUNK_VERSION_CEILING;
        let introduced_in = if extrapolated {
            None
        } else {
            INTRODUCED_IN[chunk_version as usize].map(|(major, minor, patch, revision)| {
                GameVersion::new(major, minor, patch, revision, 0)
            })
        };

        Self {
            chunk_version,
            terrain: boundary.terrain,
            biomes: boundary.biomes,
            subchunk_key_offset_window: subchunk_key_index_is_offset(
                chunk_version,
                Dimension::Overworld,
            ),
            introduced_in,
            extrapolated,
        }
    }

    /// The closed range of chunk versions that share `chunk_version`'s terrain container
    /// and biome layout -- the widest span a structural retarget
    /// ([`crate::level_chunk::Chunk::encode_as`]) may move within without converting
    /// between terrain or biome forms (legacy terrain to subchunks, non-paletted to
    /// paletted, 2D biomes to 3D). Derived directly from [`FACT_BOUNDARIES`]: a family
    /// starts at its fact-boundary entry's `from_version` and runs up to (but not
    /// including) the next entry's, or to [`u8::MAX`] for the last, open-ended entry.
    ///
    /// This is exactly the four [`FACT_BOUNDARIES`] entries, so it has four distinct
    /// families for the whole `u8` range: `0..=2`, `3..=7`, `8..=24`, and `25..=255`. The
    /// last one covers both the "paletted 3D" and "paletted 3D, negative-Y" chunk-version
    /// ranges the write-target task and `COVERAGE.md` describe separately (25-38 and
    /// 39-42): negative-Y support is a height-range consequence, not a container-format
    /// change -- no fact boundary exists at 39, and the subchunk key-index offset window
    /// (25-28 only, [`subchunk_key_index_is_offset`]) is a target-dependent detail *within*
    /// this one family, not a family split of its own.
    #[expect(
        clippy::missing_panics_doc,
        reason = "the expect is unreachable: FACT_BOUNDARIES has an entry starting at version 0"
    )]
    pub fn version_family(chunk_version: u8) -> RangeInclusive<u8> {
        let idx = FACT_BOUNDARIES
            .iter()
            .rposition(|boundary| boundary.from_version <= chunk_version)
            .expect("FACT_BOUNDARIES has an entry starting at version 0");
        let start = FACT_BOUNDARIES[idx].from_version;
        let end = FACT_BOUNDARIES
            .get(idx + 1)
            .map_or(u8::MAX, |boundary| boundary.from_version - 1);
        start..=end
    }
}

/// The world-wide `0x2b` fragment count a real write carries for `target_version`, used by
/// [`crate::level_chunk::Chunk::encode_as`]'s 2D-to-3D biome promotion (the 8-24 family
/// stamped at `target_version` and above) to build a fresh [`crate::biome::Biomes3d`] via
/// [`crate::biome::Biomes3d::from_columns`]. This is a **write-time constant per version**,
/// not a per-chunk derivation -- see [`crate::biome::Biomes3d`]'s own doc comment for why a
/// dimension-derived count is wrong for a real record.
///
/// Established by joining each corpus fixture's own chunk version with its own fragment-count
/// histogram (`tests/biomes3d_write.rs`'s `corpus_survey_fragment_counts_vs_subchunk_counts`,
/// which this function's boundaries are pinned against by
/// `fragment_count_for_matches_the_corpus_histogram`):
///
/// | Chunk version | Fixture | Fragment count |
/// | --- | --- | --- |
/// | 25 | `v1_17_20_caves_and_cliffs` | 32 (182 records) |
/// | 31 | `v1_17_40_caves_and_cliffs` | 32 (138 records) |
/// | 39 | `v1_18`, `v1_18_superflat`, `v1_18_updated_superflat` | 25 (2,833 records) |
/// | 40 | `v1_18_30`, `v1_19_30`, `v1_20_81` | 24 (4,642 records; `v1_20_81` also carries 3
/// |    |  | stray records at 25, not modeled here as a target -- see the survey's own doc |
/// |    |  | comment) |
///
/// Versions 25 and 31 agree (32), so the whole 25-38 span resolves to one boundary rather
/// than two. No fixture carries 26-30, 32-38, or 41-42; those resolve by **nearest-below**,
/// the same convention [`FormatProfile::for_version`] already uses for terrain/biome facts:
/// 26-30 and 32-38 join the 25-boundary's 32 (consistent with the confirmed value at 31
/// inside that span), and 41-42 join the 40-boundary's 24. This is stated as an assumption
/// for those nine versions, not a corpus fact -- do not read the table above as covering
/// them.
///
/// Undefined for `target_version < 25` (no chunk this crate promotes ever targets below the
/// 25+ family, and no fixture below 25 writes `0x2b` at all): such a call falls through to
/// the oldest known boundary (32) as a harmless placeholder rather than panicking, since
/// nothing in this crate ever calls it that way.
#[expect(
    clippy::missing_panics_doc,
    reason = "the expect is unreachable: every count is a nonzero literal or the nonzero fallback"
)]
pub fn fragment_count_for(target_version: u8) -> NonZeroUsize {
    const BOUNDARIES: &[(u8, usize)] = &[(25, 32), (39, 25), (40, 24)];
    let count = BOUNDARIES
        .iter()
        .rev()
        .find(|(from, _)| *from <= target_version)
        .map_or(32, |(_, count)| *count);
    NonZeroUsize::new(count).expect("every BOUNDARIES entry is a nonzero literal")
}

/// The outcome of resolving a chunk's decode version from its raw `0x2c`
/// (`ChunkVersion`) and `0x76` (`LegacyVersion`) record bytes.
///
/// This only answers *which version to decode with* -- existence-scanning
/// (does a chunk exist at all) is a different question with a different
/// answer (either key's presence says yes, with no ranking between them)
/// and is out of scope here; `ChunkRecords` already treats both keys as
/// ordinary members of a chunk's record group for that purpose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeVersion {
    /// A version byte was found: `0x2c` if present, `0x76` otherwise.
    Found(u8),
    /// Neither key was present. This is not necessarily malformed data --
    /// the earliest LevelDB-era saves wrote no version key at all, since
    /// chunk versioning did not exist yet -- so a caller that wants to
    /// treat an unversioned chunk as the oldest known format rather than
    /// refuse it can do so explicitly rather than have that choice made
    /// silently here.
    Unversioned,
}

impl DecodeVersion {
    /// Resolves the decode version from a chunk's raw `0x2c` and `0x76`
    /// record bytes (each a chunk version's single stored byte, if the
    /// record is present at all). `0x2c` wins when both are present: on a
    /// world that has been through a converter, `0x76` can hold a fixed
    /// compatibility stamp rather than the chunk's real version, so it is
    /// consulted only when `0x2c` is absent.
    pub fn select(chunk_version_0x2c: Option<u8>, legacy_version_0x76: Option<u8>) -> Self {
        match chunk_version_0x2c.or(legacy_version_0x76) {
            Some(version) => DecodeVersion::Found(version),
            None => DecodeVersion::Unversioned,
        }
    }

    /// The chunk version to decode with. [`DecodeVersion::Unversioned`]
    /// resolves to `0` -- the oldest known chunk version, matching the
    /// baseline `0x30` `LegacyTerrain` / 2D-biome format that predates
    /// version keys existing on disk at all.
    pub fn chunk_version(&self) -> u8 {
        match self {
            DecodeVersion::Found(version) => *version,
            DecodeVersion::Unversioned => 0,
        }
    }

    /// The format facts to decode with, per [`FormatProfile::for_version`]
    /// applied to [`DecodeVersion::chunk_version`].
    pub fn format_profile(&self) -> FormatProfile {
        FormatProfile::for_version(self.chunk_version())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_terrain_boundary_at_2_3() {
        assert_eq!(
            FormatProfile::for_version(2).terrain,
            TerrainLocation::LegacyTerrain
        );
        assert_eq!(
            FormatProfile::for_version(3).terrain,
            TerrainLocation::NonPalettedSubChunks
        );
    }

    #[test]
    fn paletted_terrain_boundary_at_7_8() {
        assert_eq!(
            FormatProfile::for_version(7).terrain,
            TerrainLocation::NonPalettedSubChunks
        );
        assert_eq!(
            FormatProfile::for_version(8).terrain,
            TerrainLocation::PalettedSubChunks
        );
    }

    #[test]
    fn biome_and_key_offset_boundary_at_28_29() {
        let v28 = FormatProfile::for_version(28);
        assert_eq!(v28.biomes, BiomeForm::Volumes3d);
        assert!(v28.subchunk_key_offset_window);

        let v29 = FormatProfile::for_version(29);
        assert_eq!(v29.biomes, BiomeForm::Volumes3d);
        assert!(!v29.subchunk_key_offset_window);
    }

    #[test]
    fn key_offset_window_starts_at_25_not_before() {
        assert!(!FormatProfile::for_version(24).subchunk_key_offset_window);
        assert!(FormatProfile::for_version(25).subchunk_key_offset_window);
    }

    #[test]
    fn gap_version_uses_nearest_below_facts() {
        // 16 has no fact-boundary entry of its own; it must inherit
        // version 8's facts (paletted terrain, 2D biomes) rather than error.
        let profile = FormatProfile::for_version(16);
        assert_eq!(profile.terrain, TerrainLocation::PalettedSubChunks);
        assert_eq!(profile.biomes, BiomeForm::Columns2d);
        assert!(!profile.extrapolated);
        // No release is recorded for this specific version.
        assert_eq!(profile.introduced_in, None);
    }

    #[test]
    fn version_within_known_range_is_not_extrapolated() {
        assert!(!FormatProfile::for_version(KNOWN_CHUNK_VERSION_CEILING).extrapolated);
        assert!(
            FormatProfile::for_version(KNOWN_CHUNK_VERSION_CEILING)
                .introduced_in
                .is_some()
        );
    }

    #[test]
    fn version_past_the_ceiling_extrapolates_the_newest_known_facts() {
        let profile = FormatProfile::for_version(255);
        assert!(profile.extrapolated);
        assert_eq!(profile.introduced_in, None);
        // The newest known facts (from the version-25 boundary) still come
        // through, just marked as an extrapolation rather than confirmed.
        assert_eq!(profile.terrain, TerrainLocation::PalettedSubChunks);
        assert_eq!(profile.biomes, BiomeForm::Volumes3d);
    }

    #[test]
    fn decode_version_prefers_0x2c_over_0x76() {
        assert_eq!(
            DecodeVersion::select(Some(40), Some(7)),
            DecodeVersion::Found(40)
        );
    }

    #[test]
    fn decode_version_falls_back_to_0x76_when_0x2c_absent() {
        assert_eq!(
            DecodeVersion::select(None, Some(15)),
            DecodeVersion::Found(15)
        );
    }

    #[test]
    fn decode_version_uses_0x2c_alone() {
        assert_eq!(
            DecodeVersion::select(Some(9), None),
            DecodeVersion::Found(9)
        );
    }

    #[test]
    fn decode_version_unversioned_when_both_absent() {
        assert_eq!(
            DecodeVersion::select(None, None),
            DecodeVersion::Unversioned
        );
        assert_eq!(DecodeVersion::select(None, None).chunk_version(), 0);
        assert_eq!(
            DecodeVersion::select(None, None).format_profile().terrain,
            TerrainLocation::LegacyTerrain
        );
    }

    #[test]
    fn default_write_version_is_tied_to_the_introduced_in_table_length() {
        // Not a comparison of two literals: `INTRODUCED_IN`'s own array-size type
        // annotation is written in terms of `KNOWN_CHUNK_VERSION_CEILING`, so this
        // assertion (and the `const _` beside the table) is what makes growing the table
        // without bumping the ceiling a compile error rather than a silent drift.
        assert_eq!(DEFAULT_WRITE_VERSION as usize, INTRODUCED_IN.len() - 1);
        assert_eq!(DEFAULT_WRITE_VERSION, KNOWN_CHUNK_VERSION_CEILING);
    }

    #[test]
    fn version_family_groups_match_the_fact_boundaries() {
        assert_eq!(FormatProfile::version_family(0), 0..=2);
        assert_eq!(FormatProfile::version_family(2), 0..=2);
        assert_eq!(FormatProfile::version_family(3), 3..=7);
        assert_eq!(FormatProfile::version_family(7), 3..=7);
        assert_eq!(FormatProfile::version_family(8), 8..=24);
        assert_eq!(FormatProfile::version_family(24), 8..=24);
        assert_eq!(FormatProfile::version_family(25), 25..=255);
        assert_eq!(FormatProfile::version_family(38), 25..=255);
        assert_eq!(FormatProfile::version_family(39), 25..=255);
        assert_eq!(FormatProfile::version_family(42), 25..=255);
        // Past the confirmed ceiling too -- the last family is open-ended.
        assert_eq!(FormatProfile::version_family(255), 25..=255);
    }

    #[test]
    fn version_family_25_38_and_39_42_are_one_family() {
        // The write-target task and COVERAGE.md describe these as two chunk-version
        // ranges, but both are paletted-3D at the container-format level (negative-Y is
        // a height-range fact, not a format-family split) -- see `version_family`'s doc
        // comment.
        let a = FormatProfile::version_family(27);
        let b = FormatProfile::version_family(40);
        assert_eq!(a, b);
    }

    #[test]
    fn fragment_count_for_matches_the_corpus_histogram() {
        // The four confirmed points from `tests/biomes3d_write.rs`'s corpus survey.
        assert_eq!(fragment_count_for(25).get(), 32);
        assert_eq!(fragment_count_for(31).get(), 32);
        assert_eq!(fragment_count_for(39).get(), 25);
        assert_eq!(fragment_count_for(40).get(), 24);
    }

    #[test]
    fn fragment_count_for_unconfirmed_versions_join_the_nearest_below_boundary() {
        // 26-30 and 32-38: no fixture, nearest-below join the 25-boundary's 32.
        for v in [26, 27, 28, 29, 30, 32, 33, 34, 35, 36, 37, 38] {
            assert_eq!(fragment_count_for(v).get(), 32, "version {v}");
        }
        // 41-42: no fixture, nearest-below join the 40-boundary's 24.
        assert_eq!(fragment_count_for(41).get(), 24);
        assert_eq!(fragment_count_for(42).get(), 24);
    }

    #[test]
    fn fragment_count_for_extrapolates_past_the_known_ceiling() {
        assert_eq!(fragment_count_for(255).get(), 24);
    }

    #[test]
    fn decode_version_disagreeing_case_still_prefers_0x2c() {
        // The exact shape a converter-stamped world takes: 0x76 pinned to a
        // fixed compatibility value while 0x2c carries the chunk's real
        // version. See the `converter_stamped_0x76_disagrees_with_0x2c`
        // fixture-driven test for the real on-disk example this models.
        let resolved = DecodeVersion::select(Some(40), Some(7));
        assert_eq!(resolved.chunk_version(), 40);
        assert_eq!(resolved.format_profile().biomes, BiomeForm::Volumes3d);
    }
}
