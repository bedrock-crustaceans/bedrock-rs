use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

#[derive(Facet, Schema)]
struct Typo {
    #[facet(schema::fallbak = 1)]
    a: Option<i32>,
}

fn main() {}
