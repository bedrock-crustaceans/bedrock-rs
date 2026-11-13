//! The generated vanilla tables as a user of `bedrock_addon_core::vanilla` sees them: `ActorType`
//! (the game's identifier-to-type lookup, the runtime-identifier fallback and the `Mob` default),
//! its numeric ids against the join table, the regeneration check, and the lookups
//! over the identifier lists and the command table.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use bedrock_addon_core::vanilla::{self, ActorType};
use bedrock_addon_test_support::corpus;

/// The generator itself, compiled into this test so the regeneration check runs it in-process.
#[allow(
    dead_code,
    reason = "the test calls the pure part only, not the file-writing `run`"
)]
#[path = "../../../../xtask/src/actor_types.rs"]
mod actor_types;

const JOIN_TABLE: &str = include_str!("../data/actor_types.toml");
const GENERATED: &str = include_str!("../src/vanilla/generated/actor_type.rs");

mod actor_type {
    use super::*;

    #[test]
    fn a_vanilla_identifier_names_its_type() {
        assert_eq!(
            ActorType::from_identifier("minecraft:enderman"),
            Some(ActorType::Enderman)
        );
        assert_eq!(ActorType::Enderman.id(), 0xb26);
        assert_eq!(ActorType::Enderman.identifier(), Some("minecraft:enderman"));
    }

    #[test]
    fn a_bare_vanilla_name_is_the_minecraft_identifier() {
        assert_eq!(ActorType::from_identifier("pig"), Some(ActorType::Pig));
    }

    #[test]
    fn a_custom_identifier_names_no_type() {
        assert_eq!(ActorType::from_identifier("demo:ghost"), None);
        assert_eq!(ActorType::from_identifier("demo:pig"), None);
        assert_eq!(ActorType::from_identifier(""), None);
        assert_eq!(ActorType::from_identifier("minecraft:"), None);
    }

    #[test]
    fn identifiers_are_case_sensitive() {
        assert_eq!(ActorType::from_identifier("minecraft:Pig"), None);
    }

    #[test]
    fn a_vanilla_identifier_keeps_its_type_whatever_the_runtime_identifier() {
        assert_eq!(
            ActorType::for_entity("minecraft:zombie", Some("minecraft:skeleton")),
            ActorType::Zombie
        );
    }

    #[test]
    fn a_custom_identifier_takes_its_runtime_identifiers_type() {
        assert_eq!(
            ActorType::for_entity("demo:arrowish", Some("minecraft:arrow")),
            ActorType::Arrow
        );
        assert_eq!(ActorType::Arrow.id(), 0x00c0_0050);
    }

    #[test]
    fn a_custom_identifier_without_a_runtime_identifier_is_mob() {
        let mob = ActorType::for_entity("demo:ghost", None);
        assert_eq!(mob, ActorType::Mob);
        assert_eq!(mob.id(), 0x100);
        assert_eq!(mob.identifier(), None);
    }

    #[test]
    fn a_custom_identifier_whose_runtime_identifier_is_not_vanilla_is_mob() {
        assert_eq!(
            ActorType::for_entity("demo:ghost", Some("demo:spirit")),
            ActorType::Mob
        );
        assert_eq!(
            ActorType::for_entity("demo:ghost", Some("")),
            ActorType::Mob
        );
    }

    #[test]
    fn well_known_types_have_the_games_ids() {
        // Four ids the game's entity table is known for, and a few more of the same table.
        for (identifier, id) in [
            ("minecraft:enderman", 0xb26),
            ("minecraft:wither", 0x0001_0b34),
            ("minecraft:ender_dragon", 0xb35),
            ("minecraft:boat", 0x5a),
            ("minecraft:pig", 0x130c),
            ("minecraft:player", 0x13f),
            ("minecraft:zombie", 0x0003_0b20),
            ("minecraft:villager_v2", 0x0100_0373),
            ("minecraft:xp_orb", 0x45),
        ] {
            let actor_type = ActorType::from_identifier(identifier)
                .unwrap_or_else(|| panic!("{identifier} is vanilla"));
            assert_eq!(actor_type.id(), id, "{identifier}");
        }
    }

    #[test]
    fn every_join_table_row_resolves_to_its_id() {
        let table = actor_types::parse_join_table(JOIN_TABLE).unwrap();
        assert_eq!(
            table.types.len(),
            129,
            "one row per mojang-entities.json entry"
        );
        for (identifier, id) in &table.types {
            let actor_type = ActorType::from_identifier(identifier)
                .unwrap_or_else(|| panic!("{identifier} has a type"));
            assert_eq!(actor_type.id(), *id, "{identifier}");
            assert_eq!(actor_type.identifier(), Some(identifier.as_str()));
        }
        assert_eq!(table.default, ("Mob".to_owned(), ActorType::Mob.id()));
    }

    #[test]
    fn the_mob_bit_tells_mobs_from_other_actors() {
        // `actor_type & 0x100` is the game's mob test: every mob type carries that bit.
        assert_ne!(ActorType::Mob.id() & 0x100, 0);
        assert_ne!(ActorType::Pig.id() & 0x100, 0);
        assert_eq!(ActorType::Arrow.id() & 0x100, 0);
        assert_eq!(ActorType::Minecart.id() & 0x100, 0);
    }

    #[test]
    fn the_generator_reproduces_the_committed_file() {
        let Ok(root) = std::env::var("BEDROCK_ADDON_CORPUS") else {
            corpus::skipped("the actor type generator", corpus::not_set(corpus::CORPUS));
            return;
        };
        let samples = std::path::Path::new(&root).join("bedrock-samples");
        let entities = std::fs::read_to_string(samples.join(actor_types::ENTITIES)).unwrap();
        let generated = actor_types::generate(&entities, JOIN_TABLE).unwrap();
        assert!(
            generated == GENERATED,
            "crates/addon/core/src/vanilla/generated/actor_type.rs is stale: run `cargo xtask \
             actor-types --samples <dir>`"
        );
    }

    #[test]
    fn the_generator_rejects_an_identifier_without_a_row() {
        let entities = r#"{ "data_items": [ { "name": "minecraft:pig" }, { "name": "minecraft:new_mob" } ],
            "module_type": "vanilla_data", "name": "mojang-entity", "vanilla_data_type": "entity" }"#;
        let join = "[types]\n\"minecraft:pig\" = 0x130c\n[default]\n\"Mob\" = 0x100\n";
        assert!(actor_types::generate(entities, join).is_err());
        let entities = entities.replace(r#", { "name": "minecraft:new_mob" }"#, "");
        assert!(actor_types::generate(&entities, join).is_ok());
    }

    #[test]
    fn the_generator_rejects_a_row_without_an_identifier() {
        let entities = r#"{ "data_items": [ { "name": "minecraft:pig" } ], "module_type": "vanilla_data",
            "name": "mojang-entity", "vanilla_data_type": "entity" }"#;
        let join = "[types]\n\"minecraft:pig\" = 0x130c\n\"minecraft:gone\" = 0x1\n[default]\n\"Mob\" = 0x100\n";
        assert!(actor_types::generate(entities, join).is_err());
    }
}

mod tables {
    use super::*;

    #[test]
    fn an_unknown_command_is_not_found() {
        assert!(vanilla::command("not_a_command").is_none());
        assert!(vanilla::command_enum("NOT_AN_ENUM").is_none());
    }

    #[test]
    fn every_command_is_found_by_name_and_alias() {
        for command in vanilla::commands() {
            assert_eq!(
                vanilla::command(command.name).map(|c| c.name),
                Some(command.name)
            );
            for alias in command.aliases {
                assert_eq!(vanilla::command(alias).map(|c| c.name), Some(command.name));
            }
        }
    }

    /// The blocks the game registers and `mojang-blocks.json` leaves out (technical and
    /// retired blocks: no script constant names them) are blocks of the list all the same.
    #[test]
    fn the_block_list_has_the_blocks_the_scripting_metadata_leaves_out() {
        let blocks = vanilla::ids().blocks;
        for name in [
            "minecraft:chalkboard",
            "minecraft:client_request_placeholder_block",
            "minecraft:deprecated_anvil",
            "minecraft:deprecated_purpur_block_1",
            "minecraft:deprecated_purpur_block_2",
            "minecraft:end_gateway",
            "minecraft:glowingobsidian",
            "minecraft:info_update",
            "minecraft:info_update2",
            "minecraft:invisible_bedrock",
            "minecraft:moving_block",
            "minecraft:netherreactor",
            "minecraft:reserved6",
            "minecraft:stonecutter",
        ] {
            assert!(blocks.contains(&name), "{name} is a block of the game");
        }
        // Its neighbours were there before, and no name is listed twice.
        assert!(blocks.contains(&"minecraft:end_portal"));
        let mut sorted = blocks.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), blocks.len(), "a block listed twice");
    }

    /// Against the game: every block of the game's own registry snapshot (one key per registered
    /// block) is a block of the list. Skipped when the game's files are not available.
    #[test]
    fn every_block_the_game_registers_is_in_the_block_list() {
        let Ok(root) = std::env::var("BEDROCK_ADDON_CORPUS") else {
            corpus::skipped("the block snapshot", corpus::not_set(corpus::CORPUS));
            return;
        };
        let snapshot = std::path::Path::new(&root)
            .join("gdk-dev/test/assets/BlockSnapshotServerTests")
            .join("AllBlocks_AccumulatesSnow_DataMatchesSnapshot.json");
        let Ok(text) = std::fs::read_to_string(&snapshot) else {
            corpus::skipped(
                "the block snapshot",
                format_args!("{} is missing", snapshot.display()),
            );
            return;
        };
        // One member per block: `   "minecraft:acacia_button" : [ false ],`.
        let registered: Vec<&str> = text
            .lines()
            .filter_map(|line| line.trim_start().strip_prefix('"'))
            .filter_map(|line| line.split_once('"'))
            .map(|(name, _)| name)
            .collect();
        assert!(registered.len() > 1_300, "{} blocks", registered.len());
        let blocks: std::collections::HashSet<&str> =
            vanilla::ids().blocks.iter().copied().collect();
        let missing: Vec<&str> = registered
            .into_iter()
            .filter(|name| !blocks.contains(name))
            .collect();
        assert!(
            missing.is_empty(),
            "registered blocks the list lacks: {missing:?}"
        );
    }

    #[test]
    fn every_vanilla_entity_list_entry_has_an_actor_type() {
        for identifier in vanilla::ids().entities {
            assert!(
                ActorType::from_identifier(identifier).is_some(),
                "{identifier}"
            );
        }
    }
}

mod data_crate {
    use super::*;

    #[test]
    fn the_core_tables_are_the_data_crate_s() {
        assert!(std::ptr::eq(vanilla::ids(), bedrock_addon_vanilla::ids()));
        assert!(std::ptr::eq(
            vanilla::commands(),
            bedrock_addon_vanilla::commands()
        ));
        assert!(std::ptr::eq(
            vanilla::command_enums(),
            bedrock_addon_vanilla::command_enums()
        ));
    }
}
