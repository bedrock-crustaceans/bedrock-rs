use std::{marker::PhantomData, ops::Deref, sync::Arc};

use miniz_oxide::{
    deflate::{compress_to_vec, compress_to_vec_zlib},
    inflate::{decompress_to_vec, decompress_to_vec_zlib},
};
use rusty_leveldb::{
    Compressor, CompressorList, DB, DBIterator, Options, Status, StatusCode,
    compressor::NoneCompressor,
};

pub use rusty_leveldb::CompactionMode;

use crate::{error::Result, iter::Keys};

/// On-disk block compression identifiers used by the world format.
///
/// Uncompressed blocks are stored under id 0. `zlib` (with header) is id 2 and
/// raw deflate (no zlib header or trailer) is id 4. Blocks are written with raw
/// deflate; the game selects the smaller of the raw and compressed payloads and
/// falls back to id 0 when the raw payload wins, so id 0 must always be present.
const COMPRESSOR_NONE: u8 = 0;
const COMPRESSOR_ZLIB: u8 = 2;
const COMPRESSOR_RAW_DEFLATE: u8 = 4;

/// Tables are written with a 32 KiB block size. Every read decompresses a whole block, so
/// larger blocks make point lookups of single chunk records much slower.
const BLOCK_SIZE: usize = 32 * 1024;

/// Deflate compression level. Level 1 writes about three times faster than level 6 for roughly 7% more disk space.
const COMPRESSION_LEVEL: u8 = 1;

/// `zlib`-framed deflate, matching id 2.
struct ZlibCompressor(u8);

impl Compressor for ZlibCompressor {
    fn encode(&self, block: Vec<u8>) -> rusty_leveldb::Result<Vec<u8>> {
        Ok(compress_to_vec_zlib(&block, self.0))
    }

    fn decode(&self, block: Vec<u8>) -> rusty_leveldb::Result<Vec<u8>> {
        decompress_to_vec_zlib(&block).map_err(|err| Status {
            code: StatusCode::CompressionError,
            err: err.to_string(),
        })
    }
}

/// Raw deflate without a `zlib` header or trailer, matching id 4.
struct RawDeflateCompressor(u8);

impl Compressor for RawDeflateCompressor {
    fn encode(&self, block: Vec<u8>) -> rusty_leveldb::Result<Vec<u8>> {
        Ok(compress_to_vec(&block, self.0))
    }

    fn decode(&self, block: Vec<u8>) -> rusty_leveldb::Result<Vec<u8>> {
        decompress_to_vec(&block).map_err(|err| Status {
            code: StatusCode::CompressionError,
            err: err.to_string(),
        })
    }
}

fn options(compaction_mode: CompactionMode) -> Options {
    let mut list = CompressorList::new();
    list.set_with_id(COMPRESSOR_NONE, NoneCompressor);
    list.set_with_id(COMPRESSOR_ZLIB, ZlibCompressor(COMPRESSION_LEVEL));
    list.set_with_id(
        COMPRESSOR_RAW_DEFLATE,
        RawDeflateCompressor(COMPRESSION_LEVEL),
    );

    Options {
        compressor_list: Arc::new(list),
        compressor: COMPRESSOR_RAW_DEFLATE,
        block_size: BLOCK_SIZE,
        create_if_missing: true,
        compaction_mode,
        ..Default::default()
    }
}

/// An owned copy of a value read out of the database.
#[derive(Debug)]
pub struct Buffer<'db>(Vec<u8>, PhantomData<&'db ()>);

impl Buffer<'_> {
    pub(crate) fn new(data: Vec<u8>) -> Self {
        Self(data, PhantomData)
    }
}

impl Deref for Buffer<'_> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8]> for Buffer<'_> {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<Buffer<'_>> for Vec<u8> {
    fn from(buf: Buffer<'_>) -> Self {
        buf.0
    }
}

/// A group of inserts and removals applied together by [`Database::write`]. Writing a chunk's
/// records as one batch is much cheaper than inserting them one at a time.
#[derive(Default)]
pub struct WriteBatch(rusty_leveldb::WriteBatch);

impl WriteBatch {
    pub fn new() -> Self {
        Self::default()
    }

    /// Queues an insertion of `value` at `key`.
    pub fn insert<K: AsRef<[u8]>, V: AsRef<[u8]>>(&mut self, key: K, value: V) {
        self.0.put(key.as_ref(), value.as_ref());
    }

    /// Queues the removal of `key`.
    pub fn remove<K: AsRef<[u8]>>(&mut self, key: K) {
        self.0.delete(key.as_ref());
    }

    /// The amount of queued operations.
    pub fn len(&self) -> usize {
        self.0.count() as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A LevelDB database.
pub struct Database {
    db: DB,
}

impl Database {
    /// Opens a LevelDB database at the specified `path`. This `path` should point to the `db` directory
    /// of a world, not the world itself.
    pub fn open<P: AsRef<str>>(path: P) -> Result<Self> {
        Self::open_with(path, CompactionMode::Background)
    }

    /// Opens a LevelDB database like [`open`](Self::open), choosing where compaction runs.
    /// [`CompactionMode::Background`] starts a dedicated compaction thread, [`CompactionMode::Inline`]
    /// compacts on whichever thread is writing and [`CompactionMode::Manual`] only compacts when
    /// [`compact`](Self::compact) is called.
    pub fn open_with<P: AsRef<str>>(path: P, compaction_mode: CompactionMode) -> Result<Self> {
        let db = DB::open(path.as_ref(), options(compaction_mode))?;
        Ok(Self { db })
    }

    /// Runs any compaction the database needs, on the calling thread unless the database was opened
    /// with [`CompactionMode::Background`]. Databases opened with [`CompactionMode::Manual`] should
    /// call this periodically, since reads slow down as uncompacted tables pile up.
    pub fn compact(&self) -> Result<()> {
        self.db.maybe_compact()?;
        Ok(())
    }

    /// Applies every write in `batch` atomically, as a single write to the log.
    pub fn write(&self, batch: WriteBatch) -> Result<()> {
        self.db.write(batch.0, false)?;
        Ok(())
    }

    /// Creates a forward iterator over the underlying database handle.
    pub(crate) fn new_iter(&self) -> Result<DBIterator> {
        Ok(self.db.new_iter()?)
    }

    /// Creates an iterator over all the keys in this database.
    pub fn keys(&self) -> Result<Keys<'_>> {
        Keys::new(self)
    }

    /// Attempts to retrieve the given key from the database.
    pub fn get<K>(&self, key: K) -> Result<Option<Buffer<'_>>>
    where
        K: AsRef<[u8]>,
    {
        let mut buf = Vec::new();
        if self.db.get_into(key.as_ref(), &mut buf)? {
            Ok(Some(Buffer::new(buf)))
        } else {
            Ok(None)
        }
    }

    /// Inserts a key-value pair into the database.
    pub fn insert<K, V>(&self, key: K, value: V) -> Result<()>
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        self.db.put(key.as_ref(), value.as_ref())?;
        Ok(())
    }

    /// Removes a key from the database.
    pub fn remove<K>(&self, key: K) -> Result<()>
    where
        K: AsRef<[u8]>,
    {
        self.db.delete(key.as_ref())?;
        Ok(())
    }
}
