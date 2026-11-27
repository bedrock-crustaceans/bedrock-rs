use bedrock_addon_derive::Components;

struct Toy;
struct Friction(f32);
struct Glint(bool);

fn parse() {}

#[derive(Components)]
#[component(family = Toy)]
struct ToyComponent {
    friction: Friction,
}

fn main() {}
