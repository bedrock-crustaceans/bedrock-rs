use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

#[derive(Facet, Schema)]
#[facet(untagged)]
#[repr(u8)]
enum Arms {
    #[facet(schema::unknown = "keep")]
    Count(i32),
    #[facet(schema::unknown = "report")]
    Off,
    #[facet(schema::unknown = "keep")]
    Params { a: i32 },
}

fn main() {}
