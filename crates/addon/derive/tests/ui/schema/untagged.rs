use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

#[derive(Facet, Schema)]
#[facet(untagged, schema::expected = "a value", schema::unknown = "keep")]
#[repr(u8)]
enum Arms {
    Pair(i32, i32),
    #[facet(schema::min = 0)]
    Count(i32),
    #[facet(schema::unknown = "kept")]
    Params { a: i32 },
}

#[derive(Facet, Schema)]
#[facet(schema::order = "declared")]
#[repr(u8)]
enum Slot {
    Head,
}

#[derive(Facet, Schema)]
#[facet(schema::order = "game")]
struct Ordered {
    a: i32,
}

fn main() {}
