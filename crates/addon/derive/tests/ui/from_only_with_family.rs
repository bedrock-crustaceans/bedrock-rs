use bedrock_addon_derive::Components;

struct Toy;
struct Friction(f32);
struct Glint(bool);

fn parse() {}

#[derive(Components)]
#[component(from_only, family = Toy)]
enum ToyComponentV1 {
    Friction(Friction),
    Glint(Glint),
}

fn main() {}
