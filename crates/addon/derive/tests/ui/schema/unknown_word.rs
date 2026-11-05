use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

#[derive(Facet, Schema)]
#[facet(schema::unknown = "kept")]
struct Loose {
    #[facet(schema::unknown = "drop")]
    a: Option<i32>,
}

fn main() {}
