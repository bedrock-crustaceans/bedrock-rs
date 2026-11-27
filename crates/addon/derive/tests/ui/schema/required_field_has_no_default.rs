use bedrock_addon_core::schema::Schema;
use facet::Facet;

#[derive(Facet, Schema)]
struct Boss {
    #[facet(default = 55)]
    hud_range: i32,
    color: u32,
}

fn main() {
    let _ = Boss::default();
}
