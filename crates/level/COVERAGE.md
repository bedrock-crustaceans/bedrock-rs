# Format version coverage

What `bedrock_level` reads, writes, and upgrades, by on-disk version. This is
the Phase 7 "version-coverage doc" from `TODO.md`: a table derived from what
is actually implemented and exercised, each row citing the test that proves
it, kept in sync as later phases land. When a task lands or a test's name or
scope changes, update the citing row in the same change -- a coverage claim
with no evidence next to it is the thing this file exists to prevent.

Two keys carry a chunk's format version: `0x2c` (`ChunkVersion`, current
location) and `0x76` (`LegacyVersion`, an older location some worlds still
carry alongside `0x2c`). "Chunk version" below always means the resolved
value (`0x2c` preferred, `0x76` as fallback, `DecodeVersion::select` in
`chunk_version.rs`), not either key in isolation.

## Chunk versions

| Chunk version(s) | Terrain | Biomes | Read | Write | Upgrade (content) | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| 0-2 | `0x30` LegacyTerrain | `0x2d` 2D columns | Implemented | Implemented; retarget within family 0-2 only (see "Write target" below) | Structural: N/A (this family has no `Terrain::Legacy` mutable-write path for the content pass -- see `level_chunk.rs`'s `upgrade_content` doc comment). Content: as for any chunk, per-entry, gated on a provider | Synthetic only -- no fixture predates chunk version 15 (`tests/fixtures/README.md`). `tests/legacy_terrain.rs` (`round_trips_byte_identical_with_a_synthetic_tail`, `order_convention_places_a_known_id_in_the_right_slice_and_local_position`, `nibble_packing_pins_even_low_odd_high`) pins the codec against hand-built columns, not a real record. |
| 3-7 | `0x2f` subchunks, non-paletted (`SubChunkVersion::NonPaletted`) | `0x2d` 2D columns | Implemented | Implemented; retarget within family 3-7 only | Content: per-entry, gated on a provider | Synthetic only -- same gap as above; no fixture carries a non-paletted subchunk (all real corpus subchunks are version 8 or 9, per `tests/fixtures/index.json`'s `subchunk_versions`). `SubChunk::from_disk`'s non-paletted branch is exercised only by hand-built inputs in `crates/level/src/subchunk.rs`'s own unit tests. |
| 8-24 | `0x2f` subchunks, paletted (`Limited`, subchunk version 8) | `0x2d` 2D columns | Implemented | Implemented; retarget within family 8-24 only | Content: per-entry, gated on a provider | Real fixtures: `v1_12` (15), `v1_16` (21), `v1_17_20`/`v1_17_40`/`v1_17_40_superflat` (22 -- see note below). `tests/version_matrix.rs`'s `every_chunk_round_trips_at_both_layers` round-trips every chunk of every one of these fixtures at both layers; `tests/data2d.rs` pins `0x2d` decode/encode against the same corpus; `tests/bit_layers.rs`'s `palette_entries_round_trip_byte_identical_across_all_fixtures` pins every real `0x2f` record's palette bytes. `tests/write_target.rs` retargets a real chunk from this family up to its family ceiling (24) and confirms the shortfall reported when a target outside 8-24 is requested. |
| 25-38 | `0x2f` subchunks, paletted | `0x2b` 3D volumes | Implemented | Implemented; retarget within family 25-255 (see note below on why 39-42 joins this family) | Content: per-entry, gated on a provider | Real fixtures: `v1_17_20_caves_and_cliffs` (25), `v1_17_40_caves_and_cliffs` (31). Versions 26-28 inherit the 25 boundary by nearest-below with no fixture carrying them directly -- see `chunk_version.rs`'s `FACT_BOUNDARIES` doc comment and the Phase 2 "Chunk-version -> format dispatch" TODO entry for the reasoning and the disagreement with one reference table (which places the 2D->3D boundary at 29, not 25). `tests/version_matrix.rs` covers every real chunk at 25 and 31; `tests/biome3d.rs`'s `decodes_and_round_trips_every_record` covers `0x2b` specifically. |
| 39-42 | `0x2f` subchunks, paletted, negative-Y | `0x2b` 3D volumes | Implemented | Implemented; retarget within family 25-255 | Content: per-entry, gated on a provider | Real fixtures: `v1_18`/`v1_18_superflat` (39), `v1_18_30`/`v1_19_30`/`v1_20_81` (40). No fixture carries 41 or 42; both inherit the same 25-boundary facts by nearest-below and are otherwise untested. `tests/version_matrix.rs` covers every real chunk at 39 and 40, including `v1_20_81`'s out-of-nominal-range subchunks (see `tests/height_range.rs`). `tests/write_target.rs`'s `retargeting_a_negative_y_chunk_through_the_offset_window_round_trips_semantically` retargets a real 39/40 chunk down to 27 (inside the 25-28 subchunk key-index offset window) and back, pinning the key-form change each way against a real negative-Y subchunk index. |
| 43+ | Extrapolated from 42's facts | Extrapolated from 42's facts | Best-effort (nearest-below) | Best-effort; same family as 25-255 (nearest-below facts apply) | Content: per-entry, gated on a provider | No fixture; `FormatProfile::extrapolated` flags the fact, `chunk_version.rs`'s `version_past_the_ceiling_extrapolates_the_newest_known_facts` pins the extrapolation logic itself, not real data. |

## Write target: retargeting within a format family, and what is refused

[`Chunk::encode_as`](src/level_chunk.rs) (which `Chunk::encode`/`Chunk::encode_at` and
`Level::put_chunk`/`Level::put_chunk_at` all route through) accepts a target chunk version
and honors it exactly when the target falls inside the chunk's own **format family** --
[`FormatProfile::version_family`](src/chunk_version.rs), the widest span of chunk versions
that share a terrain container and a biome layout. That is precisely the four
`FACT_BOUNDARIES` groups the table above is organized around: `0..=2`, `3..=7`, `8..=24`,
and `25..=255`. The last one deliberately spans both the "25-38" and "39-42" rows above --
negative-Y support (39+) is a height-range fact, not a container-format change, so no fact
boundary exists at 39 and the two rows share one retargeting family. Inside a family,
retargeting is exactly a re-stamp: the `0x2c` byte moves to the target, and every
version-dependent key choice the profile table encodes (currently just the subchunk
key-index offset window at chunk versions 25-28, `crate::key::subchunk_index_to_key`) is
recomputed against the *target*, not the chunk's original decode version -- so a chunk
retargeted into or out of that window gets the correct key form on either side, pinned by
`tests/write_target.rs` against a real fixture.

**Crossing families is refused, not attempted.** A target in a different family from the
chunk's own -- 2D biomes to/from 3D, legacy `0x30` terrain to/from `0x2f` subchunks, or
non-paletted to/from paletted subchunks -- needs constructing a different terrain container
or biome layout, which `encode_as` does not do. What happens instead is **not symmetric**
between the two directions, by design (`fn resolve_retarget` in `src/level_chunk.rs`):
- **Target above the family**: clamps down to the highest version reachable inside the
  chunk's *own* family (its ceiling) -- the useful "modernize an old chunk toward a newer
  target" case.
- **Target below the family**: does **not** clamp up to the family's floor. An earlier
  version of this logic did, but clamping up manufactures a version the caller never asked
  for, and for the `25..=255` family specifically it could silently walk the chunk into the
  25-28 subchunk key-index offset window as an unrequested side effect. The write instead
  stays at the chunk's own current version, unretargeted.

Either way the returned `StructuralWriteReport` names this a
`StructuralShortfall::DifferentFamily`, and **the `0x2c` stamp written is always the
achieved version, never the requested one** -- architecture decision 6's rule that a version
stamp must never claim more than what was actually converted. `tests/write_target.rs`'s
`every_fixture_reports_a_shortfall_and_stamps_the_achieved_version_across_families` pins this
for every fixture that contributes a chunk with terrain, in both directions.

**A second, independent shortfall: offset-window depth.** Even a same-family, in-window
retarget can fail. The 25-28 offset window's `+4` key-byte shift can only represent an
absolute subchunk index down to `-4` (`crate::key::SUBCHUNK_KEY_INDEX_OFFSET_WINDOW_FLOOR`)
-- the Caves-and-Cliffs preview's own extended floor (`-64`). A world with a deeper floor
has subchunks the window cannot represent as a non-negative key byte at all. Real fixture
evidence: `v1_20_81`'s recorded Overworld `subchunk_height_range` starts at `-80` (absolute
index `-5`), while every other `negative_y` fixture (`v1_17_40_caves_and_cliffs`, `v1_18`,
`v1_18_superflat`) bottoms out at exactly `-64` (index `-4`, representable). Retargeting one
of `v1_20_81`'s deep chunks into the window (or a same-family target that happens to land in
it) is refused entirely -- `achieved` stays at the chunk's own current version and
`StructuralShortfall::DepthUnrepresentableInOffsetWindow` is reported, rather than writing a
wrong, sign-flipped key byte while claiming the target was reached. Pinned by
`tests/write_target.rs`'s
`retargeting_into_the_offset_window_is_refused_for_a_chunk_too_deep_to_represent`, which
selects its fixture by scanning `index.json`'s recorded height ranges for the actual deepest
one rather than by name.

The write-target **default** is [`DEFAULT_WRITE_VERSION`](src/chunk_version.rs) -- the newest
chunk version the profile table confirms (tied to `KNOWN_CHUNK_VERSION_CEILING` by a
`const _: () = assert!(...)` beside `INTRODUCED_IN`, so growing the table is what moves the
default forward, not an independent edit), not a value declared standalone.
`tests/write_target.rs`'s `default_write_version_is_the_derived_newest_confirmed_version`
pins that a freshly opened `Level` actually uses it.

Note: the **content** axis (a palette entry's own packed block-state version) is a separate
value stamped in a separate place (a per-entry `int`, not the chunk's `0x2c` byte), reported
by `Chunk::achieved_block_state_version` and carried alongside a `StructuralWriteReport` in
its own field rather than recomputed. See the Phase 3 "upgrade chain" entry in `TODO.md` for
that pass's own status; this section is about the structural axis only.

Note on 22: `v1_17_20` and `v1_17_40`/`v1_17_40_superflat` all report chunk
version 22 despite being different game releases, because 22 is what those
worlds actually wrote -- see `INTRODUCED_IN`'s comment in `chunk_version.rs`
on the 1.17 preview cycle sharing version numbers across builds with no
individually-attributable release.

**Upgrade column reads "N/A" everywhere, honestly.** No content-upgrade
chain exists in this crate yet (`level_chunk.rs`'s module docs): a
`PaletteEntry::Legacy`/`Numeric` entry from an old world is preserved as
read, never converted toward the modern `Modern(BlockDef)` form, and
[`Chunk::achieved_chunk_version`] always reports the source version because
nothing moves a record toward a different one. What Phase 7's upgrade-path
task *can* pin without that chain is structural: `tests/upgrade_path.rs`
shows that a chunk an upgrade left untouched round-trips byte-identical to
its pre-upgrade self (`untouched_chunks_are_byte_identical_across_the_upgrade`,
65/72 real chunks in `v1_18_updated_superflat`), and that a chunk an upgrade
*did* touch matches a freshly-generated modern world's record shape --
subchunk count, palette size, biome storage form
(`upgraded_chunks_match_a_fresh_worlds_structural_shape`) -- while stopping
short of claiming the two hold the same blocks, which needs architecture
decision 9's content-mapping provider.

## Subchunk versions (`0x2f`'s own version byte)

| Version | Layout | Read | Write | Evidence |
| --- | --- | --- | --- | --- |
| 0, 2-7 | `SubChunkVersion::NonPaletted(v)`: raw id + nibble arrays, no palette | Implemented | Implemented | Synthetic only -- see chunk versions 3-7 above; no real fixture. |
| 1 | `SubChunkVersion::Legacy`: single paletted layer, no embedded index byte | Implemented | Implemented | No real fixture carries subchunk version 1 either (corpus is 8/9 only per `index.json`'s `subchunk_versions`); synthetic coverage in `subchunk.rs`'s unit tests. |
| 8 | `SubChunkVersion::Limited`: paletted, layer count, no embedded index byte | Implemented | Implemented | Real fixtures: `v1_12`, `v1_16`, `v1_17_20`, `v1_17_20_caves_and_cliffs`. `tests/version_matrix.rs` and `tests/bit_layers.rs` cover every real record. |
| 9 | `SubChunkVersion::Limitless`: paletted, layer count, embedded `i8` absolute index | Implemented | Implemented | Real fixtures: every remaining one (`v1_17_40` onward). Same test coverage as version 8, plus `tests/subchunk_key_offset.rs` for the version-25-28 key-index offset window this version's embedded byte interacts with. |

Bit widths 0 (`BitArray::Empty`), 1, 2, 3, 4, 5, 6, and 8 are all read and
written; every real layer's header width is minimal for its palette size
(pinned corpus-wide by `tests/bit_layers.rs`). The `0x7f` sentinel is
rejected on block/liquid layers (confirmed zero real occurrences,
`tests/bit_layers.rs`) and accepted on biome layers (`tests/biome3d.rs`).

## Known carve-outs (byte identity, not correctness)

This affects the raw-layer byte-identity guarantee specifically, not the
decoded value -- it is an explicit allowlisted case in
`tests/version_matrix.rs`, never a blanket "skip this key" rule:

- **Packed-index padding bits (widths 3/5/6).** The final partially-filled
  32-bit word of a packed index array can carry non-zero high bits the game
  itself did not always write as zero; a decode-then-encode canonicalizes
  them to zero. 20 real `0x2f` records corpus-wide
  (`tests/bit_layers.rs`'s `packed_index_padding_bits_are_not_always_zero`);
  `0x2b` biome fragments carry the identical finding plus a decode-invisible
  storage-header low bit (`tests/biome3d.rs`). `tests/version_matrix.rs`
  allows a byte mismatch on `SubChunk`/`Biome3d` records only when
  re-decoding both byte strings yields an equal typed value.

This carve-out, and its exact count per fixture at exhaustive coverage (set
`BEDROCK_LEVEL_EXHAUSTIVE_MATRIX=1` to run the matrix against every chunk of
every fixture rather than the default's bounded sample of the two heaviest
-- see `tests/version_matrix.rs`'s module docs), is reported by
`tests/version_matrix.rs -- --nocapture`.

**A second carve-out used to live here and no longer does.** A chunk whose
record group carried neither `0x2c` nor `0x76` used to get a `0x2c`-version-0
stamp written back by `Chunk::encode_at` unconditionally. Review of the
3,489 real corpus chunks this fired on found every one of them a modern
generation-progress stub (finalized state, empty entity lists, spawn areas,
digests -- no terrain, no biomes), for which stamping version 0 asserted
"Alpha-era legacy terrain" falsely, added a record decision 7 says was never
there, and broke decision 3's unmodified-round-trip guarantee. `Chunk::
encode_at` (see its doc comment) now synthesizes a stamp only when the
chunk's version is genuinely known (an explicit [`Chunk::new`] version) or
the chunk actually carries terrain/biome content that needs a version to
decode against; an on-disk chunk with neither a version key nor any
versioned content now stays exactly that way through a round trip.
`tests/version_matrix.rs` pins the fixed count at exactly zero
(`version_stamp_regressions`), and `tests/level_test.rs`'s
`put_chunk_can_introduce_a_brand_new_chunk` continues to cover the case that
still needs a stamp (a freshly constructed chunk with an explicit version).

## `level.dat`

Strict decode succeeds for all thirteen fixtures plus the 1.26 seed world
(`tests/fixtures_test.rs`'s `every_fixture_level_dat_parses_strictly`).
Write round-trips semantically and byte-sorts its keys
(`tests/settings_write.rs`'s `round_trip_is_semantically_equal_for_every_fixture`,
`written_output_is_globally_key_sorted`, `fresh_write_decodes_strictly_for_every_fixture`).

## What the fixture corpus does not reach

Nothing older than chunk version 15 (game version 1.12) exists in the
corpus at all -- versions 0-14 (every legacy-terrain and non-paletted-
subchunk version) are implemented against the format's documented layout
and synthetic inputs only, never against a real game-written record. See
`tests/fixtures/README.md`'s licensing/sourcing notes for why: neither
source project used for the imported fixtures ships a world that old.
