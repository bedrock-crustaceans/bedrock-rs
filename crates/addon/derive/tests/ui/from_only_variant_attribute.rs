use bedrock_addon_derive::Components;

struct Toy;
struct Friction(f32);
struct Glint(bool);

fn parse() {}

#[derive(Components)]
#[component(from_only)]
enum ToyComponentV1 {
    #[component(name = "minecraft:friction", parse = parse)]
    Friction(Friction),
    Glint(Glint),
}

fn main() {}
