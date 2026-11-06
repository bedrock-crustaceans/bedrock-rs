//! Every family reads its JSON through core's door (`json::read`): a string escape
//! outside RFC 8259 is a syntax error of the file in every family, its own reader included,
//! and never reaches facet-json, which decodes `"\é"` out of bounds (an abort in a debug
//! build, undefined behaviour in a release one). Content after the root value loads, as in
//! vanilla, and never reaches facet either.
//!
//! One list of the families' document types serves every test: a new family adds its line.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use bedrock_addon::diagnostic::{Diagnostic, DiagnosticCode, DiagnosticDetail, Severity};
use bedrock_addon::document::Content;
use bedrock_addon::load::LoadContext;
use bedrock_addon::storage::PackPath;

/// What a family made of one text: whether it loaded, and its findings.
struct Outcome {
    loaded: bool,
    findings: Vec<Diagnostic>,
}

/// The reader's message of a syntax finding: the legacy `MalformedInput` payload, or, for a
/// family on its own definition (fog's `parse_error`), the `DiagnosticDetail::Syntax` it carries.
fn syntax_message(d: &Diagnostic) -> Option<&str> {
    match &d.code {
        DiagnosticCode::MalformedInput { message, .. } => Some(message),
        code if code.name() == "MalformedInput" => match d.detail.as_deref() {
            Some(DiagnosticDetail::Syntax(message)) => Some(message),
            _ => None,
        },
        _ => None,
    }
}

impl Outcome {
    /// The messages of the `MalformedInput` findings of `severity` or above.
    fn malformed(&self, severity: Severity) -> Vec<String> {
        self.findings
            .iter()
            .filter(|d| d.severity >= severity)
            .filter_map(|d| syntax_message(d).map(str::to_owned))
            .collect()
    }

    /// The refusals of the game's JSON reader for content after the root value (a `MinecraftDocumentInput`).
    fn refusals(&self) -> Vec<&Diagnostic> {
        self.findings
            .iter()
            .filter(|d| {
                // (The item catalog tells its refusal at information level.)
                syntax_message(d) == Some("The document root must not be followed by other values.")
            })
            .collect()
    }

    /// The findings about content after the root value.
    fn trailing(&self) -> Vec<&Diagnostic> {
        self.findings
            .iter()
            .filter(|d| syntax_message(d) == Some("content after the JSON value is ignored"))
            .collect()
    }
}

#[allow(dead_code, reason = "used by the lines of the compiled families")]
fn parse<T: Content>(path: &'static str, text: &str) -> Outcome {
    let cx = LoadContext::detached();
    match T::parse(&PackPath::from_static(path), text.as_bytes(), &cx) {
        Ok(document) => Outcome {
            loaded: true,
            findings: document.diagnostics().iter().cloned().collect(),
        },
        Err(rejected) => Outcome {
            loaded: false,
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
            findings: document.diagnostics().iter().cloned().collect(),
        },
        Err(rejected) => Outcome {
            loaded: false,
            findings: rejected.diagnostics.iter().cloned().collect(),
        },
    }
}

fn pack_settings(text: &str) -> Outcome {
    match bedrock_addon::pack::settings::SavedSettings::parse(text.as_bytes()) {
        Ok(document) => Outcome {
            loaded: true,
            findings: document.diagnostics().iter().cloned().collect(),
        },
        Err(rejected) => Outcome {
            loaded: false,
            findings: rejected.diagnostics.iter().cloned().collect(),
        },
    }
}

/// A POI type is read under its experiment only: a context of a behaviour pack that has it.
#[cfg(feature = "poi")]
fn poi_type(text: &str) -> Outcome {
    use bedrock_addon::load::{Experiment, Experiments, LoadOptions};
    use bedrock_addon::pack::PackType;
    use bedrock_addon::version::MinEngineVersion;
    let mut experiments = Experiments::new();
    experiments.insert(Experiment::Poi);
    let options = LoadOptions {
        experiments,
        ..LoadOptions::default()
    };
    let cx = LoadContext::for_pack(
        &options,
        uuid::Uuid::from_u128(7),
        PackType::Behavior,
        Some(MinEngineVersion::new(1, 26, 40)),
        false,
    );
    let path = PackPath::from_static("poi/types/x.json");
    match bedrock_addon::families::poi::PoiType::parse(&path, text.as_bytes(), &cx) {
        Ok(document) => Outcome {
            loaded: true,
            findings: document.diagnostics().iter().cloned().collect(),
        },
        Err(rejected) => Outcome {
            loaded: false,
            findings: rejected.diagnostics.iter().cloned().collect(),
        },
    }
}

/// A reader of one family's file, by name.
type Reader = (&'static str, fn(&str) -> Outcome);

/// One line per document type that is read from JSON: its name and its reader at a path of
/// the family.
macro_rules! readers {
    ($( $(#[$cfg:meta])? $name:literal => $type:ty, $path:literal; )*) => {
        #[allow(unused_mut, reason = "without a content feature only the manifest is read")]
        fn readers() -> Vec<Reader> {
            let mut out: Vec<Reader> =
                vec![("manifest", manifest), ("pack settings", pack_settings)];
            #[cfg(feature = "poi")]
            out.push(("poi type", poi_type));
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
    #[cfg(feature = "voxel_shape")] "voxel shape" => bedrock_addon::families::voxel_shape::VoxelShape, "shapes/x.json";
    #[cfg(feature = "item")] "item" => bedrock_addon::families::item::Item, "items/x.json";
    #[cfg(feature = "crafting_catalog")] "crafting catalog" => bedrock_addon::families::crafting_catalog::CraftingCatalog, "item_catalog/crafting_item_catalog.json";
    #[cfg(feature = "entity")] "entity" => bedrock_addon::families::entity::Entity, "entities/x.json";
    #[cfg(all(feature = "spawn_rules", feature = "spawn_groups"))] "spawn rules" => bedrock_addon::families::spawn_rules::SpawnRules, "spawn_rules/x.json";
    #[cfg(all(feature = "spawn_rules", feature = "spawn_groups"))] "spawn group" => bedrock_addon::families::spawn_groups::SpawnGroup, "spawn_groups/x.json";
    #[cfg(feature = "biome")] "biome" => bedrock_addon::families::biome::Biome, "biomes/x.json";
    #[cfg(feature = "client_biome")] "client biome" => bedrock_addon::families::client_biome::ClientBiome, "biomes/x.client_biome.json";
    #[cfg(feature = "biomes_client")] "biomes_client" => bedrock_addon::families::biomes_client::LegacyClientBiomes, "biomes_client.json";
    #[cfg(feature = "feature")] "feature" => bedrock_addon::families::feature::Feature, "features/x.json";
    #[cfg(feature = "feature_rules")] "feature rule" => bedrock_addon::families::feature_rules::FeatureRule, "feature_rules/x.json";
    #[cfg(feature = "dimension")] "dimension" => bedrock_addon::families::dimension::Dimension, "dimensions/overworld.json";
    #[cfg(feature = "jigsaw")] "jigsaw metadata" => bedrock_addon::families::jigsaw::JigsawMetadata, "worldgen/structure_sets/x.json";
    #[cfg(feature = "loot")] "loot table" => bedrock_addon::families::loot::LootTable, "loot_tables/x.json";
    #[cfg(feature = "recipe")] "recipe" => bedrock_addon::families::recipe::Recipe, "recipes/x.json";
    #[cfg(feature = "trade")] "trade table" => bedrock_addon::families::trade::TradeTable, "trading/x.json";
    #[cfg(feature = "dialogue")] "dialogue" => bedrock_addon::families::dialogue::DialogueFile, "dialogue/x.json";
    #[cfg(feature = "animation")] "animation" => bedrock_addon::families::animation::AnimationFile, "animations/x.json";
    #[cfg(feature = "animation_controller")] "animation controller" => bedrock_addon::families::animation_controller::AnimationControllerFile, "animation_controllers/x.json";
    #[cfg(feature = "geometry")] "geometry" => bedrock_addon::families::geometry::GeometryFile, "models/entity/x.json";
    #[cfg(feature = "render_controller")] "render controller" => bedrock_addon::families::render_controller::RenderControllerFile, "render_controllers/x.json";
    #[cfg(feature = "fog")] "fog" => bedrock_addon::families::fog::FogSettings, "fogs/x.json";
    #[cfg(feature = "texture_set")] "texture set" => bedrock_addon::families::texture_set::TextureSet, "textures/blocks/x.texture_set.json";
    #[cfg(feature = "texture_atlas")] "terrain_texture" => bedrock_addon::families::texture_atlas::TextureAtlas, "textures/terrain_texture.json";
    #[cfg(feature = "texture_list")] "textures_list" => bedrock_addon::families::texture_list::TextureList, "textures/textures_list.json";
    #[cfg(feature = "flipbooks")] "flipbook_textures" => bedrock_addon::families::flipbooks::Flipbooks, "textures/flipbook_textures.json";
    #[cfg(feature = "languages")] "languages" => bedrock_addon::families::languages::Languages, "texts/languages.json";
    #[cfg(feature = "languages")] "language names" => bedrock_addon::families::languages::LanguageNames, "texts/language_names.json";
    #[cfg(feature = "function")] "tick list" => bedrock_addon::families::function::TickList, "functions/tick.json";
    #[cfg(feature = "json_ui")] "ui file" => bedrock_addon::families::json_ui::UiFile, "ui/x.json";
    #[cfg(feature = "json_ui")] "ui defs" => bedrock_addon::families::json_ui::UiDefsList, "ui/_ui_defs.json";
    #[cfg(feature = "json_ui")] "ui global variables" => bedrock_addon::families::json_ui::GlobalVariables, "ui/_global_variables.json";
    #[cfg(feature = "camera")] "camera preset" => bedrock_addon::families::camera::CameraPreset, "cameras/presets/x.json";
    #[cfg(feature = "behavior_tree")] "behavior tree" => bedrock_addon::families::behavior_tree::BehaviorTree, "behavior_trees/x.json";
}

/// Documents with one bad escape: as a value, as a key, deep in the document, and in the
/// forms facet reads without a fault but wrongly.
const BAD: &[&str] = &[
    r#"{"format_version": "1.21.60", "a": "\é"}"#,
    r#"{"format_version": "1.21.60", "\é": 1}"#,
    r#"{"format_version": "1.21.60", "a": {"b": [{"c": "x\😀y"}]}}"#,
    r#"["\é"]"#,
    r#"{"format_version": "1.21.60", "a": "\q"}"#,
    r#"{"format_version": "1.21.60", "a": "\u00é0"}"#,
    r#"{"a": "\u12"}"#,
];

/// The readers that say a file is no JSON in words of their own instead of the reader's
/// message: the jigsaw metadata file ("Failed to parse format version string of file …") and
/// the texture list, which keeps the entries before the fault with a warning.
const OWN_WORDS: &[&str] = &["jigsaw metadata", "textures_list"];

/// Every family, every bad document: the file is no JSON (a `MalformedInput` finding that
/// names the escape, or the family's own words for a file it cannot read), and the process
/// lives: a family that reached facet with `\é` would have killed this binary.
#[test]
fn every_family_reports_a_bad_escape_as_a_file_that_is_no_json() {
    let readers = readers();
    assert!(!readers.is_empty());
    let mut silent = Vec::new();
    for (name, read) in &readers {
        for text in BAD {
            let outcome = read(text);
            let named = outcome
                .malformed(Severity::Info)
                .iter()
                .any(|m| m.contains("escape"));
            let reported = if OWN_WORDS.contains(name) {
                // The texture list's line is an information.
                !outcome.findings.is_empty()
            } else {
                named
            };
            if !reported {
                silent.push(format!(
                    "{name}: `{text}` (loaded: {}): {:?}",
                    outcome.loaded, outcome.findings
                ));
            }
        }
    }
    assert!(
        silent.is_empty(),
        "{} documents without the finding:\n{}",
        silent.len(),
        silent.join("\n")
    );
}

/// A file with a bad escape is no JSON to vanilla's reader: no family loads it, but for the
/// texture list, whose reader keeps what it read before the fault (the family's own rule).
#[test]
fn a_file_with_a_bad_escape_does_not_load() {
    // families/textures_list: "clients keep only the entries read before the error".
    const KEEPS_WHAT_IT_READ: &[&str] = &["textures_list"];
    for (name, read) in readers() {
        for text in BAD {
            let outcome = read(text);
            assert_eq!(
                outcome.loaded,
                KEEPS_WHAT_IT_READ.contains(&name),
                "{name}: `{text}`: {:?}",
                outcome.findings
            );
            if !outcome.loaded {
                // A language list that is no JSON is refused with an information.
                // The legacy client biome file's line is an information.
                let least = if name.starts_with("language") || name == "biomes_client" {
                    Severity::Info
                } else {
                    Severity::Warning
                };
                assert!(
                    outcome.findings.iter().any(|d| d.severity >= least),
                    "{name}: `{text}`: rejected without a finding"
                );
            }
        }
    }
}

/// The escapes JSON has read as they always did, here through a block's identifier and a UI
/// file's text.
#[cfg(all(feature = "block", feature = "json_ui"))]
#[test]
fn the_escapes_of_json_still_decode_in_a_block_and_a_ui_file() {
    let block = parse::<bedrock_addon::families::block::Block>(
        "blocks/x.json",
        r#"{"format_version": "1.21.60", "minecraft:block": {"description": {"identifier": "demo:\u0073t\u006Fne"}}}"#,
    );
    let shown = format!("{:?}", block.findings);
    assert!(block.malformed(Severity::Info).is_empty(), "{shown}");
    assert!(block.loaded, "{shown}");

    let ui = parse::<bedrock_addon::families::json_ui::UiFile>(
        "ui/x.json",
        r#"{"namespace": "demo", "label": {"type": "label", "text": "caf\u00e9 \ud83d\ude00 \/ \"é\" \n\t\\ 😀"}}"#,
    );
    assert!(ui.loaded, "{:?}", ui.findings);
    assert!(ui.malformed(Severity::Info).is_empty(), "{:?}", ui.findings);
}

/// A block file end to end: the finding is an error on the escape itself.
#[cfg(feature = "block")]
#[test]
fn a_block_file_with_a_bad_escape_is_rejected_at_the_escape() {
    let text = "{\n  \"format_version\": \"1.21.60\",\n  \"minecraft:block\": {\n    \
                \"description\": { \"identifier\": \"demo:x\" },\n    \"components\": { \
                \"minecraft:display_name\": \"caf\\é\" }\n  }\n}";
    let outcome = parse::<bedrock_addon::families::block::Block>("blocks/x.json", text);
    assert!(!outcome.loaded);
    let errors: Vec<_> = outcome
        .findings
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert_eq!(errors.len(), 1, "{:?}", outcome.findings);
    assert!(
        syntax_message(errors[0])
            .is_some_and(|message| message.contains("bad escape sequence in string")
                && message.contains('é')),
        "{:?}",
        errors[0].code
    );
    let span = errors[0].location.span.expect("a span");
    assert_eq!(span.offset as usize, text.find('\\').unwrap());
    assert_eq!(span.line_col(text), (5, 51));
}

/// A UI file end to end, through the family's own reader.
#[cfg(feature = "json_ui")]
#[test]
fn a_ui_file_with_a_bad_escape_is_rejected() {
    let outcome = parse::<bedrock_addon::families::json_ui::UiFile>(
        "ui/x.json",
        r#"{"namespace": "demo", "label": {"type": "label", "text": "caf\é"}}"#,
    );
    assert!(!outcome.loaded);
    let messages = outcome.malformed(Severity::Error);
    assert_eq!(messages.len(), 1, "{:?}", outcome.findings);
    assert!(
        messages[0].contains("bad escape sequence in string"),
        "{messages:?}"
    );
}

// --- content after the root value ------------------------------------------------------------

/// A document each family loads, to put content after.
fn valid() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "block",
            r#"{"format_version": "1.21.60", "minecraft:block": {"description": {"identifier": "demo:x"}}}"#,
        ),
        (
            "loot table",
            r#"{"pools": [{"rolls": 1, "entries": [{"type": "item", "name": "minecraft:stone"}]}]}"#,
        ),
        (
            "recipe",
            r#"{"format_version": "1.12.0", "minecraft:recipe_shapeless": {"description": {"identifier": "demo:x"}, "tags": ["crafting_table"], "ingredients": ["minecraft:stone"], "result": "minecraft:stone"}}"#,
        ),
        (
            "dialogue",
            r#"{"format_version": "1.17.0", "minecraft:npc_dialogue": {"scenes": [{"scene_tag": "x", "text": "Hi."}]}}"#,
        ),
        (
            "behavior tree",
            r#"{"minecraft:behavior_tree": {"minecraft:name": "x", "minecraft:root": {"minecraft:wait_ticks": {"num_of_ticks": 1}}}}"#,
        ),
        ("tick list", r#"{"values": ["x"]}"#),
        (
            "crafting catalog",
            r#"{"format_version": "1.21.60", "minecraft:crafting_items_catalog": {"categories": [{"category_name": "construction", "groups": [{"group_identifier": {"name": "demo:x"}, "items": []}]}]}}"#,
        ),
        ("languages", r#"["en_US"]"#),
        (
            "geometry",
            r#"{"format_version": "1.21.0", "minecraft:geometry": [{"description": {"identifier": "geometry.x"}}]}"#,
        ),
        (
            "ui file",
            r#"{"namespace": "demo", "panel": {"type": "panel"}}"#,
        ),
        (
            "terrain_texture",
            r#"{"texture_name": "atlas.terrain", "texture_data": {}}"#,
        ),
        // The families core's versioned load reads.
        (
            "item",
            r#"{"format_version": "1.21.60", "minecraft:item": {"description": {"identifier": "demo:x"}}}"#,
        ),
        (
            "entity",
            r#"{"format_version": "1.21.60", "minecraft:entity": {"description": {"identifier": "demo:x", "is_spawnable": true, "is_summonable": true}}}"#,
        ),
        (
            "biome",
            r#"{"format_version": "1.21.110", "minecraft:biome": {"description": {"identifier": "demo:x"}, "components": {}}}"#,
        ),
        (
            "spawn rules",
            r#"{"format_version": "1.8.0", "minecraft:spawn_rules": {"description": {"identifier": "demo:x", "population_control": "animal"}, "conditions": [{"minecraft:spawns_on_surface": {}}]}}"#,
        ),
        (
            "spawn group",
            r#"{"format_version": "1.8.0", "minecraft:spawn_groups": {"description": {"identifier": "demo:x", "population_control": "monster"}, "conditions": [{"minecraft:spawns_on_surface": {}}]}}"#,
        ),
        (
            "feature rule",
            r#"{"format_version": "1.13.0", "minecraft:feature_rules": {"description": {"identifier": "demo:x", "places_feature": "demo:feature"}, "conditions": {"placement_pass": "surface_pass"}}}"#,
        ),
        (
            "dimension",
            r#"{"format_version": "1.18.0", "minecraft:dimension": {"description": {"identifier": "minecraft:overworld"}, "components": {"minecraft:dimension_bounds": {"min": -64, "max": 320}, "minecraft:generation": {"generator_type": "void"}}}}"#,
        ),
        (
            "poi type",
            r#"{"format_version": "1.26.40", "minecraft:poi_block": {"description": {"identifier": "demo:x"}, "blocks": ["minecraft:bell"]}}"#,
        ),
        (
            "trade table",
            r#"{"tiers": [{"trades": [{"wants": [{"item": "minecraft:emerald"}], "gives": [{"item": "minecraft:bread", "quantity": 6}]}]}]}"#,
        ),
        (
            "camera preset",
            r#"{"format_version": "1.19.50", "minecraft:camera_preset": {"identifier": "demo:x", "inherit_from": "minecraft:free"}}"#,
        ),
        (
            "fog",
            r#"{"format_version": "1.16.100", "minecraft:fog_settings": {"description": {"identifier": "demo:x"}}}"#,
        ),
        (
            "voxel shape",
            r#"{"format_version": "1.21.110", "minecraft:voxel_shape": {"description": {"identifier": "demo:x"}, "shape": {"boxes": [{"min": [0, 0, 0], "max": [16, 8, 16]}]}}}"#,
        ),
        (
            "pack settings",
            r#"{"format_version": "1.21.100", "minecraft:pack_settings": {"settings": [{"pack_id": "d3349771-6782-4420-875a-fcd6179a8915", "values": {}}]}}"#,
        ),
        (
            "manifest",
            r#"{"format_version": 2, "header": {"name": "x", "uuid": "00000000-0000-0000-0000-000000000001", "version": [1, 0, 0], "min_engine_version": [1, 21, 0]}, "modules": [{"type": "data", "uuid": "00000000-0000-0000-0000-000000000002", "version": [1, 0, 0]}]}"#,
        ),
    ]
}

/// What may follow a root value: anything, an unsafe string included.
const TAILS: &[&str] = &[" garbage", "é", "\n}", r#" "\é""#, r#"{"\q": "\u00é0"}"#];

/// The families whose reader is a `MinecraftDocumentInput` (`InputKind::Document`): the game's
/// JSON reader stops at content after the root value and the document is not loaded.
const DOCUMENT_INPUT: &[&str] = &[
    "block",
    "crafting catalog",
    "item",
    "biome",
    "feature rule",
    "dimension",
    "poi type",
    "trade table",
    "camera preset",
    "voxel shape",
];

/// Vanilla's `JsonCpp` reader never looks past the root value: with anything after
/// it the file loads as it loads without, in every family that reads `JsonCpp`, and what follows
/// never reaches facet. A `MinecraftDocumentInput` refuses it, still without
/// reaching facet.
#[test]
fn content_after_the_root_value_changes_nothing_that_is_loaded() {
    let readers = readers();
    let mut checked = 0;
    for (name, text) in valid() {
        let Some((_, read)) = readers.iter().find(|(reader, _)| *reader == name) else {
            continue; // the family is not compiled in
        };
        let plain = read(text);
        assert!(plain.loaded, "{name}: {:?}", plain.findings);
        assert!(plain.trailing().is_empty(), "{name}");
        for tail in TAILS {
            let outcome = read(&format!("{text}{tail}"));
            if DOCUMENT_INPUT.contains(&name) {
                assert!(!outcome.loaded, "{name} `{tail}`: {:?}", outcome.findings);
                assert_eq!(outcome.refusals().len(), 1, "{name} `{tail}`");
                continue;
            }
            assert!(outcome.loaded, "{name} `{tail}`: {:?}", outcome.findings);
            let others = |outcome: &Outcome| outcome.findings.len() - outcome.trailing().len();
            assert_eq!(others(&outcome), others(&plain), "{name} `{tail}`");
            assert!(outcome.trailing().len() <= 1, "{name} `{tail}`: twice");
        }
        checked += 1;
    }
    assert!(checked >= 2);
    #[cfg(all(
        feature = "block",
        feature = "item",
        feature = "entity",
        feature = "spawn_groups",
        feature = "spawn_rules",
        feature = "biome",
        feature = "dimension",
        feature = "feature",
        feature = "feature_rules",
        feature = "jigsaw",
        feature = "client_entity",
        feature = "atomic_client_entity",
        feature = "client_biome",
        feature = "biomes_client",
        feature = "loot",
        feature = "recipe",
        feature = "trade",
        feature = "dialogue",
        feature = "aim_assist",
        feature = "camera",
        feature = "geometry",
        feature = "render_controller",
        feature = "fog",
        feature = "material",
        feature = "deferred_settings",
        feature = "texture_atlas",
        feature = "flipbooks",
        feature = "texture_list",
        feature = "texture_set",
        feature = "languages",
        feature = "function",
        feature = "json_ui",
        feature = "ddui",
        feature = "font",
        feature = "splashes",
        feature = "loading_messages",
        feature = "credits",
        feature = "ui_texture_info",
        feature = "poi",
        feature = "behavior_tree"
    ))]
    assert_eq!(
        checked,
        valid().len(),
        "a document of the list has no reader"
    );
}

/// Who tells the author: core's versioned load, for every family it reads from
/// `JsonCpp`, with a span on the content, an information of ours; a
/// `MinecraftDocumentInput` family refuses the file instead. (The texture files' stack loaders report it themselves: terrain
/// texture rule 1a, `texture_set` ts13.) A family's own reader of a single file is as silent as
/// vanilla until its owner calls `json::trailing_content_finding`; it then moves to the list.
#[test]
fn content_after_the_root_value_is_an_information_where_the_family_reports_it() {
    const REPORTED: &[&str] = &[
        "entity",
        "spawn rules",
        "spawn group",
        "fog",
        "pack settings",
    ];
    let readers = readers();
    for (name, text) in valid() {
        let Some((_, read)) = readers.iter().find(|(reader, _)| *reader == name) else {
            continue;
        };
        let file = format!("{text}\n garbage\n");
        let outcome = read(&file);
        let trailing = outcome.trailing();
        assert_eq!(
            trailing.len(),
            usize::from(REPORTED.contains(&name)),
            "{name}: {:?}",
            outcome.findings
        );
        assert_eq!(
            outcome.refusals().len(),
            usize::from(DOCUMENT_INPUT.contains(&name)),
            "{name}: {:?}",
            outcome.findings
        );
        for finding in trailing {
            assert_eq!(finding.severity, Severity::Info, "{name}");
            if let Some(span) = finding.location.span {
                let at = span.offset as usize;
                assert_eq!(&file[at..at + span.len as usize], "garbage", "{name}");
            }
        }
    }
}
