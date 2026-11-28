use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

#[derive(Facet, Schema)]
struct Read {
    #[facet(schema::read_by = "")]
    none: Option<u8>,
    #[facet(schema::read_by = "sprite text sprite")]
    twice: Option<u8>,
}

fn main() {}
