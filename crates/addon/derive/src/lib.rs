//! The proc-macro of the `bedrock_addon` crate family.
//!
//! A proc-macro crate exports only its macros, so the one item here is
//! `#[derive(Components)]`, at the crate root: on a family's component enum it emits a
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
///
/// # Component attributes
///
/// `#[component(...)]` on a typed variant (the keys may be spread over several attributes, each
/// key at most once), and each `#[component(...)]` on the `dynamic` variant. The keys are the
/// fields of `ComponentMeta`, one to one:
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
///
/// `molang` is a typed variant's key only: on a `dynamic` row (JSON, no typed fields) and in a
/// `from_only` enum it is an error.
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
