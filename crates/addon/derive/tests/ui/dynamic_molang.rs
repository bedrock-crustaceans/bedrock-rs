use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::json::{JsonSlice, Value};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::versioning::ConvertMode;
use bedrock_addon_derive::Components;

struct Toy;

fn keep_json(
    _: JsonSlice<'_>,
    _: ConvertMode,
    _: &LoadContext,
    _: &mut Diagnostics,
) -> Option<ToyComponent> {
    None
}

#[derive(Components)]
#[component(family = Toy, dynamic = Dynamic)]
enum ToyComponent {
    #[component(name = "minecraft:health", parse = keep_json, molang)]
    Dynamic(Value),
}

fn main() {}
