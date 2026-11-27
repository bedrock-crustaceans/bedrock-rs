use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

#[derive(Facet, Schema)]
struct Boss {
    #[facet(schema::required)]
    hud_range: i32,
}

fn main() {}
