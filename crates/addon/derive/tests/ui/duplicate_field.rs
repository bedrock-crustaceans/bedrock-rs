use bedrock_addon_derive::Components;

struct Toy;
struct Friction(f32);
struct Glint(bool);

fn parse() {}

#[derive(Components)]
#[component(family = Toy)]
enum ToyComponent {
    #[component(
        name = "minecraft:friction",
        fields = [
            { name = "value", since_engine = "1.19.20" },
            { name = "value", since_engine = "1.21.0" },
        ],
        parse = parse,
    )]
    Friction(Friction),
}

fn main() {}
