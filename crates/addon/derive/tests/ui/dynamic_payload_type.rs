use bedrock_addon_derive::Components;

struct Toy;
struct Friction(f32);
struct Glint(bool);

fn parse() {}

#[derive(Components)]
#[component(from_only, unknown = Unknown, dynamic = Dynamic)]
enum ToyComponentV1 {
    Friction(Friction),
    Unknown(bedrock_addon_core::json::Value),
    Dynamic(bedrock_addon_core::document::CustomComponent),
}

fn main() {}
