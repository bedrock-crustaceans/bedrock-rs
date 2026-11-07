//! An integer too large for 64 bits is the double it spells, in every family (core's door):
//! Vanilla's readers read it so. It is never a string (a texture name, a sound
//! name, a string written back in quotes), and a file that has one loads, whatever its size:
//! the family finds there what it finds for the real number of that value.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use bedrock_addon::diagnostic::{Diagnostic, Kind, Severity};
use bedrock_addon::document::Content;
use bedrock_addon::load::LoadContext;
use bedrock_addon::storage::PackPath;

/// What a family made of one text.
struct Outcome {
    loaded: bool,
    /// `Debug` of the value.
    value: String,
    /// The value written back, where the family writes it.
    written: Option<String>,
    findings: Vec<Diagnostic>,
}

impl Outcome {
    /// The findings without their spans and source text: which finding, how severe, where.
    fn kinds(&self) -> Vec<String> {
        self.findings
            .iter()
            .map(|d| {
                format!(
                    "{} {:?} at `{}`",
                    d.code.id(),
                    d.severity,
                    d.location.json_path
                )
            })
            .collect()
    }

    fn errors(&self) -> Vec<&Diagnostic> {
        self.findings
            .iter()
            .filter(|d| d.severity >= Severity::Error)
            .collect()
    }
}

#[allow(dead_code, reason = "used by the lines of the compiled families")]
fn parse<T: Content + std::fmt::Debug>(path: &'static str, text: &str) -> Outcome {
    let cx = LoadContext::detached();
    match T::parse(&PackPath::from_static(path), text.as_bytes(), &cx) {
        Ok(document) => Outcome {
            loaded: true,
            value: format!("{:?}", document.value()),
            written: document.to_json().ok(),
            findings: document.diagnostics().iter().cloned().collect(),
        },
        Err(rejected) => Outcome {
            loaded: false,
            value: String::new(),
            written: None,
            findings: rejected.diagnostics.iter().cloned().collect(),
        },
    }
}

fn manifest(text: &str) -> Outcome {
    use bedrock_addon::manifest::{CapabilityRegistry, Manifest};
    let registry = CapabilityRegistry::default();
    match Manifest::parse(
        &PackPath::from_static("manifest.json"),
        text.as_bytes(),
        &registry,
    ) {
        Ok(document) => Outcome {
            loaded: true,
            value: format!("{:?}", document.value()),
            written: None,
            findings: document.diagnostics().iter().cloned().collect(),
        },
        Err(rejected) => Outcome {
            loaded: false,
            value: String::new(),
            written: None,
            findings: rejected.diagnostics.iter().cloned().collect(),
        },
    }
}

/// A reader of one family's file, by name.
type Reader = (&'static str, fn(&str) -> Outcome);

macro_rules! readers {
    ($( $(#[$cfg:meta])? $name:literal => $type:ty, $path:literal; )*) => {
        #[allow(unused_mut, reason = "without a content feature only the manifest is read")]
        fn readers() -> Vec<Reader> {
            let mut out: Vec<Reader> = vec![("manifest", manifest)];
            $(
                $(#[$cfg])?
                out.push(($name, |text| parse::<$type>($path, text)));
            )*
            out
        }
    };
}

readers! {
    #[cfg(feature = "block")] "block" => bedrock_addon::families::block::Block, "blocks/x.json";
    #[cfg(feature = "item")] "item" => bedrock_addon::families::item::Item, "items/x.json";
    #[cfg(feature = "crafting_catalog")] "crafting catalog" => bedrock_addon::families::crafting_catalog::CraftingCatalog, "item_catalog/crafting_item_catalog.json";
    #[cfg(feature = "entity")] "entity" => bedrock_addon::families::entity::Entity, "entities/x.json";
    #[cfg(feature = "biome")] "biome" => bedrock_addon::families::biome::Biome, "biomes/x.json";
    #[cfg(feature = "loot")] "loot table" => bedrock_addon::families::loot::LootTable, "loot_tables/x.json";
    #[cfg(feature = "recipe")] "recipe" => bedrock_addon::families::recipe::Recipe, "recipes/x.json";
    #[cfg(feature = "dialogue")] "dialogue" => bedrock_addon::families::dialogue::DialogueFile, "dialogue/x.json";
    #[cfg(feature = "geometry")] "geometry" => bedrock_addon::families::geometry::GeometryFile, "models/entity/x.json";
    #[cfg(feature = "fog")] "fog" => bedrock_addon::families::fog::FogSettings, "fogs/x.json";
    #[cfg(feature = "texture_atlas")] "terrain_texture" => bedrock_addon::families::texture_atlas::TextureAtlas, "textures/terrain_texture.json";
    #[cfg(feature = "flipbooks")] "flipbook_textures" => bedrock_addon::families::flipbooks::Flipbooks, "textures/flipbook_textures.json";
    #[cfg(feature = "function")] "tick list" => bedrock_addon::families::function::TickList, "functions/tick.json";
    #[cfg(feature = "json_ui")] "ui file" => bedrock_addon::families::json_ui::UiFile, "ui/x.json";
    #[cfg(feature = "behavior_tree")] "behavior tree" => bedrock_addon::families::behavior_tree::BehaviorTree, "behavior_trees/x.json";
}

/// A document each family loads, with `@` where a number may stand: in a member the family
/// does not read, and in members it reads as a number, a string or an integer.
fn documents() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "block",
            r#"{"format_version": "1.21.60", "minecraft:block": {"description": {"identifier": "demo:x"}, "unread": [@, {"deep": @}]}}"#,
        ),
        (
            "block",
            r#"{"format_version": "1.21.60", "minecraft:block": {"description": {"identifier": "demo:x"}, "components": {"minecraft:friction": @, "minecraft:light_emission": @, "minecraft:display_name": @}}}"#,
        ),
        (
            "block",
            r#"{"format_version": "1.21.60", "unread": @, "minecraft:block": {"description": {"identifier": "demo:x"}}}"#,
        ),
        (
            "item",
            r#"{"format_version": "1.21.60", "minecraft:item": {"description": {"identifier": "demo:x"}, "components": {"minecraft:max_stack_size": @, "minecraft:icon": @}, "unread": @}}"#,
        ),
        (
            "entity",
            r#"{"format_version": "1.21.60", "minecraft:entity": {"description": {"identifier": "demo:x", "is_spawnable": true, "is_summonable": true}, "components": {"minecraft:health": {"value": @, "max": @}}, "unread": @}}"#,
        ),
        (
            "loot table",
            r#"{"pools": [{"rolls": @, "entries": [{"type": "item", "name": "minecraft:stone", "weight": @}]}], "unread": @}"#,
        ),
        (
            "recipe",
            r#"{"format_version": "1.12.0", "minecraft:recipe_shapeless": {"description": {"identifier": "demo:x"}, "tags": ["crafting_table"], "priority": @, "ingredients": ["minecraft:stone"], "result": {"item": "minecraft:stone", "count": @}}}"#,
        ),
        (
            "dialogue",
            r#"{"format_version": "1.17.0", "minecraft:npc_dialogue": {"scenes": [{"scene_tag": "x", "text": @, "unread": @}]}}"#,
        ),
        (
            "behavior tree",
            r#"{"minecraft:behavior_tree": {"minecraft:name": "x", "minecraft:root": {"minecraft:wait_ticks": {"num_of_ticks": @}}}}"#,
        ),
        ("tick list", r#"{"values": ["x", @], "unread": @}"#),
        (
            "crafting catalog",
            r#"{"format_version": "1.21.60", "minecraft:crafting_items_catalog": {"categories": [{"category_name": "construction", "groups": [{"group_identifier": {"name": "demo:x", "icon": @}, "items": [@]}]}]}}"#,
        ),
        (
            "geometry",
            r#"{"format_version": "1.21.0", "minecraft:geometry": [{"description": {"identifier": "geometry.x", "texture_width": @, "visible_bounds_width": @}, "bones": [{"name": "b", "pivot": [@, 0, 0]}]}]}"#,
        ),
        (
            "fog",
            r#"{"format_version": "1.16.100", "minecraft:fog_settings": {"description": {"identifier": "demo:x"}, "distance": {"air": {"fog_start": @, "fog_end": @, "fog_color": [1, 1, 1], "render_distance_type": "fixed"}}}}"#,
        ),
        (
            "ui file",
            r#"{"namespace": "demo", "panel": {"type": "panel", "layer": @, "size": [@, 5], "alpha": @, "unread": @}}"#,
        ),
        (
            "terrain_texture",
            r#"{"texture_name": "atlas.terrain", "padding": @, "num_mip_levels": @, "texture_data": {"dirt": {"textures": @}, "stone": {"textures": [@, {"path": @, "weight": @}]}}}"#,
        ),
        (
            "flipbook_textures",
            r#"[{"flipbook_texture": "textures/blocks/x", "atlas_tile": "x", "ticks_per_frame": @, "frames": [@, 1], "atlas_index": @}]"#,
        ),
        (
            "manifest",
            r#"{"format_version": 2, "header": {"name": "x", "uuid": "00000000-0000-0000-0000-000000000001", "version": [1, 0, @], "min_engine_version": [1, 21, 0], "unread": @}, "modules": [{"type": "data", "uuid": "00000000-0000-0000-0000-000000000002", "version": [1, 0, 0]}], "unread": @}"#,
        ),
        (
            "manifest",
            r#"{"format_version": @, "header": {"name": @, "uuid": "00000000-0000-0000-0000-000000000001", "version": [1, 0, 0], "min_engine_version": [1, 21, 0]}, "modules": [{"type": "data", "uuid": "00000000-0000-0000-0000-000000000002", "version": [1, 0, 0]}]}"#,
        ),
    ]
}

/// The integers too large for 64 bits, by size class, with the real number each spells.
fn wide() -> Vec<(String, String)> {
    let digits = |n: usize| "1234567890".chars().cycle().take(n).collect::<String>();
    [
        "18446744073709551616".to_owned(),
        "-9223372036854775809".to_owned(),
        digits(30),
        digits(39),
        digits(40),
        format!("-{}", digits(100)),
        digits(400),
    ]
    .into_iter()
    .map(|token| {
        let value: f64 = token.parse().unwrap();
        let real = if value.is_infinite() {
            "1e999".to_owned()
        } else {
            format!("{value:?}")
        };
        (token, real)
    })
    .collect()
}

/// In every family, at every place a number can stand, a wide integer is what the real number
/// it spells is: the file loads or not as with the real, to the same value, with the same
/// findings at the same members.
#[test]
fn a_wide_integer_is_in_every_family_the_real_number_it_spells() {
    let readers = readers();
    let (mut checked, mut loaded) = (0, 0);
    let mut families = std::collections::BTreeSet::new();
    for (name, document) in documents() {
        let Some((_, read)) = readers.iter().find(|(reader, _)| *reader == name) else {
            continue; // the family is not compiled in
        };
        families.insert(name);
        let places = document.matches('@').count();
        for (token, real) in wide() {
            for place in 0..places {
                let with = |number: &str| {
                    let mut seen = 0;
                    let mut text = String::new();
                    for part in document.split_inclusive('@') {
                        match part.strip_suffix('@') {
                            Some(before) => {
                                text.push_str(before);
                                text.push_str(if seen == place { number } else { "1" });
                                seen += 1;
                            }
                            None => text.push_str(part),
                        }
                    }
                    text
                };
                let ours = read(&with(&token));
                let reference = read(&with(&real));
                let shown = format!("{name}, `{token}` at place {place} of {document}");
                assert_eq!(
                    ours.loaded, reference.loaded,
                    "{shown}: {:?}",
                    ours.findings
                );
                assert_eq!(ours.kinds(), reference.kinds(), "{shown}");
                // A family that keeps the file's text keeps the author's digits in it.
                let respelt = |text: &str| text.replace(token.as_str(), &real);
                assert_eq!(respelt(&ours.value), reference.value, "{shown}");
                assert_eq!(
                    ours.written.as_deref().map(respelt),
                    reference.written,
                    "{shown}"
                );
                // Never a syntax error of the file, and never the digits as a string.
                // (A `MinecraftDocumentInput` refuses a number no double holds with the game's JSON
                // reader's own message: the same for the wide integer and for the real.)
                assert!(
                    !ours.findings.iter().any(|d| d.location.json_path.is_empty()
                        && d.severity >= Severity::Error
                        && d.code.kind() == Kind::Syntax
                        && {
                            let told = format!(
                                "{} {}",
                                bedrock_addon::diagnostic::testing::shown(d),
                                d.code.vanilla().unwrap_or_default()
                            );
                            !told.contains("Number too big")
                                && !told.contains("the item is not loaded")
                        }),
                    "{shown}: {:?}",
                    ours.findings
                );
                let quoted = format!("\"{token}\"");
                assert!(!ours.value.contains(&quoted), "{shown}: {}", ours.value);
                if let Some(written) = &ours.written {
                    assert!(!written.contains(&quoted), "{shown}: {written}");
                    // What is written is a file the family reads.
                    assert!(read(written).loaded, "{shown}: {written}");
                }
                checked += 1;
                loaded += usize::from(ours.loaded);
            }
        }
    }
    assert!(
        checked >= 14 && loaded >= 7,
        "{checked} checked, {loaded} loaded"
    );
    // With every content feature on, every family of the list was read.
    if cfg!(all(
        feature = "block",
        feature = "item",
        feature = "entity",
        feature = "loot",
        feature = "recipe",
        feature = "dialogue",
        feature = "behavior_tree",
        feature = "function",
        feature = "geometry",
        feature = "render_controller",
        feature = "fog",
        feature = "material",
        feature = "deferred_settings",
        feature = "json_ui",
        feature = "ddui",
        feature = "font",
        feature = "splashes",
        feature = "loading_messages",
        feature = "credits",
        feature = "ui_texture_info",
        feature = "texture_atlas",
        feature = "flipbooks"
    )) {
        assert_eq!(families.len(), 15, "{families:?}");
        assert!(
            checked >= 300 && loaded >= 150,
            "{checked} checked, {loaded} loaded"
        );
    }
}

/// The reviewers' cases. A member the block does not read keeps its number: written back as
/// the shortest decimal of the double, not as a string.
#[cfg(feature = "block")]
#[test]
fn a_block_keeps_a_wide_integer_in_an_unread_member_as_a_number() {
    let text = r#"{"format_version": "1.21.60", "minecraft:block": {"description": {"identifier": "demo:x"}, "unread": [123456789012345678901234567890], "huge": 1234567890123456789012345678901234567890123}}"#;
    let block = parse::<bedrock_addon::families::block::Block>("blocks/x.json", text);
    assert!(block.loaded, "{:?}", block.findings);
    assert!(block.errors().is_empty(), "{:?}", block.findings);
    let written = block.written.expect("the block is written");
    assert!(written.contains("1.2345678901234568e29"), "{written}");
    assert!(written.contains("1.2345678901234567e42"), "{written}");
    assert!(!written.contains("\"1234"), "{written}");
    let again = parse::<bedrock_addon::families::block::Block>("blocks/x.json", &written);
    // The written text is read again to the same numbers and written the same.
    assert!(again.loaded);
    assert!(again.errors().is_empty(), "{:?}", again.findings);
    assert_eq!(again.written.as_deref(), Some(written.as_str()));
}

/// `"textures": 18446744073709551616` names no texture.
#[cfg(feature = "texture_atlas")]
#[test]
fn a_wide_integer_is_no_texture_name() {
    for token in [
        "18446744073709551616",
        "1234567890123456789012345678901234567890123",
    ] {
        let text = format!(
            r#"{{"texture_name": "atlas.terrain", "texture_data": {{"dirt": {{"textures": {token}}}, "stone": {{"textures": "textures/blocks/stone"}}}}}}"#
        );
        let atlas = parse::<bedrock_addon::families::texture_atlas::TextureAtlas>(
            "textures/terrain_texture.json",
            &text,
        );
        assert!(atlas.loaded, "{:?}", atlas.findings);
        assert!(!atlas.value.contains(token), "{}", atlas.value);
        assert!(
            atlas.value.contains("textures/blocks/stone"),
            "{}",
            atlas.value
        );
    }
}

/// A UI file and a manifest with an integer of forty digits load.
#[test]
fn a_file_with_an_integer_of_forty_digits_loads() {
    let big = "1234567890".repeat(4);
    let outcome = manifest(&format!(
        r#"{{"format_version": 2, "header": {{"name": "x", "uuid": "00000000-0000-0000-0000-000000000001", "version": [1, 0, 0], "min_engine_version": [1, 21, 0]}}, "modules": [{{"type": "data", "uuid": "00000000-0000-0000-0000-000000000002", "version": [1, 0, 0]}}], "metadata": {{"generated_with": {{"tool": ["1.0.0"]}}, "build": {big}}}}}"#
    ));
    assert!(outcome.loaded, "{:?}", outcome.findings);
    assert!(outcome.errors().is_empty(), "{:?}", outcome.findings);
    #[cfg(feature = "json_ui")]
    {
        let ui = parse::<bedrock_addon::families::json_ui::UiFile>(
            "ui/x.json",
            &format!(
                r#"{{"namespace": "demo", "panel": {{"type": "panel", "alpha": {big}, "$var": {big}}}}}"#
            ),
        );
        assert!(ui.loaded, "{:?}", ui.findings);
        assert!(!ui.value.contains(&format!("\"{big}\"")), "{}", ui.value);
    }
}
