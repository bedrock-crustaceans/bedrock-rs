//! The archive opener without `zip`: no archive can be indexed, so both steps fail. Neither
//! is reached (`on_prefix` refuses an archive before it sets `Step::Whole`); they exist so
//! the shared steps type-check.

use bytes::Bytes;

use super::super::io::Io;
use super::super::job::JobDone;
use crate::pack::OpenError;
use crate::storage::{ContentKey, StorageError};

impl super::Opener {
    pub(super) fn index_archive(
        &mut self,
        io: &mut Io,
        index: usize,
        _bytes: Bytes,
        _key: Option<ContentKey>,
    ) {
        let error = StorageError::Zip {
            fault: crate::storage::ZipFault::Unsupported,
        };
        self.finish(io, index, Some(Err(OpenError::Storage(error))));
    }

    pub(super) fn on_archive_index(&mut self, io: &mut Io, index: usize, _done: Option<JobDone>) {
        let location = self.units[index].location.clone();
        self.finish(io, index, Some(Err(OpenError::Internal { location })));
    }
}
