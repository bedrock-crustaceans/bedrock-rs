use bedrock_addon_core::diagnostic::{Class, CodeDef, Kind};
use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

// A member's code takes no argument or one, the read's subject: two do not compile.
static TWO: CodeDef<2> = CodeDef::new("demo/two", Kind::WrongType, Class::Error, "{} and {}");

#[derive(Facet, Schema)]
struct Coded {
    #[facet(schema::wrong_type = TWO)]
    count: Option<u8>,
}

fn main() {}
