use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

#[derive(Facet, Schema)]
struct Gated {
    #[facet(schema::since_format = "1.21")]
    a: Option<i32>,
    #[facet(schema::since_engine = "1.+2.0")]
    b: Option<i32>,
    #[facet(schema::since_format = SINCE)]
    c: Option<i32>,
}

const SINCE: &str = "1.21.60";

fn main() {}
