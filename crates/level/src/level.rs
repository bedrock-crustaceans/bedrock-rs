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

use crate::actor::{ActorSession, ActorStorageKey, actorprefix_key_bytes, decode_actor_digest};
use crate::chunk::ChunkRecords;
use crate::chunk_version::{DEFAULT_WRITE_VERSION, FormatProfile};
use crate::db::Database;
use crate::error::{ChunkError, Error, Result};
use crate::height_range::{self, HeightRange};
use crate::key::{Key, KeyVariant};
use crate::level_chunk::{Chunk, StructuralWriteReport};
use crate::metadata_dictionary::{self, MetadataDictionary};
use crate::provider::ContentProvider;
use crate::settings::{LevelSettings, VersionInfo};
use crate::subchunk::BlockDef;
use crate::types::ChunkPosition;
use crate::upgrade::EARLIEST_BLOCK_STATE_VERSION;

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
    /// The chunk version [`Level::put_chunk`] targets by default. Defaults to
    /// [`DEFAULT_WRITE_VERSION`], per architecture decision 6: the newest chunk version this
    /// crate has a confirmed writer for, derived from the chunk-version profile table
    /// rather than declared as a separate constant (see that constant's doc comment for the
    /// derivation). [`Level::put_chunk`] retargets through [`Chunk::encode_as`], which is a
    /// best-effort structural retarget -- see its doc comment for what "within a format
    /// family" means and what crossing families falls back to.
    write_version: u8,
    /// `true` for a [`Level::open`] handle, `false` for [`Level::open_rw`].
    /// The two constructors otherwise do identical work (see
    /// [`Level::open_rw`]'s doc comment) -- this is the one place that
    /// distinction is stored, checked at the top of every write path that
    /// touches disk: [`Level::save`] (`level.dat`/`levelname.txt`) and
    /// [`Level::put_chunk`]/[`Level::remove_chunk`] (`db/`).
    read_only: bool,
    /// The content-mapping hook (architecture decision 9). `None` is the default and is
    /// what every constructor produces -- attaching one is an explicit opt-in via
    /// [`Level::with_provider`]. Its presence is exactly what gates the content-upgrade
    /// pass [`Level::chunk`]/[`Level::chunks`] run: no provider means today's behavior
    /// exactly, byte-identical, the same carve-out `crate::provider`'s module docs describe
    /// for every other content-dependent step in this crate.
    ///
    /// `Box`, not a borrowed `&dyn ContentProvider`: a borrow would put a lifetime
    /// parameter on `Level` itself, which cascades into every type that already borrows
    /// `Level` for a different reason ([`Chunks<'a>`], [`DimensionRef<'a>`]) and turns a
    /// single added lifetime into two independent ones a caller has to reconcile at every
    /// call site, for no benefit `Level` needs: it already owns everything else it holds
    /// ([`Database`], [`LevelSettings`]), and a provider is exactly the kind of long-lived,
    /// singly-owned dependency that shape fits. A caller who genuinely needs to share one
    /// provider across several levels reaches for `Arc<dyn ContentProvider>` at their own
    /// call site (implementing `ContentProvider` for a thin wrapper around it, since `Arc`
    /// does not implement a foreign trait on the caller's behalf) rather than this type
    /// borrowing one.
    provider: Option<Box<dyn ContentProvider>>,
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
    /// The returned handle is read-only: [`Level::save`], [`Level::put_chunk`],
    /// and [`Level::remove_chunk`] all return [`Error::Invalid`] on it rather
    /// than writing, to `level.dat`/`levelname.txt` or to `db/` alike. Use
    /// [`Level::open_rw`] for a handle that actually writes through.
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
    /// the level. What now differs is every write path: only a handle from
    /// this constructor will actually write through [`Level::save`]
    /// (`level.dat`/`levelname.txt`), [`Level::put_chunk`], or
    /// [`Level::remove_chunk`] (`db/`) -- see their doc comments.
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
            write_version: DEFAULT_WRITE_VERSION,
            read_only,
            provider: None,
        })
    }

    /// Attaches a content-mapping provider, builder-style: `Level::open(path)?.with_provider(p)`.
    /// See the `provider` field's doc comment for why this takes ownership rather than a
    /// borrow, and `crate::provider`'s module docs for what attaching one changes -- most
    /// concretely, [`Level::chunk`]/[`Level::chunks`] start running the content-upgrade pass
    /// automatically once this has been called.
    pub fn with_provider(mut self, provider: impl ContentProvider + 'static) -> Self {
        self.provider = Some(Box::new(provider));
        self
    }

    /// This level's attached content-mapping provider, if any. `None` is the default for
    /// every constructor; see [`Level::with_provider`] to attach one.
    pub fn provider(&self) -> Option<&dyn ContentProvider> {
        self.provider.as_deref()
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

    /// The chunk version [`Level::put_chunk`] targets by default -- retargeting a chunk's
    /// structural container to this version wherever [`Chunk::encode_as`] can honor it
    /// within the chunk's own format family; see that method's doc comment.
    pub fn write_version(&self) -> u8 {
        self.write_version
    }

    /// Overrides the chunk version [`Level::put_chunk`] targets by default -- decision 6's
    /// level-wide pin. Use [`Level::put_chunk_at`] to override it for one write instead.
    pub fn set_write_version(&mut self, version: u8) {
        self.write_version = version;
    }

    /// The packed block-state version [`Level::chunk`]/[`Level::chunks`]' content-upgrade
    /// pass targets when a [`ContentProvider`] is attached (see [`Level::with_provider`]).
    ///
    /// # Reconciling two version notions that used to be one
    ///
    /// Before this pass existed, [`Level::write_version`] meant one thing: the `0x2c`
    /// chunk-version byte [`Level::put_chunk`] targets. That byte and a block-state
    /// entry's own packed `int` stamp are different axes stored in different places (see
    /// [`crate::level_chunk::Chunk::achieved_chunk_version`]'s doc comment), so
    /// `write_version` cannot directly *be* the content target -- it is a `u8`, and a
    /// packed block-state version is an `i32` with room for a real `major.minor.patch`.
    /// Rather than adding a second, independently settable version a caller has to keep in
    /// sync with the first, this derives the content target *from* `write_version`: every
    /// chunk version this crate's format table knows about already carries the game version
    /// that introduced it ([`FormatProfile::introduced_in`]), so the block-state target is
    /// that game version, packed. Pinning `write_version` (decision 6's caller override)
    /// therefore pins the content target too, with no second knob to forget.
    ///
    /// # This is a best-available approximation, not a corpus-verified mapping
    ///
    /// "The game version that introduced this chunk version" is not the same fact as "the
    /// block-state version every entry in a chunk at this version actually carries," and a
    /// scan of the fixture corpus's real palette stamps against their chunks' own
    /// `introduced_in` shows the two commonly disagree, always in the same direction (the
    /// stamp trailing behind): `v1_18` (chunk version 39, table says 1.18.0) carries real
    /// entries stamped 1.16.210.3; `v1_16` (chunk version 21, table says 1.16.100.58)
    /// carries entries stamped 1.16.0.16; `v1_17_40_caves_and_cliffs` (chunk version 31,
    /// table says 1.17.40) carries entries stamped 1.17.40.8. A chunk's container version
    /// advances when the game changes the *chunk* format; a block's own stamp advances only
    /// when something actually touched that block's schema, and those two events are not
    /// the same event. So this target is honestly a reasonable **upper bound** to drive the
    /// pass with -- not a claim that any particular fixture's entries are already at it --
    /// and it is exactly why decision 6's per-entry achieved-version reporting
    /// ([`crate::level_chunk::Chunk::achieved_block_state_version`], [`crate::upgrade::
    /// UpgradeReport`]) exists at all: a caller checks what was actually achieved rather
    /// than assuming this target was met.
    ///
    /// A chunk version with no recorded introducing release (`introduced_in` is `None` --
    /// two specific versions in the middle of the known range, see [`FormatProfile`]'s
    /// module docs), or whose recorded release has a component too large to fit the `u8`
    /// [`BlockDef::pack_version`] packs each field into, is treated the same way: no usable
    /// `introduced_in` at this version, so the walk falls through to the nearest lower
    /// chunk version that has one. Every fact-boundary chunk version this crate's
    /// structural decoder actually branches on is one of the ones with a known release, so
    /// a caller pinning `write_version` to a gap version still gets a real, non-fabricated
    /// target this way rather than an arbitrary default. The walk-down cannot run past
    /// version 0 without finding one (version 0 itself always resolves, and every recorded
    /// component in this table fits a `u8`), but falls back to
    /// [`EARLIEST_BLOCK_STATE_VERSION`] as a final floor rather than panicking, for a
    /// `write_version` this table has literally nothing usable recorded below.
    pub fn block_state_write_target(&self) -> i32 {
        let mut version = self.write_version;
        loop {
            if let Some(game_version) = FormatProfile::for_version(version).introduced_in
                && let Ok(major) = u8::try_from(game_version.major)
                && let Ok(minor) = u8::try_from(game_version.minor)
                && let Ok(patch) = u8::try_from(game_version.patch)
                && let Ok(revision) = u8::try_from(game_version.revision)
            {
                return BlockDef::pack_version([major, minor, patch, revision]);
            }
            match version.checked_sub(1) {
                Some(lower) => version = lower,
                None => return EARLIEST_BLOCK_STATE_VERSION,
            }
        }
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
            Some(records) => {
                let mut chunk = Chunk::decode(records, &self.db)?;
                self.upgrade_content(&mut chunk);
                Ok(Some(chunk))
            }
            None => Ok(None),
        }
    }

    /// Runs the content-upgrade pass on `chunk` if a provider is attached -- the shared
    /// step [`Level::chunk`] and [`crate::level::Chunks::next`] both need, so a chunk
    /// reached either way gets the same automatic treatment (architecture decision 4). A
    /// no-op, leaving `chunk` exactly as [`Chunk::decode`] produced it, when
    /// [`Level::provider`] is `None`.
    fn upgrade_content(&self, chunk: &mut Chunk) {
        if let Some(provider) = self.provider.as_deref() {
            let target = self.block_state_write_target();
            chunk.upgrade_content(provider, target);
        }
    }

    /// Reads and decodes every chunk in `dimension`, in one scan of the
    /// database (via [`ChunkRecords::read_many`], seeded from the position
    /// index -- see the struct docs) rather than one scan per chunk. Each
    /// item is a per-chunk `Result`, per architecture decision 11: one
    /// malformed chunk does not abort the rest of the walk. A failed item's
    /// `Err` is a [`ChunkError`], naming the position and dimension of the
    /// chunk that failed alongside the underlying [`Error`] -- a caller
    /// skipping or logging bad chunks during a bulk walk can say which ones
    /// they were, which a bare `Error` cannot.
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

    /// Writes `chunk` to `position` in `dimension` at this level's [`Level::write_version`]
    /// -- equivalent to `self.put_chunk_at(dimension, position, chunk, self.write_version())`.
    ///
    /// Two different ways to opt out of this default, for two different scopes: to pin the
    /// target for *this one write* without touching the level's own default, call
    /// [`Level::put_chunk_at`] directly with an explicit target -- most usefully
    /// `chunk.chunk_version()`, which reproduces the chunk's own on-disk version
    /// byte-identically (no retarget at all, the same as driving [`Chunk::encode_at`]
    /// outside a `Level` entirely). To change what *every* future `put_chunk` call on this
    /// level defaults to, call [`Level::set_write_version`] instead.
    ///
    /// Re-encodes `chunk`'s terrain, biomes, and preserved records (see
    /// [`Chunk::encode_as`]) and updates the position index -- and, since
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
    /// Returns [`Error::Invalid`] without writing anything on a [`Level::open`]
    /// handle -- see [`Level::open`]'s doc comment. Use [`Level::open_rw`]
    /// for a handle this actually writes through.
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
    ) -> Result<StructuralWriteReport> {
        self.put_chunk_at(dimension, position, chunk, self.write_version)
    }

    /// [`Level::put_chunk`] with an explicit per-write target, overriding
    /// [`Level::write_version`] for this call only -- architecture decision 6's per-write
    /// override ("for the whole level or for an individual write"). Everything else about
    /// this call -- the actor-write pairing, the read-only guard, the stale-record clearing
    /// at a different position, the fast-path caveat -- is exactly [`Level::put_chunk`]'s
    /// own behavior; see its doc comment.
    ///
    /// Returns the [`StructuralWriteReport`] [`Chunk::encode_as`] produced for `target`, so
    /// a caller pinning a version this chunk cannot structurally reach (a different format
    /// family -- see [`Chunk::encode_as`]'s doc comment) can tell it fell short without a
    /// second call.
    pub fn put_chunk_at(
        &mut self,
        dimension: Dimension,
        position: ChunkPosition,
        chunk: &Chunk,
        target: u8,
    ) -> Result<StructuralWriteReport> {
        if self.read_only {
            return Err(Error::Invalid(
                "a read-only level (opened via Level::open; use Level::open_rw to write chunks)",
            ));
        }

        let same_spot = dimension == chunk.dimension() && position == chunk.position();
        let (encoded, report) = chunk.encode_as(position, dimension, target)?;
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

        Ok(report)
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
    /// Returns [`Error::Invalid`] without removing anything on a
    /// [`Level::open`] handle -- see [`Level::open`]'s doc comment. Use
    /// [`Level::open_rw`] for a handle this actually writes through.
    pub fn remove_chunk(&mut self, dimension: Dimension, position: ChunkPosition) -> Result<()> {
        if self.read_only {
            return Err(Error::Invalid(
                "a read-only level (opened via Level::open; use Level::open_rw to remove chunks)",
            ));
        }

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
    /// silently doing nothing -- gating `level.dat`/`levelname.txt`, exactly
    /// the two files this method writes. [`Level::put_chunk`] and
    /// [`Level::remove_chunk`] carry the same guard for `db/` (see their own
    /// doc comments), so `open`'s read-only status (see its doc comment) is
    /// enforced everywhere a `Level` writes, not only here. A `save` that
    /// quietly no-op'd on a read-only handle would make that guarantee easy
    /// to violate by accident -- a caller who called [`Level::set_name`]
    /// then `save` on an `open` handle would see no error and no effect. An
    /// explicit error is louder and cheaper to notice than a write that
    /// silently didn't happen. Use [`Level::open_rw`] to get a handle this
    /// actually writes through.
    pub fn save(&self) -> Result<()> {
        if self.read_only {
            return Err(Error::Invalid(
                "a read-only level (opened via Level::open; use Level::open_rw to save)",
            ));
        }

        self.write_level_dat()?;
        write_atomic(
            &self.path.join("levelname.txt"),
            self.settings.level_name.as_bytes(),
        )?;

        Ok(())
    }

    /// Encodes `self.settings` through [`LevelSettings::write`] and writes
    /// the result to `level.dat` atomically. Shared by [`Self::save`] (which
    /// also writes `levelname.txt`) and [`Self::reserve_actor_session_id`]
    /// (which writes `level.dat` alone, immediately, for reasons its own doc
    /// comment states) -- both want exactly this and nothing else. Carries
    /// no read-only guard of its own; both callers check
    /// `self.read_only` before reaching here.
    fn write_level_dat(&self) -> Result<()> {
        let mut level_dat = Vec::new();
        self.settings.write(&mut level_dat)?;
        write_atomic(&self.path.join("level.dat"), &level_dat)
    }

    /// Reserves a fresh actor "session id" -- the value a caller must use as
    /// the high 32 bits of every new actor's `UniqueID` it mints while
    /// adding actors to this world, to stay collision-safe with every
    /// session the game itself has ever allocated (see
    /// [`LevelSettings::next_actor_session_id`] for the corpus and
    /// reference evidence behind that claim, and for what this crate
    /// deliberately does *not* track -- the per-session low 32 bits, and the
    /// `actorprefix` storage-key derivation, both of which stay the caller's
    /// responsibility, per `crate::actor`'s module docs).
    ///
    /// Unlike every other `LevelSettings` mutation, this writes `level.dat`
    /// immediately rather than waiting for [`Self::save`]: both reference
    /// implementations persist a claimed session before creating a single
    /// actor with it, specifically so a second process opening the same
    /// world in between cannot be handed the same session. Deferring this
    /// write to `save` (as every other in-memory settings edit does today)
    /// would reopen exactly the race this method exists to close -- a crash
    /// or an early return between reservation and `save` would leave the
    /// claimed session unrecorded on disk, ready to be handed out again.
    /// `levelname.txt` is untouched; this writes only `level.dat`, and a
    /// later `save()` call still writes both from whatever `self.settings`
    /// holds at that point, session id included.
    ///
    /// Returns [`Error::Invalid`] without reserving or writing anything on a
    /// [`Level::open`] handle, matching [`Self::save`]'s and
    /// [`Self::put_chunk`]'s own read-only guard. Also propagates
    /// [`LevelSettings::reserve_actor_session_id`]'s own error -- the
    /// counter reaching `i32::MIN` with nothing further to hand out -- in
    /// which case nothing is written either.
    pub fn reserve_actor_session_id(&mut self) -> Result<i32> {
        if self.read_only {
            return Err(Error::Invalid(
                "a read-only level (opened via Level::open; use Level::open_rw to reserve an \
                 actor session id)",
            ));
        }

        let id = self.settings.reserve_actor_session_id()?;
        self.write_level_dat()?;

        Ok(id)
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

    /// Reserves a fresh actor session (see [`Self::reserve_actor_session_id`]
    /// for what that persists, and why it must happen before any actor is
    /// minted under it) and hands back an [`ActorSession`] allocator bound
    /// to it -- the session half of a full actor-add path, paired with
    /// [`crate::actor::ChunkActors::add_new`] for the low-word half. A caller minting
    /// several new actors, across one chunk or several, calls this once and
    /// threads the returned `ActorSession` through every
    /// [`crate::actor::ChunkActors::add_new`] call in that batch, so they all land under
    /// the one reserved session rather than each claiming (and persisting)
    /// a session of its own.
    ///
    /// Returns [`Error::Invalid`] without reserving or writing anything on
    /// a [`Level::open`] handle, or once the session counter is exhausted
    /// at `i32::MIN` -- both exactly [`Self::reserve_actor_session_id`]'s
    /// own failure modes; see its doc comment.
    pub fn begin_actor_session(&mut self) -> Result<ActorSession> {
        let session = self.reserve_actor_session_id()?;
        Ok(ActorSession::new(session))
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

/// Iterator over every decoded chunk in one dimension. See [`Level::chunks`]. Yields
/// [`ChunkError`] rather than a bare [`Error`] on a per-item failure, so a caller can tell
/// which chunk it was.
pub struct Chunks<'a> {
    level: &'a Level,
    inner: std::collections::hash_map::IntoIter<(ChunkPosition, Dimension), ChunkRecords>,
}

impl Iterator for Chunks<'_> {
    type Item = std::result::Result<(ChunkPosition, Chunk), ChunkError>;

    fn next(&mut self) -> Option<Self::Item> {
        let ((position, dimension), records) = self.inner.next()?;
        Some(
            Chunk::decode(records, &self.level.db)
                .map(|mut chunk| {
                    self.level.upgrade_content(&mut chunk);
                    (position, chunk)
                })
                .map_err(|source| ChunkError {
                    position,
                    dimension,
                    source,
                }),
        )
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
