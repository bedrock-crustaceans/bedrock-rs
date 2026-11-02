use std::ops::Deref;

use crate::error::Result;

pub trait DatabaseAccess: Sized {
    type Buffer<'a>: Deref<Target = [u8]>
    where
        Self: 'a;

    /// Opens the database at `path`.
    ///
    /// # Errors
    ///
    /// Returns an error if the database cannot be opened.
    fn open<P>(path: P) -> Result<Self>
    where
        P: AsRef<str>;

    /// Retrieves the value at `key`, or `None` if it is absent.
    ///
    /// # Errors
    ///
    /// Returns an error if reading from the database fails.
    fn get<K>(&self, key: K) -> Result<Option<Self::Buffer<'_>>>
    where
        K: AsRef<[u8]>;

    /// Inserts a key-value pair.
    ///
    /// # Errors
    ///
    /// Returns an error if the write fails.
    fn insert<K, V>(&self, key: K, value: V) -> Result<()>
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>;

    /// Removes `key`.
    ///
    /// # Errors
    ///
    /// Returns an error if the write fails.
    fn remove<K>(&self, key: K) -> Result<()>
    where
        K: AsRef<[u8]>;
}
