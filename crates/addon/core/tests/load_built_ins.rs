//! The built-in provider: the generated tables as the default `BuiltIns`, the widened lookups
//! (command, command enum, enchantment alias) against the plain table functions, and a host's
//! provider on the load context preferred over the tables.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::sync::Arc;

use bedrock_addon_core::id::LevelSoundEventName;
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

/// The kinds the link pass used to answer from its own lists, now the tables'.
#[test]
fn the_generated_provider_answers_the_kinds_the_link_pass_kept() {
    let tables = GeneratedBuiltIns;
    for feature in vanilla::names::CODE_FEATURES {
        assert!(tables.contains("feature", feature), "{feature}");
    }
    assert!(!tables.contains("feature", "demo:feature"));
    for event in vanilla::sound_events::LEVEL_SOUND_EVENTS {
        assert!(tables.contains("level sound event", event), "{event}");
    }
    // As written: the kind's names are lower case.
    assert!(!tables.contains("level sound event", "Ambient"));
    for category in vanilla::names::AIM_ASSIST_CATEGORIES {
        assert!(
            tables.contains("aim assist category", category),
            "{category}"
        );
    }
    for preset in vanilla::names::AIM_ASSIST_PRESETS {
        assert!(tables.contains("aim assist preset", preset), "{preset}");
    }
    assert!(!tables.contains("aim assist preset", "minecraft:bucket"));
}

/// recipe's `potion` before the provider: `minecraft:` and the name lower-cased, if listed.
fn potion_before(name: &str) -> Option<String> {
    let id = format!("minecraft:{}", name.to_lowercase());
    vanilla::ids()
        .potion_effects
        .contains(&id.as_str())
        .then_some(id)
}

/// loot's `is_enchantment` before the provider: the bare name of an enchantment, an alias or
/// an alias's target.
fn is_enchantment_before(name: &str) -> bool {
    let ids = vanilla::ids();
    let bare = |id: &&str| id.strip_prefix("minecraft:") == Some(name);
    ids.enchantments.iter().any(bare)
        || ids
            .enchantment_aliases
            .iter()
            .any(|(alias, target)| bare(alias) || bare(target))
}

/// item's `known_effect` before the provider (over the compiled-in, never empty, list).
fn effect_before(name: &str) -> bool {
    vanilla::ids().effects.iter().any(|e| {
        e.trim_start_matches("minecraft:")
            .eq_ignore_ascii_case(name)
    })
}

#[test]
fn the_value_lookups_of_the_tables_answer_as_their_users_did() {
    let tables = GeneratedBuiltIns;
    let ids = vanilla::ids();
    assert_eq!(tables.level_sound_event("Ambient"), Some("ambient"));
    assert_eq!(tables.level_sound_event("custom.boom"), None);
    assert_eq!(tables.potion("Awkward"), Some("minecraft:awkward"));
    assert_eq!(tables.potion("minecraft:awkward"), None);
    assert!(tables.is_enchantment("sharpness"));
    assert!(tables.is_enchantment("bow_infinity"));
    assert!(!tables.is_enchantment("minecraft:sharpness"));
    assert!(!tables.is_enchantment("Sharpness"));
    assert!(tables.effect("Speed"));
    assert!(!tables.effect("minecraft:speed"));

    let bare = |id: &&'static str| id.trim_start_matches("minecraft:").to_owned();
    let mut names: Vec<String> = ids
        .potion_effects
        .iter()
        .chain(ids.enchantments)
        .chain(ids.effects)
        .chain(ids.items.iter().take(50))
        .map(bare)
        .chain(
            ids.enchantment_aliases
                .iter()
                .flat_map(|(alias, target)| [bare(alias), bare(target)]),
        )
        .collect();
    let upper: Vec<String> = names.iter().map(|name| name.to_uppercase()).collect();
    names.extend(upper);
    names.extend(["", "minecraft:", "\u{212a}", "demo"].map(str::to_owned));
    for name in &names {
        assert_eq!(
            tables.potion(name).map(str::to_owned),
            potion_before(name),
            "{name}"
        );
        assert_eq!(
            tables.is_enchantment(name),
            is_enchantment_before(name),
            "{name}"
        );
        assert_eq!(tables.effect(name), effect_before(name), "{name}");
    }
}

/// A host that answers `contains` alone, and one that also says no potion is known.
struct ContainsOnly;

impl BuiltIns for ContainsOnly {
    fn contains(&self, _kind: &'static str, _id: &str) -> bool {
        false
    }
}

struct NoPotions;

impl BuiltIns for NoPotions {
    fn contains(&self, _kind: &'static str, _id: &str) -> bool {
        false
    }
    fn potion(&self, _name: &str) -> Option<&str> {
        None
    }
    fn level_sound_event(&self, name: &str) -> Option<&str> {
        (name == "host.boom").then_some("host.boom")
    }
}

#[test]
fn a_host_that_answers_contains_keeps_the_tables_value_lookups() {
    let cx = context_with(Some(Arc::new(ContainsOnly)));
    let built_ins = cx.built_ins();
    assert!(!built_ins.contains("level sound event", "ambient"));
    assert_eq!(built_ins.level_sound_event("Ambient"), Some("ambient"));
    assert_eq!(built_ins.potion("awkward"), Some("minecraft:awkward"));
    assert!(built_ins.is_enchantment("sharpness"));
    assert!(built_ins.effect("speed"));
}

#[test]
fn a_host_value_lookup_is_believed() {
    let cx = context_with(Some(Arc::new(NoPotions)));
    assert_eq!(cx.built_ins().potion("awkward"), None);
    assert_eq!(cx.level_sound_event("ambient"), None);
    assert_eq!(
        cx.level_sound_event("host.boom"),
        Some(LevelSoundEventName::parse("host.boom"))
    );
}

#[test]
fn the_context_names_a_level_sound_event_in_the_table_s_spelling() {
    let cx = LoadContext::detached();
    let name: Option<LevelSoundEventName> = cx.level_sound_event("Item.Shield.Block");
    assert_eq!(
        name.as_ref().map(LevelSoundEventName::as_str),
        Some("item.shield.block")
    );
    assert_eq!(cx.level_sound_event("custom.boom"), None);
}
