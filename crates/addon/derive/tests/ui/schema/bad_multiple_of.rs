use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

#[derive(Facet, Schema)]
struct Steps {
    #[facet(schema::multiple_of = 0)]
    a: i32,
    #[facet(schema::multiple_of = -16)]
    b: i32,
    #[facet(schema::multiple_of = "16")]
    c: i32,
    #[facet(schema::multiple_of = a)]
    d: i32,
    #[facet(schema::multiple_of = 16, schema::multiple_of = 32)]
    e: i32,
}

fn main() {}
