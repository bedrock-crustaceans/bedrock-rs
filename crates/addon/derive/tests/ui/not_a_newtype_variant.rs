use bedrock_addon_derive::Components;

struct Toy;
struct Friction(f32);
struct Glint(bool);

fn parse() {}

#[derive(Components)]
#[component(family = Toy)]
enum ToyComponent {
    #[component(name = "minecraft:friction", parse = parse)]
    Friction { value: f32 },
}

fn main() {}
