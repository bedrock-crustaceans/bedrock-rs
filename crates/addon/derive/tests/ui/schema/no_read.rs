use bedrock_addon_core::schema::Schema;
use facet::Facet;

/// A type with a check and a Molang walk, and no read.
#[derive(Facet)]
struct Unread {
    v: u8,
}

bedrock_addon_core::impl_schema_leaf!(Unread);
bedrock_addon_core::impl_no_molang!(Unread);

#[derive(Facet, Schema)]
struct Holder {
    inner: Option<Unread>,
    plain: Unread,
}

fn main() {}
