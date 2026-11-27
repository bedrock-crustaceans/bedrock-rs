//! A whole document read with the numbers no double holds clamped
//! ([`json::FiniteBytes`](crate::json::FiniteBytes), named here by its old name too).

use super::{Document, Rejected};
pub use crate::json::FiniteBytes as FiniteText;

impl<'a> FiniteText<'a> {
    /// A whole load of one file through `read`, which is given the bytes to read: its
    /// outcome, loaded or rejected, settled in its own file.
    ///
    /// # Errors
    ///
    /// Returns the `Rejected` that `read` returns, with the finite-number findings settled into it.
    pub fn load<T>(
        bytes: &'a [u8],
        read: impl FnOnce(&[u8]) -> Result<Document<T>, Rejected>,
    ) -> Result<Document<T>, Rejected> {
        let finite = Self::of(bytes);
        let mut result = read(finite.bytes());
        match &mut result {
            Ok(document) => {
                let origin = document.origin().clone();
                finite.settle(document.diagnostics_mut(), Some(&origin));
            }
            Err(rejected) => {
                let origin = rejected.origin.clone();
                finite.settle(&mut rejected.diagnostics, Some(&origin));
            }
        }
        result
    }
}
