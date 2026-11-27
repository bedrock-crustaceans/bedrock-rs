//! What [`NoBeta`] needs to stand as a chain's `Family::Beta`: the `version_chain!` walk hands
//! the beta value to the Molang pass, and `Loaded` is `Clone` and `PartialEq` through its beta.

use super::NoBeta;
use crate::id::Kind;
use crate::molang::{MolangFields, MolangVisitor};

/// Never a value.
impl<K: Kind> Clone for NoBeta<K> {
    fn clone(&self) -> Self {
        match self.never {}
    }
}

/// Never a value.
impl<K: Kind> PartialEq for NoBeta<K> {
    fn eq(&self, _: &Self) -> bool {
        match self.never {}
    }
}

impl<K: Kind> Eq for NoBeta<K> {}

/// Never a value, so no Molang field.
impl<K: Kind> MolangFields for NoBeta<K> {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {
        match self.never {}
    }
}
