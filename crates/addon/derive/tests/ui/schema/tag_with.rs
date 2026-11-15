use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

#[derive(Facet, Schema)]
#[facet(schema::tag_with = name_of)]
struct Plain {
    n: u8,
}

#[derive(Facet, Schema)]
#[facet(schema::tag_with = name_of)]
#[repr(u8)]
enum External {
    A(u8),
}

fn main() {}
