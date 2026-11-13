# bedrock_addon_derive

The proc-macros of the `bedrock_addon` crate family. `#[derive(Components)]` on a family's component enum emits a `Component` impl per variant type, `From<T>` for the enum, and the sorted static component table the version-chain machinery looks components up in. `#[derive(Schema)]` on a typed document struct or string enum emits the impls its `schema::` attributes state; it is reached as `bedrock_addon_core::schema::Schema`, through the grammar crate `bedrock_addon_schema`. Users never depend on this crate directly: `bedrock_addon` derives its own types with it and does not re-export it.

## Installation

The crate uses edition 2024 and needs Rust 1.98 or newer. Depend on `bedrock_addon` instead.

```toml
[dependencies]
bedrock_addon = { git = "https://github.com/bedrock-crustaceans/bedrock-rs.git" }
```

## Features

| Feature | Contents | Implies |
| --- | --- | --- |
| (always) | `#[derive(Components)]`, `#[derive(Schema)]` | |

## What the derive emits

For an enum with one variant per typed component (`Friction(Friction)`), plus optionally the family's `Custom(CustomComponent)`, `Unknown(Value)` and, for entities, `Dynamic(Value)` variants:

- `impl ComponentSet for <Enum>`, whose `name` is the JSON key of a typed variant and `None` for the `custom`, `unknown` and `dynamic` ones;
- per typed variant, `impl Component for <Type>` with `Set = <Enum>`, `NAME` = the JSON key and the conversions between the type and its variant (`from_set`, `from_set_mut`, `try_from_set`, `into_set`), so `map.get::<Friction>()`, `get_mut`, `insert` and `remove` work on the entry keyed `"minecraft:friction"`;
- per typed variant, `impl From<Type> for <Enum>`, so `ComponentMap::from_iter([Friction(0.4).into(), LightEmission(15).into()])` builds a map;
- `<Enum>::TABLE: &'static ComponentTable<F>` (hidden): one `ComponentMeta` per typed component and per dynamic key, sorted by name at expansion time so it can be binary-searched; the family's `Family::components()` returns it;
- `impl MolangFields for <Enum>` (hidden trait of `molang`): the Molang walk, which collects each Molang field for the version pass and the compile check, forwards to the types of the variants marked `molang` and does nothing for the others, so `ComponentMap<Enum>` (which forwards to each typed entry below its key) takes part in a version struct's walk.

The `custom` and `unknown` variants get no impl and no table row. Their payloads, and the `dynamic` one's, must be `CustomComponent` (`custom`) and `json::Value` (`unknown`, `dynamic`); another type is a type error at the payload.

An older version's enum (`vX::Component`) shares its component types with the newest one, which already owns their `Component` impls; `#[component(from_only)]` derives only the `From` impls for it (no `family`, no variant attributes, no table, no `MolangFields`: a version struct that holds such an enum implements its walk by hand).

## Attributes

On the enum, `#[component(...)]`, each key at most once:

| Key | Value | Meaning |
| --- | --- | --- |
| `family` | a type | The family `F` of `ComponentTable<F>`. Required, except with `from_only`, which forbids it. |
| `custom` | a variant | The custom-component variant: no impl, no table row. |
| `unknown` | a variant | The variant for keys no table knows: no impl, no table row. |
| `dynamic` | a variant | The entity variant for keys vanilla parses at runtime: each `#[component(...)]` on it is one table row with no Rust type. |
| `from_only` | (flag) | Emit `From<Type> for <Enum>` per typed variant, with the payload type checks of the `custom`, `unknown` and `dynamic` variants, and nothing of the table. |
| `write` | (flag) | Emit `write_component(&self)`, the write dispatch: each typed variant through `versioning::write_payload`, the custom one as its parameters, the unknown and dynamic ones as written. |
| `write_with` | a function path | The write dispatch with `path(component)` for each typed variant instead (biome's null-stripping writer). One of `write` and `write_with`. |
| `versioned` | a generic function path | The goal parse by version, `fn<T: GoalMeta>(JsonSlice, FormatVersion, &LoadContext, &mut Diagnostics) -> Option<Enum>`: each `goal` row puts `path::<T>` in `VERSIONED`. Required by a `goal` row. |
| `typed_table` | a function path (a name in scope, or `crate::`/`::`) | `fn(&str) -> Option<Option<FormatVersion>>`, the generated game table's typed version of a key (`None`: the table does not list it): emit a test that holds every row's `typed` to it. |
| `triggers` | (flag) | Emit `triggers()`: `(key, member)` of every trigger member, from each payload's `Schema::trigger_members` and each row's `triggers = [..]`, sorted. |

On a typed variant (the keys may be spread over several attributes, each key at most once), and on each `#[component(...)]` of the `dynamic` variant; the keys up to `molang` are the fields of `ComponentMeta`, one to one:

| Key | Value | `ComponentMeta` field | When absent |
| --- | --- | --- | --- |
| `name` | `"minecraft:friction"` | `name`, and `Component::NAME` | error |
| `released` | `"1.19.20"` | `released`: the first format version that may use it | `None`: never released |
| `deprecated_since` | `"1.20.20"` | `deprecated_since` | `None` |
| `deprecated_message` | `"…"` | `deprecated_message`: the family's own text | `None`: the generic text |
| `supersedes` | `"minecraft:foil"` | `supersedes` | `None` |
| `requires` | `[expr, …]`, each a `LoadRequirement` | `requires`: unlocks it before `released` | `[]`: no gate |
| `area` | `Public`, `Vanilla` or `Internal` | `area` | `Public` |
| `fields` | `[{ name = "a.b", since_engine = "1.21.0" }, …]` | `fields` (`FieldMeta`) | `[]` |
| `parse` | a function path | `parse`: reads the component's JSON slice | error |
| `molang` | (flag) | not in `ComponentMeta`: the type implements `MolangFields`, and the enum's walk forwards to it | the variant holds no Molang |
| `typed` | `"1.26.30"` | not in `ComponentMeta`: `SCHEMA_ROWS` `typed`, `typed_since(key)`, and a goal's `GoalMeta::TYPED` | `None` |
| `shorthand` | `scalar` | `SCHEMA_ROWS` `shorthand`: a bare scalar stands for the component's one value | `None` |
| `goal` | (flag) | `SCHEMA_ROWS` `goal`; emits `impl GoalMeta for <Type>` (`TYPED` from `typed`, `priority()` from the goal's `schema::fallback`) and its `VERSIONED` entry; needs `typed` and the enum's `versioned` | not a goal |
| `triggers` | `["member", …]` | `SCHEMA_ROWS` `triggers`: trigger members the payload does not list (an untyped row's) | `[]` |

`molang`, `goal` and `shorthand` are a typed variant's keys only: on a `dynamic` row (JSON, no typed fields) they are an error, and so is `molang` in a `from_only` enum, which takes none of the enum keys after `from_only` either.

With `family` and not `from_only`, the enum also gets `SCHEMA_ROWS` (one `schema::ComponentRow` per row, sorted by name), `schema_row(key)` and `typed_since(key)`. A goal row's `priority` fallback is checked at compile time: a goal struct whose `priority` has no `schema::fallback = <integer>` fails the build.

Versions are `"major.minor.patch"` literals of decimal digits, checked at expansion time. A missing required key, an unknown or repeated key, a malformed version, a variant that is not `Name(Type)`, a component name used twice and a target that is not an enum are errors at the offending tokens. A non-enum target stops the derive at once; for an enum, the errors found in the enum's attributes and in its variants are collected and reported in one go.

## `#[derive(Schema)]`

It reads facet's own `default`, `rename`, `rename_all`, `transparent`, `flatten` and `skip` and the `schema::` keys (the grammar crate's README lists them), and emits impls only, never the item:

- on a struct: `impl Schema` with `FIELDS` (one `FieldRow` per field: names, type, kind, default, fallback, bounds, since, unknown rule), `check` (the bounds `min`, `max`, `above`, `below`, one range and one finding per field, graded by the slice's `on_member_failure` through `SchemaCx`, cross-field ones after the rest; `lowercase`, lower-cased in silence as the game does; then the walk into every field) and `trigger_members` (every `EventTrigger` field without `foreign_event`, at any depth); `impl MolangFields` (a Molang field under the walk's default spec, or its `schema::molang`; `schema::molang_with` for a hand walk; every other field nested); `impl Default` when no field is required, with a `#[cfg(test)]` test beside it that the default is the value `{}` reads (but for a transparent newtype) and reads back unchanged from what the writer makes of it; and a getter per `Option` field with a `schema::fallback`;
- on a unit-only enum: `ALL`, `NAMES`, `EXPECTED` (`schema::expected`, or the names listed), `name`, `from_name` (in any case with `schema::case_insensitive`), `parse_name`, `Display`, and empty `Schema` and `MolangFields` impls.

A field is required (plain `T`), filled on read (plain `T` with facet's `default = x`, the value written in the field's own type) or optional (`Option<T>`, with or without `schema::fallback`). The derive refuses at compile time what a struct leaves unsaid or says twice: facet's `default` on an `Option` or on the container, `fallback` on a plain field, a bound that is not a number or another field's name, a stored key whose value is not a string literal, a version that is not `major.minor.patch`, an `unknown` rule that is not `keep`, `report`, `reject` or `ignore`, a key on the wrong item (`unknown` on a string enum, any key on a `skip` field among them), `lowercase` off a `String`, `foreign_event` off an `EventTrigger`, `molang` off a Molang field. A misspelt key is facet's error.

```rust
use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct BreakDoor {
    #[facet(schema::fallback = 0)]
    pub priority: Option<i32>,
    #[facet(schema::fallback = 1.0, schema::min = 0.0)]
    pub speed: Option<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Facet, Schema)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
pub enum EquipmentSlot {
    Head,
    MainHand,
}

assert_eq!(BreakDoor::default().speed(), 1.0);
assert_eq!(EquipmentSlot::from_name("main_hand"), Some(EquipmentSlot::MainHand));
```

## Examples

A family's component enum and the `Family` impl that hands its table to the version-chain machinery:

```rust
use std::convert::Infallible;

use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::document::{Component, ComponentMap, CustomComponent};
use bedrock_addon_core::json::{JsonSlice, Value};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    ComponentTable, ConvertMode, Family, Loaded, NoStable, UpgradeCx, VersionChain,
};
use bedrock_addon_derive::Components;

pub struct Friction(pub f32);

#[derive(Components)]
#[component(family = Block, custom = Custom, unknown = Unknown)]
pub enum BlockComponent {
    #[component(name = "minecraft:friction", released = "1.19.20", parse = parse_friction)]
    Friction(Friction),
    Custom(CustomComponent),
    Unknown(Value),
}

fn parse_friction(
    _: JsonSlice<'_>,
    _: ConvertMode,
    _: &LoadContext,
    _: &mut Diagnostics,
) -> Option<BlockComponent> {
    Some(Friction(0.4).into())
}

pub struct Block;
pub enum BlockHistory {}

impl VersionChain<Block> for BlockHistory {
    const VERSIONS: &'static [FormatVersion] = &[];
    fn parse(_: JsonSlice<'_>, _: FormatVersion, _: &LoadContext, _: &mut Diagnostics) -> Option<Self> {
        None
    }
    fn upgrade_to_end(self, _: &mut UpgradeCx<'_>) -> Loaded<Block> {
        match self {}
    }
}

impl Family for Block {
    const NAME: &'static str = "block";
    const PAYLOAD_KEY: &'static str = "minecraft:block";
    const FLOOR: FormatVersion = FormatVersion { major: 1, minor: 10, patch: 0 };
    type Latest = NoStable;
    type Beta = Infallible;
    type Component = BlockComponent;
    type History = BlockHistory;
    fn components() -> &'static ComponentTable<Self> {
        BlockComponent::TABLE
    }
}

assert_eq!(Friction::NAME, "minecraft:friction");
assert_eq!(Block::components().0[0].name, "minecraft:friction");

let mut components: ComponentMap<BlockComponent> = ComponentMap::from_iter([Friction(0.4).into()]);
assert_eq!(components.get::<Friction>().map(|f| f.0), Some(0.4));
assert_eq!(components.insert(Friction(0.6)).map(|f| f.0), Some(0.4));
```

## Contributing

See [CONTRIBUTING.md](../../../CONTRIBUTING.md).

## License

Licensed under the **Apache License 2.0**; see [LICENSE](../../../LICENSE).
