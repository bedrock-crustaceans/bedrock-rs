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
    #[facet(schema::drop)]
    not_a_list: Option<u8>,
    #[facet(schema::if_kind)]
    always_read: u8,
}

fn main() {}
