use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

#[derive(Facet, Schema)]
struct Range {
    #[facet(schema::min = "zero")]
    a: f32,
    #[facet(schema::min = a + 1.0)]
    b: f32,
    #[facet(schema::max = c)]
    c: f32,
    #[facet(schema::below = missing)]
    d: f32,
    #[facet(schema::min = 0.0, schema::above = 1.0)]
    e: f32,
    #[facet(schema::max = 1.0, schema::max = 2.0)]
    f: f32,
}

fn main() {}
