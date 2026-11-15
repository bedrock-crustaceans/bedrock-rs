use bedrock_addon_core::diagnostic::{Code, codes};
use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

fn low() -> Code {
    Code::from(&codes::INVALID_VALUE)
}

#[derive(Facet, Schema)]
struct NoBound {
    #[facet(schema::low_code = low())]
    a: Option<u8>,
    #[facet(schema::max = 5, schema::low_code = low())]
    b: Option<u8>,
    #[facet(schema::kept)]
    c: Option<u8>,
}

fn main() {}
