use std::{
    ffi::{c_char, c_int},
    os::raw::c_void,
};

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum FfiStatus {
    Success,
    NotFound,
    Corrupted,
    NotSupported,
    InvalidArgument,
    IoError,
    AllocationFailed,
}

#[derive(Debug)]
#[repr(C)]
pub struct FfiData {
    pub size: c_int,
    pub data: *mut c_void,
}

#[derive(Debug)]
#[repr(C)]
pub struct FfiResult {
    pub status: FfiStatus,
    pub size: c_int,
    pub data: *mut c_void,
}

extern "C" {
    /// Open a LevelDB database.
    ///
    /// # Safety
    ///
    /// This function must receive a valid null-terminated C-style string.
    pub fn bedrockrs_db_open(path: *const c_char) -> FfiResult;

    /// Close a LevelDB database.
    /// This also frees the pointers, it must no longer be used.
    ///
    /// # Safety
    ///
    /// `db` must be a pointer previously returned by a call to [`bedrockrs_db_open`].
    pub fn bedrockrs_db_close(db: *mut c_void);

    /// Loads a value from the database.
    ///
    /// # Safety
    ///
    /// `db` must be a pointer previously returned by a call to [`bedrockrs_db_open`],
    /// `key` must be a valid buffer (does not have to be null-terminated) and `key_size`
    /// must be less than or equal to the size of `key`.
    pub fn bedrockrs_db_get(db: *mut c_void, key: *const c_char, key_size: c_int) -> FfiResult;

    /// Writes a value into the database.
    ///
    /// # Safety
    ///
    /// `db` must be a pointer previously returned by a call to [`bedrockrs_db_open`].
    /// Secondly `key` must be a valid buffer (does not have to be null-terminated) and `key_size`
    /// must be less than or equal to the size of `key`. The `value` and `value_size` must also
    /// satisfy these same conditions.
    pub fn bedrockrs_db_put(
        db: *mut c_void,
        key: *const c_char,
        key_size: c_int,
        value: *const c_char,
        value_size: c_int,
    ) -> FfiResult;

    /// Deletes a key from the database.
    ///
    /// # Safety
    ///
    /// `db` must be a pointer previously returned by a call to [`bedrockrs_db_open`],
    /// `key` must be a valid buffer (does not have to be null-terminated) and `key_size`
    /// must be less than or equal to the size of `key`.
    pub fn bedrockrs_db_remove(db: *mut c_void, key: *const c_char, key_size: c_int) -> FfiResult;

    /// Deallocates a string previously allocated by another function.
    ///
    /// # Safety
    ///
    /// `array` must be a pointer previously allocated by this FFI code.
    pub fn bedrockrs_buffer_destroy(array: *mut c_char);

    /// Creates an iterator over the database keys.
    ///
    /// # Safety
    ///
    /// `db` must be a pointer previously returned by a call to [`bedrockrs_db_open`].
    pub fn bedrockrs_iter_new(db: *mut c_void) -> FfiData;

    /// Destroys an iterator previously created with [`level_iter`].
    ///
    /// # Safety
    ///
    /// `iter` must be a pointer previously returned by a call to [`bedrockrs_iter_new`].
    pub fn bedrockrs_iter_destroy(iter: *mut c_void);

    /// Whether the iterator is still valid.
    ///
    /// # Safety
    ///
    /// `iter` must be a pointer previously returned by a call to [`bedrockrs_iter_new`].
    pub fn bedrockrs_iter_valid(iter: *const c_void) -> bool;

    /// The current key the iterator is on.
    ///
    /// # Safety
    ///
    /// `iter` must be a pointer previously returned by a call to [`bedrockrs_iter_new`].
    pub fn bedrockrs_iter_key(iter: *const c_void) -> FfiData;

    /// The current value the iterator is on.
    ///
    /// # Safety
    ///
    /// `iter` must be a pointer previously returned by a call to [`bedrockrs_iter_new`].
    pub fn bedrockrs_iter_value(iter: *const c_void) -> FfiData;

    /// Moves the iterator to the next position.
    ///
    /// # Safety
    ///
    /// `iter` must be a pointer previously returned by a call to [`bedrockrs_iter_new`].
    pub fn bedrockrs_iter_next(iter: *mut c_void);

    // /// Creates a new reusable batch.
    // pub fn batch_new() -> *mut c_void;
    // /// Adds a delete operation to the batch.
    // pub fn batch_delete(batch: *mut c_void, key: *const c_char, key_size: c_int);
    // /// Adds a put operation to the batch.
    // pub fn batch_put(batch: *mut c_void, key: *const c_char, key_size: c_int, val: *const c_char, val_size: c_int);
    // /// Clears all operations from the batch.
    // pub fn batch_clear(batch: *mut c_void);
    // /// Deallocates the batch.
    // pub fn batch_destroy(batch: *mut c_void);
    // /// Executes the batch on the provided database
    // pub fn batch_execute(db: *mut c_void, batch: *mut c_void) -> FfiResult;
}
