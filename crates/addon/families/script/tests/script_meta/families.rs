//! The other metadata families: after-events ordering, vanilla data,
//! commands and Molang queries, read strictly. Documents hand-written in the generator's shapes.

use bedrock_addon_family_script::meta::MetaError;
use bedrock_addon_family_script::meta::model::commands::CommandsFile;
use bedrock_addon_family_script::meta::model::engine::AfterEventsOrdering;
use bedrock_addon_family_script::meta::model::molang::MolangQueriesFile;
use bedrock_addon_family_script::meta::model::vanilla::{
    BiomeEntry, BlockEntry, BlockPropertyType, BlockStateValue, EnchantmentEntry, ItemEntry,
    NameEntry, VanillaDataFile,
};

#[test]
fn after_events_ordering_reads_versions_and_their_order() {
    let text = r#"{ "after_events_order_by_version": [
        { "event_order": [ { "name": "ButtonPushAfterEvent" }, { "name": "PlayerJoinAfterEvent" } ],
          "name": "@minecraft/server 1.3.0", "version": "1.3.0" },
        { "event_order": [], "name": "@minecraft/server 3.0.0-alpha", "version": "3.0.0-alpha" } ],
      "module_type": "after_events_ordering", "name": "engine-after_events_ordering" }"#;
    let file = AfterEventsOrdering::parse(text.as_bytes()).unwrap();
    let first = &file.after_events_order_by_version[0];
    assert_eq!(first.module(), "@minecraft/server");
    assert_eq!(first.event_order[1].name, "PlayerJoinAfterEvent");
    assert_eq!(file.after_events_order_by_version[1].version, "3.0.0-alpha");
}

#[test]
fn blocks_read_their_state_definitions_with_typed_values() {
    let text = r#"{ "block_properties": [
        { "name": "active", "type": "bool", "values": [ { "value": false }, { "value": true } ] },
        { "name": "age", "type": "int", "values": [ { "value": 0 }, { "value": 1 } ] },
        { "name": "facing", "type": "string", "values": [ { "value": "north" } ] } ],
      "data_items": [ { "name": "minecraft:acacia_button", "properties": [ { "name": "facing" } ],
        "raw_id": 395, "serialization_id": "tile.acacia_button" } ],
      "module_type": "vanilla_data", "name": "mojang-block", "vanilla_data_type": "block" }"#;
    let file = VanillaDataFile::<BlockEntry>::parse(text.as_bytes()).unwrap();
    assert_eq!(file.data_items[0].raw_id, 395);
    let properties = file.block_properties.unwrap();
    assert_eq!(properties[0].ty, BlockPropertyType::Bool);
    assert_eq!(properties[0].values[1].value, BlockStateValue::Bool(true));
    assert_eq!(properties[1].values[1].value, BlockStateValue::Int(1));
    assert_eq!(
        properties[2].values[0].value.property_type(),
        BlockPropertyType::String
    );
}

#[test]
fn a_block_state_value_of_another_json_type_is_an_error() {
    let text = r#"{ "block_properties": [ { "name": "age", "type": "int", "values": [ { "value": 1.5 } ] } ],
      "data_items": [], "module_type": "vanilla_data", "name": "mojang-block", "vanilla_data_type": "block" }"#;
    assert!(matches!(
        VanillaDataFile::<BlockEntry>::parse(text.as_bytes()).unwrap_err(),
        MetaError::Json(_)
    ));
}

#[test]
fn items_biomes_enchantments_and_name_lists_read() {
    let items = r#"{ "data_items": [ { "command_name": "minecraft:acacia_boat", "name": "minecraft:acacia_boat",
        "raw_id": 382, "serialization_id": "item.acacia_boat", "serialization_name": "minecraft:acacia_boat" } ],
      "module_type": "vanilla_data", "name": "mojang-item", "vanilla_data_type": "item" }"#;
    let items = VanillaDataFile::<ItemEntry>::parse(items.as_bytes()).unwrap();
    assert_eq!(items.data_items[0].serialization_id, "item.acacia_boat");
    assert!(items.block_properties.is_none());
    let biomes = r#"{ "data_items": [ { "id": 7, "name": "minecraft:plains" } ],
      "module_type": "vanilla_data", "name": "mojang-biome", "vanilla_data_type": "biome" }"#;
    let biomes = VanillaDataFile::<BiomeEntry>::parse(biomes.as_bytes()).unwrap();
    assert_eq!(biomes.data_items[0].id, 7);
    let enchantments = r#"{ "data_items": [ { "name": "minecraft:aqua_affinity" },
        { "name": "minecraft:bow_infinity", "value": "minecraft:infinity" } ],
      "module_type": "vanilla_data", "name": "mojang-enchantment", "vanilla_data_type": "enchantment" }"#;
    let enchantments = VanillaDataFile::<EnchantmentEntry>::parse(enchantments.as_bytes()).unwrap();
    assert_eq!(enchantments.data_items[0].value, None);
    assert_eq!(
        enchantments.data_items[1].value.as_deref(),
        Some("minecraft:infinity")
    );
    let potion_types = r#"{ "data_items": [ { "name": "Consume" } ],
      "module_type": "vanilla_data", "name": "mojang-potion-delivery", "vanilla_data_type": "potionDelivery" }"#;
    let potion_types = VanillaDataFile::<NameEntry>::parse(potion_types.as_bytes()).unwrap();
    assert_eq!(potion_types.data_items[0].name, "Consume");
}

#[test]
fn an_unknown_key_in_a_name_entry_is_an_error() {
    let text = r#"{ "data_items": [ { "id": 3, "name": "minecraft:pig" } ],
      "module_type": "vanilla_data", "name": "mojang-entity", "vanilla_data_type": "entity" }"#;
    let MetaError::UnknownMember { path } =
        VanillaDataFile::<NameEntry>::parse(text.as_bytes()).unwrap_err()
    else {
        panic!("not an unknown member")
    };
    assert_eq!(path.to_string(), "data_items/0/id");
}

#[test]
fn commands_read_overloads_params_and_enums() {
    let text = r#"{ "command_enums": [ { "name": "AimAssistActionSet", "values": [ { "value": "set" } ] } ],
      "commands": [ { "aliases": [ { "name": "tp" } ], "description": "Teleports entities.", "name": "teleport",
        "overloads": [ { "name": "1", "params": [
          { "is_optional": false, "name": "destination", "type": { "name": "POSITION" } },
          { "is_optional": true, "name": "x angle", "type": { "name": "VAL" } } ] } ],
        "permission_level": 1, "requires_cheats": true } ],
      "module_type": "commands", "name": "mojang-commands" }"#;
    let file = CommandsFile::parse(text.as_bytes()).unwrap();
    let command = &file.commands[0];
    assert_eq!(command.aliases[0].name, "tp");
    assert_eq!(command.permission_level, 1);
    assert!(command.requires_cheats);
    let param = &command.overloads[0].params[1];
    assert_eq!(param.name, "x angle");
    assert!(param.is_optional);
    assert_eq!(param.ty.name, "VAL");
    assert_eq!(file.command_enums[0].values[0].value, "set");
}

#[test]
fn molang_queries_read_version_ranges_and_math_functions() {
    let text = r#"{ "math_functions": [ { "description": "Absolute Value", "max_args": 1, "min_args": 1, "name": "math.abs" } ],
      "module_type": "molang", "name": "engine-molang",
      "queries": [ { "description": "Height.", "min_args": 0, "name": "query.above_top_solid", "return_type": "float",
        "version_ranges": [ { "first_version": "0.0.0", "query_sets": [ "world_gen" ] } ] },
        { "description": "Old.", "max_args": 2, "min_args": 1, "name": "query.old", "return_type": "bool",
        "version_ranges": [ { "first_version": "1.0.0", "last_version": "1.20.0", "query_sets": [ "default", "tags" ] } ] } ] }"#;
    let file = MolangQueriesFile::parse(text.as_bytes()).unwrap();
    assert_eq!(file.queries[0].max_args, None);
    assert_eq!(file.queries[1].max_args, Some(2));
    assert_eq!(
        file.queries[1].version_ranges[0].last_version.as_deref(),
        Some("1.20.0")
    );
    assert_eq!(file.math_functions[0].max_args, 1);
}
