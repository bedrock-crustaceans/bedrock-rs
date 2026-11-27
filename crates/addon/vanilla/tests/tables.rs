//! The tables as the data crate's users see them: the generated lists and the command table
//! with the sizes the committed metadata gives them, and the lookups over them.

#[test]
fn the_command_table_has_the_committed_commands_and_enums() {
    assert_eq!(bedrock_addon_vanilla::commands().len(), 84);
    assert_eq!(bedrock_addon_vanilla::command_enums().len(), 236);
    assert!(
        bedrock_addon_vanilla::commands()
            .windows(2)
            .all(|pair| pair[0].name < pair[1].name)
    );
}

#[test]
fn a_command_is_found_by_its_name_or_an_alias_exactly() {
    let teleport = bedrock_addon_vanilla::command("tp").map(|command| command.name);
    assert_eq!(teleport, Some("teleport"));
    assert!(bedrock_addon_vanilla::command("TP").is_none());
    assert!(bedrock_addon_vanilla::command_enum("GAMEMODE").is_some());
}

#[test]
fn the_identifier_lists_hold_the_game_s_identifiers() {
    let ids = bedrock_addon_vanilla::ids();
    assert!(ids.blocks.contains(&"minecraft:stone"));
    assert!(ids.items.contains(&"minecraft:stick"));
    assert_eq!(
        ids.enchantment_aliases,
        &[("minecraft:bow_infinity", "minecraft:infinity")]
    );
}

#[test]
fn the_block_state_table_has_a_row_per_listed_block() {
    let rows = bedrock_addon_vanilla::block_states();
    let listed = bedrock_addon_vanilla::ids()
        .blocks
        .iter()
        .filter(|name| rows.iter().any(|row| row.name == **name))
        .count();
    assert_eq!(listed, rows.len());
    assert!(rows.windows(2).all(|pair| pair[0].name < pair[1].name));
}

#[test]
fn stairs_carry_their_states_and_value_lists() {
    use bedrock_addon_vanilla::BlockStateValues;
    let stairs = bedrock_addon_vanilla::block_state("minecraft:oak_stairs").unwrap();
    assert_eq!(stairs.serialization_id, "tile.oak_stairs");
    let direction = stairs
        .states
        .iter()
        .find(|s| s.name == "weirdo_direction")
        .unwrap();
    assert_eq!(direction.values, BlockStateValues::Int(&[0, 1, 2, 3]));
    let upside = stairs
        .states
        .iter()
        .find(|s| s.name == "upside_down_bit")
        .unwrap();
    assert_eq!(upside.values, BlockStateValues::Bool(&[false, true]));
    let chest = bedrock_addon_vanilla::block_state("minecraft:chest").unwrap();
    assert_eq!(
        chest.states[0].values,
        BlockStateValues::Str(&["south", "west", "north", "east"])
    );
}

#[test]
fn a_block_without_states_or_a_listing_has_no_surprise() {
    assert!(
        bedrock_addon_vanilla::block_state("minecraft:stone")
            .unwrap()
            .states
            .is_empty()
    );
    assert!(bedrock_addon_vanilla::block_state("minecraft:no_such_block").is_none());
    assert!(bedrock_addon_vanilla::block_state("stone").is_none());
}
