use std::marker::PhantomData;

use rusty_leveldb::{DBIterator, LdbIterator};

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

/// An iterator over keys in a LevelDB database.
pub struct Keys<'db> {
    iter: DBIterator,
    /// Binds this struct to the `db` lifetime to ensure it does not outlive the database.
    _marker: PhantomData<&'db ()>,
}

impl<'db> Keys<'db> {
    pub fn new(db: &'db Database) -> Result<Self> {
        Ok(Self {
            iter: db.new_iter()?,
            _marker: PhantomData,
        })
    }
}

impl<'k, 'db> Iterator for &'k mut Keys<'db> {
    type Item = KvRef<'k, 'db>;

    fn next(&mut self) -> Option<KvRef<'k, 'db>> {
        if !self.iter.advance() {
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
