use bedrock_addon_core::diagnostic::{Code, codes};
use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

fn low() -> Code {
    Code::from(&codes::INVALID_VALUE)
}

fn refuse(_: &str) -> Option<Code> {
    None
}

#[derive(Facet, Schema)]
struct NoBound {
    #[facet(schema::low_code = low())]
    a: Option<u8>,
    #[facet(schema::max = 5, schema::low_code = low())]
    b: Option<u8>,
    #[facet(schema::kept)]
    c: Option<u8>,
    #[facet(schema::key_with = refuse)]
    d: Option<Vec<u8>>,
}

fn main() {}
