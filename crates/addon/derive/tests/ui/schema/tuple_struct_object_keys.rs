use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::schema::{self, Schema, UnknownRule};
use facet::Facet;

fn rule(_: &mut Pair, _: &Node<'_>, _: &mut Read<'_>) {}
fn guard(_: &Node<'_>, _: &mut Read<'_>) -> bool {
    true
}
fn unknown(_: &str, _: &Node<'_>, _: &mut Read<'_>) -> UnknownRule {
    UnknownRule::Silent
}
fn absent(_: &Node<'_>, _: &mut Read<'_>) -> Option<i32> {
    None
}

#[derive(Facet, Schema)]
#[facet(schema::after_members = rule, schema::guard = guard, schema::quiet = "a")]
#[facet(schema::unknown = "silent", schema::unknown_with = unknown)]
struct Pair(i32, i32);

#[derive(Facet, Schema)]
struct Fields(
    #[facet(schema::guard = guard)] i32,
    #[facet(schema::ends)] i32,
    #[facet(schema::scope)] i32,
    #[facet(schema::if_kind)] Option<i32>,
    #[facet(schema::reads_absent)] Option<i32>,
    #[facet(schema::absent_with = absent)] Option<i32>,
);

fn main() {}
