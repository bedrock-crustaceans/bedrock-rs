//! `ComponentMap<C>` over a toy component enum that implements `ComponentSet`
//! and `Component` by hand, as `#[derive(Components)]` will.

use bedrock_addon_core::document::{
    Component, ComponentMap, ComponentRef, ComponentSet, CustomComponent,
};
use bedrock_addon_core::json::Value;
use bedrock_addon_test_support::json;

#[derive(Clone, Debug, PartialEq)]
struct Friction(f32);

#[derive(Clone, Debug, PartialEq)]
struct LightEmission(u8);

/// The toy family's component enum.
#[derive(Clone, Debug, PartialEq)]
enum ToyComponent {
    Friction(Friction),
    LightEmission(LightEmission),
    Custom(CustomComponent),
    Unknown(Value),
}

impl ComponentSet for ToyComponent {
    fn name(&self) -> Option<&'static str> {
        match self {
            Self::Friction(_) => Some(Friction::NAME),
            Self::LightEmission(_) => Some(LightEmission::NAME),
            Self::Custom(_) | Self::Unknown(_) => None,
        }
    }
}

/// What the derive emits per typed variant.
macro_rules! component {
    ($type:ident, $name:literal) => {
        impl Component for $type {
            type Set = ToyComponent;
            const NAME: &'static str = $name;

            fn from_set(set: &ToyComponent) -> Option<&Self> {
                match set {
                    ToyComponent::$type(value) => Some(value),
                    _ => None,
                }
            }

            fn from_set_mut(set: &mut ToyComponent) -> Option<&mut Self> {
                match set {
                    ToyComponent::$type(value) => Some(value),
                    _ => None,
                }
            }

            fn try_from_set(set: ToyComponent) -> Result<Self, ToyComponent> {
                match set {
                    ToyComponent::$type(value) => Ok(value),
                    other => Err(other),
                }
            }

            fn into_set(self) -> ToyComponent {
                ToyComponent::$type(self)
            }
        }

        impl From<$type> for ToyComponent {
            fn from(value: $type) -> Self {
                Self::$type(value)
            }
        }
    };
}

component!(Friction, "minecraft:friction");
component!(LightEmission, "minecraft:light_emission");

fn names(map: &ComponentMap<ToyComponent>) -> Vec<String> {
    map.iter()
        .map(|entry| match entry {
            ComponentRef::Typed { name, .. }
            | ComponentRef::Dynamic { name, .. }
            | ComponentRef::Unknown { name, .. } => name.to_owned(),
            ComponentRef::Custom(custom) => custom.name.clone(),
            _ => unreachable!("no other entry kind"),
        })
        .collect()
}

#[test]
fn an_empty_map_is_the_default() {
    let map = ComponentMap::<ToyComponent>::new();
    assert!(map.is_empty());
    assert_eq!(map.len(), 0);
    assert_eq!(map, ComponentMap::default());
    assert!(map.get::<Friction>().is_none());
}

#[test]
fn get_finds_a_component_by_its_type() {
    let mut map = ComponentMap::new();
    map.insert(Friction(0.4));
    assert_eq!(map.get::<Friction>(), Some(&Friction(0.4)));
    assert!(map.contains::<Friction>());
    assert!(!map.contains::<LightEmission>());
    map.get_mut::<Friction>().unwrap().0 = 0.6;
    assert_eq!(map.get::<Friction>(), Some(&Friction(0.6)));
}

#[test]
fn insert_returns_the_replaced_component_and_keeps_its_place() {
    let mut map = ComponentMap::new();
    assert_eq!(map.insert(Friction(0.4)), None);
    assert_eq!(map.insert(LightEmission(15)), None);
    assert_eq!(map.insert(Friction(0.9)), Some(Friction(0.4)));
    assert_eq!(
        names(&map),
        ["minecraft:friction", "minecraft:light_emission"]
    );
    assert_eq!(map.len(), 2);
}

#[test]
fn remove_takes_the_component_out_and_keeps_the_order_of_the_rest() {
    let mut map: ComponentMap<ToyComponent> = [
        ToyComponent::from(LightEmission(1)),
        ToyComponent::from(Friction(0.4)),
    ]
    .into_iter()
    .collect();
    map.insert_unknown("demo:later", json("1"));
    assert_eq!(map.remove::<LightEmission>(), Some(LightEmission(1)));
    assert_eq!(map.remove::<LightEmission>(), None);
    assert_eq!(names(&map), ["minecraft:friction", "demo:later"]);
}

#[test]
fn from_iter_keeps_file_order_and_replaces_in_place() {
    let map: ComponentMap<ToyComponent> = [
        Friction(0.4).into(),
        LightEmission(15).into(),
        Friction(0.5).into(),
    ]
    .into_iter()
    .collect();
    assert_eq!(
        names(&map),
        ["minecraft:friction", "minecraft:light_emission"]
    );
    assert_eq!(map.get::<Friction>(), Some(&Friction(0.5)));
}

#[test]
fn extend_adds_after_the_existing_entries() {
    let mut map: ComponentMap<ToyComponent> = [LightEmission(15).into()].into_iter().collect();
    map.extend([ToyComponent::from(Friction(0.4))]);
    assert_eq!(
        names(&map),
        ["minecraft:light_emission", "minecraft:friction"]
    );
}

#[test]
fn from_iter_skips_values_without_a_typed_key() {
    // A custom or unknown variant carries no key of its type.
    let map: ComponentMap<ToyComponent> = [
        ToyComponent::Unknown(json("1")),
        ToyComponent::Custom(CustomComponent {
            name: "demo:glow".to_owned(),
            parameters: json("{}"),
            from_v1_list: false,
        }),
        ToyComponent::from(Friction(0.4)),
    ]
    .into_iter()
    .collect();
    assert_eq!(names(&map), ["minecraft:friction"]);
}

#[test]
fn iter_yields_every_entry_kind_in_file_order() {
    let mut map = ComponentMap::new();
    map.insert_typed(ToyComponent::from(Friction(0.4)), false);
    map.insert_custom(CustomComponent {
        name: "demo:glow".to_owned(),
        parameters: json(r#"{"strength": 2}"#),
        from_v1_list: false,
    });
    map.insert_dynamic("minecraft:health", json(r#"{"value": 20}"#));
    map.insert_unknown("minecraft:not_a_component", json("true"));
    map.insert_typed(ToyComponent::from(LightEmission(15)), true);
    let entries: Vec<ComponentRef<'_, ToyComponent>> = map.iter().collect();
    assert_eq!(entries.len(), 5);
    assert_eq!(
        entries[0],
        ComponentRef::Typed {
            name: "minecraft:friction",
            value: &ToyComponent::Friction(Friction(0.4)),
            synthesized: false,
        }
    );
    assert!(matches!(entries[1], ComponentRef::Custom(c) if c.name == "demo:glow"));
    assert!(
        matches!(entries[2], ComponentRef::Dynamic { name: "minecraft:health", json } if *json == self::json(r#"{"value": 20}"#))
    );
    assert!(matches!(
        entries[3],
        ComponentRef::Unknown {
            name: "minecraft:not_a_component",
            ..
        }
    ));
    assert!(matches!(
        entries[4],
        ComponentRef::Typed {
            name: "minecraft:light_emission",
            synthesized: true,
            ..
        }
    ));
}

#[test]
fn insert_typed_returns_the_replaced_value() {
    let mut map = ComponentMap::new();
    assert_eq!(
        map.insert_typed(ToyComponent::from(Friction(0.4)), true),
        None
    );
    assert_eq!(
        map.insert_typed(ToyComponent::from(Friction(0.5)), false),
        Some(ToyComponent::Friction(Friction(0.4)))
    );
    let unknown = ToyComponent::Unknown(json("1"));
    assert_eq!(
        map.insert_typed(unknown.clone(), false),
        Some(unknown),
        "no key"
    );
    assert_eq!(map.len(), 1);
}

#[test]
fn maps_are_equal_with_equal_entries_in_the_same_order() {
    let ab: ComponentMap<ToyComponent> = [Friction(0.4).into(), LightEmission(1).into()]
        .into_iter()
        .collect();
    let ba: ComponentMap<ToyComponent> = [LightEmission(1).into(), Friction(0.4).into()]
        .into_iter()
        .collect();
    assert_eq!(ab, ab.clone());
    assert_ne!(ab, ba);
}

#[test]
fn a_typed_get_ignores_a_non_typed_entry_of_the_same_key() {
    let mut map = ComponentMap::<ToyComponent>::new();
    map.insert_unknown("minecraft:friction", json("\"slippery\""));
    assert_eq!(map.get::<Friction>(), None);
    assert_eq!(map.remove::<Friction>(), None);
    assert_eq!(map.len(), 1, "the unknown entry stays");
    // A typed insert replaces it in place.
    assert_eq!(map.insert(Friction(0.4)), None);
    assert_eq!(map.get::<Friction>(), Some(&Friction(0.4)));
}

#[test]
fn remove_named_drops_any_entry_kind_and_keeps_the_order() {
    let mut map: ComponentMap<ToyComponent> = [Friction(0.4).into(), LightEmission(15).into()]
        .into_iter()
        .collect();
    map.insert_custom(CustomComponent {
        name: "demo:glow".to_owned(),
        parameters: json("{}"),
        from_v1_list: false,
    });
    map.insert_unknown("demo:other", json("1"));
    assert!(map.remove_named("minecraft:friction"));
    assert!(map.remove_named("demo:glow"));
    assert!(!map.remove_named("minecraft:friction"), "already gone");
    assert_eq!(names(&map), ["minecraft:light_emission", "demo:other"]);
}

/// A map hands each typed component to the Molang walk below its key; the other entries hold
/// no typed Molang.
#[test]
fn a_map_forwards_the_molang_walk_to_its_typed_components() {
    use bedrock_addon_core::diagnostic::JsonPath;
    use bedrock_addon_core::molang::{
        Molang, MolangFieldSpec, MolangFields, MolangVisitor, visit_fields,
    };

    /// A component with one Molang field.
    #[derive(Clone, Debug, PartialEq)]
    struct Condition(Option<Molang>);

    #[derive(Clone, Debug, PartialEq)]
    enum MolangComponent {
        Condition(Condition),
    }
    impl ComponentSet for MolangComponent {
        fn name(&self) -> Option<&'static str> {
            Some("demo:condition")
        }
    }
    impl MolangFields for MolangComponent {
        fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
            let Self::Condition(Condition(condition)) = self;
            visitor.field("condition", &MolangFieldSpec::DEFAULT, condition);
        }
    }

    let mut map = ComponentMap::new();
    map.insert_typed(
        MolangComponent::Condition(Condition(Some(Molang::Const(1.0)))),
        false,
    );
    map.insert_unknown("demo:other", json(r#"{"condition": "q.x"}"#));
    let mut base = JsonPath::new();
    base.push("components");
    let mut paths = Vec::new();
    visit_fields(&mut map, &base, |field| paths.push(field.path.to_string()));
    assert_eq!(paths, ["components/demo:condition/condition"]);
}

#[test]
fn insert_typed_as_keeps_one_entry_per_written_key() {
    let mut map = ComponentMap::new();
    map.insert_typed_as(
        "minecraft:friction.a",
        ToyComponent::from(Friction(0.4)),
        false,
    );
    map.insert_typed_as(
        "minecraft:friction.b",
        ToyComponent::from(Friction(0.5)),
        false,
    );
    assert_eq!(
        map.insert_typed_as(
            "minecraft:friction.a",
            ToyComponent::from(Friction(0.6)),
            false
        ),
        Some(ToyComponent::Friction(Friction(0.4)))
    );
    assert_eq!(
        names(&map),
        ["minecraft:friction.a", "minecraft:friction.b"]
    );
    assert_eq!(
        map.get::<Friction>(),
        None,
        "no entry under the type's own key"
    );
}

// `CustomComponent`'s `Default` is an unnamed keyed component with null parameters.
#[test]
fn a_custom_component_defaults_to_an_unnamed_keyed_one_with_null_parameters() {
    let default = CustomComponent::default();
    assert_eq!(default.name, "");
    assert!(default.parameters.is_null());
    assert!(
        !default.from_v1_list,
        "written keyed, not from the 1.21.60 list"
    );
    // What the default is for: a caller names what it sets.
    let glow = CustomComponent {
        name: "demo:glow".to_owned(),
        ..CustomComponent::default()
    };
    let mut map: ComponentMap<ToyComponent> = ComponentMap::new();
    map.insert_custom(glow.clone());
    assert_eq!(names(&map), ["demo:glow"]);
    assert_eq!(
        glow,
        CustomComponent {
            name: "demo:glow".to_owned(),
            parameters: bedrock_addon_core::json::Value::NULL,
            from_v1_list: false,
        }
    );
}
