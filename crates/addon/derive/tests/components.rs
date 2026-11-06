//! Expansion tests for `#[derive(Components)]` on a toy family: the `Component` impls, the
//! `From` conversions, the sorted component table and the metadata every attribute carries.

use std::convert::Infallible;

use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
use bedrock_addon_core::document::{Component, ComponentMap, ComponentSet, CustomComponent};
use bedrock_addon_core::json::{JsonSlice, Value};
use bedrock_addon_core::load::{Experiment, LoadContext, LoadRequirement};
use bedrock_addon_core::molang::{
    Molang, MolangFieldSpec, MolangFields, MolangVisitor, visit_fields,
};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    ComponentMeta, ComponentTable, ContextArea, ConvertMode, Family, FieldMeta, Loaded, NoStable,
    UpgradeCx, VersionChain,
};
use bedrock_addon_derive::Components;

/// A load requirement to put in `requires`.
const REQUIREMENT: LoadRequirement =
    LoadRequirement::Experiment(Experiment::UpcomingCreatorFeatures);

/// The toy family the component enum belongs to.
struct Toy;

/// The toy family's chain: uninhabited, the tests never load a document.
enum ToyAny {}

impl VersionChain<Toy> for ToyAny {
    const VERSIONS: &'static [FormatVersion] = &[];

    fn parse(
        _payload: JsonSlice<'_>,
        _v: FormatVersion,
        _cx: &LoadContext,
        _d: &mut Diagnostics,
    ) -> Option<Self> {
        None
    }

    fn upgrade_to_end(self, _cx: &mut UpgradeCx<'_>) -> Loaded<Toy> {
        match self {}
    }
}

impl Family for Toy {
    const NAME: &'static str = "toy";
    const PAYLOAD_KEY: &'static str = "minecraft:toy";
    const FLOOR: FormatVersion = version(1, 16, 0);
    type Latest = NoStable;
    type Beta = Infallible;
    type Component = ToyComponent;
    type History = ToyAny;

    fn components() -> &'static ComponentTable<Self> {
        ToyComponent::TABLE
    }
}

/// `minecraft:friction`.
#[derive(Clone, Debug, PartialEq)]
struct Friction(f32);

/// `minecraft:light_emission`.
#[derive(Clone, Debug, PartialEq)]
struct LightEmission(u8);

/// `minecraft:glint`.
#[derive(Clone, Debug, PartialEq)]
struct Glint(bool);

/// `minecraft:display_name`.
#[derive(Clone, Debug, PartialEq)]
struct DisplayName(String);

/// The toy family's component enum. Variants are deliberately out of name order.
#[derive(Components, Clone, Debug, PartialEq)]
#[component(family = Toy, custom = Custom, unknown = Unknown, dynamic = Dynamic)]
enum ToyComponent {
    #[component(name = "minecraft:light_emission", parse = parse_light_emission)]
    LightEmission(LightEmission),
    #[component(
        name = "minecraft:glint",
        released = "1.20.20",
        deprecated_since = "1.26.0",
        deprecated_message = "minecraft:glint is gone in format versions 1.26.0 and above",
        supersedes = "minecraft:foil",
        requires = [REQUIREMENT],
        area = Vanilla,
        fields = [
            { name = "", since_engine = "1.20.20" },
            { name = "value.shimmer", since_engine = "1.21.10" },
        ],
        parse = parse_glint,
    )]
    Glint(Glint),
    #[component(name = "minecraft:friction", released = "1.19.20", parse = parse_friction)]
    Friction(Friction),
    #[component(name = "minecraft:display_name", area = Internal, parse = parse_display_name)]
    DisplayName(DisplayName),
    Custom(CustomComponent),
    Unknown(Value),
    #[component(name = "minecraft:health", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:ageable", parse = keep_json)]
    Dynamic(Value),
}

const fn version(major: u16, minor: u16, patch: u16) -> FormatVersion {
    FormatVersion {
        major,
        minor,
        patch,
    }
}

#[expect(
    clippy::unnecessary_wraps,
    reason = "the signature is the function type of `ComponentMeta::parse`"
)]
fn parse_friction(
    _: JsonSlice<'_>,
    _: ConvertMode,
    _: &LoadContext,
    _: &mut Diagnostics,
) -> Option<ToyComponent> {
    Some(Friction(0.4).into())
}

#[expect(
    clippy::unnecessary_wraps,
    reason = "the signature is the function type of `ComponentMeta::parse`"
)]
fn parse_light_emission(
    _: JsonSlice<'_>,
    _: ConvertMode,
    _: &LoadContext,
    _: &mut Diagnostics,
) -> Option<ToyComponent> {
    Some(LightEmission(15).into())
}

#[expect(
    clippy::unnecessary_wraps,
    reason = "the signature is the function type of `ComponentMeta::parse`"
)]
fn parse_glint(
    _: JsonSlice<'_>,
    _: ConvertMode,
    _: &LoadContext,
    _: &mut Diagnostics,
) -> Option<ToyComponent> {
    Some(Glint(true).into())
}

#[expect(
    clippy::unnecessary_wraps,
    reason = "the signature is the function type of `ComponentMeta::parse`"
)]
fn parse_display_name(
    _: JsonSlice<'_>,
    _: ConvertMode,
    _: &LoadContext,
    _: &mut Diagnostics,
) -> Option<ToyComponent> {
    Some(DisplayName(String::from("toy")).into())
}

#[expect(
    clippy::unnecessary_wraps,
    reason = "the signature is the function type of `ComponentMeta::parse`"
)]
fn keep_json(
    _: JsonSlice<'_>,
    _: ConvertMode,
    _: &LoadContext,
    _: &mut Diagnostics,
) -> Option<ToyComponent> {
    Some(ToyComponent::Dynamic(Value::from(0)))
}

/// The function type of `ComponentMeta::parse` for the toy family.
type ParseFn =
    fn(JsonSlice<'_>, ConvertMode, &LoadContext, &mut Diagnostics) -> Option<ToyComponent>;

/// The table row of `name`, found by binary search.
fn meta(name: &str) -> &'static ComponentMeta<Toy> {
    let table = Toy::components().0;
    let index = table
        .binary_search_by(|meta| meta.name.cmp(name))
        .unwrap_or_else(|_| panic!("`{name}` is not in the table"));
    &table[index]
}

fn assert_component_set<C: ComponentSet>() {}

#[test]
fn component_name_is_the_json_key() {
    assert_eq!(Friction::NAME, "minecraft:friction");
    assert_eq!(LightEmission::NAME, "minecraft:light_emission");
    assert_eq!(<Glint as Component>::NAME, "minecraft:glint");
    assert_eq!(DisplayName::NAME, "minecraft:display_name");
}

#[test]
fn component_set_is_the_enum() {
    assert_component_set::<ToyComponent>();
    assert_component_set::<<Friction as Component>::Set>();
    let set: <Glint as Component>::Set = Glint(false).into();
    assert_eq!(set, ToyComponent::Glint(Glint(false)));
}

#[test]
fn from_wraps_each_component_in_its_variant() {
    assert_eq!(
        ToyComponent::from(Friction(0.4)),
        ToyComponent::Friction(Friction(0.4))
    );
    let components: Vec<ToyComponent> = vec![Friction(0.4).into(), LightEmission(15).into()];
    assert_eq!(
        components,
        [
            ToyComponent::Friction(Friction(0.4)),
            ToyComponent::LightEmission(LightEmission(15)),
        ]
    );
}

#[test]
fn component_map_gets_a_component_by_type() {
    let mut map: ComponentMap<ToyComponent> =
        ComponentMap::from_iter([LightEmission(7).into(), Friction(0.6).into()]);
    assert_eq!(map.len(), 2);
    assert_eq!(map.get::<Friction>(), Some(&Friction(0.6)));
    assert_eq!(map.get::<LightEmission>(), Some(&LightEmission(7)));
    assert_eq!(map.get::<Glint>(), None);
    assert!(map.contains::<Friction>());

    map.get_mut::<Friction>().unwrap().0 = 0.9;
    assert_eq!(map.get::<Friction>(), Some(&Friction(0.9)));
}

#[test]
fn component_map_inserts_and_removes_by_type() {
    let mut map: ComponentMap<ToyComponent> = ComponentMap::new();
    assert_eq!(map.insert(Friction(0.4)), None);
    assert_eq!(map.insert(Glint(true)), None);
    assert_eq!(map.insert(Friction(0.5)), Some(Friction(0.4)));
    assert_eq!(map.remove::<Friction>(), Some(Friction(0.5)));
    assert_eq!(map.remove::<Friction>(), None);
    assert_eq!(map.get::<Glint>(), Some(&Glint(true)));
}

#[test]
fn component_map_from_iter_skips_untyped_variants() {
    let custom = CustomComponent {
        name: String::from("demo:glow"),
        parameters: Value::from(1),
        from_v1_list: false,
    };
    let map: ComponentMap<ToyComponent> = [
        ToyComponent::Custom(custom),
        ToyComponent::Unknown(Value::from(2)),
        ToyComponent::Dynamic(Value::from(3)),
        Glint(false).into(),
    ]
    .into_iter()
    .collect();
    assert_eq!(map.len(), 1);
    assert_eq!(map.get::<Glint>(), Some(&Glint(false)));
}

#[test]
fn set_name_is_the_typed_key() {
    assert_eq!(
        ToyComponent::from(Friction(0.4)).name(),
        Some("minecraft:friction")
    );
    assert_eq!(
        ToyComponent::from(DisplayName(String::new())).name(),
        Some("minecraft:display_name")
    );
    let custom = CustomComponent {
        name: String::from("demo:glow"),
        parameters: Value::from(1),
        from_v1_list: false,
    };
    assert_eq!(ToyComponent::Custom(custom).name(), None);
    assert_eq!(ToyComponent::Unknown(Value::from(1)).name(), None);
    assert_eq!(ToyComponent::Dynamic(Value::from(1)).name(), None);
}

#[test]
fn conversions_reject_other_variants() {
    let mut set = ToyComponent::from(Glint(true));
    assert_eq!(Glint::from_set(&set), Some(&Glint(true)));
    assert_eq!(Friction::from_set(&set), None);
    assert_eq!(Friction::from_set_mut(&mut set), None);
    Glint::from_set_mut(&mut set).unwrap().0 = false;
    assert_eq!(Friction::try_from_set(set.clone()), Err(set.clone()));
    assert_eq!(Glint::try_from_set(set), Ok(Glint(false)));
    assert_eq!(
        Friction(0.1).into_set(),
        ToyComponent::Friction(Friction(0.1))
    );
}

/// An older version's component enum: it shares `Friction` and `Glint` with `ToyComponent`,
/// so it gets `From` only (no `Component` impl, which would conflict, and no table).
#[derive(Components, Clone, Debug, PartialEq)]
#[component(from_only, custom = Custom, unknown = Unknown)]
enum ToyComponentV1 {
    Friction(Friction),
    Glint(Glint),
    Custom(CustomComponent),
    Unknown(Value),
}

#[test]
fn from_only_enum_gets_from_and_nothing_else() {
    assert_eq!(
        ToyComponentV1::from(Friction(0.2)),
        ToyComponentV1::Friction(Friction(0.2))
    );
    assert_eq!(
        ToyComponentV1::from(Glint(true)),
        ToyComponentV1::Glint(Glint(true))
    );
    // The same types still belong to the newest enum.
    assert_eq!(
        <Friction as Component>::into_set(Friction(0.2)),
        ToyComponent::Friction(Friction(0.2))
    );
    let unknown = ToyComponentV1::Unknown(Value::from(1));
    assert!(matches!(unknown, ToyComponentV1::Unknown(_)));
    let custom = ToyComponentV1::Custom(CustomComponent {
        name: String::from("demo:glow"),
        parameters: Value::from(1),
        from_v1_list: false,
    });
    assert!(matches!(custom, ToyComponentV1::Custom(_)));
}

#[test]
fn table_is_sorted_by_name() {
    let names: Vec<&str> = Toy::components().0.iter().map(|meta| meta.name).collect();
    assert_eq!(
        names,
        [
            "minecraft:ageable",
            "minecraft:display_name",
            "minecraft:friction",
            "minecraft:glint",
            "minecraft:health",
            "minecraft:light_emission",
        ]
    );
}

#[test]
fn table_is_binary_searchable() {
    for name in [
        "minecraft:ageable",
        "minecraft:display_name",
        "minecraft:friction",
        "minecraft:glint",
        "minecraft:health",
        "minecraft:light_emission",
    ] {
        assert_eq!(meta(name).name, name);
    }
    let table = Toy::components().0;
    assert!(
        table
            .binary_search_by(|meta| meta.name.cmp("minecraft:foil"))
            .is_err()
    );
}

#[test]
fn every_attribute_lands_in_the_meta() {
    let glint = meta("minecraft:glint");
    assert_eq!(glint.released, Some(version(1, 20, 20)));
    assert_eq!(glint.deprecated_since, Some(version(1, 26, 0)));
    assert_eq!(
        glint.deprecated_message,
        Some("minecraft:glint is gone in format versions 1.26.0 and above")
    );
    assert_eq!(glint.supersedes, Some("minecraft:foil"));
    assert_eq!(glint.requires, [REQUIREMENT]);
    assert_eq!(glint.area, ContextArea::Vanilla);
    assert_eq!(
        glint.fields,
        [
            FieldMeta {
                name: "",
                since_engine: version(1, 20, 20),
            },
            FieldMeta {
                name: "value.shimmer",
                since_engine: version(1, 21, 10),
            },
        ]
    );
    assert!(std::ptr::fn_addr_eq(glint.parse, parse_glint as ParseFn));
}

#[test]
fn absent_attributes_take_their_defaults() {
    let light = meta("minecraft:light_emission");
    assert_eq!(light.released, None);
    assert_eq!(light.deprecated_since, None);
    assert_eq!(light.deprecated_message, None);
    assert_eq!(light.supersedes, None);
    assert!(light.requires.is_empty());
    assert_eq!(light.area, ContextArea::Public);
    assert!(light.fields.is_empty());
    assert!(std::ptr::fn_addr_eq(
        light.parse,
        parse_light_emission as ParseFn
    ));

    assert_eq!(
        meta("minecraft:friction").released,
        Some(version(1, 19, 20))
    );
    assert_eq!(meta("minecraft:display_name").area, ContextArea::Internal);
}

#[test]
fn dynamic_keys_are_table_rows_without_a_type() {
    let health = meta("minecraft:health");
    assert_eq!(health.released, Some(version(1, 8, 0)));
    assert!(std::ptr::fn_addr_eq(health.parse, keep_json as ParseFn));
    let ageable = meta("minecraft:ageable");
    assert_eq!(ageable.released, None);
    assert!(std::ptr::fn_addr_eq(ageable.parse, keep_json as ParseFn));
}

#[test]
fn custom_and_unknown_variants_are_left_alone() {
    // Four typed components and two dynamic keys; `Custom` and `Unknown` add no row.
    assert_eq!(Toy::components().0.len(), 6);
    let unknown = ToyComponent::Unknown(Value::from(1));
    assert!(matches!(unknown, ToyComponent::Unknown(_)));
}

/// A family whose components hold Molang fields (`#[component(molang)]`).
struct Lit;

/// `Lit`'s chain: uninhabited.
enum LitAny {}

impl VersionChain<Lit> for LitAny {
    const VERSIONS: &'static [FormatVersion] = &[];

    fn parse(
        _payload: JsonSlice<'_>,
        _v: FormatVersion,
        _cx: &LoadContext,
        _d: &mut Diagnostics,
    ) -> Option<Self> {
        None
    }

    fn upgrade_to_end(self, _cx: &mut UpgradeCx<'_>) -> Loaded<Lit> {
        match self {}
    }
}

impl Family for Lit {
    const NAME: &'static str = "lit";
    const PAYLOAD_KEY: &'static str = "minecraft:lit";
    const FLOOR: FormatVersion = version(1, 16, 0);
    type Latest = NoStable;
    type Beta = Infallible;
    type Component = LitComponent;
    type History = LitAny;

    fn components() -> &'static ComponentTable<Self> {
        LitComponent::TABLE
    }
}

/// `minecraft:condition`: one Molang field.
#[derive(Clone, Debug, PartialEq)]
struct Condition(Option<Molang>);

impl MolangFields for Condition {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visitor.field("value", &MolangFieldSpec::DEFAULT, &mut self.0);
    }
}

/// `minecraft:dim`: no Molang.
#[derive(Clone, Debug, PartialEq)]
struct Dim(u8);

/// `Lit`'s component enum: one flagged variant, one without Molang.
#[derive(Components, Clone, Debug, PartialEq)]
#[component(family = Lit, unknown = Unknown)]
enum LitComponent {
    #[component(name = "minecraft:condition", molang, parse = parse_nothing)]
    Condition(Condition),
    #[component(name = "minecraft:dim", parse = parse_nothing)]
    Dim(Dim),
    Unknown(Value),
}

fn parse_nothing(
    _: JsonSlice<'_>,
    _: ConvertMode,
    _: &LoadContext,
    _: &mut Diagnostics,
) -> Option<LitComponent> {
    None
}

/// The paths the Molang walk reaches below `components`.
fn molang_paths<T: MolangFields>(value: &mut T) -> Vec<String> {
    let mut base = JsonPath::new();
    base.push("components");
    let mut paths = Vec::new();
    visit_fields(value, &base, |field| paths.push(field.path.to_string()));
    paths
}

#[test]
fn the_molang_walk_forwards_to_flagged_variants_only() {
    let mut condition = LitComponent::Condition(Condition(Some(Molang::Const(1.0))));
    assert_eq!(molang_paths(&mut condition), ["components/value"]);
    let mut friction = LitComponent::Dim(Dim(2));
    assert!(molang_paths(&mut friction).is_empty());
    // Through a map, each component is reached below its key.
    let mut map: ComponentMap<LitComponent> = [condition, friction].into_iter().collect();
    assert_eq!(
        molang_paths(&mut map),
        ["components/minecraft:condition/value"]
    );
    // The flag adds no table key.
    let table = Lit::components();
    assert_eq!(table.0.len(), 2);
    assert!(std::ptr::fn_addr_eq(
        table.get("minecraft:condition").unwrap().parse,
        parse_nothing as fn(_, _, _, _) -> _
    ));
    // An enum without flagged variants walks nothing.
    let mut toy = ToyComponent::Friction(Friction(0.4));
    assert!(molang_paths(&mut toy).is_empty());
}

#[test]
fn the_derive_expands_inside_a_function_body() {
    #[derive(Components, Clone, Debug, PartialEq)]
    #[component(from_only, custom = Custom, unknown = Unknown)]
    enum LocalComponent {
        Friction(Friction),
        Custom(CustomComponent),
        Unknown(Value),
    }

    assert_eq!(
        LocalComponent::from(Friction(0.3)),
        LocalComponent::Friction(Friction(0.3))
    );
}
