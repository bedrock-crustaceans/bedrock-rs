use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

// Facet takes `crate::schema::min` as it takes `schema::min`; the derive reads the key written
// exactly `schema::min` and would drop this one in silence.
#[derive(Facet, Schema)]
struct Pathed {
    #[facet(crate::schema::min = 1)]
    a: i32,
    #[facet(schema::min = 1)]
    plain: i32,
}

#[derive(Facet, Schema)]
#[facet(bedrock_addon_core::schema::molang = bedrock_addon_core::molang::NONE)]
struct Qualified {
    b: Option<i32>,
}

fn main() {}
