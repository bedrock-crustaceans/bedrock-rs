//! The table against the vanilla sample packs (for the link pass): no pack defines a trim, but the packs name every pattern and material in their
//! texts, textures, recipes and loot tables. Skipped when `BEDROCK_ADDON_CORPUS` is unset.

use std::collections::BTreeSet;
use std::path::PathBuf;

use bedrock_addon_family_item::trim::TrimData;
use bedrock_addon_test_support::corpus;

/// The sample packs in the corpus (never committed), or `None` after printing the skip line of
/// `check`: the variable is not set, or the corpus has no sample packs.
fn samples(check: &str) -> Option<PathBuf> {
    let Some(root) = std::env::var_os("BEDROCK_ADDON_CORPUS") else {
        corpus::skipped(check, corpus::not_set(corpus::CORPUS));
        return None;
    };
    let samples = PathBuf::from(root).join("bedrock-samples");
    if samples.is_dir() {
        Some(samples)
    } else {
        corpus::skipped(check, format_args!("{} is missing", samples.display()));
        None
    }
}

fn pattern_ids() -> BTreeSet<String> {
    TrimData::vanilla()
        .patterns()
        .map(|pattern| pattern.pattern_id.as_str().to_owned())
        .collect()
}

fn material_ids() -> BTreeSet<String> {
    TrimData::vanilla()
        .materials()
        .iter()
        .map(|material| material.material_id.as_str().to_owned())
        .collect()
}

/// The `<id>` of every `<prefix><id>.name=` line of `en_US.lang`.
fn lang_ids(lang: &str, prefix: &str) -> BTreeSet<String> {
    lang.lines()
        .filter_map(|line| line.strip_prefix(prefix))
        .filter_map(|rest| rest.split_once(".name="))
        .map(|(id, _)| id.to_owned())
        .collect()
}

/// `en_US.lang` has one `trim_pattern.<id>.name` per pattern and one
/// `trim_material.<id>.name` per material, and no other.
#[test]
fn the_vanilla_texts_name_exactly_the_tables_patterns_and_materials() {
    let Some(samples) = samples("the trim text check") else {
        return;
    };
    let lang = std::fs::read_to_string(samples.join("resource_pack/texts/en_US.lang")).unwrap();
    assert_eq!(lang_ids(&lang, "trim_pattern."), pattern_ids());
    assert_eq!(lang_ids(&lang, "trim_material."), material_ids());
    assert_eq!(pattern_ids().len(), 18);
    assert_eq!(material_ids().len(), 11);
}

/// `textures/trims/<pattern>.png` and `<pattern>_leggings.png` per pattern, and
/// `textures/trims/color_palettes/<material>.png` per material; the palettes besides are the
/// `_darker` ones and `trim_palette`.
#[test]
fn the_vanilla_textures_cover_exactly_the_tables_patterns_and_materials() {
    let Some(samples) = samples("the trim texture check") else {
        return;
    };
    let trims = samples.join("resource_pack/textures/trims");
    let stems = |dir: &std::path::Path| -> BTreeSet<String> {
        std::fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "png"))
            .map(|path| path.file_stem().unwrap().to_str().unwrap().to_owned())
            .collect()
    };

    let mut expected = BTreeSet::new();
    for id in pattern_ids() {
        expected.insert(format!("{id}_leggings"));
        expected.insert(id);
    }
    assert_eq!(stems(&trims), expected);

    let palettes: BTreeSet<String> = stems(&trims.join("color_palettes"))
        .into_iter()
        .filter(|stem| !stem.ends_with("_darker") && stem != "trim_palette")
        .collect();
    assert_eq!(palettes, material_ids());
}

/// One duplication recipe per template item (`bolt` has a second, waxed one).
#[test]
fn the_vanilla_recipes_duplicate_exactly_the_tables_template_items() {
    let Some(samples) = samples("the trim recipe check") else {
        return;
    };
    let templates: BTreeSet<String> = std::fs::read_dir(samples.join("behavior_pack/recipes"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .filter_map(|name| {
            let (template, _) = name.split_once("_duplicate")?;
            template
                .ends_with("_armor_trim_smithing_template")
                .then(|| format!("minecraft:{template}"))
        })
        .collect();
    let expected: BTreeSet<String> = TrimData::vanilla()
        .patterns()
        .map(|pattern| pattern.item_name.as_str().to_owned())
        .collect();
    assert_eq!(templates, expected);
}

/// The string value of every `"<key>": "<value>"` member of `text`.
fn string_members<'a>(text: &'a str, key: &str) -> Vec<&'a str> {
    let needle = format!("\"{key}\"");
    text.match_indices(&needle)
        .filter_map(|(at, _)| {
            let rest = text[at + needle.len()..].trim_start().strip_prefix(':')?;
            let rest = rest.trim_start().strip_prefix('"')?;
            rest.split_once('"').map(|(value, _)| value)
        })
        .collect()
}

/// The source of `lk04`: vanilla's `set_armor_trim` loot functions name registered
/// patterns and materials (`copper` with `flow` or `bolt`).
#[test]
fn the_vanilla_loot_tables_name_registered_patterns_and_materials() {
    let Some(samples) = samples("the trim loot check") else {
        return;
    };
    let vanilla = TrimData::vanilla();
    let mut functions = 0;
    let mut patterns = BTreeSet::new();
    let mut materials = BTreeSet::new();
    for name in ["iron", "chainmail", "diamond"] {
        let path = samples.join(format!(
            "behavior_pack/loot_tables/equipment/trial_chamber_{name}.json"
        ));
        let text = std::fs::read_to_string(&path).unwrap();
        for function in text.split("\"set_armor_trim\"").skip(1) {
            functions += 1;
            // The function's own members come before the next function of the list.
            let own = function.split("\"function\"").next().unwrap();
            let pattern = string_members(own, "pattern");
            let material = string_members(own, "material");
            assert_eq!((pattern.len(), material.len()), (1, 1), "{path:?}");
            assert!(
                vanilla.pattern(pattern[0]).is_some(),
                "{path:?}: {pattern:?}"
            );
            assert!(
                vanilla.material(material[0]).is_some(),
                "{path:?}: {material:?}"
            );
            patterns.insert(pattern[0].to_owned());
            materials.insert(material[0].to_owned());
        }
    }
    assert!(functions > 0, "the tables use set_armor_trim");
    assert_eq!(
        patterns,
        BTreeSet::from(["bolt".to_owned(), "flow".to_owned()])
    );
    assert_eq!(materials, BTreeSet::from(["copper".to_owned()]));
}
