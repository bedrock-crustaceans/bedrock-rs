use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

#[derive(Facet, Schema)]
struct Wrong {
    #[facet(schema::lowercase)]
    count: Option<i32>,
    #[facet(schema::foreign_event)]
    event: Option<String>,
}

fn main() {}
