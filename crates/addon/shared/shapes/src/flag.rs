//! [`TrueOnly`]: a member whose one allowed value is `true`.

use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::schema::{SchemaCx, SchemaFields};
use facet::Facet;

/// The one value a true-only member may have: `true`. A member whose absence is false holds an
/// `Option<TrueOnly>`, so a `false` cannot be built, and writing gives back `true` or nothing.
///
/// JSON (facet's read, for a family that reads it through facet): `true`; `false` is refused
/// ([`NotTrue`]). A family with its own reader keeps the finding it gives a `false`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet)]
#[facet(opaque, proxy = bool)]
pub struct TrueOnly;

/// A `false` where only `true` may be written.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("only `true` may be written")]
pub struct NotTrue;

impl TryFrom<bool> for TrueOnly {
    type Error = NotTrue;

    fn try_from(value: bool) -> Result<Self, NotTrue> {
        if value { Ok(Self) } else { Err(NotTrue) }
    }
}

impl From<&TrueOnly> for bool {
    fn from(_: &TrueOnly) -> Self {
        true
    }
}

impl From<TrueOnly> for bool {
    fn from(_: TrueOnly) -> Self {
        true
    }
}

// A flag holds no Molang, no bound and no trigger.
impl MolangFields for TrueOnly {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

impl SchemaFields for TrueOnly {
    fn check_fields(&mut self, _: &mut SchemaCx<'_>) {}

    fn trigger_members(_: &str, _: &mut Vec<String>) {}
}
