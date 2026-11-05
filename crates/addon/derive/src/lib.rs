//! The proc-macros of the `bedrock_addon` crate family.
//!
//! A proc-macro crate exports only its macros, so the two items here are derives at the crate
//! root. `#[derive(Schema)]` emits the impls a typed document struct's `schema::` attributes
//! state; it is reached through `bedrock_addon_core::schema`. `#[derive(Components)]`: on a
//! family's component enum it emits a
//! `Component` impl per variant type, `From<T>` for the enum and the sorted component table
//! the version-chain machinery looks components up in. What it emits names items of five
//! modules of `bedrock_addon_core`: `document`, `versioning`, `version`, `molang` (the
//! Molang walk, `MolangFields` and `MolangVisitor`) and `json` (`Value`, the payload of the
//! `unknown` and `dynamic` variants). `bedrock_addon` derives its own component enums with it and
//! does not re-export it: users depend on `bedrock_addon`.
//!
//! Modules: none are public. Features: none.
//!
//! The README lists the attributes and has a compiled example of a family's component enum.

#![cfg_attr(docsrs, feature(doc_cfg))]

/// The crate exports its derive macro and nothing else; no type resolves at the root.
///
/// ```compile_fail,E0432
/// use bedrock_addon_derive::Component;
/// ```
/// ```compile_fail,E0432
/// use bedrock_addon_derive::ComponentTable;
/// ```
#[cfg(doctest)]
struct NoItemAtTheRoot;

/// Compiles the README's code blocks as doctests.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;

mod attr;
mod expand;
mod rows;
mod schema;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

/// Derives the component machinery of a family's component enum.
///
/// The enum has one variant per typed component, each holding the component type
/// (`Friction(Friction)`), plus optionally the family's `Custom(CustomComponent)`,
/// `Unknown(Value)` and, for entities, `Dynamic(Value)` variants. The derive emits:
///
/// - `impl ComponentSet for <Enum>`, whose `name` is the JSON key of a typed variant and
///   `None` for the `custom`, `unknown` and `dynamic` ones;
/// - per typed variant, `impl Component for <Type> { type Set = <Enum>; const NAME = "…"; }`
///   with the conversions between the type and its variant (`from_set`, `from_set_mut`,
///   `try_from_set`, `into_set`), so `map.get::<Friction>()`, `get_mut`, `insert` and
///   `remove` work on the entry keyed `"minecraft:friction"`;
/// - per typed variant, `impl From<Type> for <Enum>`, so a map can be built with
///   `ComponentMap::from_iter([Friction(0.4).into(), LightEmission(15).into()])`;
/// - `<Enum>::TABLE: &'static ComponentTable<F>` (hidden): one `ComponentMeta` per typed
///   component and per dynamic key, sorted by name at expansion time so it can be
///   binary-searched; the family's `Family::components()` returns it;
/// - `impl MolangFields for <Enum>` (hidden trait of `molang`): the Molang walk forwards to the
///   types of the variants marked `molang` and does nothing for the others, so a
///   `ComponentMap<Enum>` takes part in a version struct's walk.
///
/// The `custom` and `unknown` variants get no impl and no table row. Their payloads, and the
/// `dynamic` one's, must be the types the version-chain model gives them: `CustomComponent`
/// for `custom`, `json::Value` for `unknown` and `dynamic`; another type is a type error at the
/// payload.
///
/// An older version's enum (`vX::Component`) shares its component types with the newest one,
/// which already owns their `Component` impls; `#[component(from_only)]` derives only the
/// `From` impls for it (no `family`, no variant attributes, no table, no `MolangFields`: a
/// version struct that holds such an enum implements its walk by hand).
///
/// # Enum attributes
///
/// `#[component(...)]` on the enum; each key at most once.
///
/// | Key | Value | Meaning |
/// | --- | --- | --- |
/// | `family` | a type | The family `F` of `ComponentTable<F>`. Required, except with `from_only`, which forbids it. |
/// | `custom` | a variant | The custom-component variant: no impl, no table row. |
/// | `unknown` | a variant | The variant for keys no table knows: no impl, no table row. |
/// | `dynamic` | a variant | The entity variant for keys vanilla parses at runtime: each `#[component(...)]` on it is one table row with no Rust type. |
/// | `from_only` | (flag) | Emit `From<Type> for <Enum>` per typed variant, with the payload type checks of the `custom`, `unknown` and `dynamic` variants, and nothing of the table. |
/// | `write` | (flag) | Emit `write_component(&self)`, the write dispatch: each typed variant through `versioning::write_payload`, the custom one as its parameters, the unknown and dynamic ones as written. |
/// | `write_with` | a function path | The write dispatch with `path(component)` for each typed variant instead (biome's null-stripping writer). One of `write` and `write_with`. |
/// | `versioned` | a generic function path | The goal parse by version, `fn<T: GoalMeta>(JsonSlice, FormatVersion, &LoadContext, &mut Diagnostics) -> Option<Enum>`: each `goal` row puts `path::<T>` in `VERSIONED`. Required by a `goal` row. |
/// | `typed_table` | a function path | `fn(&str) -> Option<Option<FormatVersion>>`, the generated game table's typed version of a key (`None`: the table does not list it): emit a test that holds every row's `typed` to it. |
/// | `triggers` | (flag) | Emit `triggers()`: `(key, member)` of every trigger member, from each payload's `Schema::trigger_members` and each row's `triggers = [..]`, sorted. |
///
/// # Component attributes
///
/// `#[component(...)]` on a typed variant (the keys may be spread over several attributes, each
/// key at most once), and each `#[component(...)]` on the `dynamic` variant. The keys up to
/// `molang` are the fields of `ComponentMeta`, one to one:
///
/// | Key | Value | `ComponentMeta` field | When absent |
/// | --- | --- | --- | --- |
/// | `name` | `"minecraft:friction"` | `name`, and `Component::NAME` | error |
/// | `released` | `"1.19.20"` | `released`: the first format version that may use it | `None`: never released |
/// | `deprecated_since` | `"1.20.20"` | `deprecated_since` | `None` |
/// | `deprecated_message` | `"…"` | `deprecated_message`: the family's own text | `None`: the generic text |
/// | `supersedes` | `"minecraft:foil"` | `supersedes` | `None` |
/// | `requires` | `[expr, …]`, each a `LoadRequirement` | `requires`: unlocks it before `released` | `[]`: no gate |
/// | `area` | `Public`, `Vanilla` or `Internal` | `area` | `Public` |
/// | `fields` | `[{ name = "a.b", since_engine = "1.21.0" }, …]` | `fields` (`FieldMeta`) | `[]` |
/// | `parse` | a function path | `parse`: reads the component's JSON slice | error |
/// | `molang` | (flag) | not in `ComponentMeta`: the type implements `MolangFields`, and the enum's walk forwards to it | the variant holds no Molang |
/// | `typed` | `"1.26.30"` | not in `ComponentMeta`: `SCHEMA_ROWS` `typed`, `typed_since(key)`, and a goal's `GoalMeta::TYPED` | `None` |
/// | `shorthand` | `scalar` | `SCHEMA_ROWS` `shorthand`: a bare scalar stands for the component's one value | `None` |
/// | `goal` | (flag) | `SCHEMA_ROWS` `goal`; emits `impl GoalMeta for <Type>` (`TYPED` from `typed`, `priority()` from the goal's `schema::fallback`) and its `VERSIONED` entry; needs `typed` and the enum's `versioned` | not a goal |
/// | `triggers` | `["member", …]` | `SCHEMA_ROWS` `triggers`: trigger members the payload does not list (an untyped row's) | `[]` |
///
/// `molang`, `goal` and `shorthand` are a typed variant's keys only: on a `dynamic` row (JSON,
/// no typed fields) they are an error, and so is `molang` in a `from_only` enum, which takes
/// none of the enum keys after `from_only` either.
///
/// With `family` and not `from_only`, the enum also gets `SCHEMA_ROWS` (one
/// `schema::ComponentRow` per row, sorted by name), `schema_row(key)` and `typed_since(key)`. A
/// goal row's `priority` fallback is checked at compile time: a goal struct whose `priority`
/// has no `schema::fallback = <integer>` fails the build.
///
/// Versions are `"major.minor.patch"` literals of decimal digits, checked at expansion time. A
/// missing required key, an unknown or repeated key, a malformed version, a variant that is
/// not `Name(Type)`, a component name used twice and a target that is not an enum are errors
/// at the offending tokens. A non-enum target stops the derive at once; for an enum, the
/// errors found in the enum's attributes and in its variants are collected and reported in
/// one go.
///
/// # Example
///
/// ```
/// # use std::convert::Infallible;
/// # use bedrock_addon_core::diagnostic::Diagnostics;
/// # use bedrock_addon_core::json::JsonSlice;
/// # use bedrock_addon_core::load::LoadContext;
/// # use bedrock_addon_core::version::FormatVersion;
/// # use bedrock_addon_core::versioning::{
/// #     ComponentTable, Family, Loaded, NoStable, UpgradeCx, VersionChain,
/// # };
/// use bedrock_addon_core::document::{Component, CustomComponent};
/// use bedrock_addon_core::json::Value;
/// use bedrock_addon_core::versioning::{ContextArea, ConvertMode};
/// use bedrock_addon_derive::Components;
///
/// pub struct Friction(pub f32);
/// pub struct Glint(pub bool);
///
/// #[derive(Components)]
/// #[component(family = Item, custom = Custom, unknown = Unknown)]
/// pub enum ItemComponent {
///     #[component(name = "minecraft:friction", released = "1.19.20", parse = parse_friction)]
///     Friction(Friction),
///     #[component(name = "minecraft:glint", supersedes = "minecraft:foil", area = Vanilla, parse = parse_glint)]
///     Glint(Glint),
///     Custom(CustomComponent),
///     Unknown(Value),
/// }
///
/// fn parse_friction(_: JsonSlice<'_>, _: ConvertMode, _: &LoadContext, _: &mut Diagnostics) -> Option<ItemComponent> {
///     Some(Friction(0.4).into())
/// }
/// fn parse_glint(_: JsonSlice<'_>, _: ConvertMode, _: &LoadContext, _: &mut Diagnostics) -> Option<ItemComponent> {
///     Some(Glint(true).into())
/// }
/// # pub struct Item;
/// # pub enum ItemAny {}
/// # impl VersionChain<Item> for ItemAny {
/// #     const VERSIONS: &'static [FormatVersion] = &[];
/// #     fn parse(_: JsonSlice<'_>, _: FormatVersion, _: &LoadContext, _: &mut Diagnostics) -> Option<Self> { None }
/// #     fn upgrade_to_end(self, _: &mut UpgradeCx<'_>) -> Loaded<Item> { match self {} }
/// # }
/// impl Family for Item {
///     // …
/// #   const NAME: &'static str = "item";
/// #   const PAYLOAD_KEY: &'static str = "minecraft:item";
/// #   const FLOOR: FormatVersion = FormatVersion { major: 1, minor: 10, patch: 0 };
/// #   type Latest = NoStable;
/// #   type Beta = Infallible;
///     type Component = ItemComponent;
/// #   type Any = ItemAny;
///     fn components() -> &'static ComponentTable<Self> {
///         ItemComponent::TABLE
///     }
/// }
///
/// assert_eq!(Friction::NAME, "minecraft:friction");
/// assert!(matches!(ItemComponent::from(Glint(true)), ItemComponent::Glint(_)));
///
/// let glint = Item::components().get("minecraft:glint").ok_or("no glint row")?;
/// assert_eq!(glint.supersedes, Some("minecraft:foil"));
/// assert_eq!(glint.area, ContextArea::Vanilla);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[proc_macro_derive(Components, attributes(component))]
pub fn derive_components(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand::derive(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Derives the impls a typed document struct's attributes state; reached as
/// `bedrock_addon_core::schema::Schema`, beside the trait of the same name.
///
/// It reads facet's own `default`, `rename`, `rename_all`, `transparent`, `flatten` and `skip`
/// and the `schema::` keys of `#[facet(..)]` (the grammar is `bedrock_addon_schema`), and emits
/// impls only, never the item:
///
/// - on a struct: `impl Schema` (`FIELDS`, the row table; `check`, the bounds and `lowercase`
///   graded by the slice through `SchemaCx`, then the walk into every field;
///   `trigger_members`); `impl MolangFields` (a Molang field under the walk's default spec or
///   its `schema::molang`, every other field nested); `impl Default` when no field is required;
///   a getter `fn x(&self) -> T` per `Option` field with a `schema::fallback`;
/// - on a unit-only enum: `ALL`, `NAMES`, `EXPECTED`, `name`, `from_name`, `parse_name`,
///   `Display`, and empty `Schema` and `MolangFields` impls.
///
/// A field is required (plain `T`), filled on read (plain `T` with facet's `default = x`, the
/// value in the field's own type) or optional (`Option<T>`, with or without
/// `schema::fallback`). What the derive refuses at compile time: facet's `default` on an
/// `Option`, `fallback` on a plain field, a bound that is neither a number nor another field's
/// name (or names the field itself, or a field that does not exist, or doubles a side), a
/// stored key whose value is not a string literal, a version that is not `major.minor.patch`,
/// an `unknown` that is not `keep`, `report`, `reject` or `ignore`, a key on the wrong item
/// (a field's key on the container, an enum's key on a field or a struct, any key on a
/// variant or a `skip` field), `lowercase` off a `String`, `foreign_event` off an
/// `EventTrigger`, `molang` off a Molang field or beside `molang_with`, generics, a tuple
/// struct that is not transparent, and an enum with data. A misspelt key is facet's error.
///
/// The emitted paths are `::bedrock_addon_core::…`, so the user crate depends on core.
///
/// ```
/// use bedrock_addon_core::schema::{self, Schema};
/// use facet::Facet;
///
/// #[derive(Clone, Debug, PartialEq, Facet, Schema)]
/// pub struct BreakDoor {
///     #[facet(schema::fallback = 0)]
///     pub priority: Option<i32>,
///     #[facet(schema::fallback = 1.0, schema::min = 0.0)]
///     pub speed: Option<f32>,
/// }
///
/// let door = BreakDoor::default();
/// assert_eq!((door.priority, door.priority()), (None, 0));
/// assert_eq!(door.speed(), 1.0);
/// assert_eq!(BreakDoor::FIELDS[1].json, "speed");
/// ```
#[proc_macro_derive(Schema, attributes(facet))]
pub fn derive_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    schema::derive(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
