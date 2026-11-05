//! The biomes `overworld_generation_rules` turns a biome into and the biomes `replace_biomes`
//! replaces. The blocks of a biome are in `biome_block`.
//!
//! The game does not register a biome with such a miss, so each is an error. The pass tells
//! it and leaves the registry as it was loaded.

use super::super::{
    At, Findings, LinkCx, Target, Validator, invalid, kinds, unresolved, validator_miss,
};
use crate::diagnostic::{DiagnosticCode, JsonPath, Severity, VanillaText};
use crate::document::{ComponentMap, ComponentRef, Origin};
use crate::families::biome::component::{
    BiomeComponent, NameOrWeight, OverworldGenerationRules, ReplaceBiomes, WeightedBiome,
    WeightedBiomes,
};
use crate::id::BiomeId;
use crate::pack::PackSide;

/// vanilla's text for a transformation that names no biome.
const NO_BIOME: &str = "No biome found with name '{}'";

/// vanilla's text for a replacement target that names no biome.
const NO_TARGET: &str = "No biome found with name: '{}'";

/// vanilla's text for a replacement target that is no biome of the game.
const NOT_VANILLA: &str = "Must replace vanilla biomes only. Attempting to use biome: '{}'";

/// `template` with `name`.
fn text(template: &'static str, name: &str) -> VanillaText {
    VanillaText::Formatted {
        template,
        args: vec![name.to_owned()],
    }
}

/// The rules of every effective biome, the stable documents then the beta ones.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    custom_biome_count(cx, out, REGISTERABLE);
    let biomes = cx.behavior().biomes();
    for (_, document) in biomes {
        components(cx, out, document.origin(), &document.value().components);
    }
    for (_, document) in biomes.beta().into_iter().flatten() {
        components(cx, out, document.origin(), &document.value().components);
    }
}

/// The rules of one biome's components. A component an upgrade inserted is not in the file
/// and names nothing the author wrote.
fn components(
    cx: &LinkCx<'_>,
    out: &mut Findings,
    origin: &Origin,
    components: &ComponentMap<BiomeComponent>,
) {
    for entry in components.iter() {
        let ComponentRef::Typed {
            name,
            value,
            synthesized: false,
        } = entry
        else {
            continue;
        };
        let mut at = JsonPath::new();
        for key in ["minecraft:biome", "components", name] {
            at.push(key);
        }
        match value {
            BiomeComponent::OverworldGenerationRules(rules) => {
                transformations(cx, out, origin, &at, rules);
            }
            BiomeComponent::ReplaceBiomes(replace) => targets(cx, out, origin, &at, replace),
            _ => {}
        }
        #[cfg(feature = "block")]
        super::biome_block::component_blocks(cx, out, origin, &at, value);
    }
}

/// Each biome a transformation names (`"<name>"`, an entry `"<name>"` of a list, the
/// name of a `[name, weight]` entry) must be a biome of the stack or of the game; else an
/// `UnresolvedReference` error at the name with vanilla's text.
fn transformations(
    cx: &LinkCx<'_>,
    out: &mut Findings,
    origin: &Origin,
    at: &JsonPath,
    rules: &OverworldGenerationRules,
) {
    let mut check = |path: JsonPath, name: &str| {
        if let Target::Missing(hint) = cx.biome(&BiomeId::parse(name)) {
            // The linked-asset check always raises this finding to an error, so it is written as
            // the error that comes out.
            let line = At::new(origin, path.clone(), Severity::Error);
            let at = At::new(origin, path, Severity::Error).with_text(text(NO_BIOME, name));
            out.push(unresolved(at, kinds::BIOME, name, hint));
            out.push(validator_miss(
                line,
                Validator::Missing,
                1,
                kinds::BIOME,
                name,
            ));
        }
    };
    for (key, transformation) in [
        ("hills_transformation", &rules.hills_transformation),
        ("mutate_transformation", &rules.mutate_transformation),
        ("river_transformation", &rules.river_transformation),
        ("shore_transformation", &rules.shore_transformation),
    ] {
        let member = at.child(key);
        match transformation {
            None => {}
            Some(WeightedBiomes::One(name)) => check(member, name),
            Some(WeightedBiomes::List(entries)) => {
                for (index, entry) in entries.iter().enumerate() {
                    let path = member.index(index);
                    match entry {
                        WeightedBiome::Name(name) => check(path, name),
                        WeightedBiome::Pair(pair) => {
                            for (position, element) in pair.iter().enumerate() {
                                if let NameOrWeight::Name(name) = element {
                                    let path = path.index(position);
                                    check(path, name);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Each target of a replacement must be a biome of the game. One that names no biome is
/// an `UnresolvedReference` error, one that names a biome a pack of the world adds an
/// `InvalidReference` error, each at the target with vanilla's text.
///
/// A target is compared as a biome name is: a name without a namespace is of the `minecraft`
/// namespace, so `plains` and `minecraft:plains` are one biome and neither spelling is a
/// finding.
fn targets(
    cx: &LinkCx<'_>,
    out: &mut Findings,
    origin: &Origin,
    at: &JsonPath,
    replace: &ReplaceBiomes,
) {
    for (index, replacement) in replace.replacements.iter().enumerate() {
        for (position, target) in replacement.targets.iter().enumerate() {
            let name = target.0.as_str();
            let id = BiomeId::parse(name);
            let mut path = at.clone();
            path.push("replacements");
            path.push_index(index);
            path.push("targets");
            path.push_index(position);
            match cx.biome(&id) {
                Target::Missing(hint) => {
                    // The linked-asset check raises this finding to an error; it is written as the
                    // error that comes out.
                    let line = At::new(origin, path.clone(), Severity::Error);
                    let at =
                        At::new(origin, path, Severity::Error).with_text(text(NO_TARGET, name));
                    out.push(unresolved(at, kinds::BIOME, name, hint));
                    out.push(validator_miss(
                        line,
                        Validator::Missing,
                        1,
                        kinds::BIOME,
                        name,
                    ));
                }
                Target::Defined(definition) if !of_the_game(cx, &id, definition) => {
                    let at =
                        At::new(origin, path, Severity::Error).with_text(text(NOT_VANILLA, name));
                    out.push(invalid(at, kinds::BIOME, name));
                }
                _ => {}
            }
        }
    }
}

/// The game hands out custom biome ids from 30 000 and refuses at `0xffff`, so 35 535 custom
/// biomes get an id.
const REGISTERABLE: usize = 0xffff - 30_000;

/// The count the game's text prints: `MAX_CUSTOM_BIOMES`, 35 534.
const MAX_CUSTOM_BIOMES: u16 = 35_534;

/// vanilla's text for a custom biome past the limit (logged at the error level).
const COUNT_EXCEEDS: &str =
    "Custom biome count exceeds limit of {}: {} will not generate correctly";

/// The custom biomes (the biomes that are no biome of the game, each registered once, in the
/// order of the registry, the stable documents then the beta ones) past the first `limit`:
/// each an `InvalidValue` error at its identifier with vanilla's text. The game logs it when
/// it registers the biome and gives it no id; nothing here changes the registry.
fn custom_biome_count(cx: &LinkCx<'_>, out: &mut Findings, limit: usize) {
    let biomes = cx.behavior().biomes();
    let stable = biomes
        .iter()
        .map(|(_, document)| (&document.value().description.identifier, document.origin()));
    let beta = biomes
        .beta()
        .into_iter()
        .flatten()
        .map(|(_, document)| (&document.value().description.identifier, document.origin()));
    let custom = stable
        .chain(beta)
        .filter(|(id, origin)| !of_the_game(cx, id, origin));
    for (id, origin) in custom.skip(limit) {
        let text = VanillaText::Formatted {
            template: COUNT_EXCEEDS,
            args: vec![MAX_CUSTOM_BIOMES.to_string(), id.as_str().to_owned()],
        };
        let mut path = JsonPath::new();
        for key in ["minecraft:biome", "description", "identifier"] {
            path.push(key);
        }
        out.push(
            At::new(origin, path, Severity::Error)
                .diagnostic(DiagnosticCode::InvalidValue { text: Some(text) }),
        );
    }
}

/// Whether the biome `id`, which the stack defines at `definition`, is a biome of the game:
/// one the game has by itself (a pack that redefines it changes that not), or one a pack the
/// game ships defines.
fn of_the_game(cx: &LinkCx<'_>, id: &BiomeId, definition: &Origin) -> bool {
    cx.built_in(cx.biome_names(), id)
        || definition.pack.is_app_data()
        || cx
            .stack_pack(PackSide::Behavior, &definition.pack)
            .is_some_and(|pack| pack.context.is_base_game_pack)
}

#[cfg(test)]
mod tests {
    use bedrock_addon_core::load::LoadOptions;

    use super::super::super::tests::support::{File, pack, set_of};
    use super::super::super::{LinkOptions, link_with_rules};
    use super::*;

    /// A biome document of `id`.
    fn biome(id: &str) -> String {
        format!(
            r#"{{"format_version": "1.26.30", "minecraft:biome": {{"description": {{"identifier": "{id}"}}, "components": {{}}}}}}"#
        )
    }

    #[test]
    fn the_custom_ids_run_from_30000_below_0xffff_and_the_text_prints_the_limit() {
        assert_eq!(REGISTERABLE, 35_535);
        assert_eq!(MAX_CUSTOM_BIOMES, 35_534);
    }

    /// The count with a limit of two.
    fn at_two(cx: &LinkCx<'_>, out: &mut Findings) {
        custom_biome_count(cx, out, 2);
    }

    #[test]
    fn a_custom_biome_past_the_limit_is_an_error_at_its_identifier_with_the_game_text() {
        let files = [
            ("biomes/a.json", biome("demo:a")),
            // A biome of the game (redefined) takes its fixed id and counts not.
            ("biomes/plains.json", biome("minecraft:plains")),
            ("biomes/b.json", biome("demo:b")),
            ("biomes/c.json", biome("demo:c")),
        ];
        let files: Vec<File<'_>> = files
            .iter()
            .map(|(path, text)| (*path, Some(text.as_str())))
            .collect();
        let set = set_of(
            vec![pack(1, "data", &files)],
            Vec::new(),
            &LoadOptions::default(),
        );
        let found = link_with_rules(&set, LinkOptions::default(), &[at_two]);
        let found: Vec<_> = found
            .iter()
            .map(|f| {
                (
                    f.code.name(),
                    f.severity,
                    f.location.path.as_ref().map(ToString::to_string),
                    f.location.json_path.to_string(),
                    f.code.to_string(),
                )
            })
            .collect();
        assert_eq!(
            found,
            [(
                "InvalidValue",
                Severity::Error,
                Some("biomes/c.json".to_owned()),
                "minecraft:biome/description/identifier".to_owned(),
                "Custom biome count exceeds limit of 35534: demo:c will not generate correctly"
                    .to_owned()
            )]
        );
        // Below the limit, nothing.
        let all = link_with_rules(&set, LinkOptions::default(), &[run]);
        assert!(
            all.iter()
                .all(|f| !f.code.to_string().starts_with("Custom biome count")),
            "{all:#?}"
        );
    }
}
