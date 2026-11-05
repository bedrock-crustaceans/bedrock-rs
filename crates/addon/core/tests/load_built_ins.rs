//! The built-in provider: the generated tables as the default `BuiltIns`, the widened lookups
//! (command, command enum, enchantment alias) against the plain table functions, and a host's
//! provider on the load context preferred over the tables.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::sync::Arc;

use bedrock_addon_core::load::{BuiltIns, LoadContext, LoadOptions};
use bedrock_addon_core::pack_identity::PackType;
use bedrock_addon_core::vanilla::{self, Command, CommandEnum, GeneratedBuiltIns};
use uuid::Uuid;

#[test]
fn the_generated_provider_answers_a_vanilla_id_and_misses_an_unknown_one() {
    let tables = GeneratedBuiltIns;
    assert!(tables.contains("block", "minecraft:stone"));
    assert!(tables.contains("item", "minecraft:stick"));
    assert!(tables.contains("entity", "minecraft:pig"));
    assert!(tables.contains("biome", "minecraft:plains"));
    assert!(tables.contains("dimension", "minecraft:overworld"));
    assert!(!tables.contains("block", "demo:ruby"));
    assert!(!tables.contains("no such kind", "minecraft:stone"));

    assert_eq!(
        tables.command("tp").map(|command| command.name),
        Some("teleport")
    );
    assert!(tables.command("not_a_command").is_none());
    assert!(tables.command_enum("not an enum").is_none());
    assert!(tables.enchantment_alias("demo:sharpness").is_none());
}

/// What function's `find_command` answers: the first command in table order whose name or an
/// alias is `word` without ASCII case.
fn find_command_folded(word: &str) -> Option<&'static Command> {
    vanilla::commands().iter().find(|command| {
        command.name.eq_ignore_ascii_case(word)
            || command
                .aliases
                .iter()
                .any(|alias| alias.eq_ignore_ascii_case(word))
    })
}

#[test]
fn the_generated_command_lookup_folds_ascii_case_like_function() {
    let tables = GeneratedBuiltIns;
    assert_eq!(
        tables.command("TP").map(|command| command.name),
        Some("teleport")
    );
    assert_eq!(
        tables.command("Teleport").map(|command| command.name),
        Some("teleport")
    );
    for command in vanilla::commands() {
        let names = std::iter::once(command.name).chain(command.aliases.iter().copied());
        for name in names {
            for word in [
                name.to_owned(),
                name.to_ascii_uppercase(),
                name.to_ascii_lowercase(),
            ] {
                assert_eq!(tables.command(&word), find_command_folded(&word), "{word}");
            }
        }
    }
    // A word longer than any stack buffer still folds.
    let long = "A".repeat(300);
    assert_eq!(tables.command(&long), None);
    assert_eq!(tables.command_enum(&long), None);
}

#[test]
fn the_generated_lookups_agree_with_the_table_functions() {
    let tables = GeneratedBuiltIns;
    for command in vanilla::commands() {
        let names = std::iter::once(command.name).chain(command.aliases.iter().copied());
        for name in names {
            assert_eq!(tables.command(name), vanilla::command(name), "{name}");
        }
    }
    for command_enum in vanilla::command_enums() {
        for name in [
            command_enum.name.to_owned(),
            command_enum.name.to_ascii_uppercase(),
        ] {
            assert_eq!(
                tables.command_enum(&name),
                vanilla::command_enum(&name),
                "{name}"
            );
        }
    }
    let ids = vanilla::ids();
    assert!(!ids.enchantment_aliases.is_empty());
    for (alias, target) in ids.enchantment_aliases {
        let first = ids
            .enchantment_aliases
            .iter()
            .find(|(a, _)| a == alias)
            .map(|(_, t)| *t);
        assert_eq!(
            tables.enchantment_alias(alias),
            first,
            "{alias} -> {target}"
        );
    }
    for (kind, list) in [
        ("block", ids.blocks),
        ("item", ids.items),
        ("entity", ids.entities),
        ("biome", ids.biomes),
        ("dimension", ids.dimensions),
        ("camera preset", ids.camera_presets),
        ("jigsaw structure", ids.features),
    ] {
        for id in list {
            assert!(tables.contains(kind, id), "{kind} {id}");
        }
    }
}

/// A host that knows one block, renames `tp`, and spells one enchantment alias its own way;
/// the rest it leaves to the trait's defaults.
struct Host;

impl BuiltIns for Host {
    fn contains(&self, kind: &'static str, id: &str) -> bool {
        kind == "block" && id == "host:block"
    }
    fn command(&self, name: &str) -> Option<&Command> {
        match name {
            "tp" => vanilla::command("give"),
            other => vanilla::command(other),
        }
    }
    fn enchantment_alias(&self, alias: &str) -> Option<&str> {
        (alias == "host:alias").then_some("host:enchantment")
    }
}

fn context_with(built_ins: Option<Arc<dyn BuiltIns>>) -> LoadContext {
    let options = LoadOptions {
        built_ins,
        ..LoadOptions::default()
    };
    LoadContext::for_pack(&options, Uuid::nil(), PackType::Behavior, None, false)
}

#[test]
fn a_detached_context_answers_with_the_generated_tables() {
    let cx = LoadContext::detached();
    assert!(cx.host_built_ins().is_none());
    assert!(cx.built_ins().contains("block", "minecraft:stone"));
    assert_eq!(cx.built_ins().command("tp"), vanilla::command("tp"));
    let cx = context_with(None);
    assert!(cx.host_built_ins().is_none());
    assert!(cx.built_ins().contains("entity", "minecraft:pig"));
}

#[test]
fn a_host_provider_is_preferred_over_the_tables() {
    let cx = context_with(Some(Arc::new(Host)));
    assert!(cx.host_built_ins().is_some());
    let built_ins = cx.built_ins();
    // Believed, yes or no.
    assert!(built_ins.contains("block", "host:block"));
    assert!(!built_ins.contains("block", "minecraft:stone"));
    assert_eq!(
        built_ins.command("tp").map(|command| command.name),
        Some("give")
    );
    assert_eq!(
        built_ins.enchantment_alias("host:alias"),
        Some("host:enchantment")
    );
    // What the host does not answer, the trait's default answers from the tables.
    let first_enum: &CommandEnum = &vanilla::command_enums()[0];
    assert_eq!(built_ins.command_enum(first_enum.name), Some(first_enum));

    // A detached context takes a host the same way.
    let mut detached = LoadContext::detached();
    detached.set_built_ins(Some(Arc::new(Host)));
    assert!(detached.built_ins().contains("block", "host:block"));
}

#[test]
fn a_context_debug_prints_the_providers_as_present_or_absent() {
    let mut cx = LoadContext::detached();
    let absent = format!("{cx:?}");
    assert!(absent.contains(r#"built_ins: "absent""#), "{absent}");
    cx.set_built_ins(Some(Arc::new(Host)));
    let present = format!("{cx:?}");
    assert!(present.contains(r#"built_ins: "present""#), "{present}");
    #[cfg(feature = "molang")]
    assert!(present.contains(r#"molang: "absent""#), "{present}");
}
