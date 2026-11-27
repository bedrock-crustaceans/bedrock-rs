use bedrock_addon_derive::Components;

struct Toy;
struct Friction(f32);
struct Glint(bool);

fn parse() {}

#[derive(Components)]
#[component(family = Toy)]
enum ToyComponent {
    #[component(name = "minecraft:glint", parse = parse)]
    Friction(Friction),
    #[component(name = "minecraft:glint", parse = parse)]
    Glint(Glint),
}

fn main() {}
