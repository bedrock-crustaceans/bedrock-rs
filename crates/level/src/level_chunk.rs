//! The level layer's decoded chunk model.
//!
//! [`Chunk`] is what [`crate::level::Level::chunk`] hands back: one chunk's
//! terrain and biomes, decoded from its [`ChunkRecords`] group into the
//! crate's normalized in-memory shapes (`SubChunk`/`Layer` for terrain,
//! [`Biomes3d`]/[`HeightMap`] for biomes), with every record this crate does
//! not model carried through untouched (architecture decision 7). A chunk
//! read at any supported version decodes into this same shape -- there is no
//! separate "legacy chunk" type -- which is what architecture decision 4
//! means by automatic upgrading on read: the *representation* is uniform
//! across eras because every era's decoder already targets it (`SubChunk`
//! handles the non-paletted, limited, and limitless layouts uniformly;
//! `Biomes3d`/`HeightMap` cover the 3D/2D split), not because this module
//! runs a content-level translation.
//!
//! # What this wave upgrades, and what it honestly does not
//!
//! Reading a chunk here normalizes its on-disk *shape* to the crate's
//! current model regardless of source version -- a version-3 chunk's
//! non-paletted subchunk and a version-42 chunk's paletted one both end up
//! as a [`SubChunk`]/[`Layer`], and a caller never has to branch on which
//! era produced either. What it does **not** do is *content* upgrading: a
//! legacy numeric block id stays a [`PaletteEntry::Numeric`], a
//! pre-flattening `{name, val}` entry stays a [`PaletteEntry::Legacy`], and
//! a block-state schema is never advanced to a newer version's shape.
//! Architecture decision 9's carve-out is exactly this case -- content
//! mapping needs a provider this wave does not have wired in -- so
//! `PaletteEntry`'s legacy/numeric forms pass through as read, and
//! [`Chunk::encode`] writes each record back in the form it was decoded
//! from rather than at a chosen target version. [`Chunk::achieved_chunk_version`]
//! is the decision-6 reporting hook for this: today it always reports the
//! source version, honestly, because nothing in this wave moves a record
//! toward a different one.
//!
//! # Two-tier block access
//!
//! [`Chunk::block`] is the ergonomic tier: a borrowed [`PaletteEntry`] at a
//! whole-column position, `None` where the position falls in an absent
//! subchunk. [`Chunk::layer`]/[`Chunk::subchunk`]/[`Chunk::terrain`] are the
//! bulk tier: a layer's palette slice and raw index array, for callers that
//! want to scan without paying a per-block palette lookup -- see
//! [`Layer::palette`]/[`Layer::indices`].

use std::collections::BTreeMap;
use std::io::Cursor;

use bedrock_shared::world::dimension::Dimension;

use crate::Greedy;
use crate::actor::ChunkActors;
use crate::biome::{Biomes3d, COLUMNS};
use crate::chunk::ChunkRecords;
use crate::chunk_version::{DecodeVersion, FormatProfile, TerrainLocation};
use crate::db::Database;
use crate::error::Result;
use crate::height_map::HeightMap;
use crate::key::{Key, KeyVariant, subchunk_index_from_key, subchunk_index_to_key};
use crate::legacy_terrain::LegacyTerrain;
use crate::subchunk::{Layer, PaletteEntry, SubChunk, SubChunkVersion};
use crate::types::{BlockPosition, ChunkPosition};

/// A block position spanning a whole chunk column, rather than one
/// subchunk. `x`/`z` stay the usual subchunk-local `0..16` range;
/// [`BlockPosition`] cannot stand in for `y` here because it is a `u8`, and
/// a chunk column's height is signed and can exceed 255 blocks in the
/// extended range -- this type is what [`Chunk::block`] actually takes.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChunkBlockPosition {
    pub x: u8,
    pub y: i32,
    pub z: u8,
}

impl ChunkBlockPosition {
    pub const fn new(x: u8, y: i32, z: u8) -> Self {
        Self { x, y, z }
    }

    /// Splits into the owning subchunk's absolute vertical index and the
    /// position local to that subchunk.
    fn split(self) -> (i32, BlockPosition) {
        let subchunk = self.y.div_euclid(16);
        let local_y = self.y.rem_euclid(16) as u8;
        (subchunk, BlockPosition(self.x, local_y, self.z))
    }
}

impl From<(u8, i32, u8)> for ChunkBlockPosition {
    fn from((x, y, z): (u8, i32, u8)) -> Self {
        Self::new(x, y, z)
    }
}

/// A chunk's terrain, in whichever on-disk shape its format version uses.
///
/// Both forms already normalize into the same building blocks
/// ([`SubChunk`]/[`Layer`]) regardless of era -- see the module docs -- so
/// this enum exists to track *where the record lives on disk* (one `0x30`
/// blob versus per-index `0x2f` records), which [`Chunk::encode_at`] needs
/// to reproduce, rather than to model two different kinds of block data.
#[derive(Debug, Clone, PartialEq)]
pub enum Terrain {
    /// `0x30` LegacyTerrain: one record for the whole column. `None` if the
    /// chunk's format profile calls for this shape but no such record was
    /// actually present (an otherwise-empty chunk stub). The present case is
    /// boxed: a `LegacyTerrain` is over 900 bytes (eight full [`Layer`]s
    /// plus its opaque tail), and this is by far the rarer of `Terrain`'s
    /// two forms -- every chunk version this crate has a real fixture for
    /// uses [`Self::SubChunks`] instead -- so paying an indirection on this
    /// arm keeps every [`Chunk`] built with the common, modern shape from
    /// carrying that size regardless. `Option<Box<_>>` rather than
    /// `Box<Option<_>>`: the outer `Option` still occupies no extra space
    /// (niche-optimized against `Box`'s own non-null pointer), and the
    /// common `None` case allocates nothing at all, unlike a boxed `None`
    /// which would still need a heap slot for the discriminant.
    Legacy(Option<Box<LegacyTerrain>>),
    /// `0x2f` subchunks, keyed by absolute vertical index -- covers both
    /// the non-paletted and paletted on-disk layouts, since both decode
    /// into the same [`SubChunk`] type.
    SubChunks(BTreeMap<i8, SubChunk>),
}

/// A chunk's biome record, in whichever on-disk shape its format version
/// uses. Kept as the whole decoded record (not decomposed into a bare
/// biome enum plus a separate heightmap) because [`HeightMap`] and
/// [`Biomes3d`] each already bundle their heightmap with their biome data
/// on disk, and re-splitting them would need a public constructor neither
/// type has any other reason to expose.
#[derive(Debug, Clone, PartialEq)]
pub enum ChunkBiomes {
    /// `0x2d`: a 2D per-column heightmap paired with one biome id per
    /// column.
    Columns(HeightMap),
    /// `0x2b`: a paletted 3D biome volume per subchunk, with its own
    /// embedded per-column heightmap.
    Volumes(Biomes3d),
}

/// One chunk, decoded from its [`ChunkRecords`] group into this crate's
/// normalized in-memory model. See the module docs for exactly what
/// "normalized" does and does not mean in this wave.
///
/// Per the "Chunk is one type" design, this is the only chunk type: the
/// same value whether it is freshly decoded, being edited in memory, or has
/// never been part of a [`crate::level::Level`] at all ([`Chunk::new`]
/// builds one from scratch). It always carries its preserved unmodelled
/// records ([`Chunk::unmodelled_records`]) and its source version
/// ([`Chunk::chunk_version`]) regardless of which of those is true, so
/// moving one between chunks or levels never silently drops either.
#[derive(Debug, Clone)]
pub struct Chunk {
    position: ChunkPosition,
    dimension: Dimension,
    /// The version this chunk was decoded at (or created at, for
    /// [`Chunk::new`]) -- decision 10's per-chunk source-version tracking.
    source_version: DecodeVersion,
    format: FormatProfile,
    terrain: Terrain,
    biomes: Option<ChunkBiomes>,
    actors: ChunkActors,
    /// Every chunk-scoped record this crate does not model, preserved
    /// verbatim (decision 7). Also carries the on-disk `0x2c`/`0x76`
    /// version keys, which are never taken out of it at all -- this wave
    /// never rewrites them, so leaving them untouched is what makes an
    /// unmodified decode-then-encode reproduce them exactly.
    records: ChunkRecords,
}

impl Chunk {
    /// Builds an empty chunk at `position`/`dimension`, as if newly
    /// generated rather than read from disk. `chunk_version` picks the
    /// on-disk shape [`Chunk::encode`] will target (which terrain location
    /// and biome form its format profile calls for); it carries no
    /// unmodelled records and no terrain or biomes yet.
    pub fn new(position: ChunkPosition, dimension: Dimension, chunk_version: u8) -> Self {
        let format = FormatProfile::for_version(chunk_version);
        let terrain = match format.terrain {
            TerrainLocation::LegacyTerrain => Terrain::Legacy(None),
            TerrainLocation::NonPalettedSubChunks | TerrainLocation::PalettedSubChunks => {
                Terrain::SubChunks(BTreeMap::new())
            }
        };
        Self {
            position,
            dimension,
            source_version: DecodeVersion::Found(chunk_version),
            format,
            terrain,
            biomes: None,
            actors: ChunkActors::default(),
            records: ChunkRecords::new(position, dimension),
        }
    }

    /// Decodes one chunk from its raw record group. `db` is consulted only
    /// to resolve `actorprefix` records a `digp` digest names (see
    /// [`ChunkActors::read`]) -- everything else comes from `records`
    /// alone.
    ///
    /// The terrain and biome records this crate models are taken out of
    /// `records` as they are decoded; everything else -- including the
    /// `0x2c`/`0x76` version keys, `digp`, `0x32`, and every record this
    /// crate has no decoder for -- stays in it untouched, which is what
    /// [`Chunk::unmodelled_records`] returns.
    pub fn decode(mut records: ChunkRecords, db: &Database) -> Result<Self> {
        let position = records.position();
        let dimension = records.dimension();

        let version_0x2c = records
            .get(KeyVariant::ChunkVersion)
            .and_then(|bytes| bytes.first().copied());
        let version_0x76 = records
            .get(KeyVariant::LegacyVersion)
            .and_then(|bytes| bytes.first().copied());
        let source_version = DecodeVersion::select(version_0x2c, version_0x76);
        let format = source_version.format_profile();
        let chunk_version = source_version.chunk_version();

        let actors = ChunkActors::read(&records, db)?;

        let terrain = match format.terrain {
            TerrainLocation::LegacyTerrain => {
                let legacy = records
                    .take(KeyVariant::LegacyTerrain)
                    .map(|bytes| LegacyTerrain::from_disk(&mut Cursor::new(bytes.as_slice())))
                    .transpose()?
                    .map(Box::new);
                Terrain::Legacy(legacy)
            }
            TerrainLocation::NonPalettedSubChunks | TerrainLocation::PalettedSubChunks => {
                // Collected up front: `take` needs `&mut records`, and
                // `records.iter()` borrows it immutably.
                let variants: Vec<KeyVariant> = records
                    .iter()
                    .filter(|(key, _)| matches!(key.data, KeyVariant::SubChunk { .. }))
                    .map(|(key, _)| key.data)
                    .collect();

                let mut subchunks = BTreeMap::new();
                for variant in variants {
                    let KeyVariant::SubChunk { index: key_index } = variant else {
                        unreachable!("filtered to SubChunk variants above")
                    };
                    let bytes = records
                        .take(variant)
                        .expect("variant was just observed present in records");
                    let subchunk = SubChunk::from_disk_greedy(&mut Cursor::new(bytes.as_slice()))?;

                    // A version-9 (Limitless) record's own embedded index is
                    // authoritative over the key's -- see
                    // `subchunk_index_from_key`'s doc comment.
                    let absolute_index = match subchunk.version() {
                        SubChunkVersion::Limitless => subchunk.index(),
                        _ => subchunk_index_from_key(key_index, chunk_version, dimension),
                    };
                    subchunks.insert(absolute_index, subchunk);
                }
                Terrain::SubChunks(subchunks)
            }
        };

        // Presence-driven rather than profile-driven: a chunk actually
        // carrying `0x2b` is read as 3D biomes regardless of what its
        // format profile predicts, matching the corpus finding behind the
        // chunk-version-25 biome boundary (see `chunk_version.rs`'s module
        // docs) that presence, not the version table, is what a real world
        // agrees with when the two could differ.
        let biomes = if let Some(bytes) = records.take(KeyVariant::Biome3d) {
            let volumes = Biomes3d::from_disk::<Greedy, _>(&mut Cursor::new(bytes.as_slice()))?;
            Some(ChunkBiomes::Volumes(volumes))
        } else if let Some(bytes) = records.take(KeyVariant::HeightMap) {
            let columns = HeightMap::from_disk(&mut Cursor::new(bytes.as_slice()))?;
            Some(ChunkBiomes::Columns(columns))
        } else {
            None
        };

        Ok(Self {
            position,
            dimension,
            source_version,
            format,
            terrain,
            biomes,
            actors,
            records,
        })
    }

    /// This chunk's position.
    pub fn position(&self) -> ChunkPosition {
        self.position
    }

    /// This chunk's dimension.
    pub fn dimension(&self) -> Dimension {
        self.dimension
    }

    /// The version this chunk was decoded at, or created at for a chunk
    /// built with [`Chunk::new`]. Decision 10's per-chunk source-version
    /// record: preserved regardless of what a level's write target says.
    pub fn chunk_version(&self) -> u8 {
        self.source_version.chunk_version()
    }

    /// The format facts [`Chunk::decode`] resolved this chunk's version to.
    pub fn format(&self) -> FormatProfile {
        self.format
    }

    /// The version this chunk's terrain and biome records will report as
    /// achieved on the next [`Chunk::encode`]. Decision 6's per-record
    /// achieved-version reporting hook: honestly, in this wave, it always
    /// equals [`Chunk::chunk_version`], because nothing here moves a
    /// record's content toward a different version (see the module docs)
    /// -- a caller must not read this as evidence that a level's write
    /// target was reached.
    pub fn achieved_chunk_version(&self) -> u8 {
        self.source_version.chunk_version()
    }

    /// This chunk's terrain, in whichever on-disk shape it uses.
    pub fn terrain(&self) -> &Terrain {
        &self.terrain
    }

    /// Mutable access to this chunk's terrain -- the bulk-tier write path:
    /// insert, remove, or edit a [`SubChunk`]/[`Layer`] directly (see
    /// [`Layer::set`]) without a per-block round trip through this type.
    pub fn terrain_mut(&mut self) -> &mut Terrain {
        &mut self.terrain
    }

    /// Borrows the subchunk at `index` (absolute vertical index), if this
    /// chunk's terrain is the per-subchunk form and one exists there.
    /// `None` for [`Terrain::Legacy`] terrain, which has no per-subchunk
    /// records at all -- use [`Chunk::layer`] for a form that covers both.
    pub fn subchunk(&self, index: i8) -> Option<&SubChunk> {
        match &self.terrain {
            Terrain::SubChunks(subchunks) => subchunks.get(&index),
            Terrain::Legacy(_) => None,
        }
    }

    /// Borrows one layer (0 = blocks, 1 = liquid where present) of the
    /// subchunk at absolute vertical index `index`, in whichever terrain
    /// form this chunk uses. Legacy terrain only ever has a layer 0.
    pub fn layer(&self, index: i8, layer: usize) -> Option<&Layer> {
        match &self.terrain {
            Terrain::Legacy(legacy) => {
                let legacy = legacy.as_deref()?;
                if layer != 0 {
                    return None;
                }
                let slice = usize::try_from(index).ok()?;
                legacy.slices().get(slice)
            }
            Terrain::SubChunks(subchunks) => subchunks.get(&index)?.get_layer(layer),
        }
    }

    /// The ergonomic block-access tier: resolves `pos` through its owning
    /// subchunk's layer-0 palette and returns a borrow, or `None` if `pos`
    /// falls in a subchunk this chunk has no record for. See the module
    /// docs for the two-tier design and [`PaletteEntry`] for what the
    /// borrowed entry can be (a modern block, or a legacy/numeric entry on
    /// a chunk decoded from an older world).
    ///
    /// Returns `&PaletteEntry`, not `&BlockDef` as the public API sketch's
    /// provisional signature has it: resolving a legacy or numeric entry to
    /// a modern `BlockDef` needs the content-mapping provider (architecture
    /// decision 9), which this wave does not have wired in (see the module
    /// docs' "what this wave upgrades" section) -- `PaletteEntry` is what
    /// this crate can honestly hand back without one. There is likewise no
    /// `set_block` counterpart yet: mutating a block goes through
    /// [`Chunk::terrain_mut`] and [`Layer::set`] directly (which only ever
    /// inserts a modern [`crate::subchunk::BlockDef`] entry) until
    /// provider-backed editing exists to make a fused, always-modern
    /// setter meaningful.
    pub fn block(&self, pos: ChunkBlockPosition) -> Option<&PaletteEntry> {
        let (subchunk, local) = pos.split();
        let index = i8::try_from(subchunk).ok()?;
        self.layer(index, 0)?.get(local)
    }

    /// This chunk's biome/heightmap record, if it has one.
    pub fn biomes(&self) -> Option<&ChunkBiomes> {
        self.biomes.as_ref()
    }

    /// Mutable access to this chunk's biome/heightmap record.
    pub fn biomes_mut(&mut self) -> &mut Option<ChunkBiomes> {
        &mut self.biomes
    }

    /// This chunk's per-column heightmap, from whichever biome record it
    /// carries -- `0x2d`'s own heights (signed) or `0x2b`'s embedded ones
    /// (unsigned) -- widened to a common signed `i32` so the two forms can
    /// share one return type; both are small fixed-size arrays, so this
    /// copies 256 elements rather than borrowing. `None` if this chunk has
    /// neither record.
    pub fn heightmap(&self) -> Option<[i32; COLUMNS]> {
        match self.biomes.as_ref()? {
            ChunkBiomes::Columns(heightmap) => Some((*heightmap.heights()).map(i32::from)),
            ChunkBiomes::Volumes(volumes) => Some((*volumes.heightmap()).map(i32::from)),
        }
    }

    /// This chunk's resolved actor (entity) set -- see [`ChunkActors`] for
    /// what each field means. The underlying `digp`/`0x32` records are
    /// never taken out of [`Chunk::unmodelled_records`], so they always
    /// pass through a read-modify-write untouched regardless of whether a
    /// caller reads this accessor at all.
    pub fn actors(&self) -> &ChunkActors {
        &self.actors
    }

    /// The record group left over after taking out everything this type
    /// models: every chunk-scoped key this crate does not decode, plus the
    /// `0x2c`/`0x76` version stamps (never taken -- see this type's field
    /// docs) and the actor records (borrowed, never taken -- see
    /// [`Chunk::actors`]). This is *not* a full raw copy of the chunk; for
    /// that, read through [`crate::level::Level::raw`] instead.
    pub fn unmodelled_records(&self) -> &ChunkRecords {
        &self.records
    }

    /// Encodes this chunk back to its own [`Chunk::position`]/
    /// [`Chunk::dimension`]. Equivalent to
    /// `self.encode_at(self.position(), self.dimension())`.
    pub fn encode(&self) -> Result<ChunkRecords> {
        self.encode_at(self.position, self.dimension)
    }

    /// Encodes this chunk's terrain, biomes, and preserved records into a
    /// fresh [`ChunkRecords`] group keyed at `position`/`dimension`.
    ///
    /// When `position`/`dimension` match this chunk's own, the preserved
    /// group ([`Chunk::unmodelled_records`]) is reused directly: it still
    /// carries the `take`/`insert` bookkeeping from [`Chunk::decode`], so a
    /// terrain or biome record this chunk no longer has (removed from
    /// [`Chunk::terrain_mut`]/[`Chunk::biomes_mut`] since decoding) is
    /// correctly left out rather than resurrected, and
    /// [`ChunkRecords::write`] deletes its stale on-disk key. Writing at a
    /// *different* position/dimension has no such history to reuse -- the
    /// preserved records are copied over under the new key instead, and it
    /// is the caller's job (see [`crate::level::Level::put_chunk`]) to
    /// clear anything already on disk at the new spot that this encode
    /// does not cover.
    pub fn encode_at(&self, position: ChunkPosition, dimension: Dimension) -> Result<ChunkRecords> {
        let mut records = if position == self.position && dimension == self.dimension {
            self.records.clone()
        } else {
            let mut fresh = ChunkRecords::new(position, dimension);
            for (key, bytes) in self.records.iter() {
                fresh.insert(
                    Key {
                        chunk: position,
                        dimension,
                        data: key.data,
                    },
                    bytes.to_vec(),
                );
            }
            fresh
        };

        // A chunk built with `Chunk::new` (or one whose version keys were
        // otherwise never present) has no `0x2c`/`0x76` record to carry
        // through -- give it one rather than writing a chunk with no
        // version stamp at all.
        if !records.contains(KeyVariant::ChunkVersion)
            && !records.contains(KeyVariant::LegacyVersion)
        {
            records.insert(
                Key {
                    chunk: position,
                    dimension,
                    data: KeyVariant::ChunkVersion,
                },
                vec![self.source_version.chunk_version()],
            );
        }

        let chunk_version = self.source_version.chunk_version();

        match &self.terrain {
            Terrain::Legacy(legacy) => {
                if let Some(legacy) = legacy.as_deref() {
                    let mut out = Cursor::new(Vec::new());
                    legacy.to_disk(&mut out)?;
                    records.insert(
                        Key {
                            chunk: position,
                            dimension,
                            data: KeyVariant::LegacyTerrain,
                        },
                        out.into_inner(),
                    );
                }
            }
            Terrain::SubChunks(subchunks) => {
                for (&index, subchunk) in subchunks {
                    let mut out = Cursor::new(Vec::new());
                    subchunk.to_disk::<Greedy, _>(&mut out)?;
                    let key_index = subchunk_index_to_key(index, chunk_version, dimension);
                    records.insert(
                        Key {
                            chunk: position,
                            dimension,
                            data: KeyVariant::SubChunk { index: key_index },
                        },
                        out.into_inner(),
                    );
                }
            }
        }

        match &self.biomes {
            Some(ChunkBiomes::Columns(heightmap)) => {
                let mut out = Cursor::new(Vec::new());
                heightmap.to_disk(&mut out)?;
                records.insert(
                    Key {
                        chunk: position,
                        dimension,
                        data: KeyVariant::HeightMap,
                    },
                    out.into_inner(),
                );
            }
            Some(ChunkBiomes::Volumes(volumes)) => {
                let mut out = Cursor::new(Vec::new());
                volumes.to_disk(&mut out)?;
                records.insert(
                    Key {
                        chunk: position,
                        dimension,
                        data: KeyVariant::Biome3d,
                    },
                    out.into_inner(),
                );
            }
            None => {}
        }

        Ok(records)
    }
}
