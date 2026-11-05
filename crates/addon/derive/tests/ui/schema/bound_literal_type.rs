use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

#[derive(Facet, Schema)]
struct Radius {
    #[facet(default = 2.0, schema::min = 0)]
    radius: f32,
}

fn main() {}
