//! The level layer: [`Level`], the entry point architecture decisions 3 and
//! 4 describe. Opening a world and reading a chunk through this type
//! resolves the on-disk version and decodes straight into
//! [`crate::level_chunk::Chunk`]'s normalized model, with no separate call
//! from the caller -- the raw layer underneath ([`Level::raw`],
//! [`crate::chunk::ChunkRecords`]) is where byte-exact, unmodelled access
//! still lives.

use std::collections::{HashMap, HashSet};
use std::io::Cursor;
use std::path::{Path, PathBuf};

use bedrock_shared::world::dimension::Dimension;

use crate::actor::{ActorStorageKey, actorprefix_key_bytes, decode_actor_digest};
use crate::chunk::ChunkRecords;
use crate::chunk_version::KNOWN_CHUNK_VERSION_CEILING;
use crate::db::Database;
use crate::error::{Error, Result};
use crate::height_range::{self, HeightRange};
use crate::key::{Key, KeyVariant};
use crate::level_chunk::Chunk;
use crate::metadata_dictionary::{self, MetadataDictionary};
use crate::settings::{LevelSettings, VersionInfo};
use crate::types::ChunkPosition;

/// One opened Bedrock world: `level.dat` (metadata) plus the `db/` LevelDB
/// store (chunks and everything else), opened together from the world
/// directory that contains both.
///
/// # The position index, and its memory trade
///
/// [`Database`] has no prefix iteration -- only a full linear scan (see
/// [`Database::keys`]) -- so knowing which chunk positions exist at all
/// would otherwise cost a full scan on every call that needs it
/// ([`Level::chunk_positions`], [`Level::dimensions`], bulk iteration).
/// [`Level::open`] instead scans once, up front, and keeps only each
/// chunk's `(dimension, position)` pair: a `ChunkPosition` is two `i32`s, so
/// the index costs roughly 8 bytes per chunk (plus hash-map/vec overhead),
/// not the size of any chunk's actual data. A million-chunk world costs low
/// tens of megabytes for the index, which is the trade this makes
/// deliberately: paying a small, fixed, up-front memory cost so every
/// later position-only query is a map lookup instead of a rescan. Reading
/// or writing a chunk's actual content still goes through the raw layer
/// each time -- the index never caches record bytes.
pub struct Level {
    /// The world directory [`Level::open`]/[`Level::open_rw`] was given --
    /// the one containing `db/`, `level.dat`, and `levelname.txt`. Kept so
    /// [`Level::save`] knows where to write the latter two back to; `db`
    /// itself needs no such path, since every write to it already goes
    /// through `self.db` directly.
    path: PathBuf,
    db: Database,
    settings: LevelSettings,
    dictionary: Option<MetadataDictionary>,
    /// Every chunk position seen in the initial scan, grouped by dimension.
    /// See the struct docs for the memory trade this represents.
    positions: HashMap<Dimension, Vec<ChunkPosition>>,
    /// Every dimension this level has anything under, in ascending id
    /// order: dimensions with at least one chunk key from the same scan
    /// that built `positions`, unioned with any dimension the metadata
    /// dictionary names (a dictionary entry can outlive every chunk that
    /// referenced it -- see `crate::metadata_dictionary`'s module docs --
    /// so a dimension can be worth reporting even with zero live chunks).
    dimensions: Vec<Dimension>,
    /// The chunk version [`Level::put_chunk`] targets by default. Defaults
    /// to [`KNOWN_CHUNK_VERSION_CEILING`], the newest version this crate's
    /// format tables fully describe -- not a separate hardcoded constant,
    /// per architecture decision 6. Honesty caveat, stated once here rather
    /// than at every call site: this wave has no content-upgrade machinery
    /// (see `crate::level_chunk`'s module docs), so nothing currently moves
    /// a chunk's data toward this value on write -- [`Chunk::encode`]
    /// always re-encodes at the chunk's own source version. This field
    /// exists so the API surface and the caller-visible preference are in
    /// place for when that machinery lands.
    write_version: u8,
    /// `true` for a [`Level::open`] handle, `false` for [`Level::open_rw`].
    /// The two constructors otherwise do identical work (see
    /// [`Level::open_rw`]'s doc comment) -- this is the one place that
    /// distinction is actually enforced, in [`Level::save`].
    read_only: bool,
}

impl Level {
    /// Opens a Bedrock world for reading. `path` is the world directory --
    /// the one containing `db/` and `level.dat` -- not either of those
    /// directly.
    ///
    /// Builds the position index in one scan; see the struct docs for what
    /// that costs. Returns an error if `level.dat` cannot be read or
    /// decoded, or if the database fails to open -- both fatal, unlike a
    /// missing chunk, which [`Level::chunk`] reports as `Ok(None)`.
    ///
    /// The returned handle is read-only: [`Level::save`] returns an error
    /// on it rather than writing. [`Level::put_chunk`]/[`Level::remove_chunk`]
    /// are unaffected by that -- they write straight through to `db/`
    /// regardless of which constructor opened the handle, exactly as
    /// before this distinction existed; only `level.dat`/`levelname.txt`
    /// writes are gated on it. Use [`Level::open_rw`] for a handle
    /// [`Level::save`] will actually write through.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::open_impl(path.as_ref(), true)
    }

    /// Opens a Bedrock world for reading and writing. `path` is the world
    /// directory, exactly as for [`Level::open`].
    ///
    /// Distinct from [`Level::open`] per C-CUSTOM-TYPE, matching the public
    /// API design's rationale: the underlying `rusty-leveldb` handle is
    /// opened read-write either way, and there is no other write-side setup
    /// to skip, so the two constructors still do identical work *opening*
    /// the level. What now differs is [`Level::save`]: only a handle from
    /// this constructor will actually write `level.dat`/`levelname.txt` --
    /// see its doc comment.
    pub fn open_rw(path: impl AsRef<Path>) -> Result<Self> {
        Self::open_impl(path.as_ref(), false)
    }

    fn open_impl(path: &Path, read_only: bool) -> Result<Self> {
        let db_path = path.join("db");
        let db_path = db_path
            .to_str()
            .ok_or(Error::Invalid("world path is not valid UTF-8"))?;
        let db = Database::open(db_path)?;

        let level_dat_bytes = std::fs::read(path.join("level.dat"))?;
        let settings = LevelSettings::read(level_dat_bytes.as_slice())?;

        let dictionary = match db.get(metadata_dictionary::KEY)? {
            Some(bytes) => Some(MetadataDictionary::decode(bytes.as_ref())?),
            None => None,
        };

        let positions = scan_chunk_positions(&db)?;

        let mut dimensions: HashSet<Dimension> = positions.keys().copied().collect();
        if let Some(dictionary) = &dictionary {
            for (_, entry) in dictionary.iter() {
                if let Some(dimension) = entry.dimension_name().and_then(dimension_from_name) {
                    dimensions.insert(dimension);
                }
            }
        }
        let mut dimensions: Vec<Dimension> = dimensions.into_iter().collect();
        dimensions.sort_by_key(|&dimension| i32::from(dimension));

        Ok(Self {
            path: path.to_path_buf(),
            db,
            settings,
            dictionary,
            positions,
            dimensions,
            write_version: KNOWN_CHUNK_VERSION_CEILING,
            read_only,
        })
    }

    /// This level's version fields, as recorded in `level.dat`.
    pub fn version(&self) -> VersionInfo<'_> {
        self.settings.version_info()
    }

    /// This level's declared name.
    pub fn name(&self) -> &str {
        &self.settings.level_name
    }

    /// Renames this level. Takes effect the next time [`Level::save`]
    /// writes `level.dat`/`levelname.txt`; a caller that never calls `save`
    /// leaves the on-disk files untouched, same as any other in-memory-only
    /// edit. See [`Level::save`]'s doc comment for why this is the whole
    /// mutation surface this wave adds rather than exposing every
    /// [`LevelSettings`] field for editing.
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.settings.level_name = name.into();
    }

    /// The chunk version [`Level::put_chunk`] targets by default. See the
    /// struct docs' honesty caveat -- this is a preference this wave
    /// records but does not yet act on.
    pub fn write_version(&self) -> u8 {
        self.write_version
    }

    /// Overrides the chunk version [`Level::put_chunk`] targets by default.
    pub fn set_write_version(&mut self, version: u8) {
        self.write_version = version;
    }

    /// Every dimension this level has anything under, in ascending id
    /// order. See the struct docs' `dimensions` field for exactly what
    /// "has anything under" means.
    pub fn dimensions(&self) -> &[Dimension] {
        &self.dimensions
    }

    /// `dimension`'s resolved block-Y bounds -- the dictionary's recorded
    /// range if it names this dimension, otherwise the era default. See
    /// `crate::height_range` for the full precedence chain. `None` for a
    /// dimension outside the vanilla three that no dictionary entry names
    /// either.
    pub fn bounds(&self, dimension: Dimension) -> Option<HeightRange> {
        height_range::resolve(dimension, &self.settings, self.dictionary.as_ref())
    }

    /// Every chunk position known to exist in `dimension`, from the
    /// position index built at [`Level::open`] time. Does not decode
    /// anything -- see [`Level::chunk`] for that.
    pub fn chunk_positions(&self, dimension: Dimension) -> ChunkPositions<'_> {
        let positions = self
            .positions
            .get(&dimension)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        ChunkPositions {
            inner: positions.iter(),
        }
    }

    /// Reads and decodes the chunk at `position` in `dimension`.
    /// `Ok(None)` if no chunk exists there at all -- an ordinary outcome in
    /// a sparse world -- and `Err` only for a read or decode failure.
    pub fn chunk(&self, dimension: Dimension, position: ChunkPosition) -> Result<Option<Chunk>> {
        match ChunkRecords::read(&self.db, position, dimension)? {
            Some(records) => Ok(Some(Chunk::decode(records, &self.db)?)),
            None => Ok(None),
        }
    }

    /// Reads and decodes every chunk in `dimension`, in one scan of the
    /// database (via [`ChunkRecords::read_many`], seeded from the position
    /// index -- see the struct docs) rather than one scan per chunk. Each
    /// item is a per-chunk `Result`, per architecture decision 11: one
    /// malformed chunk does not abort the rest of the walk.
    ///
    /// Diverges from the API sketch's `chunks(&self, dim) -> Chunks<'_>`
    /// (infallible): building the iterator itself requires a database
    /// scan, which can fail with an I/O error independent of any single
    /// chunk's own decode, so this returns `Result<Chunks<'_>>` instead of
    /// deferring that failure into the first yielded item.
    pub fn chunks(&self, dimension: Dimension) -> Result<Chunks<'_>> {
        let empty = Vec::new();
        let positions = self.positions.get(&dimension).unwrap_or(&empty);
        let wanted = positions.iter().map(|&position| (position, dimension));
        let groups = ChunkRecords::read_many(&self.db, wanted)?;
        Ok(Chunks {
            level: self,
            inner: groups.into_iter(),
        })
    }

    /// Writes `chunk` to `position` in `dimension`, re-encoding its
    /// terrain, biomes, and preserved records (see
    /// [`Chunk::encode_at`]) and updating the position index -- and, since
    /// `chunk`'s actor edits reach only as far as [`Chunk::encode_at`]
    /// itself can carry them (its regenerated `digp` value, chunk-scoped
    /// like everything else in [`ChunkRecords`]), also applies
    /// [`Chunk::encode_actor_writes`] against this level's own database:
    /// the `actorprefix` half decision 13's recomputed-digest policy needs,
    /// which has no chunk coordinates of its own and so can never be a
    /// member of the [`ChunkRecords`] group `encode_at` returns. A caller
    /// going through this method never has to apply that separately, or
    /// even know it exists -- an actor added or removed via
    /// [`Chunk::actors_mut`] before this call reaches disk as a matched
    /// pair, new `digp` value and new/deleted `actorprefix` records alike.
    /// (A caller driving [`Chunk::encode_at`] directly, without a `Level`,
    /// still owns that second write itself -- see that method's own doc
    /// comment.)
    ///
    /// Writes straight through to `db/` regardless of whether this handle
    /// came from [`Level::open`] or [`Level::open_rw`] -- see [`Level::open`]'s
    /// doc comment for the read-only boundary this crate currently draws
    /// (`level.dat`/`levelname.txt` only, via [`Level::save`]).
    ///
    /// When `dimension`/`position` match `chunk`'s own (the common case: a
    /// chunk read from this level, edited, and written back), this reuses
    /// `chunk`'s own preserved-record bookkeeping to delete any record it
    /// no longer has, at no extra scan cost. Writing to a *different*
    /// position or dimension than the chunk's own has no such history to
    /// consult, so it costs one extra full scan ([`ChunkRecords::read`]) to
    /// find and clear whatever the target position already holds that the
    /// new content does not cover -- unavoidable given the backend's
    /// scan-only access (see [`Database::keys`]).
    ///
    /// **The two writes below are not transactional.**
    /// [`ActorWriteSet::apply`](crate::actor::ActorWriteSet::apply) and
    /// [`ChunkRecords::write`] are separate calls against `db/`, with no
    /// shared transaction wrapping them, so a process failure between them
    /// leaves whichever ran first durable and the other not. That ordering
    /// is chosen deliberately, applying the actor writes *before* the record
    /// group, so the failure modes are both recoverable rather than lossy: a
    /// crash after only the actor writes land leaves a newly added actor's
    /// `actorprefix` record durably on disk but not yet referenced by any
    /// `digp` (an ordinary dangling-style entry, resolved on the next
    /// successful `put_chunk`) and leaves a removed actor's `actorprefix`
    /// record briefly un-GC'd (an ordinary orphan). The reverse order would
    /// instead risk a `digp` referencing a record that was never written at
    /// all -- a real loss, not just a stale pointer -- if the process died
    /// after the group write but before the actor write.
    ///
    /// **Caveat on the fast path's stale-record guarantee.** "Matches
    /// `chunk`'s own" is judged purely by coordinates
    /// (`dimension`/`position`), not by which `Level` `chunk` actually came
    /// from -- a `Chunk` carries no back-reference to the level that
    /// decoded it (by design; see the "Chunk is one type" note in
    /// `crate::level_chunk`'s module docs). So the fast path's stale-record
    /// clearing is only as good as `chunk`'s own preserved-record history:
    /// it clears exactly what *that chunk's own origin* no longer has, not
    /// what `self` (the level being written to) currently holds at
    /// `position`. Writing a chunk decoded from a *different* `Level` back
    /// into this one, at coordinates that happen to coincide with
    /// something already in `self`, takes the fast path (coordinates
    /// match) but will not clear whatever stray records `self` has there
    /// that the other level's chunk never knew about. A same-level
    /// read-modify-write is always safe; a cross-level move at coinciding
    /// coordinates is not guaranteed to be -- pick a position this level
    /// has nothing at, or accept that stray records may survive.
    pub fn put_chunk(
        &mut self,
        dimension: Dimension,
        position: ChunkPosition,
        chunk: &Chunk,
    ) -> Result<()> {
        let same_spot = dimension == chunk.dimension() && position == chunk.position();
        let encoded = chunk.encode_at(position, dimension)?;
        let actor_writes = chunk.encode_actor_writes()?;

        if !same_spot && let Some(existing) = ChunkRecords::read(&self.db, position, dimension)? {
            let mut key_buf = Vec::new();
            for (key, _) in existing.iter() {
                if !encoded.contains(key.data) {
                    key_buf.clear();
                    key.serialize(&mut key_buf)?;
                    self.db.remove(&key_buf)?;
                }
            }
        }

        actor_writes.apply(&self.db)?;
        encoded.write(&self.db)?;

        let positions = self.positions.entry(dimension).or_default();
        if !positions.contains(&position) {
            positions.push(position);
        }
        if !self.dimensions.contains(&dimension) {
            self.dimensions.push(dimension);
            self.dimensions.sort_by_key(|&d| i32::from(d));
        }

        Ok(())
    }

    /// Deletes every record of the chunk at `position` in `dimension`, if
    /// one exists, and removes it from the position index. A no-op, not an
    /// error, if no chunk is there.
    ///
    /// Also deletes that chunk's own `actorprefix` records -- every storage
    /// key its `digp` digest names, read before the chunk's own records are
    /// taken. This includes a dangling entry (a digest key with no backing
    /// record) exactly as readily as a resolved one: deleting a key that
    /// turns out to already be absent is a documented no-op (see
    /// [`crate::actor::ActorWriteSet::apply`]), so there is no reason to
    /// filter dangling entries out first -- including them is simply safe,
    /// self-cleaning GC rather than a special case to avoid. What this does
    /// *not* touch is any `actorprefix` key some *other* chunk's digest
    /// names, or a pre-existing orphan neither this nor any digest ever
    /// named -- exactly the same scope [`Chunk::encode_actor_writes`]'s GC
    /// keeps to for an edit rather than a removal.
    ///
    /// Writes straight through to `db/` regardless of whether this handle
    /// came from [`Level::open`] or [`Level::open_rw`] -- see [`Level::open`]'s
    /// doc comment for the read-only boundary this crate currently draws
    /// (`level.dat`/`levelname.txt` only, via [`Level::save`]).
    pub fn remove_chunk(&mut self, dimension: Dimension, position: ChunkPosition) -> Result<()> {
        if let Some(mut records) = ChunkRecords::read(&self.db, position, dimension)? {
            let actor_storage_keys: Vec<ActorStorageKey> = records
                .get(KeyVariant::ActorDigest)
                .map(decode_actor_digest)
                .transpose()?
                .unwrap_or_default();

            let variants: Vec<_> = records.iter().map(|(key, _)| key.data).collect();
            for variant in variants {
                records.take(variant);
            }
            records.write(&self.db)?;

            for storage_key in actor_storage_keys {
                self.db.remove(actorprefix_key_bytes(&storage_key))?;
            }
        }

        if let Some(positions) = self.positions.get_mut(&dimension) {
            positions.retain(|&p| p != position);
        }

        Ok(())
    }

    /// The raw layer underneath: keys and values exactly as they are on
    /// disk, with no version resolution or decoding. See architecture
    /// decision 3.
    pub fn raw(&self) -> &Database {
        &self.db
    }

    /// Persists pending changes: (re)writes `level.dat` and its
    /// `levelname.txt` sidecar from this level's current settings.
    ///
    /// [`Level::put_chunk`]/[`Level::remove_chunk`] already write straight
    /// through to `db/` on every call (see their own doc comments), so
    /// those need no separate flush here -- `save`'s only remaining job is
    /// the two small metadata files that sit beside `db/` rather than in
    /// it. `level.dat` is written through [`LevelSettings::write`]; see its
    /// doc comment for the exact field-set and field-order contract that
    /// gives the file. `levelname.txt` is written as [`Level::name`]'s
    /// current value, plain text, nothing else -- the whole of what that
    /// sidecar ever holds.
    ///
    /// Settings themselves stay effectively immutable this wave beyond
    /// [`Level::set_name`], the one mutator `save` makes meaningful right
    /// now: there is no public way yet to edit a game rule, an ability, or
    /// any other `LevelSettings` field. So an `open_rw`-edit-`save` cycle
    /// that never calls `set_name` writes back exactly the settings that
    /// were read, modulo `LevelSettings::write`'s own fidelity contract
    /// (semantically faithful; not always byte-identical -- defaulted
    /// boolean fields the source world omitted are re-emitted explicit).
    /// Widening the mutation surface further is deferred until a caller
    /// need motivates which field to expose next, rather than opening every
    /// field speculatively.
    ///
    /// # Read-only handles
    ///
    /// Returns [`Error::Invalid`] on a [`Level::open`] handle instead of
    /// silently doing nothing. This is the one place `open`'s read-only
    /// status (see its doc comment) is actually enforced -- it gates only
    /// `level.dat`/`levelname.txt`, exactly the two files this method
    /// writes, not `db/` (see [`Level::put_chunk`]/[`Level::remove_chunk`],
    /// unaffected either way). A `save` that quietly no-op'd on a read-only
    /// handle would make that guarantee easy to violate by accident -- a
    /// caller who called [`Level::set_name`] then `save` on an `open`
    /// handle would see no error and no effect. An explicit error is louder
    /// and cheaper to notice than a write that silently didn't happen. Use
    /// [`Level::open_rw`] to get a handle this actually writes through.
    pub fn save(&self) -> Result<()> {
        if self.read_only {
            return Err(Error::Invalid(
                "a read-only level (opened via Level::open; use Level::open_rw to save)",
            ));
        }

        let mut level_dat = Vec::new();
        self.settings.write(&mut level_dat)?;
        write_atomic(&self.path.join("level.dat"), &level_dat)?;
        write_atomic(
            &self.path.join("levelname.txt"),
            self.settings.level_name.as_bytes(),
        )?;

        Ok(())
    }

    /// A borrowing, read-only convenience view over one dimension. Every
    /// operation it offers also exists directly on `Level` with a
    /// `dimension` parameter -- see the public API design's `DimensionRef`
    /// section for why this is a convenience over that API rather than a
    /// second way to model a dimension.
    pub fn dimension(&self, dimension: Dimension) -> DimensionRef<'_> {
        DimensionRef {
            level: self,
            dimension,
        }
    }
}

/// Writes `contents` to `path` atomically: to a fresh temp file in the same
/// directory, then an atomic same-filesystem rename over `path`. Guards
/// [`Level::save`] against a reader (the game, or another tool scanning the
/// world directory) ever observing a partially written `level.dat` or
/// `levelname.txt` -- a plain truncate-in-place write has no such
/// guarantee, since a crash or a concurrent read between the truncate and
/// the last byte sees a corrupt file where a working one used to be. The
/// rename itself is atomic because the temp file is created next to its
/// target rather than in a separate temp directory, which keeps both on
/// the same filesystem (a cross-filesystem rename is not atomic, and on
/// some platforms is not even a rename).
fn write_atomic(path: &Path, contents: &[u8]) -> Result<()> {
    let dir = path
        .parent()
        .ok_or(Error::Invalid("path has no parent directory"))?;
    let file_name = path
        .file_name()
        .ok_or(Error::Invalid("path has no file name"))?;

    let mut tmp_name = std::ffi::OsString::from(".");
    tmp_name.push(file_name);
    tmp_name.push(".tmp");
    let tmp_path = dir.join(tmp_name);

    std::fs::write(&tmp_path, contents)?;
    std::fs::rename(&tmp_path, path)?;

    Ok(())
}

/// One pass over every key in `db`, keeping only each chunk-scoped key's
/// `(dimension, position)` pair -- never the value bytes. This is what
/// keeps [`Level::open`]'s scan cheap relative to
/// [`ChunkRecords::read_all`]: the latter is for when a caller actually
/// wants every chunk's raw records in memory at once, which building a
/// position index does not need.
fn scan_chunk_positions(db: &Database) -> Result<HashMap<Dimension, Vec<ChunkPosition>>> {
    let mut positions: HashMap<Dimension, Vec<ChunkPosition>> = HashMap::new();
    let mut seen: HashSet<(Dimension, ChunkPosition)> = HashSet::new();

    let mut keys = db.keys()?;
    for kv in &mut keys {
        let key_bytes = kv.key();
        let mut cursor = Cursor::new(key_bytes.as_ref());
        let Ok(key) = Key::deserialize(&mut cursor) else {
            continue;
        };
        if !key.data.is_chunk_scoped() {
            continue;
        }
        if seen.insert((key.dimension, key.chunk)) {
            positions.entry(key.dimension).or_default().push(key.chunk);
        }
    }

    Ok(positions)
}

/// Maps a `LevelChunkMetaDataDictionary` entry's `DimensionName` field back
/// to a [`Dimension`] -- the inverse of the mapping
/// `crate::height_range::dictionary_dimension_name` uses, kept separate
/// since that one is private to its own module and this crate's dimension
/// scan is a different concern (enumerating dimensions, not resolving one
/// dimension's height range).
fn dimension_from_name(name: &str) -> Option<Dimension> {
    match name {
        "Overworld" => Some(Dimension::Overworld),
        "Nether" => Some(Dimension::Nether),
        "TheEnd" => Some(Dimension::End),
        _ => None,
    }
}

/// Iterator over a dimension's known chunk positions. See
/// [`Level::chunk_positions`].
pub struct ChunkPositions<'a> {
    inner: std::slice::Iter<'a, ChunkPosition>,
}

impl Iterator for ChunkPositions<'_> {
    type Item = ChunkPosition;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().copied()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ExactSizeIterator for ChunkPositions<'_> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl DoubleEndedIterator for ChunkPositions<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().copied()
    }
}

/// Iterator over every decoded chunk in one dimension. See [`Level::chunks`].
pub struct Chunks<'a> {
    level: &'a Level,
    inner: std::collections::hash_map::IntoIter<(ChunkPosition, Dimension), ChunkRecords>,
}

impl Iterator for Chunks<'_> {
    type Item = Result<(ChunkPosition, Chunk)>;

    fn next(&mut self) -> Option<Self::Item> {
        let ((position, _dimension), records) = self.inner.next()?;
        Some(Chunk::decode(records, &self.level.db).map(|chunk| (position, chunk)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ExactSizeIterator for Chunks<'_> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

/// A borrowing, read-only convenience view over one dimension of a
/// [`Level`]. See [`Level::dimension`] and the public API design's
/// `DimensionRef` section: every method here delegates to the matching
/// `Level` method with this view's `dimension`, and holds no state of its
/// own beyond the borrow and the id.
#[derive(Clone, Copy)]
pub struct DimensionRef<'a> {
    level: &'a Level,
    dimension: Dimension,
}

impl<'a> DimensionRef<'a> {
    /// The dimension this view is over.
    pub fn dimension(&self) -> Dimension {
        self.dimension
    }

    /// See [`Level::bounds`].
    pub fn bounds(&self) -> Option<HeightRange> {
        self.level.bounds(self.dimension)
    }

    /// See [`Level::chunk_positions`].
    pub fn chunk_positions(&self) -> ChunkPositions<'a> {
        self.level.chunk_positions(self.dimension)
    }

    /// See [`Level::chunk`].
    pub fn chunk(&self, position: ChunkPosition) -> Result<Option<Chunk>> {
        self.level.chunk(self.dimension, position)
    }

    /// See [`Level::chunks`].
    pub fn chunks(&self) -> Result<Chunks<'a>> {
        self.level.chunks(self.dimension)
    }
}
