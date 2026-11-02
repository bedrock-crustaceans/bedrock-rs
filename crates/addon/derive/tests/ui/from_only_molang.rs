use bedrock_addon_derive::Components;

struct Condition(bool);

#[derive(Components)]
#[component(from_only)]
enum ToyComponentV1 {
    #[component(molang)]
    Condition(Condition),
}

fn main() {}
