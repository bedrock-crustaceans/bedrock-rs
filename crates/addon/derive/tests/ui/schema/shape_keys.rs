use bedrock_addon_core::json::Value;
use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

#[derive(Facet, Schema)]
struct Shapes {
    #[facet(schema::iterated)]
    one: Option<u8>,
    #[facet(schema::elements = "skip")]
    list: Option<Vec<u8>>,
    #[facet(schema::or_kept = "told")]
    plain: u8,
    #[facet(schema::otherwise)]
    other: Option<u8>,
}

#[derive(Facet, Schema)]
#[facet(untagged)]
#[repr(u8)]
enum Arms {
    Count(u8),
    #[facet(schema::otherwise)]
    Pair { a: u8 },
    #[facet(schema::otherwise)]
    Rest(Value),
}

fn main() {}
