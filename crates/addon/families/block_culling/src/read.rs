//! Reading a `minecraft:block_culling_rules` payload as vanilla's strict cereal slices do: every
//! failure fails the file, an unknown member included, and every failure of the payload is
//! reported (`versioning::read_payload_with`, unknown members first, the members in the order
//! of the types), each with the span of its value. What the reader tells in the kit's words is
//! given vanilla's breadcrumb and text afterwards ([`word`]).
//!
//! The members `condition` and `cull_against_full_and_opaque` of a rule are `since_format`
//! 1.21.80: below it they are unknown members, which is the 1.20.60 slice.

use facet::Facet;

use super::{Condition, Description, Facing, Rule};
use bedrock_addon_core::diagnostic::cereal;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Code, Diagnostic, Diagnostics, Disposition, JsonPath, JsonPathSegment, Kind, Severity,
};
use bedrock_addon_core::json::JsonSlice;
use bedrock_addon_core::json::kit::{Node, Profile, Read};
use bedrock_addon_core::schema::{FromJson, Schema};
use bedrock_addon_core::versioning::{NullHandling, ParseCx, Scalars, read_payload_with};

/// The cereal name of [`Facing`], which vanilla's enum text prints.
const FACING: &str = "Facing";
/// The cereal name of [`Condition`].
const CONDITION: &str = "Rule Condition";

/// A payload as read.
#[derive(Facet, Schema)]
pub(super) struct Payload {
    /// `description`.
    pub(super) description: Description,
    /// `rules`.
    pub(super) rules: Vec<Rule>,
}

/// Reads the payload `payload`, reported below `cx.at`. `None` when vanilla's parse fails;
/// `d` then holds every reason.
pub(super) fn read(
    payload: JsonSlice<'_>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<Payload> {
    let before = d.len();
    let read = read_payload_with(payload, cx, d, |read| {
        read.profile(Profile::DECLARED_ORDER)
            .scalars(Scalars::Strict)
            .null(NullHandling::Rejected)
            .unknown_severity(Severity::Error)
    });
    word(d, before, cx.at.segments().count());
    // An unknown member fails the file too, though the read builds the value.
    read.filter(|_| !d.iter().skip(before).any(Diagnostic::rejects))
}

/// Gives the findings of `d` from `first` on vanilla's text: the generic ones its cereal
/// reader logs after the breadcrumb of their path below the payload (`below` steps in), as
/// `-> rules[0] -> direction: <text>`. A plain wrong type and a plain invalid value are
/// vanilla's own unworded lines.
fn word(d: &mut Diagnostics, first: usize, below: usize) {
    for (index, mut finding) in std::mem::take(d).into_iter().enumerate() {
        if index >= first
            && !finding.is(&core_codes::INVALID_TYPE)
            && !finding.is(&core_codes::INVALID_VALUE)
        {
            let at: JsonPath = finding.location.json_path.segments().skip(below).fold(
                JsonPath::new(),
                |at, step| match step {
                    JsonPathSegment::Key(key) => at.child(key),
                    JsonPathSegment::Index(index) => at.index(index),
                },
            );
            // By kind: the generic findings reach here (a missing field, an unknown member, a
            // wrong type, an enum name, a value out of range) and no other definition of these
            // kinds.
            let code = match finding.code.kind() {
                Kind::MissingRequired => cereal::missing_required_field(),
                Kind::UnknownMember(Disposition::Kept) => cereal::member_not_in_schema(),
                _ => finding.code.clone(),
            };
            let line = format!("{}: {code}", breadcrumb(&at));
            finding.code = match code.kind() {
                Kind::MissingRequired => core_codes::MISSING_REQUIRED_FIELD_WORDED.with([line]),
                Kind::UnknownMember(Disposition::Kept) => {
                    core_codes::UNRECOGNIZED_FIELD_WORDED.with([line])
                }
                Kind::WrongType => core_codes::INVALID_TYPE_WORDED.with([line]),
                Kind::InvalidEnumValue => core_codes::INVALID_ENUM_VALUE_WORDED.with([line]),
                Kind::InvalidValue => core_codes::INVALID_VALUE_WORDED.with([line]),
                _ => code,
            };
        }
        d.push(finding);
    }
}

/// A string naming one of a cereal enum `bound`'s values, `names` (a `null` is the enum's
/// own wrong type: the enum reads it).
fn named<T>(
    node: &Node<'_>,
    read: &mut Read<'_>,
    bound: &str,
    names: &[&str],
    from_name: fn(&str) -> Option<T>,
) -> Option<T> {
    let Some(found) = node.value().as_string() else {
        read.refuse(node, cereal::enum_expects_string(bound));
        return None;
    };
    let value = from_name(found.as_str());
    if value.is_none() {
        read.refuse(node, cereal::enum_expects(bound, names, found.as_str()));
    }
    value
}

impl FromJson for Facing {
    const HOLDS_NULL: bool = true;

    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        named(
            node,
            read,
            FACING,
            &Self::ALL.map(Self::name),
            Self::from_name,
        )
    }
}

impl FromJson for Condition {
    const HOLDS_NULL: bool = true;

    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        named(
            node,
            read,
            CONDITION,
            &Self::ALL.map(Self::name),
            Self::from_name,
        )
    }
}

/// `cube`: an integer of at least 0 (`schema::read_with`). The minimum is inclusive.
pub(super) fn cube(node: &Node<'_>, read: &mut Read<'_>) -> Option<i32> {
    let Some(number) = node.value().as_number() else {
        read.refuse(node, cereal::invalid_numeric_value());
        return None;
    };
    let wide = number.to_f64_lossy();
    if wide < 0.0 {
        read.refuse(node, cereal::NumericRange::at_least(0.0).code());
        return None;
    }
    // A fraction or an index beyond `int` is not a cube index; vanilla's text for it is not
    // known.
    #[allow(clippy::cast_possible_truncation)] // whole and in range, checked here
    let index = wide as i32;
    if wide.fract() != 0.0 || wide > f64::from(i32::MAX) {
        read.refuse(node, Code::from(&core_codes::INVALID_VALUE));
        return None;
    }
    Some(index)
}

/// Vanilla's breadcrumb of a path below the payload: `-> rules[0] -> direction`.
pub(super) fn breadcrumb(at: &JsonPath) -> String {
    let mut out = String::new();
    for segment in at.segments() {
        match segment {
            JsonPathSegment::Key(key) => {
                if !out.is_empty() {
                    out.push(' ');
                }
                out.push_str("-> ");
                out.push_str(key);
            }
            JsonPathSegment::Index(index) => {
                out.push('[');
                out.push_str(&index.to_string());
                out.push(']');
            }
        }
    }
    out
}

// Leaves of the derived check and Molang walk: neither holds a bound or Molang.
bedrock_addon_core::impl_schema_leaf!(Facing, Condition);
bedrock_addon_core::impl_no_molang!(Facing, Condition);
