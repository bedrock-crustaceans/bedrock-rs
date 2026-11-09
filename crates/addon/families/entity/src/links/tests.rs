//! The member tables, the member patterns and what an entity's references name.

use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::json::{JsonSlice, Value};
use bedrock_addon_core::load::{Experiment, LoadContext};
use bedrock_addon_core::versioning::{Loaded, load};

use super::table::{self, Row};
use super::*;

/// Every table, by name.
fn tables() -> Vec<(&'static str, &'static [Row])> {
    vec![
        ("BLOCK_NAMES", table::BLOCK_NAMES),
        ("BLOCKS", table::BLOCKS),
        ("BLOCKS_AS_ITEMS", table::BLOCKS_AS_ITEMS),
        ("LOOT_TABLES", table::LOOT_TABLES),
        ("TRADE_TABLES", table::TRADE_TABLES),
        ("ENTITIES", table::ENTITIES),
        ("ITEMS", table::ITEMS),
        ("SOUNDS", table::SOUNDS),
        ("PARTICLES", table::PARTICLES),
        ("POIS", table::POIS),
    ]
}

/// A table is sorted by key and pattern and has no row twice (the order of a component's
/// findings is the table's), and every key is one the component table of the entity family
/// has: a row for a key the game does not bind would never match.
#[test]
fn every_table_is_sorted_unique_and_of_known_components() {
    for (name, rows) in tables() {
        assert!(!rows.is_empty(), "{name}");
        assert!(
            rows.windows(2).all(|pair| pair[0] < pair[1]),
            "{name} is sorted and unique"
        );
        for (key, pattern) in rows {
            assert!(
                EntityComponent::TABLE.get(key).is_some(),
                "{name}: {key} is a component"
            );
            assert!(
                !pattern.is_empty() && !pattern.contains(".."),
                "{name}: {key} {pattern}"
            );
        }
    }
}

/// The two members the game reads as lists of block names (`std::vector<std::string>` in
/// its definitions of `home` and of the `raid_garden` goal) are no rows of the table of the
/// members that hold a block DESCRIPTOR: no `{name, states}` and no `{tags}` can stand there.
#[test]
fn a_list_of_block_names_is_no_row_of_the_descriptor_table() {
    let names = [
        ("minecraft:behavior.raid_garden", "blocks[]"),
        ("minecraft:home", "home_block_list[]"),
    ];
    for row in names {
        assert!(
            !table::BLOCKS.contains(&row),
            "{row:?} is a list of names and is listed as a descriptor member"
        );
    }
    assert_eq!(table::BLOCK_NAMES, names);
    // A reference tells the two tables apart by the component: no component is in both.
    for (key, _) in table::BLOCK_NAMES {
        assert!(table::BLOCKS.iter().all(|(other, _)| other != key), "{key}");
    }
}

fn json(text: &str) -> Value {
    bedrock_addon_core::json::from_str::<Value>(text).expect("json")
}

/// What `pattern` names below `text`: each path and value.
fn named(text: &str, pattern: &str) -> Vec<(String, String)> {
    let value = json(text);
    let mut found = Vec::new();
    members(
        &value,
        pattern,
        &JsonPath::new().child("c"),
        &mut |at: JsonPath, value| {
            let text = bedrock_addon_core::json::to_json(value).expect("writes");
            // Written without the layout of the writer.
            let text: String = text.split_whitespace().collect();
            found.push((at.to_string(), text));
        },
    );
    found
}

fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter()
        .map(|(at, value)| ((*at).to_owned(), (*value).to_owned()))
        .collect()
}

#[test]
fn a_pattern_names_a_member_each_element_of_a_list_or_the_one_value_in_its_place() {
    assert_eq!(named(r#"{"a": "x"}"#, "a"), pairs(&[("c/a", r#""x""#)]));
    // `[]`: each element of an array, with its index; a value that is no array, itself.
    assert_eq!(
        named(r#"{"a": ["x", "y"]}"#, "a[]"),
        pairs(&[("c/a/0", r#""x""#), ("c/a/1", r#""y""#)])
    );
    assert_eq!(named(r#"{"a": "x"}"#, "a[]"), pairs(&[("c/a", r#""x""#)]));
    assert_eq!(named(r#"{"a": []}"#, "a[]"), pairs(&[]));
    // Without `[]` an array is the value.
    assert_eq!(named(r#"{"a": ["x"]}"#, "a"), pairs(&[("c/a", r#"["x"]"#)]));
    // Nested, through a list of objects or one object.
    assert_eq!(
        named(r#"{"a": [{"b": 1}, {"z": 2}, {"b": [3, 4]}]}"#, "a[].b[]"),
        pairs(&[("c/a/0/b", "1"), ("c/a/2/b/0", "3"), ("c/a/2/b/1", "4")])
    );
    assert_eq!(
        named(r#"{"a": {"b": 1}}"#, "a[].b"),
        pairs(&[("c/a/b", "1")])
    );
    assert_eq!(
        named(r#"{"a": {"b": {"d": 1}}}"#, "a.b.d"),
        pairs(&[("c/a/b/d", "1")])
    );
}

#[test]
fn a_pattern_names_nothing_where_the_member_is_absent_or_its_parent_no_object() {
    assert_eq!(named(r#"{"z": 1}"#, "a"), pairs(&[]));
    assert_eq!(named(r#"{"a": 1}"#, "a.b"), pairs(&[]));
    assert_eq!(named(r#"{"a": [1, "x"]}"#, "a[].b"), pairs(&[]));
    assert_eq!(named("[]", "a"), pairs(&[]));
    assert_eq!(named("5", "a[]"), pairs(&[]));
}

/// `*` names each member of an object that is itself an object (the values of a map): a
/// member that holds a string, a number or a list is passed over, at the end of a pattern as
/// in its middle.
#[test]
fn a_star_names_only_object_members() {
    assert_eq!(
        named(r#"{"a": {"x": {"b": 1}, "y": {"b": 2}, "z": 3}}"#, "a.*.b"),
        pairs(&[("c/a/x/b", "1"), ("c/a/y/b", "2")])
    );
    assert_eq!(
        named(r#"{"a": {"x": {}, "y": "s", "z": [1], "w": 4}}"#, "a.*"),
        pairs(&[("c/a/x", "{}")])
    );
    assert_eq!(named(r#"{"a": 1}"#, "a.*"), pairs(&[]));
}

/// The newest entity `components` and `events` load to (JSON text of the two objects).
fn entity(components: &str, events: &str) -> Entity {
    let text = format!(
        r#"{{"format_version": "1.26.50", "minecraft:entity": {{
            "description": {{"identifier": "demo:pet"}},
            "components": {components},
            "events": {events}
        }}}}"#
    );
    // The Brain components (the memory sensors) are read under their experiment only.
    let mut cx = LoadContext::detached();
    cx.experiments.insert(Experiment::BrainPrototype);
    let outcome = load::<Entity>(JsonSlice::new(&text), &cx);
    match outcome.value {
        Some(Loaded::Stable(entity)) => entity,
        other => panic!("{other:?}: {:#?}", outcome.diagnostics),
    }
}

/// Each reference of `kind` as its component, its path (from below the payload key) and its
/// value as JSON text.
fn said(references: &References<'_>, kind: ReferenceKind) -> Vec<(String, String, String)> {
    let payload = format!("{}/", <Entity as Family>::PAYLOAD_KEY);
    references
        .of(kind)
        .into_iter()
        .map(|found| {
            let at = found.at.to_string();
            let at = at.strip_prefix(&payload).unwrap_or(&at).to_owned();
            (found.component.to_owned(), at, text(&found.reference))
        })
        .collect()
}

/// The name `reference` names, quoted as the member writes it.
fn text(reference: &Reference<'_>) -> String {
    let name = match reference {
        Reference::Block(BlockDescriptor::Block { name, .. }) | Reference::BlockName(name) => {
            name.as_str().to_owned()
        }
        Reference::BlockAsItem(ItemDescriptor::Item { id, .. })
        | Reference::Item {
            descriptor: ItemDescriptor::Item { id, .. },
            ..
        } => id.as_str().to_owned(),
        Reference::Entity { name, .. } => name.as_str().to_owned(),
        Reference::LootTable(name)
        | Reference::TradeTable(name)
        | Reference::Sound(name)
        | Reference::Poi(name)
        | Reference::Particle { name, .. } => (*name).to_owned(),
        other => format!("{other:?}"),
    };
    format!("{name:?}")
}

fn rows(list: &[(&str, &str, &str)]) -> Vec<(String, String, String)> {
    list.iter()
        .map(|(component, at, value)| {
            (
                (*component).to_owned(),
                (*at).to_owned(),
                (*value).to_owned(),
            )
        })
        .collect()
}

/// One member of every table, in `components` and in a component group: each kind names its
/// own members, with the component and the path the entity holds it at; the blocks of a
/// descriptor and of a list of names come in one walk, in document order.
#[test]
fn references_name_the_members_each_table_lists() {
    let entity = entity(
        r#"{
            "minecraft:break_blocks": {"breakable_blocks": ["demo:b"]},
            "minecraft:home": {"home_block_list": ["demo:h"]},
            "minecraft:trail": {"block_type": "demo:t"},
            "minecraft:loot": {"table": "loot_tables/l.json"},
            "minecraft:trade_table": {"table": "trading/t.json"},
            "minecraft:transformation": {"into": "demo:y"},
            "minecraft:heartbeat": {"sound_event": "heartbeat"},
            "minecraft:memory_sensor.find_nearest_poi": {"sensor_entries": [{"poi_name": "demo:poi"}]}
        }"#,
        "{}",
    );
    let references = entity.references();
    let c = "components/minecraft:";
    assert_eq!(
        said(&references, ReferenceKind::Block),
        rows(&[
            (
                "minecraft:break_blocks",
                &format!("{c}break_blocks/breakable_blocks/0"),
                r#""demo:b""#
            ),
            (
                "minecraft:home",
                &format!("{c}home/home_block_list/0"),
                r#""demo:h""#
            ),
            (
                "minecraft:trail",
                &format!("{c}trail/block_type"),
                r#""demo:t""#
            ),
        ])
    );
    assert_eq!(
        said(&references, ReferenceKind::LootTable),
        rows(&[(
            "minecraft:loot",
            &format!("{c}loot/table"),
            r#""loot_tables/l.json""#
        )])
    );
    assert_eq!(
        said(&references, ReferenceKind::TradeTable),
        rows(&[(
            "minecraft:trade_table",
            &format!("{c}trade_table/table"),
            r#""trading/t.json""#
        )])
    );
    assert_eq!(
        said(&references, ReferenceKind::Entity),
        rows(&[(
            "minecraft:transformation",
            &format!("{c}transformation/into"),
            r#""demo:y""#
        )])
    );
    assert_eq!(
        said(&references, ReferenceKind::Sound),
        rows(&[(
            "minecraft:heartbeat",
            &format!("{c}heartbeat/sound_event"),
            r#""heartbeat""#
        )])
    );
    assert_eq!(
        said(&references, ReferenceKind::Poi),
        rows(&[(
            "minecraft:memory_sensor.find_nearest_poi",
            &format!("{c}memory_sensor.find_nearest_poi/sensor_entries/0/poi_name"),
            r#""demo:poi""#
        )])
    );
    for kind in [
        ReferenceKind::BlockAsItem,
        ReferenceKind::Item,
        ReferenceKind::Particle,
    ] {
        assert_eq!(said(&references, kind), rows(&[]), "{kind:?}");
    }
}

/// The members of the item, block-as-item and particle tables, in a component group: the
/// group's path is the one held.
#[test]
fn references_name_the_members_of_a_component_group() {
    let text = r#"{
        "format_version": "1.26.50", "minecraft:entity": {
            "description": {"identifier": "demo:pet"},
            "components": {},
            "component_groups": {"g": {
                "minecraft:tameable": {"tame_items": ["demo:bone"]},
                "minecraft:behavior.avoid_block": {"priority": 1, "target_blocks": ["demo:a"]},
                "minecraft:explode": {"particle_effect": "demo:boom"}
            }}
        }}"#;
    let outcome = load::<Entity>(JsonSlice::new(text), &LoadContext::detached());
    let Some(Loaded::Stable(entity)) = outcome.value else {
        panic!("{:#?}", outcome.diagnostics);
    };
    let references = entity.references();
    let g = "component_groups/g/minecraft:";
    assert_eq!(
        said(&references, ReferenceKind::Item),
        rows(&[(
            "minecraft:tameable",
            &format!("{g}tameable/tame_items/0"),
            r#""demo:bone""#
        )])
    );
    assert_eq!(
        said(&references, ReferenceKind::BlockAsItem),
        rows(&[(
            "minecraft:behavior.avoid_block",
            &format!("{g}behavior.avoid_block/target_blocks/0"),
            r#""demo:a""#
        )])
    );
    assert_eq!(
        said(&references, ReferenceKind::Particle),
        rows(&[(
            "minecraft:explode",
            &format!("{g}explode/particle_effect"),
            r#""demo:boom""#
        )])
    );
}

/// `emit_particle.particle` of an event's body and of a node nested in it, in document order.
#[test]
fn emitted_particles_are_named_in_every_event_node() {
    let entity = entity(
        "{}",
        r#"{
            "demo:a": {"emit_particle": {"particle": "demo:p1"}, "sequence": [
                {"emit_particle": {"particle": "demo:p2"}},
                {"randomize": [{"emit_particle": {"particle": "demo:p3"}}]}
            ]},
            "demo:b": {"emit_particle": {"particle": "demo:p4"}}
        }"#,
    );
    let found: Vec<(String, String)> = entity
        .references()
        .emitted_particles()
        .into_iter()
        .map(|found| {
            assert_eq!(found.component, "emit_particle");
            let Reference::Particle { name, .. } = found.reference else {
                panic!("{:?}", found.reference);
            };
            (found.at.to_string(), name.to_owned())
        })
        .collect();
    let e = "minecraft:entity/events/";
    let expected = [
        (format!("{e}demo:a/emit_particle/particle"), "demo:p1"),
        (
            format!("{e}demo:a/sequence/0/emit_particle/particle"),
            "demo:p2",
        ),
        (
            format!("{e}demo:a/sequence/1/randomize/0/emit_particle/particle"),
            "demo:p3",
        ),
        (format!("{e}demo:b/emit_particle/particle"), "demo:p4"),
    ];
    let expected: Vec<(String, String)> = expected
        .into_iter()
        .map(|(at, name)| (at, name.to_owned()))
        .collect();
    assert_eq!(found, expected);
}

/// Each reference of `kind` with its path from below the payload key.
fn read_as(references: &References<'_>, kind: ReferenceKind) -> Vec<(String, Reference<'static>)> {
    let payload = format!("{}/", <Entity as Family>::PAYLOAD_KEY);
    references
        .of(kind)
        .into_iter()
        .map(|found| {
            let at = found.at.to_string();
            let at = at.strip_prefix(&payload).unwrap_or(&at).to_owned();
            (at, owned(found.reference))
        })
        .collect()
}

/// `reference` with its borrowed names leaked, so a test can hold it past the entity.
fn owned(reference: Reference<'_>) -> Reference<'static> {
    let leak = |text: &str| -> &'static str { Box::leak(text.to_owned().into_boxed_str()) };
    match reference {
        Reference::Entity { name, event } => Reference::Entity {
            name,
            event: event.map(leak),
        },
        Reference::LootTable(path) => Reference::LootTable(leak(path)),
        Reference::TradeTable(path) => Reference::TradeTable(leak(path)),
        Reference::Sound(name) => Reference::Sound(leak(name)),
        Reference::Particle { name, effect } => Reference::Particle {
            name: leak(name),
            effect,
        },
        Reference::Poi(name) => Reference::Poi(leak(name)),
        Reference::Block(descriptor) => Reference::Block(descriptor),
        Reference::BlockName(id) => Reference::BlockName(id),
        Reference::BlockAsItem(descriptor) => Reference::BlockAsItem(descriptor),
        Reference::Offspring { mate, child } => Reference::Offspring { mate, child },
        Reference::Mate(id) => Reference::Mate(id),
        Reference::Item {
            descriptor,
            or_item_tag,
        } => Reference::Item {
            descriptor,
            or_item_tag,
        },
    }
}

/// A member of the descriptor table names a block descriptor, in any of its forms; a member
/// of a list of names names a block by its name, an empty one included.
#[test]
fn a_block_member_names_a_descriptor_and_a_name_list_names_blocks() {
    let entity = entity(
        r#"{
            "minecraft:break_blocks": {"breakable_blocks": ["demo:b", {"name": "demo:o"}, {"tags": "q.any_tag('t')"}]},
            "minecraft:home": {"home_block_list": ["demo:h", ""]}
        }"#,
        "{}",
    );
    let c = "components/minecraft:";
    let descriptor = |text: &str| Reference::Block(BlockDescriptor::from(json(text)));
    assert_eq!(
        read_as(&entity.references(), ReferenceKind::Block),
        vec![
            (
                format!("{c}break_blocks/breakable_blocks/0"),
                descriptor(r#""demo:b""#)
            ),
            (
                format!("{c}break_blocks/breakable_blocks/1"),
                descriptor(r#"{"name": "demo:o"}"#)
            ),
            (
                format!("{c}break_blocks/breakable_blocks/2"),
                descriptor(r#"{"tags": "q.any_tag('t')"}"#)
            ),
            (
                format!("{c}home/home_block_list/0"),
                Reference::BlockName(BlockId::parse("demo:h"))
            ),
            (
                format!("{c}home/home_block_list/1"),
                Reference::BlockName(BlockId::parse(""))
            ),
        ]
    );
}

/// An entity member names the identifier as written and the event of its `<event>`; an
/// empty member names nothing.
#[test]
fn an_entity_member_names_the_entity_and_its_event() {
    let entity = entity(
        r#"{
            "minecraft:transformation": {"into": "demo:y<demo:grow>"},
            "minecraft:shooter": {"def": "demo:arrow"},
            "minecraft:spawn_on_death": {"entity_to_spawn": ""}
        }"#,
        "{}",
    );
    let references = entity.references();
    let found = read_as(&references, ReferenceKind::Entity);
    let c = "components/minecraft:";
    let texts: Vec<(String, String, Option<&str>)> = found
        .iter()
        .map(|(at, reference)| match reference {
            Reference::Entity { name, event } => (at.clone(), name.as_str().to_owned(), *event),
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(
        texts,
        vec![
            (
                format!("{c}transformation/into"),
                "demo:y<demo:grow>".to_owned(),
                Some("demo:grow")
            ),
            (format!("{c}shooter/def"), "demo:arrow".to_owned(), None),
        ]
    );
}

/// Each pair of `offspring_pairs` names its mate (the key) and its child (the value), at the
/// pair; each key of `breeds_with` names a mate, at its member.
#[test]
fn offspring_pairs_name_mate_and_child() {
    let entity = entity(
        r#"{
            "minecraft:breedable": {"breeds_with": {"demo:p": {}}},
            "minecraft:offspring": {"offspring_pairs": {"demo:m": "demo:c", "demo:n": ""}}
        }"#,
        "{}",
    );
    let c = "components/minecraft:";
    let id = |text: &str| EntityId::parse(text);
    assert_eq!(
        read_as(&entity.references(), ReferenceKind::Entity),
        vec![
            (
                format!("{c}breedable/breeds_with/demo:p"),
                Reference::Mate(id("demo:p"))
            ),
            (
                format!("{c}offspring/offspring_pairs/demo:m"),
                Reference::Offspring {
                    mate: Some(id("demo:m")),
                    child: Some(id("demo:c")),
                }
            ),
            (
                format!("{c}offspring/offspring_pairs/demo:n"),
                Reference::Offspring {
                    mate: Some(id("demo:n")),
                    child: None,
                }
            ),
        ]
    );
}

/// An item member names its descriptor in the form it is written; `shareables.items[].item`
/// may be an item tag; an empty name names nothing.
#[test]
fn an_item_member_names_its_descriptor_form() {
    let entity = entity(
        r#"{
            "minecraft:shareables": {"items": [{"item": "demo:food", "craft_into": "demo:pie"}]},
            "minecraft:tameable": {"tame_items": ["demo:bone", {"item": "demo:x"}, ""]}
        }"#,
        "{}",
    );
    let found = read_as(&entity.references(), ReferenceKind::Item);
    let forms: Vec<(String, String, ItemDescriptorForm, bool)> = found
        .into_iter()
        .map(|(at, reference)| match reference {
            Reference::Item {
                descriptor: ItemDescriptor::Item { id, form, .. },
                or_item_tag,
            } => (at, id.as_str().to_owned(), form, or_item_tag),
            other => panic!("{other:?}"),
        })
        .collect();
    let c = "components/minecraft:";
    let row = |at: &str, id: &str, form, or_item_tag| {
        (format!("{c}{at}"), id.to_owned(), form, or_item_tag)
    };
    assert_eq!(
        forms,
        vec![
            row(
                "shareables/items/0",
                "demo:food",
                ItemDescriptorForm::Item,
                true
            ),
            row(
                "shareables/items/0/craft_into",
                "demo:pie",
                ItemDescriptorForm::String,
                false
            ),
            row(
                "tameable/tame_items/0",
                "demo:bone",
                ItemDescriptorForm::String,
                false
            ),
            row(
                "tameable/tame_items/1",
                "demo:x",
                ItemDescriptorForm::Item,
                false
            ),
        ]
    );
}

/// `explode.particle_effect` and an event's `emit_particle.particle` name an effect; the
/// other particle members name a particle type.
#[test]
fn explode_names_an_effect_and_emit_particle_an_effect() {
    let entity = entity(
        r#"{
            "minecraft:explode": {"particle_effect": "demo:boom"},
            "minecraft:interact": {"interactions": [{"particle_on_start": {"particle_type": "smoke"}}]}
        }"#,
        r#"{"demo:a": {"emit_particle": {"particle": "demo:spark"}}}"#,
    );
    let references = entity.references();
    let c = "components/minecraft:";
    assert_eq!(
        read_as(&references, ReferenceKind::Particle),
        vec![
            (
                format!("{c}explode/particle_effect"),
                Reference::Particle {
                    name: "demo:boom",
                    effect: true
                }
            ),
            (
                format!("{c}interact/interactions/0/particle_on_start/particle_type"),
                Reference::Particle {
                    name: "smoke",
                    effect: false
                }
            ),
        ]
    );
    let emitted: Vec<Reference<'_>> = references
        .emitted_particles()
        .into_iter()
        .map(|found| found.reference)
        .collect();
    assert_eq!(
        emitted,
        vec![Reference::Particle {
            name: "demo:spark",
            effect: true
        }]
    );
}

/// A path, a sound or a point of interest is the member's name; an empty one names nothing.
#[test]
fn a_name_member_names_its_text_and_an_empty_one_nothing() {
    let entity = entity(
        r#"{
            "minecraft:loot": {"table": ""},
            "minecraft:equipment": {"table": "loot_tables/e.json"},
            "minecraft:economy_trade_table": {"table": "trading/t.json"},
            "minecraft:heartbeat": {"sound_event": ""},
            "minecraft:celebrate_hunt": {"celebrate_sound": "celebrate"},
            "minecraft:memory_sensor.find_nearest_poi": {"sensor_entries": [{"poi_name": ""}, {"poi_name": "demo:poi"}]}
        }"#,
        "{}",
    );
    let references = entity.references();
    let c = "components/minecraft:";
    assert_eq!(
        read_as(&references, ReferenceKind::LootTable),
        vec![(
            format!("{c}equipment/table"),
            Reference::LootTable("loot_tables/e.json")
        )]
    );
    assert_eq!(
        read_as(&references, ReferenceKind::TradeTable),
        vec![(
            format!("{c}economy_trade_table/table"),
            Reference::TradeTable("trading/t.json")
        )]
    );
    assert_eq!(
        read_as(&references, ReferenceKind::Sound),
        vec![(
            format!("{c}celebrate_hunt/celebrate_sound"),
            Reference::Sound("celebrate")
        )]
    );
    assert_eq!(
        read_as(&references, ReferenceKind::Poi),
        vec![(
            format!("{c}memory_sensor.find_nearest_poi/sensor_entries/1/poi_name"),
            Reference::Poi("demo:poi")
        )]
    );
}

/// The keys of the components the family keeps as JSON, from its component list's source.
fn dynamic_components() -> Vec<&'static str> {
    const COMPONENTS: &str = include_str!("../component/mod.rs");
    COMPONENTS
        .lines()
        .filter(|line| line.contains("#[component(name = ") && line.contains("parse = keep_json"))
        .map(|line| {
            line.split('"')
                .nth(1)
                .expect("the component name is quoted")
        })
        .collect()
}

/// The descriptor table of `dynamic` is the block and item members of the components kept as
/// JSON, no row more and none less: a member one list gains, the other must.
#[test]
fn the_descriptor_members_are_the_block_and_item_members_of_dynamic_components() {
    let dynamic = dynamic_components();
    assert!(dynamic.len() > 100, "{}", dynamic.len());
    let mut of_the_tables: Vec<(&str, &str)> =
        [table::BLOCKS, table::BLOCKS_AS_ITEMS, table::ITEMS]
            .into_iter()
            .flatten()
            .copied()
            .filter(|(component, _)| dynamic.contains(component))
            .collect();
    of_the_tables.sort_unstable();
    of_the_tables.dedup();
    assert_eq!(crate::DESCRIPTORS, of_the_tables);
    // Sorted, and no component in it that is typed.
    assert!(crate::DESCRIPTORS.is_sorted());
    assert_eq!(crate::DESCRIPTORS.len(), 42);
}
