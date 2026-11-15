//! The bridge: `FromJson` for a facet type no family has converted yet, read by today's typed
//! door (`versioning::parse_payload`'s steps) at the node's text, its findings handed to the
//! read in the passes they belong to. It exists so that every `#[derive(Schema)]` type compiles
//! with its derived read from the first day; the run deletes it, and every use of it, once the
//! last type is converted (`rg from_json_via_facet`).

use facet::Facet;

use crate::diagnostic::codes as core_codes;
use crate::diagnostic::{Code, Diagnostic, Diagnostics, Location, Phase};
use crate::json::kit::{Node, Pass, Read};
use crate::json::{JsonSlice, NullRule, ScalarRead, parse_with_rules, reject_nulls};
use crate::schema::UnknownRule;

/// `T` read by the typed door at `node` (its text, or the value written out where the node has
/// none, its findings then without spans), under the read's reading: a failed typed read is a
/// failed member ([`Read::refuse_with`]); its `null` errors, the members it read as absent, its
/// unknown members and its misread scalars are told in their [`Pass`]es; the unknown members a
/// struct of it keeps go to the document's table at the node's path.
pub fn read_via_facet<T: for<'a> Facet<'a>>(node: &Node<'_>, read: &mut Read<'_>) -> Option<T> {
    if read.stopped() {
        return None;
    }
    let written;
    let (slice, spans) = if let Some(text) = node.text() {
        (text, true)
    } else {
        written = crate::json::to_json(node.value()).unwrap_or_default();
        (JsonSlice::new(&written), false)
    };
    let at = node.path();
    let reading = read.reading();
    let rule = reading.null_rule();
    let ScalarRead {
        value,
        told,
        json,
        absent,
    } = match parse_with_rules::<T>(slice, rule, reading.scalar_rules()) {
        Ok(read) => read,
        Err(failure) => {
            let mut finding = crate::versioning::failure_finding(&failure, at, slice);
            if !spans {
                finding.location.span = None;
            }
            read.refuse_with(finding);
            return None;
        }
    };
    if rule == Some(NullRule::Rejected) {
        let mut nulls = Diagnostics::default();
        if reject_nulls(&value, slice, at, &mut nulls) {
            for mut finding in nulls {
                if !spans {
                    finding.location.span = None;
                }
                read.defer(Pass::Null, finding);
            }
            return None;
        }
    }
    let Some(json) = json else {
        return Some(value);
    };
    let mut absent_told = Diagnostics::default();
    crate::json::tell_null_members(&absent, at, &mut absent_told);
    for finding in absent_told {
        read.defer(Pass::Absent, finding);
    }
    let mut kept = crate::document::Unknown::new();
    // The word of the field that holds the value is the first object's below it.
    let unknown = crate::json::held_unknown_members(
        &value,
        &json,
        reading.keeps_unknown,
        read.field_unknown(),
        &mut kept,
    );
    if read.keepable() {
        read.findings_mut().unknown_mut().absorb(at, kept);
    }
    let severity = read.unknown_grade();
    for (path, rule) in unknown {
        let code = match rule {
            UnknownRule::Silent => continue,
            UnknownRule::Reject => Code::from(&core_codes::DROPPED_FIELD),
            UnknownRule::Ignore => Code::from(&core_codes::IGNORED_MEMBER),
            UnknownRule::Component => Code::from(&core_codes::UNRECOGNIZED_COMPONENT),
            UnknownRule::Keep | UnknownRule::Report => Code::from(&core_codes::UNRECOGNIZED_FIELD),
        };
        let mut full = at.clone();
        full.append(&path);
        read.defer(
            Pass::Unknown,
            Diagnostic::new(
                code,
                severity,
                Phase::Parse,
                Location::new(None, None, full, None),
            ),
        );
    }
    for told in told {
        read.defer(Pass::Misread, told.finding(at));
    }
    Some(value)
}

/// `FromJson` for each facet type named, read by the bridge ([`read_via_facet`]): a type no
/// family has converted to the derived read yet. Each use carries a `// bridge: <why>` comment;
/// the run deletes the bridge, and every use of it, when the last type is converted. A generic
/// type is named as `impl<T: Bound> Type<T>`.
///
/// ```
/// use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
/// use bedrock_addon_core::json::kit::{Node, Read};
/// use bedrock_addon_core::json::{JsonSlice, Value, from_str_jsonc};
/// use bedrock_addon_core::schema::FromJson;
/// use bedrock_addon_core::versioning::Reading;
///
/// #[derive(Debug, PartialEq, facet::Facet)]
/// struct Legacy {
///     level: u8,
/// }
///
/// // bridge: the example's type is read by facet
/// bedrock_addon_core::from_json_via_facet!(Legacy);
///
/// let text = r#"{"level": 3}"#;
/// let json: Value = from_str_jsonc(text).unwrap();
/// let mut found = Diagnostics::default();
/// let mut read = Read::new(Reading::DOCUMENT, &mut found);
/// let node = Node::with_text(&json, JsonPath::new(), JsonSlice::new(text));
/// assert_eq!(Legacy::from_json(&node, &mut read), Some(Legacy { level: 3 }));
/// ```
#[macro_export]
macro_rules! from_json_via_facet {
    ($(impl<$($param:ident $(: $bound:path)?),+> $ty:ty),+ $(,)?) => {
        $(
            impl<$($param $(: $bound)?),+> $crate::schema::FromJson for $ty
            where
                $ty: for<'__a> $crate::__private::Facet<'__a>,
            {
                fn from_json(
                    node: &$crate::json::kit::Node<'_>,
                    read: &mut $crate::json::kit::Read<'_>,
                ) -> ::core::option::Option<Self> {
                    $crate::schema::read_via_facet::<Self>(node, read)
                }
            }
        )+
    };
    ($($ty:ty),+ $(,)?) => {
        $(
            impl $crate::schema::FromJson for $ty {
                fn from_json(
                    node: &$crate::json::kit::Node<'_>,
                    read: &mut $crate::json::kit::Read<'_>,
                ) -> ::core::option::Option<Self> {
                    $crate::schema::read_via_facet::<Self>(node, read)
                }
            }
        )+
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::JsonPath;
    use crate::versioning::Reading;

    #[derive(Debug, PartialEq, facet::Facet)]
    struct Inner {
        level: u8,
    }

    #[test]
    fn a_bridged_type_tells_the_door_s_findings_at_its_node() {
        let text = r#"{"level": "3", "extra": 1}"#;
        let json: crate::json::Value = crate::json::from_str_jsonc(text).unwrap();
        let mut found = Diagnostics::default();
        let mut read = Read::new(Reading::DOCUMENT, &mut found);
        let node = Node::with_text(&json, JsonPath::of_keys(["inner"]), JsonSlice::new(text));
        let value = read_via_facet::<Inner>(&node, &mut read);
        assert_eq!(value, Some(Inner { level: 3 }));
        let told: Vec<String> = found
            .iter()
            .map(|f| format!("{} {}", f.location.json_path, f.code.id()))
            .collect();
        assert_eq!(
            told,
            [
                "inner/extra core/unrecognized_field",
                "inner/level core/invalid_type"
            ]
        );
    }

    use crate::schema::{self, Schema};

    /// A bridged struct, held by a field that says `schema::unknown = "keep"`.
    #[derive(Debug, PartialEq, facet::Facet, Schema)]
    struct Holds {
        #[facet(schema::unknown = "keep")]
        held: Option<Bridged>,
    }

    #[derive(Debug, PartialEq, facet::Facet)]
    struct Bridged {
        level: u8,
    }

    // bridge: the test's type is read by facet
    crate::from_json_via_facet!(Bridged);
    crate::impl_schema_leaf!(Bridged);
    crate::impl_no_molang!(Bridged);

    /// The word of the field that holds a bridged struct is the door's for that struct's
    /// object: its unknown members are kept, as `parse_payload` keeps them.
    #[test]
    fn the_word_of_the_field_that_holds_a_bridged_struct_is_honoured() {
        use crate::load::LoadContext;
        use crate::version::{FormatVersion, FormatVersionInput};
        use crate::versioning::{ParseCx, parse_payload, read_payload};
        let load = LoadContext::detached();
        let cx = ParseCx {
            load: &load,
            written: FormatVersionInput {
                version: Some(FormatVersion::new(1, 0, 0)),
                beta_literal: false,
                prerelease: false,
            },
            original_version: FormatVersion::new(1, 0, 0),
            at: JsonPath::new(),
        };
        let text = JsonSlice::new(r#"{"held": {"level": 1, "zz": 2}}"#);
        let kept = |d: &Diagnostics| -> Vec<String> {
            d.unknown()
                .iter()
                .map(|(at, key, _)| format!("{at}/{key}"))
                .collect()
        };
        let mut door = Diagnostics::default();
        let _: Option<Holds> = parse_payload(text, &cx, &mut door);
        let mut read = Diagnostics::default();
        let _: Option<Holds> = read_payload(text, &cx, &mut read);
        assert_eq!(kept(&door), ["held/zz"]);
        assert_eq!(kept(&read), kept(&door));
    }
}
