use bedrock_addon_derive::Components;

struct Toy;
struct Friction(f32);
struct Glint(bool);

fn parse() {}

#[derive(Components)]
#[component(family = Toy)]
enum ToyComponent {
    #[component(name = "minecraft:friction", parse = parse)]
    #[component(name = "minecraft:friction_v2")]
    Friction(Friction),
}

fn main() {}
