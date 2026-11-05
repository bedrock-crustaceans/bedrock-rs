# bedrock_addon_derive

The proc-macro of the `bedrock_addon` crate family: `#[derive(Components)]` on a family's component enum emits a `Component` impl per variant type, `From<T>` for the enum, and the sorted static component table the version-chain machinery looks components up in. Users never depend on this crate directly: `bedrock_addon` derives its own component enums with it and does not re-export it.

## Installation

The crate uses edition 2024 and needs Rust 1.98 or newer. Depend on `bedrock_addon` instead.

```toml
[dependencies]
bedrock_addon = { git = "https://github.com/bedrock-crustaceans/bedrock-rs.git" }
```

## Features

| Feature | Contents | Implies |
| --- | --- | --- |
| (always) | `#[derive(Components)]` | |

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

On a typed variant (the keys may be spread over several attributes, each key at most once), and on each `#[component(...)]` of the `dynamic` variant; the keys are the fields of `ComponentMeta`, one to one:

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

`molang` is a typed variant's key only: on a `dynamic` row (JSON, no typed fields) and in a `from_only` enum it is an error.

Versions are `"major.minor.patch"` literals of decimal digits, checked at expansion time. A missing required key, an unknown or repeated key, a malformed version, a variant that is not `Name(Type)`, a component name used twice and a target that is not an enum are errors at the offending tokens. A non-enum target stops the derive at once; for an enum, the errors found in the enum's attributes and in its variants are collected and reported in one go.

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
pub enum BlockAny {}

impl VersionChain<Block> for BlockAny {
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
    const DIRECTORIES: &'static [&'static str] = &["blocks"];
    const FLOOR: FormatVersion = FormatVersion { major: 1, minor: 10, patch: 0 };
    type Latest = NoStable;
    type Beta = Infallible;
    type Component = BlockComponent;
    type Any = BlockAny;
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
