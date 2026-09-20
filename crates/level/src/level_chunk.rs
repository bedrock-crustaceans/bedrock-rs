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
use crate::provider::{
    ContentProvider, ContextResolution, RelativeOffset, StateUpgrade, VersionedBlockDef,
};
use crate::subchunk::{BlockDef, Layer, PaletteEntry, SubChunk, SubChunkVersion, from_offset};
use crate::types::{BlockPosition, ChunkPosition};
use crate::upgrade::{
    EARLIEST_BLOCK_STATE_VERSION, EntryLocation, EntryOutcome, NeighbourOutcome,
    PRE_NBT_BLOCKSTATE_VERSION, UnconvertedReason, UpgradeReport,
};

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
    /// The content-upgrade pass's report, if [`Chunk::upgrade_content`] has run --
    /// [`crate::level::Level::chunk`] runs it automatically whenever a
    /// [`ContentProvider`] is attached (architecture decision 4), so this is `Some` for
    /// every chunk a caller gets back that way, and `None` for a chunk decoded or built
    /// with no provider involved at all (the no-provider path this crate's other tests
    /// pin as byte-identical to before this pass existed). See [`Chunk::content_upgrade`].
    content_upgrade: Option<UpgradeReport>,
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
            content_upgrade: None,
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
            content_upgrade: None,
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

    /// The **structural container** version this chunk's terrain and biome records will
    /// report as achieved on the next [`Chunk::encode`] -- the `0x2c` chunk-version byte
    /// that picks a terrain layout and a biome layout (see [`FormatProfile`]), not the
    /// per-palette-entry block-state schema version [`Chunk::achieved_block_state_version`]
    /// reports. The two are different axes stamped in different places on disk (one byte
    /// on the chunk record, a packed `int` on every individual palette entry), and this
    /// wave is what first makes the distinction matter: nothing here moves the *structural*
    /// container version toward a different one (see the module docs), so this always
    /// equals [`Chunk::chunk_version`] regardless of whether a content-upgrade pass ran --
    /// a caller must not read this as evidence that a level's write target was reached, nor
    /// as a proxy for how far this chunk's block states were actually upgraded.
    pub fn achieved_chunk_version(&self) -> u8 {
        self.source_version.chunk_version()
    }

    /// This chunk's honestly achieved block-state (content) version: the minimum achieved
    /// packed version across every palette entry the last [`Chunk::upgrade_content`] pass
    /// visited (decision 6 -- the chunk is only as upgraded as its least-upgraded entry).
    /// `None` if no pass has run yet ([`Chunk::content_upgrade`] is `None`) or the pass
    /// visited no entries at all. See [`crate::upgrade::UpgradeReport::
    /// achieved_block_state_version`] for the exact floor conventions an unconverted entry
    /// contributes.
    pub fn achieved_block_state_version(&self) -> Option<i32> {
        self.content_upgrade
            .as_ref()
            .and_then(UpgradeReport::achieved_block_state_version)
    }

    /// The last content-upgrade pass's full report, if [`Chunk::upgrade_content`] has run.
    /// [`crate::level::Level::chunk`]/[`crate::level::Level::chunks`] run it automatically
    /// whenever a [`ContentProvider`] is attached to the [`crate::level::Level`]; a chunk
    /// read or built with no provider involved carries `None` here, honestly, rather than an
    /// empty report that would suggest a pass ran and found nothing to do.
    pub fn content_upgrade(&self) -> Option<&UpgradeReport> {
        self.content_upgrade.as_ref()
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

    /// Walks every palette entry of every layer in this chunk's terrain and asks
    /// `provider` to bring each one forward toward `target` (a packed block-state
    /// version), then revisits every position whose entry `provider` flags via
    /// [`ContentProvider::needs_neighbours`]. See [`crate::upgrade`]'s module docs for the
    /// two version floors this uses, and [`crate::provider::ContentProvider::
    /// resolve_with_neighbours`]'s doc comment for the collect-then-apply shape the
    /// neighbour pass follows.
    ///
    /// Stores the resulting [`UpgradeReport`] on this chunk (see
    /// [`Chunk::content_upgrade`]), replacing any previous one, and returns a borrow of it.
    ///
    /// Not called by [`Chunk::decode`]/[`Chunk::new`] themselves -- [`crate::level::Level::
    /// chunk`]/[`crate::level::Level::chunks`] call this automatically once a chunk is
    /// decoded, whenever a [`ContentProvider`] is attached to the level (architecture
    /// decision 4: automatic, with no separate call for a caller going through `Level` to
    /// remember). A caller driving `Chunk::decode` directly, outside a `Level`, calls this
    /// itself if it wants the pass at all.
    ///
    /// # Scope: `Terrain::SubChunks` only
    ///
    /// Only [`Terrain::SubChunks`] terrain is walked. A [`Terrain::Legacy`] chunk (chunk
    /// versions 0-2, `0x30` LegacyTerrain) is left untouched: those chunks predate
    /// flattening entirely, so every one of their entries is already
    /// [`PaletteEntry::Numeric`], and correctly upgrading them needs the same
    /// [`ContentProvider::numeric_block_state`] lookup this pass already applies to a
    /// paletted chunk's own `Numeric` entries -- the gap here is not a different rule, only
    /// that `LegacyTerrain` currently exposes no mutable access to its slices for this pass
    /// to write a converted entry back through. Reported honestly rather than worked
    /// around: a `Terrain::Legacy` chunk's [`Chunk::content_upgrade`] report is always
    /// empty, and its [`Chunk::achieved_block_state_version`] is always `None`.
    pub fn upgrade_content(
        &mut self,
        provider: &dyn ContentProvider,
        target: i32,
    ) -> &UpgradeReport {
        let mut report = UpgradeReport::default();
        self.upgrade_palette_entries(provider, target, &mut report);
        self.run_neighbour_pass(provider, &mut report);
        self.content_upgrade.insert(report)
    }

    /// The schema-upgrade half of [`Chunk::upgrade_content`]: `Legacy`/`Numeric` entries
    /// resolved through the provider's legacy lookups, `Modern` entries advanced through
    /// [`ContentProvider::upgrade_block_state`]. Runs once per palette *entry*, never once
    /// per position -- see [`EntryLocation`]'s doc comment.
    fn upgrade_palette_entries(
        &mut self,
        provider: &dyn ContentProvider,
        target: i32,
        report: &mut UpgradeReport,
    ) {
        let Terrain::SubChunks(subchunks) = &mut self.terrain else {
            return;
        };

        for (&subchunk_index, subchunk) in subchunks.iter_mut() {
            let mut layer_index = 0usize;
            while let Some(layer) = subchunk.get_layer_mut(layer_index) {
                let count = layer.palette().len();
                for palette_index in 0..count {
                    let outcome = layer.update_palette_entry(palette_index as u16, |entry| {
                        upgrade_one_entry(entry, provider, target)
                    });
                    report.push_entry(
                        EntryLocation {
                            subchunk: subchunk_index,
                            layer: layer_index,
                            palette_index: palette_index as u16,
                        },
                        outcome,
                    );
                }
                layer_index += 1;
            }
        }
    }

    /// The neighbour-revisit half of [`Chunk::upgrade_content`]. Collect-then-apply, per
    /// [`ContentProvider::resolve_with_neighbours`]'s doc comment: every flagged position is
    /// resolved under a shared borrow of this chunk's post-schema-upgrade state before
    /// anything is written back, so a same-pass neighbour never sees another same-pass
    /// neighbour's already-computed resolution.
    ///
    /// Only layer 0 (blocks) is considered -- every known neighbour-dependent case (stair
    /// shape, fence/wall connections, redstone routing, double-chest pairing, note-block
    /// instrument, stem attachment) is a block-layer property; see the Phase 3 TODO entry
    /// this pass implements.
    ///
    /// # Cross-chunk boundary: a documented gap, not a silent one
    ///
    /// A flagged position's neighbour callback answers `None` whenever the requested offset
    /// would land outside this chunk's own `x`/`z` range (`0..16`), rather than reading into
    /// a different chunk. `Level::chunk` decodes one chunk at a time and does not keep any
    /// other chunk resident to answer such a read; honoring a genuine cross-chunk lookup
    /// would mean opening and decoding a second chunk per boundary position, which this
    /// pass does not do. This matches [`ContentProvider::resolve_with_neighbours`]'s own
    /// contract for an absent neighbour (`None`, never synthetic air) and is exactly the
    /// gap architecture decision 11 asks to be stated rather than papered over. A vertical
    /// offset is not subject to this limit: it is answered from this same chunk's other
    /// subchunks (or `None` if the target subchunk is not present), since a chunk column
    /// always owns its own full height range.
    fn run_neighbour_pass(&mut self, provider: &dyn ContentProvider, report: &mut UpgradeReport) {
        let flagged = self.collect_flagged_positions(provider);
        if flagged.is_empty() {
            return;
        }

        let resolved: Vec<(ChunkBlockPosition, ContextResolution)> = {
            let this: &Chunk = self;
            flagged
                .into_iter()
                .map(|(pos, block)| {
                    let outcome = provider.resolve_with_neighbours(block, &mut |offset| {
                        neighbour_block(this, pos, offset)
                    });
                    (pos, outcome)
                })
                .collect()
        };

        for (pos, outcome) in resolved {
            match outcome {
                ContextResolution::Resolved(block) => {
                    let (subchunk_index, local) = pos.split();
                    if let Ok(subchunk_index) = i8::try_from(subchunk_index)
                        && let Terrain::SubChunks(subchunks) = &mut self.terrain
                        && let Some(layer) = subchunks
                            .get_mut(&subchunk_index)
                            .and_then(|subchunk| subchunk.get_layer_mut(0))
                    {
                        layer.set(local, block);
                    }
                    report.push_neighbour(pos, NeighbourOutcome::Resolved);
                }
                ContextResolution::NoEntry(_) => {
                    report.push_neighbour(pos, NeighbourOutcome::NoEntry);
                }
                ContextResolution::Malformed(_, reason) => {
                    report.push_neighbour(pos, NeighbourOutcome::Malformed(reason.0));
                }
            }
        }
    }

    /// Every layer-0 position whose (already schema-upgraded) entry `provider` flags via
    /// [`ContentProvider::needs_neighbours`], paired with a clone of that entry's resolved
    /// block -- cloned so the pass can move on to a mutable borrow afterward, per
    /// [`Chunk::run_neighbour_pass`]'s collect-then-apply shape.
    fn collect_flagged_positions(
        &self,
        provider: &dyn ContentProvider,
    ) -> Vec<(ChunkBlockPosition, crate::subchunk::BlockDef)> {
        let mut flagged = Vec::new();
        let Terrain::SubChunks(subchunks) = &self.terrain else {
            return flagged;
        };

        for (&subchunk_index, subchunk) in subchunks.iter() {
            let Some(layer) = subchunk.get_layer(0) else {
                continue;
            };
            for offset in 0..4096usize {
                let local = from_offset(offset);
                let Some(entry) = layer.get(local) else {
                    continue;
                };
                let Some(block) = entry.as_block() else {
                    continue;
                };
                if provider.needs_neighbours(block) {
                    let absolute_y = (subchunk_index as i32) * 16 + local.1 as i32;
                    flagged.push((
                        ChunkBlockPosition::new(local.0, absolute_y, local.2),
                        block.clone(),
                    ));
                }
            }
        }

        flagged
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

/// Reads the block at `pos` offset by `offset`, for the neighbour pass's callback -- `None`
/// at this chunk's own `x`/`z` boundary (a documented gap, see
/// [`Chunk::run_neighbour_pass`]) or wherever [`Chunk::block`] itself would answer `None`
/// (an absent subchunk, or a non-`Modern` entry there).
fn neighbour_block(
    chunk: &Chunk,
    pos: ChunkBlockPosition,
    offset: RelativeOffset,
) -> Option<&BlockDef> {
    let x = pos.x as i32 + offset.dx;
    let z = pos.z as i32 + offset.dz;
    let y = pos.y + offset.dy;

    if !(0..16).contains(&x) || !(0..16).contains(&z) {
        return None;
    }

    let neighbour = ChunkBlockPosition::new(x as u8, y, z as u8);
    chunk.block(neighbour).and_then(PaletteEntry::as_block)
}

/// The schema-upgrade decision for one palette entry: a `Legacy`/`Numeric` entry is
/// resolved through the provider's legacy lookups (continuing on to
/// [`ContentProvider::upgrade_block_state`] if the lookup's own version falls short of
/// `target`); a `Modern` entry goes straight to [`ContentProvider::upgrade_block_state`],
/// from its own stamp or [`EARLIEST_BLOCK_STATE_VERSION`] if it has none.
///
/// Takes `entry` by value and returns it (or the value the provider itself handed back
/// unchanged) in the `Err` half of the result on every declined path, never a clone --
/// see [`Layer::update_palette_entry`], which this is written to be passed straight to as
/// its closure. `Ok` carries the `BlockDef` to write back as `PaletteEntry::Modern`.
fn upgrade_one_entry(
    entry: PaletteEntry,
    provider: &dyn ContentProvider,
    target: i32,
) -> (std::result::Result<BlockDef, PaletteEntry>, EntryOutcome) {
    match entry {
        PaletteEntry::Legacy { ref name, val } => match provider.legacy_block_state(name, val) {
            Some(mapped) => finish_legacy_resolution(mapped, provider, target),
            None => (
                Err(entry),
                EntryOutcome::unconverted(PRE_NBT_BLOCKSTATE_VERSION, UnconvertedReason::NoEntry),
            ),
        },
        PaletteEntry::Numeric { id, data } => match provider.numeric_block_state(id, data) {
            Some(mapped) => finish_legacy_resolution(mapped, provider, target),
            None => (
                Err(entry),
                EntryOutcome::unconverted(PRE_NBT_BLOCKSTATE_VERSION, UnconvertedReason::NoEntry),
            ),
        },
        PaletteEntry::Modern(block) => {
            if let Some(version) = block.version
                && version >= target
            {
                // Already at or past the target -- confirmed, not merely assumed, since
                // this entry actually carries a stamp. Nothing to ask the provider.
                return (
                    Err(PaletteEntry::Modern(block)),
                    EntryOutcome::Converted(version),
                );
            }
            let source = block.version.unwrap_or(EARLIEST_BLOCK_STATE_VERSION);
            match provider.upgrade_block_state(block, source, target) {
                StateUpgrade::Upgraded(VersionedBlockDef { mut block, version }) => {
                    block.version = Some(version);
                    (Ok(block), EntryOutcome::Converted(version))
                }
                StateUpgrade::NoEntry(block) => {
                    let achieved = block.version.unwrap_or(EARLIEST_BLOCK_STATE_VERSION);
                    (
                        Err(PaletteEntry::Modern(block)),
                        EntryOutcome::unconverted(achieved, UnconvertedReason::NoEntry),
                    )
                }
                StateUpgrade::Malformed(block, reason) => {
                    let achieved = block.version.unwrap_or(EARLIEST_BLOCK_STATE_VERSION);
                    (
                        Err(PaletteEntry::Modern(block)),
                        EntryOutcome::unconverted(achieved, UnconvertedReason::Malformed(reason.0)),
                    )
                }
            }
        }
    }
}

/// Continues a successful legacy/numeric resolution on toward `target` via
/// [`ContentProvider::upgrade_block_state`], if the resolution's own version does not
/// already reach it.
///
/// The three outcomes of that follow-up call are *not* symmetric with
/// [`upgrade_one_entry`]'s own `Modern` branch, because a legacy/numeric resolution that
/// already succeeded is real, provider-approved progress the schema step cannot take back
/// -- only fail to extend:
/// - [`StateUpgrade::Upgraded`]: carried further, as usual -- `Converted` at the new
///   version.
/// - [`StateUpgrade::NoEntry`]: the schema step has no rule to advance this entry past
///   `version`, but the entry *did* move from `Legacy`/`Numeric` to `Modern`, which is
///   real progress even though nothing beyond it happened -- still `Converted(version)`,
///   and the palette is written at that version.
/// - [`StateUpgrade::Malformed`]: different from `NoEntry` -- the provider looked at the
///   resolved entry and found it does not match what its own version promises. The
///   palette slot still gets the resolved `Modern` entry at `version` (that resolution
///   was real and is kept), but the outcome reported is
///   [`EntryOutcome::unconverted`] with the provider's reason, per architecture decisions
///   6 and 11: a malformed compound is not silently reported as fully converted, even
///   though the slot's content did change.
fn finish_legacy_resolution(
    mapped: VersionedBlockDef,
    provider: &dyn ContentProvider,
    target: i32,
) -> (std::result::Result<BlockDef, PaletteEntry>, EntryOutcome) {
    let VersionedBlockDef { mut block, version } = mapped;
    block.version = Some(version);

    if version >= target {
        return (Ok(block), EntryOutcome::Converted(version));
    }

    match provider.upgrade_block_state(block, version, target) {
        StateUpgrade::Upgraded(VersionedBlockDef { mut block, version }) => {
            block.version = Some(version);
            (Ok(block), EntryOutcome::Converted(version))
        }
        StateUpgrade::NoEntry(block) => (Ok(block), EntryOutcome::Converted(version)),
        StateUpgrade::Malformed(block, reason) => (
            Ok(block),
            EntryOutcome::unconverted(version, UnconvertedReason::Malformed(reason.0)),
        ),
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

    /// A minimal in-test provider covering every outcome the schema-upgrade pass can
    /// produce, mirroring `provider.rs`'s own `TestProvider` but kept separate: that one
    /// is private to `provider.rs`'s test module, and this module needs its own fixed
    /// inputs to drive `Chunk::upgrade_content` end to end rather than the trait in
    /// isolation.
    struct TestProvider;

    fn versioned(name: &str, version: i32) -> VersionedBlockDef {
        VersionedBlockDef {
            block: BlockDef {
                name: name.to_string(),
                version: None,
                states: Default::default(),
            },
            version,
        }
    }

    impl ContentProvider for TestProvider {
        fn legacy_block_state(&self, name: &str, val: i16) -> Option<VersionedBlockDef> {
            match (name, val) {
                ("legacy:wood", 0) => Some(versioned("minecraft:oak_planks", 1)),
                // Resolves cleanly, but the name it resolves to is the one
                // `upgrade_block_state` always rejects below -- the fixture for the
                // "legacy resolution succeeds, schema step finds it malformed" case.
                ("legacy:malformed", 0) => Some(versioned("malformed:entry", 5)),
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
                    crate::provider::MalformedState("bad state".to_string()),
                );
            }
            if source == 99 {
                return StateUpgrade::NoEntry(state);
            }
            if source == 1 && target == 3 {
                // A rule exists but only reaches an intermediate version.
                return StateUpgrade::Upgraded(versioned(&state.name, 2));
            }
            if source < target {
                return StateUpgrade::Upgraded(versioned(&state.name, target));
            }
            StateUpgrade::NoEntry(state)
        }

        fn needs_neighbours(&self, state: &BlockDef) -> bool {
            state.name == "minecraft:oak_stairs"
        }

        fn resolve_with_neighbours<'n>(
            &self,
            state: BlockDef,
            neighbour: &mut dyn FnMut(RelativeOffset) -> Option<&'n BlockDef>,
        ) -> ContextResolution {
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

        fn biome_name(&self, _id: i32) -> Option<&str> {
            None
        }
    }

    /// A modern palette-entry NBT compound: `name`, `version` (as whichever tag `version`
    /// carries, `Int` here since that is the tag every real record uses), `states`.
    fn modern_entry_nbt(name: &str, version: Option<i32>) -> nbtx::Value {
        let mut compound = nbtx::Compound::new();
        compound.insert("name".into(), nbtx::Value::String(name.into()));
        if let Some(version) = version {
            compound.insert("version".into(), nbtx::Value::Int(version));
        }
        compound.insert(
            "states".into(),
            nbtx::Value::Compound(nbtx::Compound::new()),
        );
        nbtx::Value::Compound(compound)
    }

    /// One layer-0, one-subchunk `Terrain::SubChunks` chunk whose sole palette entry is
    /// `first`, built from the zero-bits-per-index wire form (header byte `0x00`, one NBT
    /// compound, no index array at all -- see `Layer::from_disk`'s doc comment). Starting
    /// from this and growing the palette through the public [`Layer::set`] afterward avoids
    /// hand-packing a bit-indexed array by hand.
    fn chunk_with_one_entry(entry: nbtx::Value) -> Chunk {
        let mut bytes = vec![SubChunkVersion::Legacy.raw(), 0u8];
        bytes.extend(nbtx::to_le_bytes(&entry).unwrap());
        let subchunk =
            SubChunk::from_disk::<crate::Greedy, _>(&mut Cursor::new(bytes.as_slice())).unwrap();

        let mut chunk = Chunk::new(ChunkPosition(0, 0), Dimension::Overworld, 40);
        if let Terrain::SubChunks(subchunks) = chunk.terrain_mut() {
            subchunks.insert(0, subchunk);
        }
        chunk
    }

    #[test]
    fn upgrade_content_resolves_legacy_and_continues_toward_target() {
        let mut chunk = chunk_with_one_entry(nbtx::Value::Compound(nbtx::Compound::from_iter([
            ("name".into(), nbtx::Value::String("legacy:wood".into())),
            ("val".into(), nbtx::Value::Short(0)),
        ])));

        let report = chunk.upgrade_content(&TestProvider, 3).clone();
        let (_, outcome) = report.entries().next().unwrap();
        // `legacy:wood` resolves at version 1, then the schema pass carries it from 1 to
        // 3 only as far as 2 (the fixed `(1, 3) => ... 2` rule) -- falls short of the
        // target, still `Converted`.
        assert_eq!(outcome, &EntryOutcome::Converted(2));
        assert_eq!(chunk.achieved_block_state_version(), Some(2));

        let entry = chunk.layer(0, 0).unwrap().palette().first().unwrap();
        let block = entry.as_block().expect("resolved to Modern");
        assert_eq!(block.name, "minecraft:oak_planks");
        assert_eq!(block.version, Some(2));
    }

    #[test]
    fn upgrade_content_leaves_an_unrecognized_legacy_entry_unconverted() {
        let mut chunk = chunk_with_one_entry(nbtx::Value::Compound(nbtx::Compound::from_iter([
            ("name".into(), nbtx::Value::String("legacy:unknown".into())),
            ("val".into(), nbtx::Value::Short(0)),
        ])));

        let report = chunk.upgrade_content(&TestProvider, 3).clone();
        let (_, outcome) = report.entries().next().unwrap();
        assert_eq!(
            outcome,
            &EntryOutcome::unconverted(PRE_NBT_BLOCKSTATE_VERSION, UnconvertedReason::NoEntry)
        );

        let entry = chunk.layer(0, 0).unwrap().palette().first().unwrap();
        assert!(matches!(entry, PaletteEntry::Legacy { .. }));
    }

    #[test]
    fn upgrade_content_reports_malformed_and_leaves_the_entry_unchanged() {
        let mut chunk = chunk_with_one_entry(modern_entry_nbt("malformed:entry", Some(1)));

        let report = chunk.upgrade_content(&TestProvider, 5).clone();
        let (_, outcome) = report.entries().next().unwrap();
        match outcome {
            EntryOutcome::Unconverted { achieved, reason } => {
                assert_eq!(*achieved, 1);
                assert_eq!(
                    *reason,
                    UnconvertedReason::Malformed("bad state".to_string())
                );
            }
            other => panic!("expected Unconverted, got {other:?}"),
        }

        let entry = chunk.layer(0, 0).unwrap().palette().first().unwrap();
        assert_eq!(entry.as_block().unwrap().name, "malformed:entry");
        assert_eq!(entry.as_block().unwrap().version, Some(1));
    }

    /// A `Legacy` entry the provider resolves cleanly to `V`, whose *schema* step then
    /// reports `Malformed`: the legacy resolution's progress is real and kept (the palette
    /// slot becomes `Modern` at `V`, not left as `Legacy`), but the outcome reported is
    /// `Unconverted` with the provider's reason, not `Converted` -- the blocker this test
    /// pins: `finish_legacy_resolution` must not collapse `Malformed` into `NoEntry`'s
    /// "still progress" treatment, since a malformed compound is a different, worse
    /// situation than "no rule exists" (architecture decisions 6 and 11).
    #[test]
    fn upgrade_content_legacy_resolution_then_malformed_schema_step_keeps_progress_but_reports_unconverted()
     {
        let mut chunk = chunk_with_one_entry(nbtx::Value::Compound(nbtx::Compound::from_iter([
            (
                "name".into(),
                nbtx::Value::String("legacy:malformed".into()),
            ),
            ("val".into(), nbtx::Value::Short(0)),
        ])));

        let report = chunk.upgrade_content(&TestProvider, 10).clone();
        let (_, outcome) = report.entries().next().unwrap();
        match outcome {
            EntryOutcome::Unconverted { achieved, reason } => {
                assert_eq!(*achieved, 5);
                assert_eq!(
                    *reason,
                    UnconvertedReason::Malformed("bad state".to_string())
                );
            }
            other => panic!("expected Unconverted, got {other:?}"),
        }

        // The palette slot itself did move: legacy resolution succeeded and is kept,
        // stamped at the version that resolution reached, even though the outcome above
        // reports `Unconverted`.
        let entry = chunk.layer(0, 0).unwrap().palette().first().unwrap();
        let block = entry
            .as_block()
            .expect("legacy resolution succeeded, so the slot is Modern");
        assert_eq!(block.name, "malformed:entry");
        assert_eq!(block.version, Some(5));

        // The chunk-wide minimum reflects that real, if capped, progress.
        assert_eq!(chunk.achieved_block_state_version(), Some(5));
    }

    #[test]
    fn upgrade_content_stamps_a_stampless_modern_entry_from_the_nbt_blockstate_floor() {
        let mut chunk = chunk_with_one_entry(modern_entry_nbt("minecraft:stone", None));

        chunk.upgrade_content(&TestProvider, EARLIEST_BLOCK_STATE_VERSION + 1);

        let entry = chunk.layer(0, 0).unwrap().palette().first().unwrap();
        assert_eq!(
            entry.as_block().unwrap().version,
            Some(EARLIEST_BLOCK_STATE_VERSION + 1)
        );
    }

    #[test]
    fn upgrade_content_leaves_an_already_target_entry_untouched_and_uncalled() {
        let mut chunk = chunk_with_one_entry(modern_entry_nbt("minecraft:stone", Some(42)));

        let report = chunk.upgrade_content(&TestProvider, 10).clone();
        let (_, outcome) = report.entries().next().unwrap();
        // Target 10 <= the entry's own stamp of 42: already there, confirmed rather than
        // merely assumed, since the entry actually carries a real stamp.
        assert_eq!(outcome, &EntryOutcome::Converted(42));
    }

    #[test]
    fn achieved_block_state_version_is_the_minimum_across_entries() {
        let mut chunk = chunk_with_one_entry(modern_entry_nbt("minecraft:stone", Some(1)));
        if let Terrain::SubChunks(subchunks) = chunk.terrain_mut() {
            let layer = subchunks.get_mut(&0).unwrap().get_layer_mut(0).unwrap();
            layer.set(
                BlockPosition(1, 0, 0),
                BlockDef {
                    name: "legacy:unknown".to_string(),
                    version: None,
                    states: Default::default(),
                },
            );
        }
        // The second `set` above always inserts a *Modern* entry (see `Layer::set`'s doc
        // comment -- there is no public way to construct a `Legacy` one), so both palette
        // entries here are `Modern`: one stamped `1`, the other stampless. The unstamped
        // one's `NoEntry` outcome (source == target, `(1, 1)` matches no rule) is exactly
        // what should drag the chunk-wide minimum down to the stampless floor.
        let report = chunk
            .upgrade_content(&TestProvider, EARLIEST_BLOCK_STATE_VERSION)
            .clone();
        assert_eq!(report.entries().count(), 2);
        assert_eq!(
            chunk.achieved_block_state_version(),
            Some(EARLIEST_BLOCK_STATE_VERSION)
        );
    }

    #[test]
    fn neighbour_pass_resolves_from_a_same_chunk_neighbour_above() {
        let mut chunk = chunk_with_one_entry(modern_entry_nbt("minecraft:oak_stairs", Some(1)));
        if let Terrain::SubChunks(subchunks) = chunk.terrain_mut() {
            let layer = subchunks.get_mut(&0).unwrap().get_layer_mut(0).unwrap();
            // Every position defaults to the stairs entry (index 0); place air directly
            // above the position under test, (0, 0, 0).
            layer.set(
                BlockPosition(0, 1, 0),
                BlockDef {
                    name: "minecraft:air".to_string(),
                    version: Some(1),
                    states: Default::default(),
                },
            );
        }

        chunk.upgrade_content(&TestProvider, 1);

        let stairs = chunk
            .block(ChunkBlockPosition::new(0, 0, 0))
            .and_then(PaletteEntry::as_block)
            .expect("resolved stairs entry");
        assert_eq!(
            stairs.states.get("shape"),
            Some(&nbtx::Value::String("outer_left".into()))
        );

        let report = chunk.content_upgrade().unwrap();
        let outcomes: Vec<_> = report.neighbour_resolutions().collect();
        assert!(
            outcomes
                .iter()
                .any(|(pos, outcome)| *pos == ChunkBlockPosition::new(0, 0, 0)
                    && *outcome == &NeighbourOutcome::Resolved)
        );
        // A stairs position with another stairs block above it (the default everywhere
        // else) has no `air` neighbour, so it is flagged and revisited but left
        // unresolved.
        assert!(
            outcomes
                .iter()
                .any(|(pos, outcome)| *pos != ChunkBlockPosition::new(0, 0, 0)
                    && *outcome == &NeighbourOutcome::NoEntry)
        );
    }

    #[test]
    fn no_provider_pass_leaves_content_upgrade_absent() {
        let chunk = chunk_with_one_entry(modern_entry_nbt("minecraft:stone", Some(1)));
        assert!(chunk.content_upgrade().is_none());
        assert_eq!(chunk.achieved_block_state_version(), None);
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
