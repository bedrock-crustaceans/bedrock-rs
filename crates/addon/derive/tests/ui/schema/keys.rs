use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::schema::{self, KeyPattern, Keys, Schema};
use facet::Facet;
use indexmap::IndexMap;

const NAMES: Keys = Keys::new(KeyPattern::IDENTIFIER);

fn names(_: &Node<'_>, _: &mut Read<'_>) -> Option<IndexMap<String, u8>> {
    None
}

#[derive(Facet, Schema)]
struct Bad {
    #[facet(schema::keys = NAMES)]
    a: Vec<u8>,
    #[facet(schema::keys = NAMES, schema::read_with = names)]
    b: IndexMap<String, u8>,
}

fn main() {}
