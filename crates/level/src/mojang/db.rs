use std::{
    ffi::{c_char, c_int, c_void, CStr, CString},
    marker::PhantomData,
    ops::Deref,
    ptr::NonNull,
};

use crate::{error::{Error, Result}, iter::Keys};
use crate::{
    ffi::{self, FfiStatus},
    key::Key,
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
    pub fn open<P: AsRef<str>>(path: P) -> Result<Self> {
        let ffi_path = CString::new(path.as_ref())?;

        // Safety:
        //
        //
        let result = unsafe { ffi::bedrockrs_db_open(ffi_path.as_ptr()) };

        if result.status == FfiStatus::Success {
            let ptr = NonNull::new(result.data)
                .expect("`db_open` pointer was null despite successful status");

            Ok(Self { ptr })
        } else {
            let err = unsafe { translate_ffi_error(result) };

            Err(err)
        }
    }

    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }

    pub fn iter(&self) -> Keys<'_> {
        Keys::new(self)
    }

    pub fn get(&self, key: Key) -> Result<Option<Buffer<'_>>> {
        let mut raw_key = Vec::with_capacity(key.size_hint());
        key.serialize(&mut raw_key)?;

        let result = unsafe {
            ffi::bedrockrs_db_get(
                self.ptr.as_ptr(),
                raw_key.as_ptr().cast::<c_char>(),
                raw_key.len() as c_int,
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

unsafe fn translate_ffi_error(result: ffi::FfiResult) -> Error {
    let ffi_err = CStr::from_ptr(result.data.cast::<c_char>());
    let str = match ffi_err.to_str() {
        Ok(str) => str,
        Err(err) => return Error::InvalidUtf8(err),
    };

    let owned = str.to_owned();

    ffi::bedrockrs_buffer_destroy(result.data.cast::<c_char>());
    Error::LevelDbError(owned)
}
