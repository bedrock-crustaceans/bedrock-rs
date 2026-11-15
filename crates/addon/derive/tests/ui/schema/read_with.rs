use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

fn level(_: &Node<'_>, _: &mut Read<'_>) -> Option<u8> {
    None
}

#[derive(Facet, Schema)]
struct Inner {
    v: Option<u8>,
}

#[derive(Facet, Schema)]
struct Reads {
    #[facet(flatten, schema::read_with = level)]
    inner: Inner,
    #[facet(schema::read_with = level)]
    extra: UnknownMembers,
    #[facet(schema::read_with = level, schema::read_with = level)]
    twice: u8,
}

fn main() {}
