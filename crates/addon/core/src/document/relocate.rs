//! A document or rejection read on its own, put in the file of a pack it was walked from.

use super::{Document, Origin, Rejected};

impl<T> Document<T> {
    /// This document as read from `origin`: its origin replaced, every finding put in that
    /// pack and file. For a file parsed by `Content::parse` (which reads a file on its own, in
    /// no pack) during a walk.
    pub(crate) fn relocated(mut self, origin: Origin) -> Self {
        self.diagnostics.locate(&origin);
        self.origin = origin;
        self
    }
}

impl Rejected {
    /// [`Document::relocated`] of a rejection.
    pub(crate) fn relocated(mut self, origin: Origin) -> Self {
        self.diagnostics.locate(&origin);
        self.origin = origin;
        self
    }
}
