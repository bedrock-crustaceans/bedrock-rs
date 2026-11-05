use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

// A `skip` field is not read, so the door never reads its stored keys either.
#[derive(Facet, Schema)]
struct Skipped {
    #[facet(skip, schema::since_format = "1.21.60")]
    a: Option<i32>,
    #[facet(skip, schema::since_engine = "1.20.0")]
    b: Option<i32>,
    #[facet(skip, schema::unknown = "keep")]
    c: Option<i32>,
}

fn main() {}
