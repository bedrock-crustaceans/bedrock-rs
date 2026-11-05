use bedrock_addon_derive::Components;

struct Toy;
struct Float;
struct Friction(f32);

fn parse() {}

#[derive(Components)]
#[component(family = Toy, dynamic = Dynamic)]
enum ToyComponent {
    #[component(name = "minecraft:behavior.float", goal, parse = parse)]
    Float(Float),
    #[component(name = "minecraft:friction", shorthand = object, parse = parse)]
    Friction(Friction),
    #[component(name = "minecraft:behavior.vex_move", goal, parse = parse)]
    Dynamic(Value),
}

fn main() {}
