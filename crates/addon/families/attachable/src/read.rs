//! Reading an attachable document as vanilla's strict schemas do: the reader of the schema
//! client entities and attachables share (`visual::cereal`), with the attachable's profile and
//! its own member, `item`.

use facet::Facet;

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, JsonPath, Severity, Span};
use bedrock_addon_core::json::{JsonSlice, Value, Width};
use bedrock_addon_core::molang::Molang;
use bedrock_addon_core::versioning::ParseCx;
use bedrock_addon_shared_visual::cereal::{
    self, ConditionTypes, Described, OwnMember, Profile, is_condition_key, required,
};

pub(crate) use bedrock_addon_shared_visual::cereal::{
    check_animate, check_controllers, check_identifier, check_map, check_min_engine_version,
    check_variables, is_map_key, not_valid_here, wrong_type,
};

use super::{ItemBinding, codes};

/// The payload key.
pub(crate) const PAYLOAD: &str = cereal::ATTACHABLE;
/// The payload key of the client entity family, which the same root schema declares.
pub(crate) const CLIENT_ENTITY: &str = cereal::CLIENT_ENTITY;
/// The pattern of an object-form `item` key, as vanilla prints it.
pub(crate) const ITEM_KEY: &str = "[a-zA-Z0-9_.:-]+";

/// How an attachable reads the shared schema: with `item`, whose `null`, conditions and
/// failures are this family's (`check_item`, `item_type_error`). The drifts from the client
/// entity's reader are pinned in `tests/attachable/drifts.rs` (D3), `tests/attachable/
/// numbers.rs` (D2) and `scalar_tests.expected` (D1).
const PROFILE: Profile = Profile {
    payload: PAYLOAD,
    other_payload: CLIENT_ENTITY,
    notes_other_payload: false,
    rules: crate::READING.scalar_rules(),
    null_span: false,
    // `min_engine_version` is kept as JSON, whose numbers are doubles.
    min_engine_version_width: Some(Width::Double),
    condition_types: ConditionTypes::ByPlace,
    own: Some(OwnMember {
        name: "item",
        refused: item_type_error,
    }),
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

/// The finding for an `item` the typed reader refuses, which it reports at `item` whatever
/// is wrong inside: a value that is neither a string nor an object is reported by
/// `wrong_item` (an `InvalidValue` error with the schema's text); in an object, the first
/// member whose value is no number, boolean or string is no Molang.
fn item_type_error(described: &Described<'_>, span: Span) -> Diagnostic {
    // `item` (object form): "null, array, object → unknown
    // child schema option type".
    let item_at = described.at.with(&["item"]);
    let item = described
        .json
        .as_object()
        .and_then(|description| description.get("item"));
    let not_molang = item.and_then(Value::as_object).and_then(|members| {
        members.iter().find(|(_, value)| {
            value.as_string().is_none() && value.as_number().is_none() && value.as_bool().is_none()
        })
    });
    match not_molang {
        Some((key, _)) => wrong_type("molang", item_at.with(&[key.as_str()]), Some(span)),
        None => wrong_item(item_at, Some(span)),
    }
}

/// An `item` that is neither a string nor an object.
fn wrong_item(at: JsonPath, span: Option<Span>) -> Diagnostic {
    let mut found = Diagnostic::of(codes::UNKNOWN_ITEM_TYPE.with(["string, object"])).at(at);
    found.location.span = span;
    found
}

/// The rules of `description.item`, at 1.10.0 and up;
/// `json` is the description as written.
/// - a `null` is no item (the typed reader takes it for an absent member);
/// - the string form takes any string; one outside the object form's key pattern is warned
///   about (an information of the crate's own) and kept;
/// - every key of the object form fully matches `[a-zA-Z0-9_.:-]+`, and the object holds at
///   least one member;
/// - the members after the one vanilla reads are dropped, with an information of the
///   crate's own;
/// - a number as the condition of the member that binds reads as the empty text, which is
///   never true: an information of the crate's own, the binding kept.
pub(crate) fn check_item(
    item: Option<&ItemBinding>,
    json: &Value,
    at: &JsonPath,
    d: &mut Diagnostics,
) {
    let item_at = at.with(&["item"]);
    let Some(item) = item else {
        if json.as_object().is_some_and(|o| o.get("item").is_some()) {
            d.push(wrong_item(item_at, None));
        }
        return;
    };
    let members = match item {
        ItemBinding::Item(written) => {
            if !is_condition_key(written) {
                d.push(
                    Diagnostic::of(&core_codes::INVALID_VALUE)
                        .severity(Severity::Info)
                        .at(item_at),
                );
            }
            return;
        }
        ItemBinding::Conditional(members) => members,
    };
    // The pattern child is required and matched in full.
    for key in members.keys() {
        if !is_condition_key(key) {
            d.push(not_valid_here(key, item_at.with(&[key])));
        }
    }
    if members.is_empty() {
        d.push(required(ITEM_KEY, item_at.clone()));
    }
    for key in item.ignored() {
        d.push(Diagnostic::of(&core_codes::DROPPED_FIELD).at(item_at.with(&[key])));
    }
    // `asString("")` of a number is the empty text.
    let bound = members
        .iter()
        .min_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    if let Some((key, Molang::Const(_))) = bound {
        d.push(
            Diagnostic::of(&core_codes::INVALID_VALUE)
                .severity(Severity::Info)
                .at(item_at.with(&[key])),
        );
    }
}
