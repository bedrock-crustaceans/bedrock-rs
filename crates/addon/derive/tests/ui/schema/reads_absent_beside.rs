use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

fn beside(_: &Node<'_>, _: &Node<'_>, _: &mut Read<'_>) -> Option<i32> {
    None
}
fn absent(_: &Node<'_>, _: &mut Read<'_>) -> Option<i32> {
    None
}

#[derive(Facet, Schema)]
struct Beside {
    #[facet(schema::reads_absent, schema::read_beside = beside)]
    a: Option<i32>,
    #[facet(schema::reads_absent, schema::absent_with = absent)]
    b: Option<i32>,
}

fn main() {}
