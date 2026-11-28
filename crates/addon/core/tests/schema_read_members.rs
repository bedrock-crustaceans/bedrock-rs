//! A field fed by several members of its struct's object (`schema::members`): read once at its
//! place, whether or not any member is written, none of them an unknown member, the read
//! handed each member as written and the field's default.

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, JsonPath, codes};
use bedrock_addon_core::json::kit::{MemberOrder, Node, Profile, Read, UnknownAt};
use bedrock_addon_core::json::{Value, from_str_jsonc};
use bedrock_addon_core::schema::{self, FromJson, FromMembers, Schema};
use bedrock_addon_core::versioning::Reading;
use facet::Facet;

/// A count, or the name of the entry that holds it, and the member it was read from.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
struct Count {
    value: u32,
    entry: String,
    from: Option<String>,
}
bedrock_addon_core::impl_schema_leaf!(Count);
bedrock_addon_core::impl_no_molang!(Count);

impl Count {
    fn entry(entry: &str) -> Self {
        Self {
            entry: entry.to_owned(),
            ..Self::default()
        }
    }
}

impl FromMembers<2> for Count {
    fn from_members(
        _: &Node<'_>,
        members: &[Option<Node<'_>>; 2],
        default: Self,
        read: &mut Read<'_>,
    ) -> Option<Self> {
        let [count, entry] = members;
        // Written together, the count wins and the entry is not read.
        if let Some(count) = count {
            return Some(Self {
                value: u32::from_json(count, read)?,
                entry: String::new(),
                from: Some(count.path().to_string()),
            });
        }
        let Some(entry) = entry else {
            return Some(default);
        };
        Some(Self {
            entry: read.string(entry)?.to_owned(),
            from: Some(entry.path().to_string()),
            ..default
        })
    }
}

/// Whether `one` or `other` was written, by a hand read that tells `one` missing when neither
/// is.
#[allow(
    clippy::unnecessary_wraps,
    clippy::option_option,
    reason = "a `schema::members` read of an `Option` field"
)]
fn which(
    object: &Node<'_>,
    members: &[Option<Node<'_>>; 2],
    default: Option<String>,
    read: &mut Read<'_>,
) -> Option<Option<String>> {
    let [one, other] = members;
    let written = match (one, other) {
        (Some(_), Some(_)) => "both",
        (Some(_), None) => "one",
        (None, Some(_)) => "other",
        (None, None) => {
            read.push(
                Diagnostic::of(&codes::MISSING_REQUIRED_FIELD).at(object.path().child("one")),
            );
            return Some(default);
        }
    };
    Some(Some(written.to_owned()))
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Repeat {
    first: Option<String>,
    #[facet(schema::members = ["times", "times_id"], default = Count::entry("times"))]
    count: Count,
    #[facet(schema::members = ["one", "other"], schema::read_with = which)]
    pick: Option<String>,
    last: Option<String>,
}

/// `text` read as a `Repeat` by `profile`: the value and each finding as `path code`.
fn read(text: &str, profile: Profile) -> (Option<Repeat>, Vec<String>) {
    let json: Value = from_str_jsonc(text).expect("JSON");
    let mut found = Diagnostics::default();
    let mut read = Read::new(Reading::OWN, &mut found).profile(profile);
    let value = Repeat::from_json(&Node::new(&json, JsonPath::new()), &mut read);
    read.finish();
    let told = found
        .iter()
        .map(|f| format!("{} {}", f.location.json_path, f.code.id()))
        .collect();
    (value, told)
}

const DECLARED: Profile = Profile::EVERY_FAILURE
    .with_members(MemberOrder::Declaration)
    .with_unknown(UnknownAt::Last);

#[test]
fn a_field_fed_by_two_members_reads_them_at_its_place_and_neither_is_unknown() {
    let (value, told) = read(
        r#"{"last": 1, "times_id": 2, "first": 3, "zz": 0}"#,
        DECLARED,
    );
    assert_eq!(
        told,
        [
            "first core/invalid_type",
            "times_id core/invalid_type",
            "one core/missing_required_field",
            "last core/invalid_type",
            "zz core/unrecognized_field",
        ],
        "the members are read where the field is declared, and are no unknown members"
    );
    assert_eq!(value, None);
    let (value, told) = read(r#"{"times_id": "laps", "one": true}"#, DECLARED);
    assert!(told.is_empty(), "{told:?}");
    let value = value.expect("read");
    assert_eq!(
        value.count,
        Count {
            value: 0,
            entry: "laps".to_owned(),
            from: Some("times_id".to_owned())
        }
    );
    assert_eq!(value.pick.as_deref(), Some("one"));
}

#[test]
fn two_members_written_together_are_merged_by_the_read() {
    let (value, told) = read(
        r#"{"times": 3, "times_id": "laps", "other": 1, "one": 2}"#,
        DECLARED,
    );
    assert!(told.is_empty(), "{told:?}");
    let value = value.expect("read");
    assert_eq!(
        value.count,
        Count {
            value: 3,
            entry: String::new(),
            from: Some("times".to_owned())
        }
    );
    assert_eq!(value.pick.as_deref(), Some("both"));
}

#[test]
fn a_field_none_of_whose_members_is_written_is_read_with_its_default() {
    for profile in [DECLARED, Profile::EVERY_FAILURE] {
        let (value, told) = read(r#"{"first": "a"}"#, profile);
        assert_eq!(
            told,
            ["one core/missing_required_field"],
            "the read is called"
        );
        let value = value.expect("read");
        assert_eq!(value.count, Count::entry("times"));
        assert_eq!(value.pick, None);
    }
}

#[test]
fn in_file_order_the_field_is_read_where_the_file_has_the_first_of_its_members() {
    let (_, told) = read(
        r#"{"last": 1, "times_id": 2, "first": 3, "times": "x"}"#,
        Profile::EVERY_FAILURE,
    );
    assert_eq!(
        told,
        [
            "last core/invalid_type",
            "times core/invalid_type",
            "first core/invalid_type",
            "one core/missing_required_field",
        ],
        "a field none of whose members the file has is read after the members"
    );
}
