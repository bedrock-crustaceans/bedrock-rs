use bedrock_addon_derive::Components;

struct Toy;
struct Friction(f32);
struct Glint(bool);

fn parse() {}

#[derive(Components)]
#[component(from_only, custom = Custom, unknown = Unknown)]
enum ToyComponentV1 {
    Friction(Friction),
    Custom(String),
    Unknown(bedrock_addon_core::json::Value),
}

fn main() {}
