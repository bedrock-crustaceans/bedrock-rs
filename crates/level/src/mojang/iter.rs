use std::{ffi::c_void, marker::PhantomData, ptr::NonNull};

use crate::db::{Buffer, Database};

/// A key-value tuple returned by the [`Keys`] iterator.
pub struct KvRef<'k, 'db> {
    iter: NonNull<c_void>,
    /// Binds this ref to the parent [`Keys`] iterator.
    _marker: PhantomData<&'k Keys<'db>>,
}

impl<'k, 'db> KvRef<'k, 'db> {
    /// Returns the key of this tuple.
    pub fn key(&self) -> Buffer<'static> {
        // Safety: A `KvRef` can only be created when the iterator is in a valid state.
        // The iterator cannot become invalid since the existence of this mutable ref prevents updates
        // of the iterator. This means the iterator must be valid and this method is safe to call.
        let result = unsafe { leveldb_sys::iter_key(self.iter.as_ptr()) };

        // something must go very wrong...
        assert!(
            !result.data.is_null(),
            "iterator key pointer was null, this is a bug."
        );

        // Safety: The pointer is nonzero by the assertion above, it has also been allocated as a byte array of
        // the size given by `result.size` in C++. It has been written to (and therefore initialised) and comes
        // from a single allocation.
        unsafe {
            let slice =
                std::slice::from_raw_parts_mut(result.data.cast::<u8>(), result.size as usize);

            // Safety: The slice was created by an FFI function, hence it is safe to create a guard.
            // It is also safe to create it with a static lifetime since the data from the iterator is copied to Rust rather
            // than being bound to the iterator.
            Buffer::from_slice(slice)
        }
    }

    /// Returns the value in this tuple.
    pub fn value(&self) -> Buffer<'static> {
        // Safety: A `KvRef` can only be created when the iterator is in a valid state.
        // The iterator cannot become invalid since the existence of this mutable ref prevents updates
        // of the iterator. This means the iterator must be valid and this method is safe to call.
        let result = unsafe { leveldb_sys::iter_value(self.iter.as_ptr()) };

        assert!(
            !result.data.is_null(),
            "iterator value pointer was null, this is a bug."
        );

        // Safety: The pointer is nonzero by the assertion above, it has also been allocated as a byte array of
        // the size given by `result.size` in C++. It has been written to (and therefore initialised) and comes
        // from a single allocation.
        unsafe {
            let slice =
                std::slice::from_raw_parts_mut(result.data.cast::<u8>(), result.size as usize);

            // Safety: The slice was created by an FFI function, hence it is safe to create a guard.
            // It is also safe to create it with a static lifetime since the data from the iterator is copied to Rust rather
            // than being bound to the iterator.
            Buffer::from_slice(slice)
        }
    }
}

/// An iterator over keys in a LevelDB database.
pub struct Keys<'db> {
    index: usize,
    /// A pointer to the leveldb iterator. This is passed to the C++ code.
    iter: NonNull<c_void>,
    /// Binds this struct to the `db` lifetime to ensure it does not outlive the database.
    _marker: PhantomData<&'db ()>,
}

impl<'db> Keys<'db> {
    pub fn new(db: &'db Database) -> Self {
        let result = unsafe { leveldb_sys::iter_new(db.as_ptr()) };

        Self {
            index: 0,
            iter: NonNull::new(result.data).expect("database iter was null"),
            _marker: PhantomData,
        }
    }
}

impl<'k, 'db> Iterator for &'k mut Keys<'db> {
    type Item = KvRef<'k, 'db>;

    fn next(&mut self) -> Option<KvRef<'k, 'db>> {
        if self.index != 0 {
            // Safety: `bedrockrs_iter_next` is safe to call as long as the iterator has not been destroyed.
            // The only method of destroying an iterator is the `Drop` implementation of `Keys` which cannot have been
            // called yet.
            unsafe {
                leveldb_sys::iter_next(self.iter.as_ptr());
            }
        }

        let valid = unsafe { leveldb_sys::iter_valid(self.iter.as_ptr()) };

        self.index += 1;
        valid.then_some(KvRef {
            iter: self.iter,
            _marker: PhantomData,
        })
    }
}

impl<'db> Drop for Keys<'db> {
    fn drop(&mut self) {
        unsafe {
            // Safety: This piece of code is the only line in the codebase that can destroy iterators.
            // Since this is a `Drop` implementation it means that this specific iterator must exist and
            // can therefore safely be deleted.
            leveldb_sys::iter_destroy(self.iter.as_ptr());
        }
    }
}

impl<'db> From<&'db Database> for Keys<'db> {
    fn from(db: &'db Database) -> Self {
        Keys::new(db)
    }
}
