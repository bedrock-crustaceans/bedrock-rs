use bedrock_addon_derive::Components;

struct Toy;
struct Friction(f32);
struct Glint(bool);

fn parse() {}

#[derive(Components)]
#[component(family = Toy)]
enum ToyComponent {
    #[component(released = "1.19.20", parse = parse)]
    Friction(Friction),
}

fn main() {}
