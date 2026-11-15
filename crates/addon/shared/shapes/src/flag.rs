//! [`TrueOnly`]: a member whose one allowed value is `true`.

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::json::kit::{Expected, Node, Read};
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::schema::{FromJson, SchemaCx, SchemaFields};
use facet::Facet;

/// The one value a true-only member may have: `true`. A member whose absence is false holds an
/// `Option<TrueOnly>`, so a `false` cannot be built, and writing gives back `true` or nothing.
///
/// JSON (facet's read, for a family that reads it through facet): `true`; `false` is refused
/// ([`NotTrue`]). The derived read takes a bool, and tells a `false` as the game's schema node
/// tells a bool outside its allowed-value set `{true}`: `"Value 'false' not allowed"`, an error.
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

/// A bool: `true`, or `false` told as a value its allowed-value set `{true}` refuses.
impl FromJson for TrueOnly {
    const EXPECTED: Option<Expected> = Some(Expected::Bool);

    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        if !bool::from_json(node, read)? {
            read.error(node, core_codes::VALUE_NOT_ALLOWED.with(["false"]));
            return None;
        }
        Some(Self)
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
