//! Checks for the tests of a family that moves a raise site from a legacy [`DiagnosticCode`] to a
//! [`CodeDef`].

use super::def::kind_of;
use super::{CodeDef, DiagnosticCode};

/// Asserts that `def` replaces `legacy` as [`CodeDef::was`] promises: it has the legacy
/// variant's [`name`](DiagnosticCode::name) and [`kind`](super::Kind).
///
/// # Panics
///
/// When the name or the kind differs.
#[allow(
    clippy::needless_pass_by_value,
    reason = "a test passes the legacy code as a literal"
)]
pub fn assert_replaces<const N: usize>(def: &CodeDef<N>, legacy: DiagnosticCode) {
    assert_eq!(
        def.name(),
        legacy.name(),
        "{} replaces {} under its name",
        def.id(),
        legacy.name()
    );
    assert_eq!(
        def.kind(),
        kind_of(&legacy),
        "{} keeps the kind of {}",
        def.id(),
        legacy.name()
    );
}
