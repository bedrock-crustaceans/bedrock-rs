//! Reading a client entity document as vanilla's strict schemas do: the reader of the schema
//! client entities and attachables share (`visual::cereal`), with the client entity's profile.

use facet::Facet;

use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::json::JsonSlice;
use bedrock_addon_core::versioning::ParseCx;
use bedrock_addon_shared_visual::cereal::{self, ConditionTypes, Described, Profile};

pub(crate) use bedrock_addon_shared_visual::cereal::{
    check_animate, check_controllers, check_identifier, check_map, check_min_engine_version,
    check_variables, is_map_key, not_valid_here,
};

/// The payload key.
pub(crate) const PAYLOAD: &str = cereal::CLIENT_ENTITY;
/// The payload key of the attachable family, which the same root schema declares.
pub(crate) const ATTACHABLE: &str = cereal::ATTACHABLE;

/// How a client entity reads the shared schema. A file with both payloads loads the client
/// entity alone and the game says nothing; the crate notes the payload that is not loaded,
/// and not written, with an information at the member, whatever it holds. The drifts from the
/// attachable's reader are pinned in `tests/client_entity/drifts.rs` (D2, D3) and
/// `scalar_tests.expected` (D1).
const PROFILE: Profile = Profile {
    payload: PAYLOAD,
    other_payload: ATTACHABLE,
    notes_other_payload: true,
    rules: crate::READING.scalar_rules(),
    null_span: true,
    min_engine_version_width: None,
    condition_types: ConditionTypes::ByName,
    own: None,
};

/// The root and payload walk: [`cereal::describe`] with this family's profile.
pub(crate) fn describe<'a>(
    document: JsonSlice<'a>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<Described<'a>> {
    cereal::describe(document, &PROFILE, cx, d)
}

/// [`cereal::typed`] with this family's profile.
pub(crate) fn typed<D: for<'x> Facet<'x>>(
    described: &Described<'_>,
    names_only: bool,
    floats: &[&[&str]],
    statements: bool,
    d: &mut Diagnostics,
) -> Option<D> {
    cereal::typed(described, &PROFILE, names_only, floats, statements, d)
}
