use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

#[derive(Facet, Schema)]
#[facet(schema::min = 0, schema::string_only)]
struct Misplaced {
    #[facet(schema::string_only)]
    a: Option<i32>,
    #[facet(skip, schema::fallback = 1)]
    b: Option<i32>,
}

#[derive(Facet, Schema)]
#[facet(schema::finite)]
#[repr(u8)]
enum Slot {
    #[facet(schema::string_only)]
    Head,
}

fn main() {}
