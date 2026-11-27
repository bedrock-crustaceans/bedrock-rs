//! The generic shapes held by a `#[derive(Facet, Schema)]` struct, as a family holds them: the
//! derive's check and Molang walks reach them through their hand `SchemaFields` and
//! `MolangFields` impls (the derive takes no generics, and the leaf macros none either).

use bedrock_addon_test_support::value as common;

use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
use bedrock_addon_core::schema::{self, Schema, SchemaCx};
use bedrock_addon_core::versioning::Reading;
use bedrock_addon_shared_shapes::HexOr;
use common::round_trip;
use facet::Facet;

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Holder {
    #[facet(schema::min = 0)]
    count: i32,
    color: HexOr<[i32; 3]>,
    tint: Option<HexOr<[f32; 3]>>,
}

#[test]
fn a_schema_struct_holds_the_generic_shapes() {
    let mut holder: Holder =
        round_trip(r##"{"count": -1, "color": "#a0b0c0", "tint": [1.0, 0.5, 0.0]}"##);
    assert_eq!(holder.color, HexOr::Hex("#a0b0c0".to_owned()));
    assert_eq!(holder.tint, Some(HexOr::Array([1.0, 0.5, 0.0])));
    // The derive's check walks the struct: its own bound tells, the shapes walk nothing.
    let mut found = Diagnostics::default();
    let mut cx = SchemaCx::new(Reading::DOCUMENT, JsonPath::new(), &mut found);
    holder.check(&mut cx);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(Holder::triggers().is_empty());
}
