use bedrock_addon_derive::Components;

struct Toy;
struct Friction(f32);
struct Glint(bool);
struct Health(u32);

fn parse() {}

// Every error of the enum is reported, in variant order, not only the first.
#[derive(Components)]
#[component(family = Toy)]
enum ToyComponent {
    Friction(Friction),
    #[component(name = "minecraft:glint", released = "one", parse = parse)]
    Glint(Glint),
    #[component(name = "minecraft:health")]
    Health(Health),
}

fn main() {}
