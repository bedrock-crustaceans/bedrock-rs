use std::{
    io::{Cursor, Read},
    ops::Deref,
};

use crate::error::Result;

pub trait CursorExt {
    fn has_remaining(&self) -> bool;
}

impl<R: AsRef<[u8]>> CursorExt for Cursor<R>
where
    Cursor<R>: Read,
{
    fn has_remaining(&self) -> bool {
        self.get_ref().as_ref().len() as u64 != self.position()
    }
}

pub trait DatabaseAccess: Sized {
    type Buffer<'a>: Deref<Target = [u8]>
    where
        Self: 'a;

    fn open<P>(path: P) -> Result<Self>
    where
        P: AsRef<str>;

    fn get<K>(&self, key: K) -> Result<Option<Self::Buffer<'_>>>
    where
        K: AsRef<[u8]>;

    fn insert<K, V>(&self, key: K, value: V) -> Result<()>
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>;

    fn remove<K>(&self, key: K) -> Result<()>
    where
        K: AsRef<[u8]>;
}
