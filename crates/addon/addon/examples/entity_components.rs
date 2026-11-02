//! Read an entity's components by typed access: `components.get::<T>()` for a component the
//! crate has a type for, and the JSON as written for a component vanilla itself still parses at
//! run time (`minecraft:health`, `minecraft:movement`, `minecraft:type_family`).
//!
//! The stacks are built from bytes in memory, as in `in_memory_pack`.
//!
//! Run with `cargo run -p bedrock_addon --example entity_components --no-default-features --features entity`.

mod common;

use bedrock_addon::document::{ComponentMap, ComponentRef};
use bedrock_addon::families::entity::component::{CollisionBox, EntityComponent, Physics};
use bedrock_addon::id::EntityId;
use bedrock_addon::json::Value;
use bedrock_addon::pack::StackSet;

const WALKER: EntityId = EntityId::from_static("demo:walker");

/// The JSON of the component `name`, when it is one the crate keeps as written.
fn dynamic<'a>(components: &'a ComponentMap<EntityComponent>, name: &str) -> Option<&'a Value> {
    components.iter().find_map(|entry| match entry {
        ComponentRef::Dynamic { name: key, json } if key == name => Some(json),
        _ => None,
    })
}

/// The number at `member` of a JSON object.
fn number(json: &Value, member: &str) -> Option<f64> {
    json.as_object()?.get(member)?.as_number()?.to_f64()
}

fn describe(stacks: &StackSet) -> Option<Summary> {
    let walker = stacks.behavior.entities().get(&WALKER)?;

    // Typed: the component is a struct with its defaults applied.
    let collision = walker.components.get::<CollisionBox>()?;
    println!(
        "collision box {} x {}",
        collision.width.0, collision.height.0
    );
    if let Some(physics) = walker.components.get::<Physics>() {
        println!("falls: {}", physics.has_gravity.0);
    }

    // Kept as written: the value is the component's JSON.
    let health = dynamic(&walker.components, "minecraft:health")?;
    let max_health = number(health, "max").or_else(|| number(health, "value"))?;
    println!("max health {max_health}");

    let mut speeds = Vec::new();
    for (group, components) in &walker.component_groups {
        if let Some(speed) = dynamic(components.components(), "minecraft:movement")
            .and_then(|json| number(json, "value"))
        {
            println!("{group}: speed {speed}");
            speeds.push((group.clone(), speed));
        }
    }

    let families: Vec<&str> = dynamic(&walker.components, "minecraft:type_family")
        .and_then(|json| json.as_object()?.get("family")?.as_array())
        .map(|family| {
            family
                .iter()
                .filter_map(|name| Some(name.as_string()?.as_str()))
                .collect()
        })
        .unwrap_or_default();
    println!(
        "defined in {} ({}), families {families:?}",
        walker.origin().pack.name(),
        walker.origin().path
    );
    if walker.version().upgraded {
        println!("written as {:?}", walker.version().written);
    }
    Some(Summary {
        width: collision.width.0,
        max_health,
        speeds,
        families: families.into_iter().map(str::to_owned).collect(),
        upgraded: walker.version().upgraded,
    })
}

/// What `describe` read, for the assertions below.
#[derive(Debug, PartialEq)]
struct Summary {
    width: f32,
    max_health: f64,
    speeds: Vec<(String, f64)>,
    families: Vec<String>,
    upgraded: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stacks = common::stack_set()?;
    let summary = describe(&stacks).expect("demo:walker is on the behaviour stack");
    assert_eq!(
        summary,
        Summary {
            width: 0.5,
            max_health: 12.0,
            speeds: vec![
                ("demo:slow".to_owned(), 0.1),
                ("demo:fast".to_owned(), 0.35)
            ],
            families: vec!["demo".to_owned(), "walker".to_owned()],
            upgraded: true,
        }
    );
    assert!(
        !stacks.behavior.report().has_errors(),
        "{}",
        stacks.behavior.report()
    );
    Ok(())
}
