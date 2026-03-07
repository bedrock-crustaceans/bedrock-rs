use std::{
    ffi::{CStr, CString, c_char, c_int, c_void},
    ops::Deref,
    ptr::NonNull,
};

use crate::ffi::{self, FfiStatus};
use crate::{
    error::{Error, Result},
    iter::Keys,
};

/// Smart pointer around a LevelDB buffer, ensuring the buffer is deallocated after use.
#[derive(Debug)]
pub struct Buffer<'db>(&'db mut [u8]);

impl<'db> Buffer<'db> {
    /// Creates a `Guard` from the given slice.
    ///
    /// This is not implemented as a `From` trait so that `Guard` can only be constructed
    /// inside of this crate and to label it as unsafe.
    ///
    /// # Safety
    ///
    /// A `Guard` must only be created from a slice that was allocated by
    /// `LevelDb` or `Keys`.
    /// The caller must also ensure that the slice is not referenced anywhere else in the program.
    pub(crate) unsafe fn from_slice(slice: &'db mut [u8]) -> Self {
        Self(slice)
    }
}

impl<'db> Deref for Buffer<'db> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        self.0
    }
}

impl<'db> AsRef<[u8]> for Buffer<'db> {
    fn as_ref(&self) -> &[u8] {
        self.0
    }
}

impl<'db> From<Buffer<'db>> for Vec<u8> {
    fn from(buf: Buffer<'db>) -> Self {
        buf.0.to_owned()
    }
}

impl<'db> Drop for Buffer<'db> {
    fn drop(&mut self) {
        // Safety:
        //
        // The slice in self has been allocated by the database, assuming the safety
        // conditions were followed when creating this guard.
        unsafe {
            ffi::bedrockrs_buffer_destroy(self.0.as_mut_ptr().cast::<i8>());
        }
    }
}

/// A LevelDB database.
pub struct Database {
    /// Pointer to the C++ `Database` struct, containing the database and
    ptr: NonNull<c_void>,
}

impl Database {
    /// Opens a LevelDB database at the specified `path`. This `path` should point to the `db` directory
    /// of a world, not the world itself.
    pub fn open<P: AsRef<str>>(path: P) -> Result<Self> {
        let ffi_path = CString::new(path.as_ref())?;

        // Safety: This is safe to call since `ffi_path` is a valid nul-terminated string.
        let result = unsafe { ffi::bedrockrs_db_open(ffi_path.as_ptr()) };

        if result.status == FfiStatus::Success {
            let ptr = NonNull::new(result.data)
                .expect("`db_open` pointer was null despite successful status");

            Ok(Self { ptr })
        } else {
            // Safety: This is safe because the result is a fail and therefore `result.data` contains
            // a nul-terminated string.
            let err = unsafe { translate_ffi_error(result) };

            Err(err)
        }
    }

    /// Yields the FFI database pointer.
    pub(crate) fn as_ptr(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }

    /// Creates an iterator over all the keys in this database.
    pub fn iter(&self) -> Keys<'_> {
        Keys::new(self)
    }

    /// Attempts to retrieve the given key from the database.
    pub fn get<K>(&self, key: K) -> Result<Option<Buffer<'_>>>
    where
        K: AsRef<[u8]>,
    {
        // Safety: This is safe to call since `key` is a valid Rust slice and `self.ptr`
        // has been allocated by a call to `bedrockrs_db_open` in `Database::open`.
        let result = unsafe {
            ffi::bedrockrs_db_get(
                self.as_ptr(),
                key.as_ref().as_ptr().cast::<c_char>(),
                key.as_ref().len() as c_int,
            )
        };

        match result.status {
            FfiStatus::Success => {
                assert!(!result.data.is_null(), "`db_get` result data was null");

                // Safety: This is safe because `result.size` is the exact size of the buffer and
                // `result.data` is indeed an array of `u8`.
                let data = unsafe {
                    std::slice::from_raw_parts_mut(result.data as *mut u8, result.size as usize)
                };

                // Safety: This is safe because `data` was created by a LevelDB allocation and is a valid
                // Rust slice.
                let guard = unsafe { Buffer::from_slice(data) };

                Ok(Some(guard))
            }
            FfiStatus::NotFound => Ok(None),
            // Safety: This is safe because the result is a fail and therefore `result.data` points to a
            // nul-terminated string.
            _ => Err(unsafe { translate_ffi_error(result) }),
        }
    }

    /// Inserts a key-value pair into the database.
    pub fn insert<K, V>(&self, key: K, value: V) -> Result<()>
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        // Safety: This is safe to call since `key` and `value` are valid Rust slices and `self.ptr`
        // has been allocated by a call to `bedrockrs_db_open` in `Database::open`.
        let result = unsafe {
            ffi::bedrockrs_db_put(
                self.as_ptr(),
                key.as_ref().as_ptr().cast::<c_char>(),
                key.as_ref().len() as c_int,
                value.as_ref().as_ptr().cast::<c_char>(),
                value.as_ref().len() as c_int,
            )
        };

        if result.status == FfiStatus::Success {
            Ok(())
        } else {
            // Safety: This is safe because the result is a fail and therefore `result.data` points to a
            // nul-terminated string.
            Err(unsafe { translate_ffi_error(result) })
        }
    }

    /// Removes a key from the database.
    pub fn remove<K>(&self, key: K) -> Result<()>
    where
        K: AsRef<[u8]>,
    {
        // Safety: This is safe to call since `key` is a valid Rust slice and `self.ptr`
        // has been allocated by a call to `bedrockrs_db_open` in `Database::open`.
        let result = unsafe {
            ffi::bedrockrs_db_remove(
                self.as_ptr(),
                key.as_ref().as_ptr().cast::<c_char>(),
                key.as_ref().len() as c_int,
            )
        };

        match result.status {
            FfiStatus::Success | FfiStatus::NotFound => Ok(()),
            // Safety: This is safe because the result is a fail and therefore `result.data` points to a
            // nul-terminated string.
            _ => Err(unsafe { translate_ffi_error(result) }),
        }
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        unsafe {
            ffi::bedrockrs_db_close(self.ptr.as_ptr());
        }
    }
}

// Safety: This is safe because `leveldb` is internally synchronised. The docs mention explicitly
// that `leveldb` is thread safe.
unsafe impl Send for Database {}

// Safety: This is safe because `leveldb` is internally synchronised. The docs mention explicitly
// that `leveldb` is thread safe.
unsafe impl Sync for Database {}

/// # Safety
///
/// This function must only be called if `result.success` is not `FfiResult::Success` and
/// `result.data` is a C-style string ending in a nul terminator.
unsafe fn translate_ffi_error(result: ffi::FfiResult) -> Error {
    assert_ne!(result.status, FfiStatus::Success);

    let ffi_err = unsafe { CStr::from_ptr(result.data.cast::<c_char>()) };
    let str = match ffi_err.to_str() {
        Ok(str) => str,
        Err(err) => return Error::InvalidUtf8(err),
    };

    let owned = str.to_owned();

    unsafe { ffi::bedrockrs_buffer_destroy(result.data.cast::<c_char>()) };
    Error::LevelDbError(owned)
}
