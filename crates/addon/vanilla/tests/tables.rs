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
