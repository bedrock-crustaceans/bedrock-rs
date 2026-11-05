//! The fixtures the unknown-member tests read: every JSON file of `tests/fixtures`, the document
//! types read from them (by the pack path a type claims), and the texts with an unknown member
//! put into their objects.

#![allow(dead_code, reason = "each test file uses part of the helpers")]

use std::path::{Path, PathBuf};

use ba::json::Value;
use bedrock_addon as ba;

/// FNV-1a, 64 bits: a pin of a text.
pub fn hash(text: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.bytes() {
        h ^= u64::from(byte);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

/// `path` is a `.json` file below `dir`.
pub fn below(path: &str, dir: &str, extension: &str) -> bool {
    path.starts_with(&format!("{dir}/")) && path.ends_with(extension)
}

/// Every `.json` file below `dir`, sorted, with its path relative to `root`.
pub fn files(root: &Path, dir: &Path, out: &mut Vec<(String, PathBuf)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            files(root, &path, out);
        } else if path.extension().is_some_and(|ext| ext == "json") {
            let relative = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            out.push((relative, path));
        }
    }
}

/// `value` with `"zz_unknown": {"zz_inner": 1}` put into each object at a depth `at` takes
/// (the root object is at 0).
pub fn inject(value: &Value, depth: usize, at: &dyn Fn(usize) -> bool) -> Value {
    if let Some(array) = value.as_array() {
        let items: facet_value::VArray = array.iter().map(|v| inject(v, depth + 1, at)).collect();
        return items.into();
    }
    let Some(object) = value.as_object() else {
        return value.clone();
    };
    let mut out = facet_value::VObject::new();
    for (key, member) in object {
        out.insert(key.as_str(), inject(member, depth + 1, at));
    }
    if at(depth) {
        let mut inner = facet_value::VObject::new();
        inner.insert("zz_inner", Value::from(1i64));
        out.insert("zz_unknown", Value::from(inner));
    }
    out.into()
}

/// The texts a file is read as: as written, then with unknown members put in.
pub fn variants(bytes: &[u8]) -> Vec<(&'static str, Vec<u8>)> {
    let mut out = vec![("as written", bytes.to_vec())];
    let Ok(text) = std::str::from_utf8(bytes) else {
        return out;
    };
    let Ok(value) = ba::json::from_str_jsonc::<Value>(text) else {
        return out;
    };
    let depths: [(&'static str, &dyn Fn(usize) -> bool); 5] = [
        ("every object", &|_| true),
        ("depth 0..=1", &|d| d <= 1),
        ("depth 2", &|d| d == 2),
        ("depth 3", &|d| d == 3),
        ("depth 4..", &|d| d >= 4),
    ];
    for (name, at) in depths {
        if let Ok(text) = ba::json::to_json(&inject(&value, 0, at)) {
            out.push((name, text.into_bytes()));
        }
    }
    out
}

/// Every document type of the list, each as `$m!(name, type, claims[, chain])`.
macro_rules! each_kind {
    ($m:ident) => {{
        use ba::families as f;
        use fixtures::below;
        vec![
            $m!(
                "block",
                f::block::Block,
                |k| below(k, "blocks", ".json"),
                chain
            ),
            $m!("item", f::item::Item, |k| below(k, "items", ".json"), chain),
            $m!(
                "entity",
                f::entity::Entity,
                |k| below(k, "entities", ".json"),
                chain
            ),
            $m!(
                "biome",
                f::biome::Biome,
                |k| below(k, "biomes", ".json"),
                chain
            ),
            $m!(
                "client_biome",
                f::client_biome::ClientBiome,
                |k| below(k, "biomes", ".json"),
                chain
            ),
            $m!("spawn_rules", f::spawn_rules::SpawnRules, |k| below(
                k,
                "spawn_rules",
                ".json"
            )),
            $m!(
                "feature",
                f::feature::Feature,
                |k| below(k, "features", ".json"),
                chain
            ),
            $m!("feature_rules", f::feature_rules::FeatureRule, |k| below(
                k,
                "feature_rules",
                ".json"
            )),
            $m!(
                "dimension",
                f::dimension::Dimension,
                |k| below(k, "dimensions", ".json"),
                chain
            ),
            $m!(
                "trade",
                f::trade::TradeTable,
                |k| below(k, "trading", ".json"),
                chain
            ),
            $m!("jigsaw_structure", f::jigsaw::JigsawStructure, |k| below(
                k,
                "worldgen/structures",
                ".json"
            )),
            $m!("template_pool", f::jigsaw::TemplatePool, |k| below(
                k,
                "worldgen/template_pools",
                ".json"
            )),
            $m!("structure_set", f::jigsaw::StructureSet, |k| below(
                k,
                "worldgen/structure_sets",
                ".json"
            )),
            $m!("processor_list", f::jigsaw::ProcessorList, |k| below(
                k,
                "worldgen/processors",
                ".json"
            )),
            $m!(
                "server_sound_definitions",
                f::server_sound_definitions::SoundMetadataFile,
                |k| k == "sounds/sound_definitions.json"
            ),
            $m!("recipe", f::recipe::Recipe, |k| below(
                k, "recipes", ".json"
            )),
            $m!("dialogue", f::dialogue::DialogueFile, |k| below(
                k, "dialogue", ".json"
            )),
            $m!("loot", f::loot::LootTable, |k| below(
                k,
                "loot_tables",
                ".json"
            )),
            $m!("behavior_tree", f::behavior_tree::BehaviorTree, |k| below(
                k,
                "behavior_trees",
                ".json"
            )),
            $m!("ui_texture_info", f::ui_texture_info::UiTextureInfo, |k| {
                below(k, "textures/ui", ".json")
            }),
            $m!(
                "fog",
                f::fog::FogSettings,
                |k| below(k, "fogs", ".json"),
                chain
            ),
            $m!("lighting", f::deferred_settings::LightingSettings, |k| k
                == "lighting/global.json"),
            $m!(
                "atmosphere",
                f::deferred_settings::AtmosphereSettings,
                |k| below(k, "atmospherics", ".json")
            ),
            $m!("water", f::deferred_settings::WaterSettings, |k| below(
                k, "water", ".json"
            )),
            $m!(
                "color_grading",
                f::deferred_settings::ColorGradingSettings,
                |k| below(k, "color_grading", ".json")
            ),
        ]
    }};
}
pub(crate) use each_kind;
