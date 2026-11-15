use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

#[derive(Facet, Schema)]
struct Inner {
    v: Option<u8>,
}

fn inner(_: &Node<'_>, _: &mut Read<'_>) -> Option<Inner> {
    None
}

#[derive(Facet, Schema)]
#[facet(tag = "type", content = "value")]
#[repr(u8)]
enum Adjacent {
    One(Inner),
}

#[derive(Facet, Schema)]
#[facet(tag = "type")]
#[repr(u8)]
enum Internal {
    One(#[facet(schema::read_with = inner)] Inner),
}

fn main() {}
