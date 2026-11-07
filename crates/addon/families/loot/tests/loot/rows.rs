//! The fixtures: one test per row, fixtures under
//! `tests/fixtures/loot/loot/`.

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, Severity};
use bedrock_addon_core::document::{Content, UnknownMembers};
use bedrock_addon_core::id::{EntityId, ItemId, LootTablePath};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_family_loot::{
    BannerPattern, BannerType, ConditionKind, EnchantOption, EntryKind, FunctionKind, ItemColor,
    ItemData, ItemDataKind, LootCondition, LootFunction, LootTable, Pool, RandomValueBounds,
    SpecificEnchant,
};
use facet_value::VString;

use super::common::{
    at_least, cx, cx_at, findings, fixture, load, load_fixture, load_in, path, table_path, text,
};

fn item(name: &str) -> ItemId {
    ItemId::new(name).unwrap()
}

#[test]
fn lt01_pools_only_loads_without_a_version_warning() {
    let table = load_fixture("pools_only.json");
    assert!(findings(&table).is_empty(), "{:?}", findings(&table));
    assert_eq!(table.version().written, None);
    let pools = table.pools.as_deref().unwrap();
    assert_eq!(pools.len(), 3);

    let first = &pools[0];
    assert_eq!(first.rolls, Some(RandomValueBounds::Exact(1.0)));
    let entries = first.entries();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].weight, Some(1));
    let EntryKind::Item { name, functions } = &entries[0].kind else {
        panic!("an item entry: {:?}", entries[0].kind);
    };
    assert_eq!(*name, Some(item("minecraft:rotten_flesh")));
    assert_eq!(
        functions.as_deref().unwrap(),
        [
            LootFunction::new(FunctionKind::SetCount {
                count: Some(RandomValueBounds::new(0.0, 2.0)),
            }),
            LootFunction::new(FunctionKind::LootingEnchant {
                count: Some(RandomValueBounds::new(0.0, 1.0)),
            }),
        ]
    );

    let second = &pools[1];
    assert_eq!(
        second.conditions.as_deref().unwrap(),
        [
            LootCondition::new(ConditionKind::KilledByPlayerOrPets),
            LootCondition::new(ConditionKind::RandomChanceWithLooting {
                chance: Some(0.025),
                looting_multiplier: Some(0.01),
            }),
        ]
    );
    assert_eq!(second.entries().len(), 3);

    let third = &pools[2];
    assert_eq!(
        third.conditions.as_deref().unwrap()[1],
        LootCondition::new(ConditionKind::PassengerOfEntity {
            entity_type: Some(EntityId::new("minecraft:chicken").unwrap()),
        })
    );
    assert_eq!(third.entries()[0].weight, None);
    assert_eq!(third.entries()[0].weight(), 1);
    let _ = (Code::from(&core_codes::UNRECOGNIZED_FIELD), Severity::Info);
}

#[test]
fn lt02_a_format_version_is_ignored_whatever_its_value() {
    let plain = load_fixture("pools_only.json");
    let old = load_fixture("with_format_version.json");
    let text = String::from_utf8(fixture("with_format_version.json")).unwrap();
    let future = load(&text.replace("\"1.8.0\"", "\"99.0.0\""));
    assert_eq!(plain.pools.as_deref().map(<[_]>::len), Some(3));
    for table in [&old, &future] {
        assert!(findings(table).is_empty(), "{:?}", findings(table));
        assert_eq!(table.version().written, None);
        assert_eq!(table.pools, plain.pools);
    }
}

#[test]
fn lt04_rolls_bonus_rolls_and_tiers_take_every_shape() {
    let table = load_fixture("rolls_shapes.json");
    assert!(findings(&table).is_empty(), "{:?}", findings(&table));
    let pools = table.pools.as_deref().unwrap();
    assert_eq!(pools[0].rolls, Some(RandomValueBounds::Exact(2.0)));
    assert_eq!(pools[1].rolls, Some(RandomValueBounds::Exact(1.5)));
    assert_eq!(pools[2].rolls, Some(RandomValueBounds::new(1.0, 3.0)));
    assert_eq!(pools[3].rolls, None);
    assert_eq!((pools[3].rolls().min(), pools[3].rolls().max()), (0.0, 0.0));
    assert_eq!(pools[4].bonus_rolls, Some(RandomValueBounds::Exact(2.0)));
    assert_eq!(pools[5].bonus_rolls, Some(RandomValueBounds::new(0.5, 1.5)));
    let tiers = pools[6].tiers.as_ref().unwrap();
    assert_eq!(
        (
            tiers.initial_range(),
            tiers.bonus_rolls(),
            tiers.bonus_chance()
        ),
        (2, 3, 0.095)
    );
    let defaults = pools[7].tiers.as_ref().unwrap();
    assert_eq!(
        (
            defaults.initial_range(),
            defaults.bonus_rolls(),
            defaults.bonus_chance()
        ),
        (0, 0, 0.0)
    );
}

#[test]
fn lt05_an_entry_without_weight_or_quality_gets_1_and_0() {
    let table = load_fixture("weight_defaults.json");
    let entry = &table.pools.as_deref().unwrap()[0].entries()[0];
    assert_eq!((entry.weight, entry.quality), (None, None));
    assert_eq!((entry.weight(), entry.quality()), (1, 0));
    assert!(matches!(entry.kind, EntryKind::Item { .. }));
}

#[test]
fn lt06_an_entry_with_pools_carries_a_sub_table() {
    let table = load_fixture("inline_pools.json");
    assert!(findings(&table).is_empty(), "{:?}", findings(&table));
    let entry = &table.pools.as_deref().unwrap()[0].entries()[0];
    assert_eq!(entry.weight, Some(2));
    let sub = entry.pools.as_deref().unwrap();
    assert_eq!(sub.len(), 1);
    let EntryKind::Item { name, .. } = &sub[0].entries()[0].kind else {
        panic!("an item entry");
    };
    assert_eq!(*name, Some(item("minecraft:apple")));
}

#[test]
fn lt15_bounds_are_floats_in_every_shape_and_kept_unordered() {
    let table = load_fixture("bounds_shapes.json");
    assert!(findings(&table).is_empty(), "{:?}", findings(&table));
    let EntryKind::Item { functions, .. } = &table.pools.as_deref().unwrap()[0].entries()[0].kind
    else {
        panic!("an item entry");
    };
    let counts: Vec<RandomValueBounds> = functions
        .as_deref()
        .unwrap()
        .iter()
        .map(|f| match f.kind {
            FunctionKind::SetCount { count } => count.unwrap(),
            ref other => panic!("not set_count: {other:?}"),
        })
        .collect();
    assert_eq!(
        counts,
        [
            RandomValueBounds::Exact(2.0),
            RandomValueBounds::Exact(1.5),
            RandomValueBounds::Range {
                min: Some(1.0),
                max: None
            },
            RandomValueBounds::new(3.0, 1.0),
        ]
    );
    assert_eq!((counts[2].min(), counts[2].max()), (1.0, 0.0));
    assert_eq!((counts[3].min(), counts[3].max()), (3.0, 1.0));
}

#[test]
fn lt16_a_loot_table_reference_drops_everything_up_to_the_last_colon() {
    let table = load_fixture("loot_table_ref_namespace.json");
    let entry = &table.pools.as_deref().unwrap()[0].entries()[0];
    assert_eq!(
        entry.kind,
        EntryKind::LootTable {
            name: Some(LootTablePath::parse("minecraft:loot_tables/chests/x.json")),
        }
    );
    assert_eq!(entry.table().unwrap().as_str(), "loot_tables/chests/x.json");
}

#[test]
fn lt18_random_difficulty_chance_defaults_each_difficulty_to_default_chance() {
    let table = load_fixture("difficulty_chance.json");
    let condition = &table.pools.as_deref().unwrap()[0]
        .conditions
        .as_deref()
        .unwrap()[0];
    assert_eq!(
        condition.kind.difficulty_chances(),
        Some([0.5, 0.5, 0.5, 1.0])
    );
}

#[test]
fn lt11_an_array_root_is_an_invalid_table() {
    let bytes = fixture("invalid_root.json");
    let rejected = LootTable::parse(&table_path(), &bytes, &cx()).unwrap_err();
    let errors: Vec<_> = rejected
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert_eq!(errors.len(), 1, "{:?}", rejected.diagnostics);
    assert!(errors[0].code.id() == "loot/invalid_table" && errors[0].code.vanilla().is_some());
    assert_eq!(
        text(&errors[0].code),
        "Loot table for loot_tables/test.json is invalid."
    );
}

#[test]
fn lt03_an_unknown_entry_type_is_dropped_with_vanillas_text() {
    let table = load_fixture("entry_types.json");
    let entries = table.pools.as_deref().unwrap()[0].entries();
    assert_eq!(entries.len(), 3);
    assert!(matches!(
        entries[0].kind,
        EntryKind::Item { name: None, .. }
    ));
    assert!(matches!(
        entries[1].kind,
        EntryKind::LootTable { name: None }
    ));
    assert!(matches!(entries[2].kind, EntryKind::Empty));
    // the validation rules: an error in vanilla's log; a kept error, the table loads.
    let found = findings(&table);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].code.id() == "loot/entry_type" && found[0].code.vanilla().is_some());
    assert_eq!(found[0].severity, Severity::Error);
    assert_eq!(path(found[0]), "pools/0/entries/0");
    assert_eq!(
        text(&found[0].code),
        "Every entry of \"entries\" must specify a type of either \"item\", \"loot_table\", or \"empty\". Given: test"
    );
}

/// The functions of the first entry of the first pool.
fn first_functions(table: &LootTable) -> Vec<LootFunction> {
    match &table.pools.as_deref().unwrap()[0].entries()[0].kind {
        EntryKind::Item { functions, .. } => functions.clone().unwrap_or_default(),
        other => panic!("not an item: {other:?}"),
    }
}

#[test]
fn lt07_a_function_name_loses_everything_up_to_the_last_colon() {
    let table = load_fixture("function_namespace.json");
    let functions = first_functions(&table);
    assert_eq!(functions.len(), 3);
    let count = FunctionKind::SetCount {
        count: Some(RandomValueBounds::Exact(2.0)),
    };
    assert!(functions.iter().all(|f| f.kind == count));
    let namespaces: Vec<Option<&str>> = functions.iter().map(|f| f.namespace.as_deref()).collect();
    assert_eq!(namespaces, [None, Some("minecraft:"), Some("foo:bar:")]);
    assert_eq!(functions[2].function_name(), "foo:bar:set_count");

    let found = findings(&table);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].code.id() == "loot/unsupported_function" && found[0].code.vanilla().is_some());
    assert_eq!(found[0].severity, Severity::Error);
    assert_eq!(path(found[0]), "pools/0/entries/0/functions/3");
    assert_eq!(
        text(&found[0].code),
        "Unsupported looting function: set_counts"
    );
}

#[test]
fn lt21_trader_material_type_is_not_a_loot_function() {
    let table = load_fixture("trader_material_in_loot.json");
    assert!(first_functions(&table).is_empty());
    let found = findings(&table);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].code.id() == "loot/unsupported_function" && found[0].code.vanilla().is_some());
    assert_eq!(
        text(&found[0].code),
        "Unsupported looting function: trader_material_type"
    );
}

#[test]
fn lt08_a_namespaced_condition_name_is_unknown_and_dropped_with_an_information() {
    let table = load_fixture("condition_namespace.json");
    let conditions = table.pools.as_deref().unwrap()[0]
        .conditions
        .as_deref()
        .unwrap();
    assert_eq!(
        conditions,
        [LootCondition::new(ConditionKind::RandomChance {
            chance: Some(0.5)
        })]
    );
    let found = findings(&table);
    assert_eq!(found.len(), 1, "{found:?}");
    // Vanilla drops it with a developer-log line only: an information with that line.
    assert_eq!(found[0].code.id(), "loot/unknown_condition");
    assert_eq!(found[0].severity, Severity::Info);
    assert_eq!(path(found[0]), "pools/0/conditions/0");
}

#[test]
fn lt22_vanillas_entity_properties_condition_is_dropped_and_furnace_smelt_kept() {
    let table = load_fixture("entity_properties_condition.json");
    let functions = first_functions(&table);
    assert_eq!(functions.len(), 1);
    assert_eq!(functions[0].kind, FunctionKind::FurnaceSmelt);
    assert_eq!(functions[0].conditions.as_deref(), Some(&[][..]));
    let found = findings(&table);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].code.id(), "loot/unknown_condition");
    assert_eq!(found[0].severity, Severity::Info);
    assert_eq!(path(found[0]), "pools/0/entries/0/functions/0/conditions/0");
    assert!(at_least(&table, Severity::Error).is_empty());
}

#[test]
fn lt10_a_pool_without_entries_is_the_game_s_content_log_error() {
    // The game logs the text at error level and keeps the pool, empty, and the table: the
    // game's severity, an error. The
    // loot reader keeps the table and the pool, empty, with it.
    let table = load_fixture("entries_missing.json");
    let pools = table.pools.as_deref().unwrap();
    assert_eq!(pools.len(), 1);
    assert_eq!(pools[0].entries, None);
    let found = findings(&table);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].code.id() == "loot/entries_missing" && found[0].code.vanilla().is_some());
    assert_eq!(found[0].severity, Severity::Error);
    assert_eq!(path(found[0]), "pools/0");
    // A kept error: the table loads with it.
    assert!(found[0].kept, "{found:?}");
    assert_eq!(
        text(&found[0].code),
        "There must be an \"entries\" array inside the loot table pool."
    );
}

/// `table` with every `extra` emptied.
fn without_extra(mut table: LootTable) -> LootTable {
    fn functions(functions: &mut [LootFunction]) {
        for function in functions {
            function.extra = UnknownMembers::default();
        }
    }
    fn pools(pools: &mut [Pool]) {
        for pool in pools {
            pool.extra = UnknownMembers::default();
            for entry in pool.entries.iter_mut().flatten() {
                entry.extra = UnknownMembers::default();
                if let EntryKind::Item {
                    functions: Some(list),
                    ..
                } = &mut entry.kind
                {
                    functions(list);
                }
            }
        }
    }
    table.extra = UnknownMembers::default();
    pools(table.pools.as_deref_mut().unwrap_or_default());
    table
}

#[test]
fn lt23_keys_vanilla_does_not_read_are_kept_and_reported_at_info() {
    let table = load_fixture("vanilla_extra_keys.json");
    let plain = load(
        r#"{"pools": [{"rolls": 1, "entries": [{"type": "item", "name": "minecraft:wool",
            "functions": [{"function": "looting_enchant", "count": {"min": 0, "max": 1}},
                          {"function": "set_count", "count": {"min": 1, "max": 3}}]}]}]}"#,
    );
    assert_eq!(without_extra(table.value().clone()), *plain.value());
    assert_eq!(
        table
            .extra
            .get("type")
            .and_then(|v| v.as_string())
            .map(VString::as_str),
        Some("minecraft:chest")
    );
    let found = findings(&table);
    assert!(
        found
            .iter()
            .all(|d| d.severity == Severity::Info && d.is(&core_codes::UNRECOGNIZED_FIELD))
    );
    let paths: Vec<String> = found.iter().map(|d| path(d)).collect();
    assert_eq!(
        paths,
        [
            "pools/0/entries/0/functions/0/limit",
            "pools/0/entries/0/functions/1/add",
            "pools/0/entries/0/aux_val",
            "pools/0/entries/0/count",
            "type",
        ]
    );
}

#[test]
fn lt13_random_aux_value_takes_data_or_values_as_set_data_does() {
    let table = load_fixture("random_aux_value_alias.json");
    assert!(findings(&table).is_empty(), "{:?}", findings(&table));
    let functions = first_functions(&table);
    let data: Vec<&ItemData> = functions
        .iter()
        .map(|f| match &f.kind {
            FunctionKind::RandomAuxValue(data) => data,
            other => panic!("not random_aux_value: {other:?}"),
        })
        .collect();
    assert_eq!(data[0].kind(), ItemDataKind::RandomAuxValue);
    assert_eq!((data[0].range().min(), data[0].range().max()), (0.0, 15.0));
    assert_eq!(data[1].kind(), ItemDataKind::SetItemData);
    assert_eq!((data[1].range().min(), data[1].range().max()), (3.0, 3.0));
}

#[test]
fn lt14_set_data_without_data_or_values_is_kept_with_vanillas_warning() {
    let table = load_fixture("set_data_no_key.json");
    let functions = first_functions(&table);
    assert_eq!(functions.len(), 1);
    let FunctionKind::SetData(data) = &functions[0].kind else {
        panic!("not set_data: {:?}", functions[0].kind);
    };
    assert_eq!(data.kind(), ItemDataKind::Unset);
    assert_eq!((data.range().min(), data.range().max()), (0.0, 0.0));
    let found = findings(&table);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].code.id() == "loot/set_data_key" && found[0].code.vanilla().is_some());
    assert_eq!(found[0].severity, Severity::Warning);
    assert_eq!(path(found[0]), "pools/0/entries/0/functions/0");
    assert_eq!(
        text(&found[0].code),
        "Invalid json keyname for specifying the data range for set_data function. Only 'data' and 'values' are supported."
    );
}

#[test]
fn lt12_carry_over_block_entity_data_needs_upcoming_creator_features() {
    let text_ = String::from_utf8(fixture("carry_over.json")).unwrap();
    let load_at = |text_: &str, cx: &LoadContext| load_in(text_, cx).unwrap();

    // UCF off: skipped with the retail text, as a missing toggle.
    let off = load_at(&text_, &cx_at(1, 26, 50, false));
    assert!(first_functions(&off).is_empty());
    let found = findings(&off);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].code.id() == "loot/ucf_function" && found[0].code.vanilla().is_some());
    assert_eq!(found[0].severity, Severity::Error);
    assert_eq!(path(found[0]), "pools/0/entries/0/functions/0");
    assert_eq!(
        text(&found[0].code),
        "Unsupported looting function outside of Upcoming Creator Features Experiment: carry_over_block_entity_data"
    );

    // UCF on: accepted, `dynamic_properties` absent (false).
    let on = load_at(&text_, &cx_at(1, 26, 50, true));
    assert!(findings(&on).is_empty(), "{:?}", findings(&on));
    assert_eq!(
        first_functions(&on)[0].kind,
        FunctionKind::CarryOverBlockEntityData {
            dynamic_properties: None
        }
    );

    // UCF on, a string: vanilla's error text, the default kept (a kept error).
    let yes = text_.replace(
        r#""carry_over_block_entity_data"}"#,
        r#""carry_over_block_entity_data", "dynamic_properties": "yes"}"#,
    );
    let bad = load_at(&yes, &cx_at(1, 26, 50, true));
    assert_eq!(
        first_functions(&bad)[0].kind,
        FunctionKind::CarryOverBlockEntityData {
            dynamic_properties: None
        }
    );
    let found = findings(&bad);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].code.id() == "loot/dynamic_properties" && found[0].code.vanilla().is_some());
    assert_eq!(found[0].severity, Severity::Error);
    assert_eq!(
        path(found[0]),
        "pools/0/entries/0/functions/0/dynamic_properties"
    );
    assert_eq!(
        text(&found[0].code),
        "Error loading loot table \"carry_over_block_entity_data\" dynamic_properties as it is not a bool"
    );

    // Below engine 1.26.40 the name is unknown, as on that engine.
    let old = load_at(&text_, &cx_at(1, 26, 30, true));
    assert!(first_functions(&old).is_empty());
    let found = findings(&old);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(
        text(&found[0].code),
        "Unsupported looting function: carry_over_block_entity_data"
    );
}

#[test]
fn lt24_biome_has_tag_is_a_condition_from_engine_1_26_30() {
    let text_ = String::from_utf8(fixture("biome_has_tag.json")).unwrap();
    let table = load_in(&text_, &cx_at(1, 26, 30, false)).unwrap();
    assert!(findings(&table).is_empty(), "{:?}", findings(&table));
    let conditions = table.pools.as_deref().unwrap()[0]
        .conditions
        .clone()
        .unwrap();
    assert_eq!(
        conditions,
        [
            LootCondition::new(ConditionKind::BiomeHasTag {
                tag: Some("overworld".to_owned())
            }),
            LootCondition::new(ConditionKind::BiomeHasTag { tag: None }),
        ]
    );

    let old = load_in(&text_, &cx_at(1, 26, 20, false)).unwrap();
    assert_eq!(
        old.pools.as_deref().unwrap()[0].conditions.as_deref(),
        Some(&[][..])
    );
    let found = findings(&old);
    assert_eq!(found.len(), 2, "{found:?}");
    assert!(
        found
            .iter()
            .all(|d| d.severity == Severity::Info && d.code.id() == "loot/unknown_condition")
    );
}

/// The `set_banner_details` parameters of `function`.
fn banner(
    function: &LootFunction,
) -> (
    Option<BannerType>,
    Option<ItemColor>,
    Option<Vec<BannerPattern>>,
) {
    match &function.kind {
        FunctionKind::SetBannerDetails {
            banner_type,
            base_color,
            patterns,
        } => (*banner_type, *base_color, patterns.clone()),
        other => panic!("not set_banner_details: {other:?}"),
    }
}

#[test]
fn lt17_banner_details_keep_vanillas_limits() {
    let table = load_fixture("banner_limits.json");
    let functions = first_functions(&table);
    assert_eq!(functions.len(), 4, "every banner function is kept");
    let found = findings(&table);
    let at = |index: usize| {
        found
            .iter()
            .filter(|d| path(d).starts_with(&format!("pools/0/entries/0/functions/{index}")))
            .collect::<Vec<_>>()
    };

    // Seven patterns: vanilla's error, the first six kept.
    let (_, _, patterns) = banner(&functions[0]);
    let names: Vec<String> = patterns.unwrap().into_iter().map(|p| p.pattern).collect();
    assert_eq!(names, ["bo", "bri", "cre", "flo", "moj", "sku"]);
    let first = at(0);
    assert_eq!(first.len(), 1, "{first:?}");
    assert_eq!(first[0].code.id(), "loot/banner_max");
    assert_eq!(first[0].severity, Severity::Error);
    assert_eq!(path(first[0]), "pools/0/entries/0/functions/0/patterns");
    assert_eq!(
        text(&first[0].code),
        "Banner exceeds maximum number of 6 patterns! Patterns given: 7"
    );

    // A non-default type with patterns: vanilla's error, no patterns.
    let (banner_type, _, patterns) = banner(&functions[1]);
    assert_eq!(banner_type, Some(BannerType::IllagerCaptain));
    assert_eq!(patterns, None);
    let second = at(1);
    assert_eq!(second.len(), 1, "{second:?}");
    assert_eq!(second[0].code.id(), "loot/banner_non_default");
    assert_eq!(
        text(&second[0].code),
        "Non-default banner types cannot have custom patterns"
    );

    // An unknown base colour: vanilla's error (the value as `toStyledString` prints it), white.
    let (_, base_color, _) = banner(&functions[2]);
    assert_eq!(base_color, None);
    let third = at(2);
    assert_eq!(third.len(), 1, "{third:?}");
    assert_eq!(third[0].code.id(), "loot/banner_base_color");
    assert_eq!(path(third[0]), "pools/0/entries/0/functions/2/base_color");
    assert_eq!(
        text(&third[0].code),
        "Invalid banner base_color input: \"not_a_color\"\n"
    );

    // A pattern with an unknown colour is kept as white, with vanilla's error. The pattern
    // names are kept as written and not checked, so `nope` is kept.
    let (_, _, patterns) = banner(&functions[3]);
    let patterns = patterns.unwrap();
    assert_eq!(patterns.len(), 2);
    assert_eq!(patterns[1].color, None);
    let fourth = at(3);
    assert_eq!(fourth.len(), 1, "{fourth:?}");
    assert_eq!(fourth[0].code.id(), "loot/banner_pattern_color");
    assert_eq!(
        path(fourth[0]),
        "pools/0/entries/0/functions/3/patterns/1/color"
    );
    assert_eq!(
        text(&fourth[0].code),
        "Invalid color input in banner pattern \"plaid\"\n"
    );
    assert_eq!(found.len(), 4, "{found:?}");
}

/// The `enchantments` of an `enchant_book_for_trading` function.
fn book_options(function: &LootFunction) -> Option<Vec<EnchantOption>> {
    match &function.kind {
        FunctionKind::EnchantBookForTrading { enchantments, .. } => enchantments.clone(),
        other => panic!("not enchant_book_for_trading: {other:?}"),
    }
}

#[test]
fn lt19_enchant_book_options_drop_what_names_no_enchantment() {
    let table = load_fixture("enchant_book_options.json");
    let functions = first_functions(&table);
    assert_eq!(functions.len(), 3);
    let found = findings(&table);

    // One option left: sharpness, its levels the enchantment's own (left out).
    assert_eq!(
        book_options(&functions[0]).unwrap(),
        [EnchantOption {
            name: "sharpness".to_owned(),
            min: None,
            max: None,
            extra: UnknownMembers::default(),
        }]
    );
    let names: Vec<_> = found
        .iter()
        .filter(|d| path(d).starts_with("pools/0/entries/0/functions/0/"))
        .collect();
    assert_eq!(names.len(), 2, "{names:?}");
    for (finding, (at, shown)) in names.iter().zip([("1", "nope"), ("2", "5")]) {
        assert!(finding.code.id() == "loot/enchant_name" && finding.code.vanilla().is_some());
        // An error in vanilla's log; the function and the table load.
        assert_eq!(finding.severity, Severity::Error);
        assert_eq!(
            path(finding),
            format!("pools/0/entries/0/functions/0/enchantments/{at}")
        );
        assert_eq!(
            text(&finding.code),
            format!(
                "Error loading \"enchant_book_for_trading\" function, \"name\": {shown} is not a valid enchantment name"
            )
        );
    }

    // Not an array: vanilla's error, no options.
    assert_eq!(book_options(&functions[1]), None);
    let array: Vec<_> = found
        .iter()
        .filter(|d| path(d).starts_with("pools/0/entries/0/functions/1"))
        .collect();
    assert_eq!(array.len(), 1, "{array:?}");
    assert!(array[0].code.id() == "loot/enchant_array" && array[0].code.vanilla().is_some());
    assert_eq!(path(array[0]), "pools/0/entries/0/functions/1/enchantments");
    assert_eq!(
        text(&array[0].code),
        "Error loading \"enchant_book_for_trading\" function, \"enchantments\": should be type array"
    );

    // `null`: no options, no finding.
    assert_eq!(book_options(&functions[2]), None);
    assert_eq!(found.len(), 3, "{found:?}");
}

#[test]
fn lt20_specific_enchants_drops_an_unknown_id_with_a_warning_deviation() {
    let table = load_fixture("specific_enchants_shapes.json");
    let functions = first_functions(&table);
    let FunctionKind::SpecificEnchants { enchants } = &functions[0].kind else {
        panic!("not specific_enchants: {:?}", functions[0].kind);
    };
    let enchants = enchants.as_deref().unwrap();
    assert_eq!(enchants.len(), 2);
    assert_eq!(enchants[0], SpecificEnchant::new("sharpness"));
    assert_eq!(enchants[0].levels(), (1, 1));
    assert_eq!(enchants[1].id, "unbreaking");
    assert_eq!(enchants[1].levels(), (1, 3));
    // Vanilla drops `nope` without a word; the warning is ours.
    let found = findings(&table);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].is(&core_codes::UNRESOLVED_REFERENCE));
    assert_eq!(found[0].severity, Severity::Info);
    assert_eq!(path(found[0]), "pools/0/entries/0/functions/0/enchants/2");
}

#[test]
fn lt09_an_unknown_item_name_keeps_the_entry_and_is_not_a_load_finding() {
    // The name resolves against the item registry and the built-ins in the link pass (20 L1,
    // X14): the load keeps the entry and the name as written and reports nothing; the
    // warning itself is `link::lt09_an_unknown_item_name_is_warned_with_vanillas_text`
    let table = load_fixture("bad_item.json");
    let entries = table.pools.as_deref().unwrap()[0].entries();
    assert_eq!(entries.len(), 2);
    assert_eq!(
        entries[0].kind,
        EntryKind::Item {
            name: Some(ItemId::parse("minecraft:not_an_item")),
            functions: None,
        }
    );
    assert!(findings(&table).is_empty(), "{:?}", findings(&table));
}
