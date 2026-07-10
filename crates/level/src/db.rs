use std::{marker::PhantomData, ops::Deref, sync::Arc};

use miniz_oxide::{
    deflate::{compress_to_vec, compress_to_vec_zlib},
    inflate::{decompress_to_vec, decompress_to_vec_zlib},
};
use rusty_leveldb::{
    Compressor, CompressorList, DB, DBIterator, Options, Status, StatusCode,
    compressor::NoneCompressor,
};

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

/// Tables are written with a 4 MiB block size.
const BLOCK_SIZE: usize = 4 * 1024 * 1024;

/// Default deflate compression level.
const COMPRESSION_LEVEL: u8 = 6;

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

fn options() -> Options {
    let mut list = CompressorList::new();
    list.set_with_id(COMPRESSOR_NONE, NoneCompressor);
    list.set_with_id(COMPRESSOR_ZLIB, ZlibCompressor(COMPRESSION_LEVEL));
    list.set_with_id(COMPRESSOR_RAW_DEFLATE, RawDeflateCompressor(COMPRESSION_LEVEL));

    let mut opt = Options::default();
    opt.compressor_list = Arc::new(list);
    opt.compressor = COMPRESSOR_RAW_DEFLATE;
    opt.block_size = BLOCK_SIZE;
    opt.create_if_missing = true;

    opt
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

/// A LevelDB database.
pub struct Database {
    db: DB,
}

impl Database {
    /// Opens a LevelDB database at the specified `path`. This `path` should point to the `db` directory
    /// of a world, not the world itself.
    pub fn open<P: AsRef<str>>(path: P) -> Result<Self> {
        let db = DB::open(path.as_ref(), options())?;
        Ok(Self { db })
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
