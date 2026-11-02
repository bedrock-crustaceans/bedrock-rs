//! The schema: every function and every condition reads its parameters,
//! with no finding when each has the JSON type vanilla reads.

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::id::{EntityId, ItemId, LootTablePath};
use bedrock_addon_core::shared::{IntRange, RangeForm};
use bedrock_addon_family_loot::{
    BannerPattern, BannerType, ConditionKind, EnchantOption, EntryKind, FunctionKind, ItemColor,
    ItemData, LootCondition, LootFunction, RandomValueBounds, SpecificEnchant, StewEffect, Tiers,
    ToolEnchantment,
};

use super::common::{at_least, load};

/// One function of each kind (but the experimental one), every parameter written.
const ALL_FUNCTIONS: &str = r#"
        {"function": "set_count", "count": 3},
        {"function": "set_data", "data": {"min": 1, "max": 2}},
        {"function": "set_damage", "damage": 0.5},
        {"function": "looting_enchant", "count": {"min": 0, "max": 1}},
        {"function": "enchant_with_levels", "levels": 30, "treasure": true},
        {"function": "enchant_book_for_trading", "base_cost": 1, "per_level_cost": 2, "base_random_cost": 3, "per_level_random_cost": 4,
         "enchantments": [{"name": "sharpness", "min": 1, "max": 5}]},
        {"function": "enchant_randomly", "treasure": false},
        {"function": "furnace_smelt"},
        {"function": "set_data_from_color_index"},
        {"function": "enchant_random_gear", "chance": 0.25},
        {"function": "random_aux_value", "values": {"min": 0, "max": 6}},
        {"function": "random_block_state", "block_state": "age", "values": {"min": 0, "max": 7}},
        {"function": "random_dye"},
        {"function": "exploration_map", "destination": "buriedtreasure"},
        {"function": "set_banner_details", "type": "default", "base_color": "red",
         "patterns": [{"pattern": "bo", "color": "blue"}]},
        {"function": "explosion_decay"},
        {"function": "set_name", "name": "Excalibur"},
        {"function": "set_lore", "lore": ["one", "two"]},
        {"function": "specific_enchants", "enchants": ["sharpness", {"id": "unbreaking", "level": [1, 3]}]},
        {"function": "fill_container", "loot_table": "loot_tables/chests/x.json"},
        {"function": "set_actor_id", "id": "minecraft:pig"},
        {"function": "set_book_contents", "author": "a", "title": "t", "pages": ["p1", "p2"]},
        {"function": "set_stew_effect", "effects": [{"id": 3}]},
        {"function": "set_ominous_bottle_amplifier", "amplifier": {"min": 0, "max": 4}},
        {"function": "set_armor_trim", "material": "copper", "pattern": "bolt"},
        {"function": "set_potion", "id": "healing"}
        "#;

/// One condition of each kind, every parameter written.
const ALL_CONDITIONS: &str = r#"
        {"condition": "killed_by_player"},
        {"condition": "killed_by_player_or_pets"},
        {"condition": "killed_by_entity", "entity_type": "minecraft:skeleton"},
        {"condition": "entity_killed", "entity_type": "minecraft:slime"},
        {"condition": "random_chance", "chance": 0.5},
        {"condition": "random_difficulty_chance", "default_chance": 0.5, "peaceful": 0, "easy": 0.1, "normal": 0.2, "hard": 0.8},
        {"condition": "random_chance_with_looting", "chance": 0.025, "looting_multiplier": 0.01},
        {"condition": "random_regional_difficulty_chance", "max_chance": 0.15},
        {"condition": "has_mark_variant", "value": -1},
        {"condition": "has_variant", "value": 2},
        {"condition": "match_tool", "item": "minecraft:shears", "count": {"range_min": 1, "range_max": 2}, "durability": 5,
         "enchantments": [{"enchantment": "silk_touch", "levels": {"range_min": 1}}],
         "minecraft:match_tool_filter_any": ["minecraft:is_pickaxe"],
         "minecraft:match_tool_filter_all": ["minecraft:is_tool"],
         "minecraft:match_tool_filter_none": ["minecraft:is_axe"]},
        {"condition": "damaged_by_entity", "entity_type": "minecraft:fireball"},
        {"condition": "passenger_of_entity", "entity_type": "minecraft:chicken"},
        {"condition": "is_baby"},
        {"condition": "has_property", "domain": "minecraft:p"},
        {"condition": "bool_property", "domain": "minecraft:has_flower", "value": true},
        {"condition": "int_property", "domain": "minecraft:i", "value": 3},
        {"condition": "float_property", "domain": "minecraft:f", "value": 0.5},
        {"condition": "enum_property", "domain": "minecraft:e", "value": "red"},
        {"condition": "biome_has_tag", "tag": "overworld"}
        "#;

/// The functions of the table's one item entry.
fn functions(text: &str) -> Vec<LootFunction> {
    let table = load(text);
    assert!(
        at_least(&table, Severity::Info).is_empty(),
        "{:?}",
        table.diagnostics()
    );
    let pools = table.pools.as_deref().unwrap();
    match &pools[0].entries()[0].kind {
        EntryKind::Item { functions, .. } => functions.clone().unwrap(),
        other => panic!("not an item: {other:?}"),
    }
}

/// A table whose one item entry has `functions`.
fn with_functions(functions: &str) -> String {
    format!(
        r#"{{"pools": [{{"rolls": 1, "entries": [{{"type": "item", "name": "minecraft:stick", "functions": [{functions}]}}]}}]}}"#
    )
}

/// The conditions of the table's one pool.
fn conditions(conditions: &str) -> Vec<LootCondition> {
    let text = format!(
        r#"{{"pools": [{{"rolls": 1, "conditions": [{conditions}], "entries": [{{"type": "empty"}}]}}]}}"#
    );
    let table = load(&text);
    assert!(
        at_least(&table, Severity::Info).is_empty(),
        "{:?}",
        table.diagnostics()
    );
    table.pools.as_deref().unwrap()[0]
        .conditions
        .clone()
        .unwrap()
}

fn kinds(functions: Vec<LootFunction>) -> Vec<FunctionKind> {
    functions.into_iter().map(|f| f.kind).collect()
}

fn range(min: i32, max: i32, form: RangeForm) -> IntRange {
    IntRange::new(min, max, form)
}

#[test]
fn every_function_reads_its_parameters() {
    let text = with_functions(ALL_FUNCTIONS);
    let expected = vec![
        FunctionKind::SetCount {
            count: Some(RandomValueBounds::Exact(3.0)),
        },
        FunctionKind::SetData(ItemData {
            data: Some(RandomValueBounds::new(1.0, 2.0)),
            values: None,
        }),
        FunctionKind::SetDamage {
            damage: Some(RandomValueBounds::Exact(0.5)),
        },
        FunctionKind::LootingEnchant {
            count: Some(RandomValueBounds::new(0.0, 1.0)),
        },
        FunctionKind::EnchantWithLevels {
            levels: Some(RandomValueBounds::Exact(30.0)),
            treasure: Some(true),
        },
        FunctionKind::EnchantBookForTrading {
            base_cost: Some(1),
            per_level_cost: Some(2),
            base_random_cost: Some(3),
            per_level_random_cost: Some(4),
            enchantments: Some(vec![EnchantOption {
                name: "sharpness".to_owned(),
                min: Some(1),
                max: Some(5),
                extra: Default::default(),
            }]),
        },
        FunctionKind::EnchantRandomly {
            treasure: Some(false),
        },
        FunctionKind::FurnaceSmelt,
        FunctionKind::SetDataFromColorIndex,
        FunctionKind::EnchantRandomGear { chance: Some(0.25) },
        FunctionKind::RandomAuxValue(ItemData {
            data: None,
            values: Some(RandomValueBounds::new(0.0, 6.0)),
        }),
        FunctionKind::RandomBlockState {
            block_state: Some("age".to_owned()),
            values: Some(RandomValueBounds::new(0.0, 7.0)),
        },
        FunctionKind::RandomDye,
        FunctionKind::ExplorationMap {
            destination: Some("buriedtreasure".to_owned()),
        },
        FunctionKind::SetBannerDetails {
            banner_type: Some(BannerType::Default),
            base_color: Some(ItemColor::Red),
            patterns: Some(vec![BannerPattern {
                pattern: "bo".to_owned(),
                color: Some(ItemColor::Blue),
                extra: Default::default(),
            }]),
        },
        FunctionKind::ExplosionDecay,
        FunctionKind::SetName {
            name: Some("Excalibur".to_owned()),
        },
        FunctionKind::SetLore {
            lore: Some(vec!["one".to_owned(), "two".to_owned()]),
        },
        FunctionKind::SpecificEnchants {
            enchants: Some(vec![
                SpecificEnchant::new("sharpness"),
                SpecificEnchant {
                    id: "unbreaking".to_owned(),
                    level: Some(range(1, 3, RangeForm::Array)),
                    extra: Default::default(),
                },
            ]),
        },
        FunctionKind::FillContainer {
            loot_table: Some(LootTablePath::new("loot_tables/chests/x.json").unwrap()),
        },
        FunctionKind::SetActorId {
            id: Some(EntityId::new("minecraft:pig").unwrap()),
        },
        FunctionKind::SetBookContents {
            author: Some("a".to_owned()),
            title: Some("t".to_owned()),
            pages: Some(vec!["p1".to_owned(), "p2".to_owned()]),
        },
        FunctionKind::SetStewEffect {
            effects: Some(vec![StewEffect {
                id: Some(3),
                extra: Default::default(),
            }]),
        },
        FunctionKind::SetOminousBottleAmplifier {
            amplifier: Some(RandomValueBounds::new(0.0, 4.0)),
        },
        FunctionKind::SetArmorTrim {
            material: Some("copper".to_owned()),
            pattern: Some("bolt".to_owned()),
        },
        FunctionKind::SetPotion {
            id: Some("healing".to_owned()),
        },
    ];
    assert_eq!(kinds(functions(&text)), expected);
}

#[test]
fn every_condition_reads_its_parameters() {
    let read = conditions(ALL_CONDITIONS);
    let entity = |id: &str| Some(EntityId::new(id).unwrap());
    let mut levels = IntRange::new(1, 0, RangeForm::RangeMinMax);
    levels.missing_max = true;
    let expected = vec![
        ConditionKind::KilledByPlayer,
        ConditionKind::KilledByPlayerOrPets,
        ConditionKind::KilledByEntity {
            entity_type: entity("minecraft:skeleton"),
        },
        ConditionKind::EntityKilled {
            entity_type: entity("minecraft:slime"),
        },
        ConditionKind::RandomChance { chance: Some(0.5) },
        ConditionKind::RandomDifficultyChance {
            default_chance: Some(0.5),
            peaceful: Some(0.0),
            easy: Some(0.1),
            normal: Some(0.2),
            hard: Some(0.8),
        },
        ConditionKind::RandomChanceWithLooting {
            chance: Some(0.025),
            looting_multiplier: Some(0.01),
        },
        ConditionKind::RandomRegionalDifficultyChance {
            max_chance: Some(0.15),
        },
        ConditionKind::HasMarkVariant { value: Some(-1) },
        ConditionKind::HasVariant { value: Some(2) },
        ConditionKind::MatchTool {
            item: Some(ItemId::new("minecraft:shears").unwrap()),
            count: Some(range(1, 2, RangeForm::RangeMinMax)),
            durability: Some(range(5, 5, RangeForm::Scalar)),
            enchantments: Some(vec![ToolEnchantment {
                enchantment: Some("silk_touch".to_owned()),
                levels: Some(levels),
                extra: Default::default(),
            }]),
            filter_any: Some(vec!["minecraft:is_pickaxe".to_owned()]),
            filter_all: Some(vec!["minecraft:is_tool".to_owned()]),
            filter_none: Some(vec!["minecraft:is_axe".to_owned()]),
        },
        ConditionKind::DamagedByEntity {
            entity_type: entity("minecraft:fireball"),
        },
        ConditionKind::PassengerOfEntity {
            entity_type: entity("minecraft:chicken"),
        },
        ConditionKind::IsBaby,
        ConditionKind::HasProperty {
            domain: Some("minecraft:p".to_owned()),
        },
        ConditionKind::BoolProperty {
            domain: Some("minecraft:has_flower".to_owned()),
            value: Some(true),
        },
        ConditionKind::IntProperty {
            domain: Some("minecraft:i".to_owned()),
            value: Some(3),
        },
        ConditionKind::FloatProperty {
            domain: Some("minecraft:f".to_owned()),
            value: Some(0.5),
        },
        ConditionKind::EnumProperty {
            domain: Some("minecraft:e".to_owned()),
            value: Some("red".to_owned()),
        },
        ConditionKind::BiomeHasTag {
            tag: Some("overworld".to_owned()),
        },
    ];
    let read: Vec<ConditionKind> = read.into_iter().map(|c| c.kind).collect();
    assert_eq!(read, expected);
}

#[test]
fn pool_tiers_and_entry_members_are_read() {
    let table = load(
        r#"{"pools": [{"rolls": 1, "bonus_rolls": 2, "tiers": {"initial_range": 2, "bonus_rolls": 3, "bonus_chance": 0.095},
            "entries": [{"type": "loot_table", "name": "loot_tables/chests/x.json", "weight": 5, "quality": -1}]}]}"#,
    );
    assert!(
        at_least(&table, Severity::Info).is_empty(),
        "{:?}",
        table.diagnostics()
    );
    let pool = &table.pools.as_deref().unwrap()[0];
    assert_eq!(pool.bonus_rolls, Some(RandomValueBounds::Exact(2.0)));
    assert_eq!(
        pool.tiers,
        Some(Tiers {
            initial_range: Some(2),
            bonus_rolls: Some(3),
            bonus_chance: Some(0.095),
            extra: Default::default(),
        })
    );
    let entry = &pool.entries()[0];
    assert_eq!((entry.weight, entry.quality), (Some(5), Some(-1)));
    assert_eq!(
        entry.kind,
        EntryKind::LootTable {
            name: Some(LootTablePath::new("loot_tables/chests/x.json").unwrap()),
        }
    );
}

#[test]
fn every_parameter_is_written_back_under_its_own_key() {
    // Value level: what is read is written, and reads back equal.
    let conditions_text = format!(
        r#"{{"pools": [{{"rolls": 1, "conditions": [{ALL_CONDITIONS}], "entries": [{{"type": "empty"}}]}}]}}"#
    );
    for text in [with_functions(ALL_FUNCTIONS), conditions_text] {
        let table = load(&text);
        let written = bedrock_addon_core::document::Content::to_json(table.value()).unwrap();
        let again = load(&written);
        assert!(
            at_least(&again, Severity::Info).is_empty(),
            "{:?}",
            again.diagnostics()
        );
        assert_eq!(again.value(), table.value(), "{written}");
        assert_eq!(
            bedrock_addon_core::document::Content::to_json(again.value()).unwrap(),
            written
        );
    }
}
