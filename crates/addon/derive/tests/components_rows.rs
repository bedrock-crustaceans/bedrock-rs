//! Expansion tests for the schema keys of `#[derive(Components)]` on a toy family: the second
//! table (`typed`, `shorthand`, `goal`, `triggers`), `GoalMeta` and the `VERSIONED` table of
//! the goal rows, the write dispatch (`write`, `write_with`), `triggers()`, and the emitted
//! test that holds every row's `typed` to a game table (it runs with this file's tests).

use std::convert::Infallible;

use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::document::CustomComponent;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::schema::{self, ComponentRow, GoalMeta, Schema, Shorthand};
use bedrock_addon_core::shared::EventTrigger;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    ComponentTable, ConvertMode, Family, Loaded, NoStable, UpgradeCx, VersionChain, write_payload,
};
use bedrock_addon_derive::Components;
use facet::Facet;

struct Toy;

enum ToyHistory {}

impl VersionChain<Toy> for ToyHistory {
    const VERSIONS: &'static [FormatVersion] = &[];

    fn parse(
        _: JsonSlice<'_>,
        _: FormatVersion,
        _: &LoadContext,
        _: &mut Diagnostics,
    ) -> Option<Self> {
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
    type History = ToyHistory;

    fn components() -> &'static ComponentTable<Self> {
        ToyComponent::TABLE
    }
}

/// `minecraft:friction`: written as a bare number.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(transparent)]
pub struct Friction(#[facet(default = 0.4)] pub f32);

/// `minecraft:interact`: one trigger of this entity, one of another's.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct Interact {
    pub on_interact: Option<EventTrigger>,
    #[facet(schema::foreign_event)]
    pub on_spawn: Option<EventTrigger>,
}

/// `minecraft:behavior.float`: a goal.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct Float {
    #[facet(schema::fallback = 0)]
    pub priority: Option<i32>,
    pub control_flags: Option<Vec<String>>,
}

#[allow(clippy::large_enum_variant, reason = "a toy enum, built a few times")]
#[derive(Clone, Debug, PartialEq, Components)]
#[component(
    family = Toy,
    custom = Custom,
    unknown = Unknown,
    dynamic = Dynamic,
    write,
    versioned = parse_versioned,
    // An absolute path: the generated test module names it as written.
    typed_table = crate::game_typed,
    triggers,
)]
pub enum ToyComponent {
    #[component(name = "minecraft:interact", typed = "1.21.0", parse = parse_any)]
    Interact(Interact),
    #[component(
        name = "minecraft:friction",
        released = "1.19.20",
        typed = "1.19.20",
        shorthand = scalar,
        parse = parse_any
    )]
    Friction(Friction),
    #[component(name = "minecraft:behavior.float", typed = "1.26.60", goal, parse = parse_any)]
    Float(Float),
    Custom(CustomComponent),
    Unknown(Value),
    #[component(name = "minecraft:behavior.vex_move", triggers = ["on_done", "on_start[]"], parse = parse_any)]
    Dynamic(Value),
}

fn parse_any(
    _: JsonSlice<'_>,
    _: ConvertMode,
    _: &LoadContext,
    _: &mut Diagnostics,
) -> Option<ToyComponent> {
    None
}

/// A goal parse by version, as a family writes it: generic over the goal's `GoalMeta`.
fn parse_versioned<T: GoalMeta>(
    _: JsonSlice<'_>,
    version: FormatVersion,
    _: &LoadContext,
    _: &mut Diagnostics,
) -> Option<ToyComponent> {
    (version >= T::TYPED).then(|| ToyComponent::Unknown(Value::from(0)))
}

/// The game table's typed version of a key, as a family adapts its generated table: `None`
/// where the table does not list the key, `Some(None)` where it lists no typed version.
#[allow(clippy::option_option, reason = "the signature `typed_table` takes")]
fn game_typed(key: &str) -> Option<Option<FormatVersion>> {
    match key {
        "minecraft:friction" => Some(Some(FormatVersion::new(1, 19, 20))),
        "minecraft:interact" => Some(Some(FormatVersion::new(1, 21, 0))),
        "minecraft:behavior.float" => Some(Some(FormatVersion::new(1, 26, 60))),
        "minecraft:behavior.vex_move" => Some(None),
        _ => None,
    }
}

#[test]
fn the_second_table_is_sorted_and_carries_each_rows_schema_keys() {
    let names: Vec<&str> = ToyComponent::SCHEMA_ROWS.iter().map(|r| r.name).collect();
    assert_eq!(
        names,
        [
            "minecraft:behavior.float",
            "minecraft:behavior.vex_move",
            "minecraft:friction",
            "minecraft:interact",
        ]
    );
    assert_eq!(
        ToyComponent::schema_row("minecraft:friction"),
        Some(&ComponentRow {
            name: "minecraft:friction",
            typed: Some(FormatVersion::new(1, 19, 20)),
            shorthand: Some(Shorthand::Scalar),
            goal: false,
            triggers: &[],
        })
    );
    assert!(ToyComponent::schema_row("minecraft:behavior.float").is_some_and(|row| row.goal));
    assert_eq!(
        ToyComponent::typed_since("minecraft:interact"),
        Some(FormatVersion::new(1, 21, 0))
    );
    assert_eq!(
        ToyComponent::typed_since("minecraft:behavior.vex_move"),
        None
    );
    assert_eq!(ToyComponent::typed_since("minecraft:nothing"), None);
}

#[test]
fn a_goal_row_gives_the_goal_its_typed_version_and_priority_and_a_versioned_parse() {
    assert_eq!(Float::TYPED, FormatVersion::new(1, 26, 60));
    let float = Float {
        priority: None,
        control_flags: None,
    };
    assert_eq!(
        GoalMeta::priority(&float),
        0,
        "the Schema getter's fallback"
    );
    let set = Float {
        priority: Some(4),
        ..float
    };
    assert_eq!(GoalMeta::priority(&set), 4);
    let keys: Vec<&str> = ToyComponent::VERSIONED.iter().map(|(k, _)| *k).collect();
    assert_eq!(keys, ["minecraft:behavior.float"]);
}

#[test]
fn the_write_dispatch_writes_each_variant() -> Result<(), EncodeError> {
    let friction = ToyComponent::Friction(Friction(0.5)).write_component()?;
    assert_eq!(friction, write_payload(&Friction(0.5))?);
    let custom = ToyComponent::Custom(CustomComponent {
        name: "toy:custom".to_owned(),
        parameters: Value::from(7),
        from_v1_list: false,
    });
    assert_eq!(custom.write_component()?, Value::from(7));
    assert_eq!(
        ToyComponent::Dynamic(Value::from(3)).write_component()?,
        Value::from(3)
    );
    assert_eq!(
        ToyComponent::Unknown(Value::from(2)).write_component()?,
        Value::from(2)
    );
    Ok(())
}

#[test]
fn triggers_list_the_typed_members_and_the_rows_own_sorted() {
    let triggers: Vec<(&str, String)> = ToyComponent::triggers();
    let triggers: Vec<(&str, &str)> = triggers.iter().map(|(k, m)| (*k, m.as_str())).collect();
    assert_eq!(
        triggers,
        [
            ("minecraft:behavior.vex_move", "on_done"),
            ("minecraft:behavior.vex_move", "on_start[]"),
            ("minecraft:interact", "on_interact"),
        ]
    );
}

/// `minecraft:climate`: a component whose writer strips `null` members.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct Climate {
    pub downfall: Option<f32>,
}

/// A writer that marks what it wrote, standing in for biome's null-stripping `written`.
fn written<'f, T: Facet<'f>>(component: &T) -> Result<Value, EncodeError> {
    let json = write_payload(component)?;
    Ok(Value::from(format!("written {json:?}").as_str()))
}

struct ToyBiome;

enum ToyBiomeHistory {}

impl VersionChain<ToyBiome> for ToyBiomeHistory {
    const VERSIONS: &'static [FormatVersion] = &[];

    fn parse(
        _: JsonSlice<'_>,
        _: FormatVersion,
        _: &LoadContext,
        _: &mut Diagnostics,
    ) -> Option<Self> {
        None
    }

    fn upgrade_to_end(self, _: &mut UpgradeCx<'_>) -> Loaded<ToyBiome> {
        match self {}
    }
}

impl Family for ToyBiome {
    const NAME: &'static str = "toy_biome";
    const PAYLOAD_KEY: &'static str = "minecraft:biome";
    const FLOOR: FormatVersion = FormatVersion::new(1, 16, 0);
    type Latest = NoStable;
    type Beta = Infallible;
    type Component = BiomeComponent;
    type History = ToyBiomeHistory;

    fn components() -> &'static ComponentTable<Self> {
        BiomeComponent::TABLE
    }
}

#[derive(Clone, Debug, PartialEq, Components)]
#[component(family = ToyBiome, unknown = Unknown, write_with = written)]
pub enum BiomeComponent {
    #[component(name = "minecraft:climate", parse = parse_biome)]
    Climate(Climate),
    Unknown(Value),
}

fn parse_biome(
    _: JsonSlice<'_>,
    _: ConvertMode,
    _: &LoadContext,
    _: &mut Diagnostics,
) -> Option<BiomeComponent> {
    None
}

#[test]
fn write_with_routes_every_typed_arm_through_the_enums_writer() -> Result<(), EncodeError> {
    let climate = Climate {
        downfall: Some(0.5),
    };
    let written_by_enum = BiomeComponent::Climate(climate.clone()).write_component()?;
    assert_eq!(written_by_enum, written(&climate)?);
    assert_eq!(
        BiomeComponent::Unknown(Value::from(1)).write_component()?,
        Value::from(1)
    );
    Ok(())
}
