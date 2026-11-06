use std::convert::Infallible;

use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::json::{JsonSlice, Value};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::schema::{GoalMeta, Schema};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    ComponentTable, ConvertMode, Family, Loaded, NoStable, UpgradeCx, VersionChain,
};
use bedrock_addon_derive::Components;

struct Toy;
enum ToyAny {}

impl VersionChain<Toy> for ToyAny {
    const VERSIONS: &'static [FormatVersion] = &[];
    fn parse(_: JsonSlice<'_>, _: FormatVersion, _: &LoadContext, _: &mut Diagnostics) -> Option<Self> {
        None
    }
    fn upgrade_to_end(self, _: &mut UpgradeCx<'_>) -> Loaded<Toy> {
        match self {}
    }
}

impl Family for Toy {
    const NAME: &'static str = "toy";
    const PAYLOAD_KEY: &'static str = "minecraft:toy";
    const FLOOR: FormatVersion = FormatVersion::new(1, 16, 0);
    type Latest = NoStable;
    type Beta = Infallible;
    type Component = ToyComponent;
    type History = ToyAny;
    fn components() -> &'static ComponentTable<Self> {
        ToyComponent::TABLE
    }
}

// A `priority` with no `schema::fallback`: nothing to answer for an absent one.
#[derive(facet::Facet, Schema)]
struct Float {
    priority: Option<i32>,
}

// Not a `Schema` struct at all.
struct Drift {
    priority: Option<i32>,
}

fn parse(_: JsonSlice<'_>, _: ConvertMode, _: &LoadContext, _: &mut Diagnostics) -> Option<ToyComponent> {
    None
}

fn versioned<T: GoalMeta>(_: JsonSlice<'_>, _: FormatVersion, _: &LoadContext, _: &mut Diagnostics) -> Option<ToyComponent> {
    None
}

#[derive(Components)]
#[component(family = Toy, unknown = Unknown, versioned = versioned)]
enum ToyComponent {
    #[component(name = "minecraft:behavior.float", typed = "1.26.60", goal, parse = parse)]
    Float(Float),
    #[component(name = "minecraft:behavior.drift", typed = "1.26.60", goal, parse = parse)]
    Drift(Drift),
    Unknown(Value),
}

fn main() {}
