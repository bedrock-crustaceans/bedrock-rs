use bedrock_addon_core::schema::{self, Schema, SchemaCx};
use facet::Facet;

fn rule(_: &mut Rules, _: &mut SchemaCx<'_>) {}

#[derive(Facet, Schema)]
#[facet(schema::min_len = 1)]
struct Misplaced {
    a: Option<Vec<u8>>,
}

#[derive(Facet, Schema)]
struct Rules {
    #[facet(schema::rule = rule)]
    a: Option<u8>,
    #[facet(schema::min_len = "one")]
    b: Option<Vec<u8>>,
    #[facet(schema::max_len = 1, schema::max_len = 2)]
    c: Option<Vec<u8>>,
}

#[derive(Facet, Schema)]
#[facet(schema::rule = rule, schema::rule = rule)]
#[repr(u8)]
enum Slot {
    Head,
}

fn main() {}
