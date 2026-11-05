use bedrock_addon_core::schema::Schema;
use facet::Facet;

// facet's container `default` fills a missing member from the field type's `Default`, not from
// the field's own `default = ..`: a `Schema` struct states each field's instead. (The hand
// `Default` is what facet's container `default` needs; it keeps facet's own error out.)
#[derive(Facet, Schema)]
#[facet(default)]
struct Filled {
    #[facet(default = 2.0)]
    radius: f32,
    count: i32,
}

impl Default for Filled {
    fn default() -> Self {
        Self { radius: 2.0, count: 0 }
    }
}

fn main() {}
