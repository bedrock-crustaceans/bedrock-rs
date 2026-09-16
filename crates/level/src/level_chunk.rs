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
use crate::actor::{
    ActorStorageKey, ActorWriteSet, ChunkActors, decode_actor_digest, encode_actor_digest,
};
use crate::biome::{Biomes3d, COLUMNS};
use crate::chunk::ChunkRecords;
use crate::chunk_version::{DecodeVersion, FormatProfile, TerrainLocation};
use crate::db::Database;
use crate::error::{Error, Result};
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

/// See [`Chunk::heightmap_origin`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum HeightmapOrigin {
    /// The heightmap is the value this chunk was decoded with (or has none
    /// at all), untouched.
    Preserved,
    /// The heightmap was recomputed from this chunk's own terrain.
    Recomputed,
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
    /// The `digp` digest exactly as decoded -- every storage key it named,
    /// dangling entries included, in on-disk order. `Chunk::decode`'s own
    /// snapshot, kept alongside `actors` (which drops dangling entries
    /// entirely) purely so [`Chunk::encode_actor_writes`] can diff "what the
    /// digest used to name" against "what it names now" and GC exactly the
    /// keys that fell out. Empty for a chunk with no `digp` record at all,
    /// including every [`Chunk::new`] chunk.
    actor_digest_at_decode: Vec<ActorStorageKey>,
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
            actor_digest_at_decode: Vec::new(),
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
        // Independent of `actors` deliberately: `ChunkActors::read` already
        // drops a dangling entry out of `resolved`, but the GC diff in
        // `encode_actor_writes` needs the digest exactly as it was written,
        // dangling entries included, to know which keys the *old* digest
        // vouched for.
        let actor_digest_at_decode = records
            .get(KeyVariant::ActorDigest)
            .map(decode_actor_digest)
            .transpose()?
            .unwrap_or_default();

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
            actor_digest_at_decode,
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

    /// Whether this chunk's heightmap reflects its stored value or a fresh
    /// recomputation from terrain -- the `achieved_chunk_version`-shaped
    /// reporting hook decision 13's heightmap row calls for ("recompute
    /// with a provider, else preserve").
    ///
    /// Always [`HeightmapOrigin::Preserved`] in this wave, honestly: nothing
    /// in this crate recomputes a heightmap yet. Two things are missing,
    /// both outside this module's own file ownership so they are reported
    /// rather than built here: (1) [`crate::provider::ContentProvider`] has
    /// no "is this entry air" query -- classifying a legacy/numeric entry
    /// needs content knowledge this crate does not have, per decision 9,
    /// and a name-based check for modern entries alone would still be
    /// guessing at a fixed string this crate is not supposed to know,
    /// per decision 8; (2) [`HeightMap`]/[`Biomes3d`] expose their stored
    /// heights only by shared reference (see [`Chunk::heightmap`]), with no
    /// setter or reconstruction path to write a recomputed array back into
    /// either. This method exists now so the reporting shape is in place
    /// once both land, the same way [`Chunk::achieved_chunk_version`]
    /// existed before anything moved a record toward a different version.
    pub fn heightmap_origin(&self) -> HeightmapOrigin {
        HeightmapOrigin::Preserved
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
    /// what each field means. The `0x32` legacy list is never taken out of
    /// [`Chunk::unmodelled_records`], so it always passes through a
    /// read-modify-write untouched regardless of whether a caller reads
    /// this accessor at all. `digp` is different: [`Chunk::encode_at`]
    /// regenerates it from this value's resolved set on every encode (see
    /// [`Chunk::actors_mut`]), so an edit made through the mutable accessor
    /// is what a subsequent encode actually writes.
    pub fn actors(&self) -> &ChunkActors {
        &self.actors
    }

    /// Mutable access to this chunk's actor set -- the write-side surface
    /// for [`ChunkActors::add`]/[`ChunkActors::remove`]. An edit made here
    /// has no effect on disk until this chunk is encoded (regenerating
    /// `digp` -- see [`Chunk::encode_at`]) and [`Chunk::encode_actor_writes`]
    /// is applied (writing/deleting the affected `actorprefix` records) --
    /// both of which `crate::level::Level::put_chunk` does for a caller
    /// going through it, in one call.
    pub fn actors_mut(&mut self) -> &mut ChunkActors {
        &mut self.actors
    }

    /// Moves this chunk's legacy inline `0x32` entities into the modern
    /// `digp`/`actorprefix` scheme, pairing each legacy entity (in on-disk
    /// order) with the storage key `keys` yields for it and clearing the
    /// `0x32` record on the next encode.
    ///
    /// An explicit opt-in, not something [`Chunk::decode`]/[`Chunk::encode`]
    /// ever does on their own: this format's storage key is derived from
    /// the entity's `UniqueID` (see `crate::actor`'s module docs), a
    /// derivation this crate does not implement, so `keys` must supply one
    /// per legacy entity from wherever the caller can produce or has
    /// already computed it -- there is no honest way to synthesize one
    /// here. This also does not address *when* migrating is appropriate:
    /// the digest scheme is understood to be tied to a chunk-version-era
    /// boundary, and this crate has no write-target machinery yet (see the
    /// Phase 5 TODO entry) to know whether this chunk's own version is one
    /// the digest scheme applies to -- that judgment stays the caller's.
    ///
    /// Errors (without mutating anything) if `keys` does not yield exactly
    /// one key per legacy entity; on success, returns the number migrated.
    pub fn migrate_legacy_entities(
        &mut self,
        keys: impl IntoIterator<Item = ActorStorageKey>,
    ) -> Result<usize> {
        let keys: Vec<ActorStorageKey> = keys.into_iter().collect();
        if keys.len() != self.actors.legacy.len() {
            return Err(Error::Invalid(
                "migrate_legacy_entities: key count does not match legacy entity count",
            ));
        }

        let legacy = std::mem::take(&mut self.actors.legacy);
        let migrated = legacy.len();
        for (storage_key, value) in keys.into_iter().zip(legacy) {
            self.actors.add(storage_key, value);
        }
        self.records.take(KeyVariant::Entity);

        Ok(migrated)
    }

    /// Computes the `actorprefix` writes and deletions this chunk's actor
    /// edits require -- the counterpart to [`Chunk::encode_at`] regenerating
    /// `digp`, covering the part of decision 13's actor-digest policy that
    /// [`ChunkRecords`] cannot carry (`actorprefix` keys have no chunk
    /// coordinates, so they can never be members of that group -- see
    /// `crate::chunk`'s module docs).
    ///
    /// **This is not applied by [`Chunk::encode`]/[`Chunk::encode_at`] or by
    /// writing the resulting [`ChunkRecords`].** [`crate::level::Level::put_chunk`]
    /// calls this and applies the result for you -- a caller going through
    /// it never needs to touch this method at all. It matters directly only
    /// for a caller driving [`Chunk::encode_at`] on its own, outside a
    /// `Level` (against the raw layer, or some other database entirely):
    /// that caller must call this too and apply the returned
    /// [`ActorWriteSet`] to the same database itself, or added/removed
    /// actors will not actually reach disk (`digp` will point at records
    /// that were never written, or stop pointing at ones that were never
    /// deleted).
    pub fn encode_actor_writes(&self) -> Result<ActorWriteSet> {
        self.actors.write_plan(&self.actor_digest_at_decode)
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
    ///
    /// # The version stamp is written only where there is something real to stamp
    ///
    /// A record group can arrive at this point missing both `0x2c` and
    /// `0x76` for two structurally different reasons, and they get
    /// different treatment because only one of them is asserting a fact
    /// about this chunk's version:
    ///
    /// - [`Chunk::new`] always knows a real, explicit version --
    ///   [`DecodeVersion::Found`], the version the caller passed in -- it
    ///   simply has not written it into a group yet, because a group built
    ///   from scratch starts with nothing in it. Writing that explicit
    ///   version is not a claim about anything; it is the version the
    ///   caller already chose. This case always gets its stamp, independent
    ///   of whether the chunk has any terrain or biomes yet -- a caller
    ///   building a chunk up piece by piece should not have its version
    ///   stamp appear and disappear as it adds and removes content.
    /// - A **decoded chunk whose group never had either key**
    ///   ([`DecodeVersion::Unversioned`]) is different: nothing observed on
    ///   disk said what version this chunk is. Real worlds carry exactly
    ///   this shape for generation-progress stubs (a `digp`, a
    ///   finalized-state byte, hardcoded spawn areas -- administrative
    ///   records with no terrain or biome data at all, never a legacy
    ///   pre-versioning chunk in the modern corpus). Synthesizing `0x2c = 0`
    ///   for one of those would assert "Alpha-era legacy terrain" for a
    ///   chunk that carries none, add a record decision 7 says was never
    ///   there, and break the byte-identical unmodified-round-trip
    ///   guarantee decision 3 makes for exactly this case. This crate has no
    ///   way to tell "genuinely versionless administrative stub" apart from
    ///   "genuinely pre-versioning legacy terrain" except by looking at
    ///   whether real terrain or biomes are actually present -- so an
    ///   `Unversioned` chunk only gets a synthesized stamp when it has real
    ///   terrain or biome content to decode that stamp against; an
    ///   `Unversioned` chunk with neither stays exactly as versionless on
    ///   the way out as it was on the way in.
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

        let has_versioned_content = matches!(&self.terrain, Terrain::Legacy(Some(_)))
            || matches!(&self.terrain, Terrain::SubChunks(subchunks) if !subchunks.is_empty())
            || self.biomes.is_some();
        let should_stamp_version = match self.source_version {
            DecodeVersion::Found(_) => true,
            DecodeVersion::Unversioned => has_versioned_content,
        };

        // See this method's doc comment for why the stamp is conditioned on
        // `should_stamp_version` rather than written unconditionally for
        // any group missing both version keys.
        if should_stamp_version
            && !records.contains(KeyVariant::ChunkVersion)
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

        // Decision 13: `digp` is a recomputed index, not a preserved cache
        // -- regenerate it from this chunk's own resolved actor set rather
        // than carrying the on-disk bytes through. For an untouched chunk
        // this reproduces the original bytes exactly: `ChunkActors::read`
        // builds `resolved` in on-disk digest order, and the real corpus
        // carries no dangling entries to drop (see `crate::actor`'s module
        // docs), so recomputing is a no-op in content even though it is not
        // a no-op in code. A chunk edited through `Chunk::actors_mut` gets
        // the digest its new actor set actually implies, dangling entries
        // included, none of which survive being recomputed from a set that
        // never had them (self-healing, not merely GC of what was
        // explicitly removed).
        let new_digest_keys = self.actors.digest_keys();
        if new_digest_keys.is_empty() {
            if !self.actor_digest_at_decode.is_empty() {
                // The digest held entries at decode and holds none now --
                // every actor this chunk had was removed. Delete the key
                // entirely rather than leave a stale record with zero
                // entries, per the Phase 5 "deleting empty digp" task.
                records.take(KeyVariant::ActorDigest);
            }
            // Otherwise there was nothing to begin with -- no key, or a key
            // already carrying zero entries, the common shape for a chunk
            // with no actors (see the corpus counts in `crate::actor`'s
            // test suite) -- and nothing was added either. Leave `records`
            // exactly as it already is (the clone/copy above already
            // carries whatever the original had, present-empty or absent),
            // which is what keeps this path byte-identical for the
            // no-actors-either-way case rather than deleting a key real
            // worlds keep present.
        } else {
            records.insert(
                Key {
                    chunk: position,
                    dimension,
                    data: KeyVariant::ActorDigest,
                },
                encode_actor_digest(&new_digest_keys),
            );
        }

        Ok(records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actor::{decode_actor_digest, encode_actor_record};

    /// A synthetic, never-persisted chunk -- these tests are about the
    /// mutation surface and encode-time bookkeeping in this module, not
    /// about real on-disk shapes (the fixture-driven suite in
    /// `tests/actor_write.rs` covers those).
    fn new_chunk() -> Chunk {
        Chunk::new(ChunkPosition(0, 0), Dimension::Overworld, 40)
    }

    #[test]
    fn adding_an_actor_to_a_fresh_chunk_produces_a_digest_and_a_matching_upsert() {
        let mut chunk = new_chunk();
        let key: ActorStorageKey = [7; 8];
        let value = nbtx::Value::Byte(1);
        chunk.actors_mut().add(key, value.clone());

        let encoded = chunk.encode().unwrap();
        let digest_bytes = encoded
            .get(KeyVariant::ActorDigest)
            .expect("digp key present after adding an actor");
        assert_eq!(decode_actor_digest(digest_bytes).unwrap(), vec![key]);

        let writes = chunk.encode_actor_writes().unwrap();
        assert_eq!(
            writes.upserts,
            vec![(key, encode_actor_record(&value).unwrap())]
        );
        assert!(writes.deletions.is_empty());
    }

    #[test]
    fn removing_the_only_actor_deletes_the_digest_key_entirely() {
        let mut chunk = new_chunk();
        let key: ActorStorageKey = [3; 8];
        chunk.actors_mut().add(key, nbtx::Value::Byte(0));
        chunk.actors_mut().remove(&key);

        let encoded = chunk.encode().unwrap();
        assert!(!encoded.contains(KeyVariant::ActorDigest));
    }

    #[test]
    fn a_chunk_with_no_actors_encodes_with_no_digest_key_at_all() {
        let chunk = new_chunk();
        let encoded = chunk.encode().unwrap();
        assert!(!encoded.contains(KeyVariant::ActorDigest));
    }

    #[test]
    fn migrate_legacy_entities_rejects_a_key_count_mismatch_without_mutating() {
        let mut chunk = new_chunk();
        chunk.actors_mut().legacy = vec![nbtx::Value::Byte(1), nbtx::Value::Byte(2)];
        let before = chunk.actors().legacy.clone();

        let result = chunk.migrate_legacy_entities([[1u8; 8]]);

        assert!(result.is_err());
        assert_eq!(chunk.actors().legacy, before);
        assert!(chunk.actors().resolved.is_empty());
    }

    #[test]
    fn migrate_legacy_entities_moves_each_entity_to_its_given_key_in_order() {
        let mut chunk = new_chunk();
        chunk.actors_mut().legacy = vec![nbtx::Value::Byte(1), nbtx::Value::Byte(2)];

        let migrated = chunk.migrate_legacy_entities([[1u8; 8], [2u8; 8]]).unwrap();

        assert_eq!(migrated, 2);
        assert!(chunk.actors().legacy.is_empty());
        assert_eq!(chunk.actors().digest_keys(), vec![[1u8; 8], [2u8; 8]]);

        let encoded = chunk.encode().unwrap();
        assert!(!encoded.contains(KeyVariant::Entity));
        assert!(encoded.contains(KeyVariant::ActorDigest));
    }

    #[test]
    fn heightmap_origin_is_always_preserved_in_this_wave() {
        let chunk = new_chunk();
        assert_eq!(chunk.heightmap_origin(), HeightmapOrigin::Preserved);
    }
}
