use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

#[derive(Facet, Schema)]
struct Inner {
    v: Option<u8>,
}

#[derive(Facet, Schema)]
struct Fed {
    #[facet(flatten, schema::members = ["a", "b"])]
    inner: Inner,
    #[facet(schema::members = ["c", "d"])]
    no_default: u8,
    #[facet(schema::members = "e", default)]
    not_a_list: u8,
    #[facet(schema::members = ["f", "f"], default)]
    twice: u8,
    #[facet(schema::members = ["g"], schema::reads_null)]
    one_member: Option<u8>,
}

fn main() {}
