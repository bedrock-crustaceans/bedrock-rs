//! The `blocks.json` entries against the blocks: what `blocks.json` says of blocks the behaviour
//! stack defines or the game has.

use bedrock_addon_core::diagnostic::builders;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, Severity};
use bedrock_addon_core::document::Origin;
use bedrock_addon_core::id::BlockId;
use bedrock_addon_core::load::Experiment;
use bedrock_addon_core::pack_identity::PackSide;
use indexmap::IndexMap;

use super::super::{At, Findings, LinkCx, Target, kinds, unresolved};
use super::blocks_json::{BLOCKS_JSON, Writers, creator_origin, entry_key, entry_path};
use super::told::{Told, formatted};
use crate::families::block::component::{Geometry, MaterialInstances};
use crate::families::blocks_json::entry_settings;

/// The text for a key that names no block: the key.
const NO_SUCH_BLOCK: &str =
    "The block named {} used in a \"blocks.json\" file does not exist in the registry";
/// The text of the three block-graphics lines below: the block's full name.
const EXPONENT_UNUSED: &str = "Block {} has an \"ambient_occlusion_exponent\" in 'blocks.json' file, this value is unused because the value in \"minecraft:material_instances\" component is used instead";
const CUSTOM_BLOCK_OVERRIDE: &str = "{}: trying to override the Geometry component with blocks.json settings for a custom block. This isn't supported. Please remove any legacy texture definition or block shape specification for this block.";
const TEXTURES_MISSING: &str = "{}: textures are missing to be able to override Geometry component";
/// Every template of this module with the number of its arguments, for the arity test.
#[cfg(test)]
const TEMPLATES: &[(&str, usize)] = &[
    (NO_SUCH_BLOCK, 1),
    (EXPONENT_UNUSED, 1),
    (CUSTOM_BLOCK_OVERRIDE, 1),
    (TEXTURES_MISSING, 1),
];

/// A `blocks.json` key names a block: a block of the behaviour stack, one of the game's, or a
/// name the client still resolves to one of the game's
/// ([`LEGACY_BLOCK_NAMES`](crate::vanilla::names::LEGACY_BLOCK_NAMES), with any host). Any other key is an
/// `UnresolvedReference` warning with vanilla's text, at the key: the client skips the entry.
///
/// The key was normalised when the file was read: a bare key is `minecraft:` and the key in
/// lower case, a key in the `minecraft` namespace is lower-cased whole, a key of another
/// namespace is compared as written.
pub(super) fn keys(cx: &LinkCx<'_>, out: &mut Findings) {
    let mut told = Told::default();
    for (id, document) in cx.resource().client_blocks() {
        let Target::Missing(hint) = cx.block(id) else {
            continue;
        };
        let origin = document.origin();
        let key = entry_key(id, origin);
        if !told.first(origin, key, "", "") {
            continue;
        }
        // The client logs the key at the warning level and skips the entry: a warning.
        let at = At::new(origin, entry_path(key, None), Severity::Warning)
            .with_text(formatted(NO_SUCH_BLOCK, &[key]));
        out.push(unresolved(at, kinds::BLOCK, key, hint));
    }
}

/// A block with `minecraft:material_instances` among its base components takes its ambient
/// occlusion from the component: an `ambient_occlusion_exponent` other than 1.0 that the
/// merged `blocks.json` gives it is unused. `NoEffect` warning with vanilla's text, at the
/// member of the entry of the pack that wrote it.
pub(super) fn exponents(cx: &LinkCx<'_>, out: &mut Findings) {
    let client = cx.resource().client_blocks();
    let mut writers = Writers::default();
    for (id, document) in cx.behavior().blocks() {
        if !document.value().components.contains::<MaterialInstances>() {
            continue;
        }
        let Some(entry) = client.get(id) else {
            continue;
        };
        // The client compares with its default, 1.0, exactly.
        if entry.value().ambient_occlusion_exponent().to_bits() == 1.0_f32.to_bits() {
            continue;
        }
        let member = "ambient_occlusion_exponent";
        let origin = writers.of(cx, out, id, member, entry.origin());
        let path = entry_path(entry_key(id, &origin), Some(member));
        let code = builders::worded(
            &core_codes::NO_EFFECT,
            &core_codes::NO_EFFECT_WORDED,
            Some(formatted(EXPONENT_UNUSED, &[id.as_str()]).as_str()),
        );
        // The client logs this at the warning level: a warning.
        out.push(At::new(&origin, path, Severity::Warning).diagnostic(code));
    }
}

/// What the packs' `blocks.json` files say of one block's drawing members.
#[derive(Debug)]
struct Drawing {
    /// Some pack's entry has `textures`.
    textures: bool,
    /// The entry of the highest pack that is not the base game's and has one of the four
    /// members: where the finding is told.
    creator: Option<Origin>,
}

/// Whether Upcoming Creator Features is on for the stacks.
fn upcoming_creator_features(cx: &LinkCx<'_>) -> bool {
    let options = cx.resource().options();
    options.upcoming_creator_features
        || options
            .experiments
            .contains(Experiment::UpcomingCreatorFeatures)
}

/// With Upcoming Creator Features, a block drawn from its `minecraft:geometry` whose
/// `blocks.json` entry in a pack that is not the base game's sets `textures`, `blockshape`,
/// `ambient_occlusion_exponent` or `isotropic`:
///
/// - a custom block: the settings are not supported (`NoEffect` warning, vanilla's text);
/// - a block of the `minecraft` namespace with `textures` in some pack: the client draws it
///   the old way, from `blocks.json` (a lint: `ShadowedDefinition` information);
/// - one without `textures` in any pack: the settings are not applied (`NoEffect` warning,
///   vanilla's text).
///
/// Each is told at the entry of the highest such pack. A block counts as drawn from its
/// geometry when a behaviour pack of the stack defines it with `minecraft:geometry` among its
/// base components; which of the game's own blocks are is not known here, and nothing is said
/// of them. Without the experiment nothing is said at all.
pub(super) fn tessellation(cx: &LinkCx<'_>, out: &mut Findings) {
    if !upcoming_creator_features(cx) {
        return;
    }
    let blocks = cx.behavior().blocks();
    if blocks.iter().next().is_none() {
        return;
    }
    // The stack's packs, the top one first: the first creator entry of a block is the highest.
    let mut drawn: IndexMap<BlockId, Drawing> = IndexMap::new();
    for pack in cx.resource().stack_packs() {
        for file in pack.files() {
            if file.key.as_str() != BLOCKS_JSON {
                continue;
            }
            let Some(bytes) = cx.bytes(PackSide::Resource, &pack.pack, &file.path, out) else {
                continue;
            };
            let base_game = pack.context.is_base_game_pack;
            for entry in entry_settings(&bytes, base_game) {
                let drawing = drawn.entry(entry.id).or_insert(Drawing {
                    textures: false,
                    creator: None,
                });
                drawing.textures |= entry.textures;
                if !base_game && drawing.creator.is_none() {
                    drawing.creator = Some(creator_origin(pack, &file.path, entry.key));
                }
            }
        }
    }
    for (id, drawing) in &drawn {
        let Some(origin) = &drawing.creator else {
            continue;
        };
        let Some(block) = blocks.get(id) else {
            continue;
        };
        if !block.value().components.contains::<Geometry>() {
            continue;
        }
        let path = entry_path(entry_key(id, origin), None);
        let custom = id
            .namespace()
            .is_none_or(|namespace| namespace != "minecraft");
        if custom || !drawing.textures {
            // Both lines come from the geometry fallback, after which the block keeps its
            // geometry: the first is logged at the warning level, the second at the error
            // level, so it is a kept error.
            let (template, severity) = if custom {
                (CUSTOM_BLOCK_OVERRIDE, Severity::Warning)
            } else {
                (TEXTURES_MISSING, Severity::Error)
            };
            let code = builders::worded(
                &core_codes::NO_EFFECT,
                &core_codes::NO_EFFECT_WORDED,
                Some(formatted(template, &[id.as_str()]).as_str()),
            );
            let finding = At::new(origin, path, severity).diagnostic(code);
            out.push(if custom { finding } else { finding.into_kept() });
        } else if cx.lints() {
            let code = Code::from(&core_codes::SHADOWED_DEFINITION);
            out.push(At::new(origin, path, Severity::Info).diagnostic(code));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Write as _;

    use bedrock_addon_core::diagnostic::{Diagnostics, Severity};
    use bedrock_addon_core::load::LoadOptions;
    use bedrock_addon_core::pack_identity::PackOrigin;

    use super::super::super::tests::support::{
        all_said, block, hosted, leak, pack, set, set_of, set_with,
    };
    use super::super::super::{LinkOptions, link_with_rules};
    use super::super::told::formatted;
    use super::{TEMPLATES, keys, tessellation};
    use crate::pack::tests::pack_from;

    #[test]
    fn every_template_takes_the_arguments_it_is_given() {
        for (template, arity) in TEMPLATES {
            assert_eq!(template.matches("{}").count(), *arity, "{template}");
            let args = vec!["x"; *arity];
            assert!(!formatted(template, &args).contains("{}"));
        }
    }

    /// `n` keys, one in four of them of no block: each costs the two lookups of the rule
    /// (the aliases, then the blocks), a missing one its near miss too.
    #[test]
    fn n_keys_cost_n() {
        fn text(text: &str) -> &'static str {
            std::str::from_utf8(leak(text)).unwrap_or_default()
        }
        let input = |n: usize| {
            let mut file = String::from(r#"{"format_version": "1.21.40""#);
            let mut behavior: Vec<(&'static str, &'static str)> = Vec::new();
            for index in 0..n {
                if index % 4 == 0 {
                    // Missing, with a block that differs in case only to find.
                    let _ = write!(file, r#", "demo:Lamp_{}": {{}}"#, index + 1);
                } else {
                    let _ = write!(file, r#", "demo:lamp_{index}": {{}}"#);
                    behavior.push((
                        text(&format!("blocks/lamp_{index}.json")),
                        text(&block(&format!("demo:lamp_{index}"), "{}")),
                    ));
                }
            }
            file.push('}');
            let world = set(&behavior, &[("blocks.json", file.as_str())]);
            // The collections are loaded before the clock starts: the rule is what is timed.
            assert_eq!(world.resource.client_blocks().iter().count(), n);
            assert_eq!(world.behavior.blocks().iter().count(), behavior.len());
            (world, n.div_ceil(4))
        };
        crate::test_cost::assert_linear_at(
            "link: n blocks.json keys",
            3_000,
            input,
            |(world, missing)| {
                let found = link_with_rules(world, LinkOptions::default(), &[keys]);
                assert_eq!(found.len(), *missing);
                found
            },
        );
    }

    /// A world whose base-game behaviour pack defines `minecraft:lamp` and `minecraft:vine`
    /// with a geometry, with the base game's `blocks.json` and a creator pack's above it.
    fn vanilla_world(base: &str, creator: &str, options: &LoadOptions) -> crate::pack::StackSet {
        fn manifest(n: u8, module: &str) -> String {
            format!(
                r#"{{ "format_version": 2,
  "header": {{ "name": "pack {n}", "description": "", "uuid": "00000000-0000-4000-8000-0000000000{n:02}",
              "version": [1, 0, 0], "min_engine_version": [1, 21, 0] }},
  "modules": [{{ "type": "{module}", "uuid": "00000000-0000-4000-9000-0000000000{n:02}", "version": [1, 0, 0] }}] }}"#
            )
        }
        let drawn = r#"{"minecraft:geometry": "minecraft:geometry.full_block", "minecraft:material_instances": {"*": {"texture": "lamp"}}}"#;
        let behavior = pack_from(
            &manifest(1, "data"),
            &[
                (
                    "blocks/lamp.json",
                    Some(leak(&block("minecraft:lamp", "{}").replace(
                        r#""components": {}"#,
                        &format!(r#""components": {drawn}"#),
                    ))),
                ),
                (
                    "blocks/vine.json",
                    Some(leak(&block("minecraft:vine", "{}").replace(
                        r#""components": {}"#,
                        &format!(r#""components": {drawn}"#),
                    ))),
                ),
            ],
            PackOrigin::Package,
        );
        let base = pack_from(
            &manifest(2, "resources"),
            &[("blocks.json", Some(leak(base)))],
            PackOrigin::Package,
        );
        let world = set_of(
            vec![behavior],
            vec![
                pack(3, "resources", &[("blocks.json", Some(creator))]),
                base,
            ],
            options,
        );
        let blocks: Vec<&str> = world
            .behavior
            .blocks()
            .iter()
            .map(|(id, _)| id.as_str())
            .collect();
        assert_eq!(
            blocks,
            ["minecraft:lamp", "minecraft:vine"],
            "the fixture loads"
        );
        world
    }

    fn creator_features() -> LoadOptions {
        LoadOptions {
            upcoming_creator_features: true,
            ..LoadOptions::default()
        }
    }

    /// What a finding says, with its pack and vanilla's text (`None` for a finding of the
    /// crate's own).
    fn told(found: &Diagnostics) -> Vec<(&'static str, Severity, String, String, Option<String>)> {
        found
            .iter()
            .map(|finding| {
                let text = matches!(
                    finding.code.id(),
                    "core/no_effect_worded" | "core/shadowed_definition"
                )
                .then(|| finding.vanilla())
                .flatten();
                (
                    finding.code.id(),
                    finding.severity,
                    finding
                        .location
                        .pack
                        .as_ref()
                        .map_or_else(String::new, |pack| pack.name().to_owned()),
                    finding.location.json_path.to_string(),
                    text,
                )
            })
            .collect()
    }

    const BASE: &str = r#"{"format_version": "1.21.40", "lamp": {"textures": "lamp", "sound": "glass"}, "vine": {"sound": "vine"}}"#;
    const CREATOR: &str = r#"{"format_version": "1.21.40", "lamp": {"isotropic": true}, "vine": {"ambient_occlusion_exponent": 0.5}}"#;

    #[test]
    fn a_block_of_the_game_drawn_from_its_geometry_with_creator_settings() {
        let world = vanilla_world(BASE, CREATOR, &creator_features());
        let found = link_with_rules(&world, LinkOptions::default(), &[tessellation]);
        assert_eq!(
            told(&found),
            [
                // `lamp` has textures in a pack (the base game's), so the client draws it the
                // old way: the crate's own lint, an information.
                (
                    "core/shadowed_definition",
                    Severity::Info,
                    "pack 3".to_owned(),
                    "lamp".to_owned(),
                    None
                ),
                // `vine` has textures in no pack; the line is at the error level and the block
                // keeps its geometry: a kept error.
                (
                    "core/no_effect_worded",
                    Severity::Error,
                    "pack 3".to_owned(),
                    "vine".to_owned(),
                    Some("minecraft:vine: textures are missing to be able to override Geometry component".to_owned())
                ),
            ]
        );
        // The shadowing is a lint; the missing textures are not.
        let found = link_with_rules(
            &world,
            LinkOptions {
                lints: false,
                ..Default::default()
            },
            &[tessellation],
        );
        let codes: Vec<&str> = found.iter().map(|finding| finding.code.id()).collect();
        assert_eq!(codes, ["core/no_effect_worded"]);
    }

    #[test]
    fn settings_of_a_pack_that_is_not_the_base_game_s_need_the_experiment() {
        // Without the experiment: nothing.
        let world = vanilla_world(BASE, CREATOR, &LoadOptions::default());
        let found = link_with_rules(&world, LinkOptions::default(), &[tessellation]);
        assert!(found.is_empty(), "{found:#?}");
        // The base game's own settings are no creator's: a creator pack with `sound` alone.
        let creator = r#"{"format_version": "1.21.40", "lamp": {"sound": "stone"}, "vine": {"sound": "stone"}}"#;
        let base = r#"{"format_version": "1.21.40", "lamp": {"textures": "lamp"}, "vine": {"isotropic": true}}"#;
        let world = vanilla_world(base, creator, &creator_features());
        let found = link_with_rules(&world, LinkOptions::default(), &[tessellation]);
        assert!(found.is_empty(), "{found:#?}");
        // A block of the game no behaviour pack defines: how it is drawn is not known.
        let creator = r#"{"format_version": "1.21.40", "dirt": {"isotropic": true}}"#;
        let world = vanilla_world(BASE, creator, &creator_features());
        let found = link_with_rules(&world, LinkOptions::default(), &[tessellation]);
        assert!(found.is_empty(), "{found:#?}");
    }

    #[test]
    fn a_file_that_is_not_resident_is_told_once_and_skipped() {
        let lamp = block("demo:lamp", "{}").replace(
            r#""components": {}"#,
            r#""components": {"minecraft:geometry": "minecraft:geometry.full_block", "minecraft:material_instances": {"*": {"texture": "lamp"}}}"#,
        );
        let world = set_of(
            vec![pack(
                1,
                "data",
                &[("blocks/lamp.json", Some(lamp.as_str()))],
            )],
            vec![pack(2, "resources", &[("blocks.json", None)])],
            &creator_features(),
        );
        let found = link_with_rules(&world, LinkOptions::default(), &[tessellation]);
        let codes: Vec<&str> = found.iter().map(|finding| finding.code.id()).collect();
        assert_eq!(codes, ["core/file_not_loaded"]);
    }

    #[test]
    fn a_key_split_onto_many_blocks_is_one_finding() {
        // Without a version the file is read as 1.1.0, where `wool` stands for sixteen
        // blocks; this host has none of them.
        let world = set_with(
            &[],
            &[("blocks.json", r#"{"wool": {"sound": "cloth"}}"#)],
            &hosted(&[]),
        );
        assert!(world.resource.client_blocks().iter().count() > 1);
        let found = link_with_rules(&world, LinkOptions::default(), &[keys]);
        assert_eq!(
            all_said(&found),
            [(
                "core/unresolved_reference_worded",
                Severity::Warning,
                "wool".to_owned(),
                "block",
                "wool".to_owned()
            )]
        );
        // The same file where the game has its blocks: nothing.
        let world = set(&[], &[("blocks.json", r#"{"wool": {"sound": "cloth"}}"#)]);
        let found = link_with_rules(&world, LinkOptions::default(), &[keys]);
        assert!(found.is_empty(), "{found:#?}");
    }
    /// The `blocks.json` of every resource pack directly below `packs`.
    #[allow(
        clippy::disallowed_methods,
        clippy::disallowed_types,
        reason = "a test reads the game's files"
    )]
    fn files_below(packs: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        let Ok(entries) = std::fs::read_dir(packs) else {
            return;
        };
        let mut found: Vec<std::path::PathBuf> = entries
            .flatten()
            .map(|entry| entry.path().join("blocks.json"))
            .filter(|file| file.is_file())
            .collect();
        found.sort();
        out.append(&mut found);
    }

    /// No key of the game's own `blocks.json` files misses its block, and the crate's one list
    /// of legacy block names is exactly what the game's files need: the identifiers those files
    /// give a block that the generated list lacks, and the names the game's entity files, spawn
    /// rules and features write.
    #[allow(
        clippy::disallowed_methods,
        clippy::disallowed_types,
        reason = "a test reads the game's files"
    )]
    #[test]
    fn the_legacy_block_names_are_exactly_what_the_vanilla_files_need() {
        use std::collections::BTreeSet;

        use super::super::super::tests::support::{vanilla_behavior_text, written_in};
        use crate::vanilla::names::LEGACY_BLOCK_NAMES;

        let Some(game_files) = std::env::var_os("BEDROCK_ADDON_CORPUS").filter(|v| !v.is_empty())
        else {
            eprintln!("skipped: BEDROCK_ADDON_CORPUS is not set");
            return;
        };
        let game_files = std::path::Path::new(&game_files);
        let mut files = Vec::new();
        files_below(
            &game_files.join("gdk-retail").join("resource_packs"),
            &mut files,
        );
        files_below(
            &game_files.join("gdk-dev").join("resource_packs"),
            &mut files,
        );
        let samples = game_files
            .join("bedrock-samples")
            .join("resource_pack")
            .join("blocks.json");
        assert!(samples.is_file(), "{}", samples.display());
        files.push(samples);
        assert!(files.len() > 60, "{} blocks.json files", files.len());
        let vanilla: BTreeSet<&str> = bedrock_addon_core::vanilla::ids()
            .blocks
            .iter()
            .copied()
            .collect();
        let mut needed = BTreeSet::new();
        let mut keys_read = 0usize;
        let mut missed: Vec<(String, Vec<String>)> = Vec::new();
        for file in &files {
            let text = std::fs::read_to_string(file)
                .unwrap_or_else(|error| panic!("{}: {error}", file.display()));
            let world = set_of(
                Vec::new(),
                vec![pack(
                    2,
                    "resources",
                    &[("blocks.json", Some(text.as_str()))],
                )],
                &LoadOptions::default(),
            );
            let blocks = world.resource.client_blocks();
            assert!(blocks.rejected().is_empty(), "{}", file.display());
            for (id, _) in blocks {
                keys_read += 1;
                if !vanilla.contains(id.as_str()) {
                    needed.insert(id.as_str().to_owned());
                }
            }
            // The rule itself, over the file: no key without its block.
            let found = link_with_rules(&world, LinkOptions::default(), &[keys]);
            let told: Vec<String> = found.iter().map(|f| f.code.to_string()).collect();
            if !told.is_empty() {
                missed.push((file.display().to_string(), told));
            }
        }
        assert!(keys_read > 4_000, "{keys_read} blocks read");
        assert!(missed.is_empty(), "{missed:#?}");
        // The names of the behaviour packs' files: the rest of the list, each written.
        let text = vanilla_behavior_text(
            game_files,
            &["entities", "spawn_rules", "spawn_groups", "features"],
        );
        let listed: BTreeSet<String> = LEGACY_BLOCK_NAMES.iter().map(|n| (*n).to_owned()).collect();
        let unlisted: Vec<&String> = needed.difference(&listed).collect();
        assert!(unlisted.is_empty(), "blocks.json needs {unlisted:?}");
        assert_eq!(needed.len(), 17, "{needed:?}");
        for name in listed.difference(&needed) {
            assert!(written_in(&text, name), "{name} is in no file");
            assert!(!vanilla.contains(name.as_str()), "{name} is generated");
        }
    }
}
