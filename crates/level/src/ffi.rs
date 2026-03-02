use std::{ffi::{c_char, c_int}, os::raw::c_void};

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
    AllocationFailed
}

#[derive(Debug)]
#[repr(C)]
pub struct FfiData {
    pub size: c_int,
    pub data: *mut c_void
}

#[derive(Debug)]
#[repr(C)]
pub struct FfiResult {
    pub status: FfiStatus,
    pub size: c_int,
    pub data: *mut c_void
}

extern "C" {
     /// Open a LevelDB database.
    pub fn bedrockrs_db_open(path: *const c_char) -> FfiResult;
    /// Close a LevelDB database.
    /// This also frees the pointers, it must no longer be used.
    pub fn bedrockrs_db_close(database: *mut c_void);
    /// Loads a value from the database.
    pub fn bedrockrs_db_get(database: *mut c_void, key: *const c_char, key_size: c_int) -> FfiResult;
    /// Writes a value into the database.
    pub fn bedrockrs_db_put(database: *mut c_void, key: *const c_char, key_size: c_int, value: *const c_char, value_size: c_int) -> FfiResult;
    /// Deletes a key from the database.
    pub fn bedrockrs_db_remove(database: *mut c_void, key: *const c_char, key_size: c_int) -> FfiResult;
    /// Deallocates a string previously allocated by another function.
    pub fn bedrockrs_buffer_destroy(array: *mut c_char);
    // /// Creates an iterator over the database keys.
    // pub fn iter_new(database: *mut c_void) -> FfiData;
    // /// Destroys an iterator previously created with [`level_iter`].
    // pub fn iter_destroy(iter: *mut c_void);
    // /// Whether the iterator is still valid.
    // pub fn iter_valid(iter: *const c_void) -> bool;
    // /// The current key the iterator is on.
    // pub fn iter_key(iter: *const c_void) -> FfiData;
    // /// The current value the iterator is on.
    // pub fn iter_value(iter: *const c_void) -> FfiData;
    // /// Moves the iterator to the next position.
    // pub fn iter_next(iter: *mut c_void);
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