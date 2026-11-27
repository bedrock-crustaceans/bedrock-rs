use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

#[derive(Facet, Schema)]
#[facet(schema::quiet = 5)]
struct NotNames {
    a: Option<i32>,
}

#[derive(Facet, Schema)]
struct OnAField {
    #[facet(schema::after_read = nothing, schema::quiet = "a")]
    a: Option<i32>,
}

#[derive(Facet, Schema)]
#[facet(schema::after_read = nothing, schema::after_read = nothing)]
struct Twice {
    a: Option<i32>,
}

fn main() {}
