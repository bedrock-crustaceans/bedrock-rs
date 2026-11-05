use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

#[derive(Facet, Schema)]
#[facet(schema::min = 0, schema::expected = "a name")]
struct Misplaced {
    #[facet(schema::case_insensitive)]
    a: Option<i32>,
    #[facet(skip, schema::fallback = 1)]
    b: Option<i32>,
}

#[derive(Facet, Schema)]
#[facet(schema::lowercase)]
#[repr(u8)]
enum Slot {
    #[facet(schema::expected = "a slot")]
    Head,
}

fn main() {}
