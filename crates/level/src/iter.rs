use std::marker::PhantomData;

use leveldb::DBIterator;

use crate::{
    db::{Buffer, Database},
    error::Result,
};

/// A key-value tuple returned by the [`Keys`] iterator.
pub struct KvRef<'k, 'db> {
    key: Vec<u8>,
    value: Vec<u8>,
    /// Binds this ref to the parent [`Keys`] iterator.
    _marker: PhantomData<&'k Keys<'db>>,
}

impl KvRef<'_, '_> {
    /// Returns the key of this tuple.
    pub fn key(&self) -> Buffer<'static> {
        Buffer::new(self.key.clone())
    }

    /// Returns the value in this tuple.
    pub fn value(&self) -> Buffer<'static> {
        Buffer::new(self.value.clone())
    }
}

/// An iterator over keys in a `LevelDB` database.
///
/// Besides [`Iterator`], which returns owned copies, records can be read in place with
/// [`next_ref`](Keys::next_ref), and [`seek`](Keys::seek) skips ahead to a key, e.g. to the
/// first record of a chunk.
pub struct Keys<'db> {
    iter: DBIterator,
    /// Set by [`seek`](Keys::seek): the iterator already stands on the record the next call returns.
    sought: bool,
    /// Binds this struct to the `db` lifetime to ensure it does not outlive the database.
    _marker: PhantomData<&'db ()>,
}

impl<'db> Keys<'db> {
    /// Creates an iterator over the keys of `db`.
    ///
    /// # Errors
    ///
    /// Returns an error if the database cannot create an iterator.
    pub fn new(db: &'db Database) -> Result<Self> {
        Ok(Self {
            iter: db.new_iter()?,
            sought: false,
            _marker: PhantomData,
        })
    }

    /// Moves to the first record whose key is at or after `key`: the next call to
    /// [`next`](Iterator::next) or [`next_ref`](Keys::next_ref) returns it.
    pub fn seek(&mut self, key: &[u8]) {
        self.iter.seek(key);
        self.sought = true;
    }

    /// Moves to the next record like [`next`](Iterator::next), but returns its key and value
    /// without copying them. They are valid until the iterator moves again.
    pub fn next_ref(&mut self) -> Option<(&[u8], &[u8])> {
        if !self.step() {
            return None;
        }
        Some((self.iter.key()?, self.iter.value()?))
    }

    /// The record the iterator stands on: the one the last call to [`next_ref`](Keys::next_ref)
    /// or [`next`](Iterator::next) returned, or after [`seek`](Keys::seek) the one the next call
    /// returns. `None` past the last record.
    pub fn current(&self) -> Option<(&[u8], &[u8])> {
        Some((self.iter.key()?, self.iter.value()?))
    }

    /// The error that ended the iteration early, if any. The iterator stops at a record it cannot
    /// read as if it had reached the last one; this tells the two apart.
    ///
    /// # Errors
    ///
    /// Returns the read error that stopped the iterator.
    pub fn status(&self) -> Result<()> {
        Ok(self.iter.status()?)
    }

    /// Moves to the record the next call returns; `false` past the last one.
    fn step(&mut self) -> bool {
        if std::mem::take(&mut self.sought) {
            self.iter.valid()
        } else {
            self.iter.next()
        }
    }
}

impl<'k, 'db> Iterator for &'k mut Keys<'db> {
    type Item = KvRef<'k, 'db>;

    fn next(&mut self) -> Option<KvRef<'k, 'db>> {
        if !self.step() {
            return None;
        }

        let (key, value) = self.iter.current()?;
        Some(KvRef {
            key,
            value,
            _marker: PhantomData,
        })
    }
}
