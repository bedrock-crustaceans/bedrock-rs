use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

// A string enum has no members, so no unknown ones.
#[derive(Facet, Schema)]
#[facet(schema::unknown = "keep")]
#[repr(u8)]
enum Slot {
    Head,
}

fn main() {}
