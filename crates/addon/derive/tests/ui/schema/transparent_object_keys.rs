use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

fn rule(_: &mut Wrapped, _: &Node<'_>, _: &mut Read<'_>) {}
fn guard(_: &Node<'_>, _: &mut Read<'_>) -> bool {
    true
}

#[derive(Facet, Schema)]
#[facet(transparent, schema::after_read = rule, schema::after_members = rule, schema::guard = guard, schema::quiet = "a")]
struct Wrapped(i32);

fn main() {}
