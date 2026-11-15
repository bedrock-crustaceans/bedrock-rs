use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

#[derive(Facet, Schema)]
struct Shapes {
    #[facet(schema::iterated)]
    one: Option<u8>,
    #[facet(schema::or_kept = "told")]
    plain: u8,
    #[facet(schema::reads_null)]
    never_null: u8,
}

fn main() {}
