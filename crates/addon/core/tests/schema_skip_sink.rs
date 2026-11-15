//! A `skip` field of type `UnknownMembers` is not the sink of the unknown members: the one
//! that is not `skip` keeps them, and the `skip` one is what its `Default` makes it.

use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::json::{Value, from_str_jsonc};
use bedrock_addon_core::schema::{FromJson, Schema};
use bedrock_addon_core::versioning::Reading;
use facet::Facet;

#[derive(Clone, Debug, Default, PartialEq, Facet, Schema)]
struct Held {
    #[facet(default)]
    name: String,
    #[facet(skip)]
    apart: UnknownMembers,
    extra: UnknownMembers,
}

#[test]
fn a_skip_unknown_members_field_keeps_nothing() {
    let json: Value = from_str_jsonc(r#"{"name": "n", "zz": 1}"#).expect("JSON");
    let mut found = Diagnostics::default();
    let mut read = Read::new(Reading::OWN, &mut found);
    let held = Held::from_json(&Node::new(&json, JsonPath::new()), &mut read).expect("read");
    assert_eq!(held.name, "n");
    assert_eq!(
        held.extra.iter().map(|(key, _)| key).collect::<Vec<_>>(),
        ["zz"]
    );
    assert!(held.apart.is_empty());
}
