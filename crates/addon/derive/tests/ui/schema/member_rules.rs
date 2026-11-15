use bedrock_addon_core::diagnostic::codes;
use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

#[derive(Facet, Schema)]
struct Inner {
    v: Option<u8>,
}

#[derive(Facet, Schema)]
struct Rules {
    #[facet(schema::finite)]
    name: Option<String>,
    #[facet(schema::missing = codes::INVALID_VALUE)]
    optional: Option<u8>,
    #[facet(default = 1, schema::missing = codes::INVALID_VALUE)]
    defaulted: u8,
    #[facet(flatten, schema::wrong_type = codes::INVALID_VALUE)]
    inner: Inner,
}

#[derive(Facet, Schema)]
#[facet(schema::string_only)]
struct NotAnEnum {
    v: u8,
}

fn main() {}
