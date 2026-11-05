use bedrock_addon_derive::Components;

struct Toy;
struct Friction(f32);

fn parse() {}
fn written() {}

#[derive(Components)]
#[component(family = Toy, write, write_with = written)]
enum ToyComponent {
    #[component(name = "minecraft:friction", parse = parse)]
    Friction(Friction),
}

#[derive(Components)]
#[component(from_only, write, triggers)]
enum OldComponent {
    Friction(Friction),
}

fn main() {}
