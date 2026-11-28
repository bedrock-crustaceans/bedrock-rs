use bedrock_addon_core::diagnostic::codes;
use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

#[derive(Facet, Schema)]
#[facet(schema::arity = codes::DROPPED_FIELD)]
struct Named {
    a: Option<i32>,
}

#[derive(Facet, Schema)]
#[facet(schema::arity = codes::DROPPED_FIELD)]
struct Lacking(i32, Option<i32>);

#[derive(Facet, Schema)]
struct Moved(#[facet(skip)] i32, i32);

#[derive(Facet, Schema)]
struct Member(#[facet(rename = "a")] i32, #[facet(schema::required)] Option<i32>);

#[derive(Facet, Schema)]
struct OnAField(#[facet(schema::arity = codes::DROPPED_FIELD)] i32);

fn main() {}
