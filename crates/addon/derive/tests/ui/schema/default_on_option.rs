use bedrock_addon_core::schema::Schema;
use facet::Facet;

#[derive(Facet, Schema)]
struct Goal {
    #[facet(default = 1)]
    priority: Option<i32>,
    #[facet(default)]
    speed: Option<f32>,
}

fn main() {}
