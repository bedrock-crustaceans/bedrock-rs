use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

#[derive(Facet, Schema)]
#[facet(tag = "type", schema::expected = "a shape", schema::unknown = "keep")]
#[repr(u8)]
enum Internal {
    Count(u8),
    #[facet(schema::min = 0)]
    Pair { a: u8 },
    #[facet(schema::unknown = "keep")]
    Bare,
}

#[derive(Facet, Schema)]
#[repr(u8)]
enum External {
    Both(u8, u8),
    One(u8),
}

fn main() {}
