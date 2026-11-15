use bedrock_addon_core::diagnostic::{Code, codes};
use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

fn skipped() -> Code {
    Code::from(&codes::INVALID_VALUE)
}

#[derive(Facet, Schema)]
struct Bad {
    #[facet(schema::skip_code = skipped())]
    a: Vec<u8>,
    #[facet(schema::elements = "fail", schema::skip_code = skipped())]
    b: Vec<u8>,
    #[facet(schema::elements = "drop", schema::skip_code = skipped())]
    c: Option<u8>,
}

fn main() {}
