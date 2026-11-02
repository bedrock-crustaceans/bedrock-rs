use bedrock_addon_derive::Components;

struct Toy;
struct Friction(f32);
struct Glint(bool);

fn parse() {}

// Enum-level errors do not hide the variants' errors: all three are reported.
#[derive(Components)]
#[component(custom = Custom, colour = "red")]
enum ToyComponent {
    Friction(Friction),
    Custom(bedrock_addon_core::document::CustomComponent),
}

fn main() {}
