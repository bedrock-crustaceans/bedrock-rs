use bedrock_addon_derive::Components;

struct Toy;
struct Friction(f32);
struct Glint(bool);

fn parse() {}

#[derive(Components)]
#[component(family = Toy, custom = Custom)]
enum ToyComponent {
    #[component(name = "minecraft:friction", parse = parse)]
    Friction(Friction),
}

fn main() {}
