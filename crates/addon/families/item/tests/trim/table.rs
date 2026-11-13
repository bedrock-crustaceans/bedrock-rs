//! The generated constants and their lookups.

use bedrock_addon_core::id::{Kind, Syntax, TrimMaterialId, TrimPatternId, kind};
use bedrock_addon_core::version::SemVersion;
use bedrock_addon_family_item::trim::TrimData;

use super::common::{
    item, material_entry, material_rows, pattern_entry, pattern_rows, vanilla_table,
};

#[test]
fn tr01_the_vanilla_table_is_the_18_patterns_and_11_materials_in_build_order() {
    let expected = vanilla_table();
    assert_eq!(expected.patterns.len(), 18, "the fixture's patterns");
    assert_eq!(expected.materials.len(), 11, "the fixture's materials");

    let vanilla = TrimData::vanilla();
    assert_eq!(pattern_rows(vanilla), expected.patterns);
    assert_eq!(material_rows(vanilla), expected.materials);
    assert!(vanilla.diagnostics().is_empty());
}

#[test]
fn tr02_the_reverse_index_finds_patterns_and_materials_by_id_and_by_item() {
    let vanilla = TrimData::vanilla();

    let ward = vanilla.pattern("ward").expect("the pattern `ward`");
    assert_eq!(
        ward.item_name.as_str(),
        "minecraft:ward_armor_trim_smithing_template"
    );
    let by_item = vanilla
        .pattern_for_item(&item("minecraft:ward_armor_trim_smithing_template"))
        .expect("the pattern of the ward template");
    assert_eq!(by_item.pattern_id.as_str(), "ward");

    let resin = vanilla
        .material_for_item(&item("minecraft:resin_brick"))
        .expect("the material of resin bricks");
    assert_eq!(resin.material_id.as_str(), "resin");
    let lapis = vanilla.material("lapis").expect("the material `lapis`");
    assert_eq!(lapis.item_name.as_str(), "minecraft:lapis_lazuli");
}

#[test]
fn every_entry_of_the_table_is_found_both_ways() {
    let vanilla = TrimData::vanilla();
    let expected = vanilla_table();
    for row in &expected.patterns {
        let by_id = vanilla.pattern(&row.pattern_id).expect(&row.pattern_id);
        let by_item = vanilla
            .pattern_for_item(&item(&row.item_name))
            .expect(&row.item_name);
        assert_eq!(by_id, by_item);
        assert_eq!(by_id.pattern_id.as_str(), row.pattern_id);
        assert_eq!(by_id.item_name.as_str(), row.item_name);
    }
    for row in &expected.materials {
        let by_id = vanilla.material(&row.material_id).expect(&row.material_id);
        let by_item = vanilla
            .material_for_item(&item(&row.item_name))
            .expect(&row.item_name);
        assert_eq!(by_id, by_item);
        assert_eq!(by_id.material_id.as_str(), row.material_id);
        assert_eq!(by_id.color.as_str(), row.color);
        assert_eq!(by_id.item_name.as_str(), row.item_name);
    }
}

#[test]
fn a_name_outside_the_table_finds_nothing() {
    let vanilla = TrimData::vanilla();
    assert_eq!(vanilla.pattern("spiral"), None);
    assert_eq!(vanilla.material("obsidian"), None);
    assert_eq!(vanilla.pattern_for_item(&item("minecraft:stick")), None);
    assert_eq!(vanilla.material_for_item(&item("minecraft:stick")), None);
    // A pattern is not a material and a template is not an ingredient.
    assert_eq!(vanilla.material("ward"), None);
    assert_eq!(vanilla.pattern("quartz"), None);
    assert_eq!(
        vanilla.material_for_item(&item("minecraft:ward_armor_trim_smithing_template")),
        None
    );
    assert_eq!(vanilla.pattern_for_item(&item("minecraft:quartz")), None);
}

/// Vanilla's ids are `HashedString`s: bare, compared byte for byte.
#[test]
fn a_pattern_or_material_id_is_compared_as_written() {
    let vanilla = TrimData::vanilla();
    assert!(vanilla.pattern("ward").is_some());
    assert!(vanilla.material("quartz").is_some());
    for id in ["Ward", "minecraft:ward", "ward ", ""] {
        assert_eq!(vanilla.pattern(id), None, "{id:?}");
    }
    for id in ["Quartz", "minecraft:quartz", "quartz ", ""] {
        assert_eq!(vanilla.material(id), None, "{id:?}");
    }
}

/// The id kinds of the two tables (`TrimDataPacket`): free text, no namespace.
#[test]
fn the_trim_id_kinds_are_free_text_compared_as_written() {
    assert_eq!(TrimPatternId::parse("ward"), TrimPatternId::parse("ward"));
    assert_ne!(TrimPatternId::parse("ward"), TrimPatternId::parse("Ward"));
    assert_ne!(
        TrimMaterialId::parse("quartz"),
        TrimMaterialId::parse("minecraft:quartz")
    );
    assert_eq!(<kind::TrimPattern as Kind>::SYNTAX, Syntax::Free);
    assert_eq!(<kind::TrimMaterial as Kind>::SYNTAX, Syntax::Free);
    assert_eq!(<kind::TrimPattern as Kind>::NAME, "trim pattern");
    assert_eq!(<kind::TrimMaterial as Kind>::NAME, "trim material");
}

/// An item is looked up as an `ItemId` compares: the namespace defaults to `minecraft`.
#[test]
fn an_item_is_found_as_item_identifiers_compare() {
    let vanilla = TrimData::vanilla();
    let bare = vanilla
        .pattern_for_item(&item("ward_armor_trim_smithing_template"))
        .expect("the bare template name");
    assert_eq!(bare.pattern_id.as_str(), "ward");
    assert_eq!(
        vanilla
            .material_for_item(&item("quartz"))
            .map(|material| material.material_id.as_str()),
        Some("quartz")
    );
    assert_eq!(
        vanilla.pattern_for_item(&item("demo:ward_armor_trim_smithing_template")),
        None
    );
    assert_eq!(vanilla.material_for_item(&item("demo:quartz")), None);
}

/// The build whose vanilla packs first ship each entry.
#[test]
fn every_entry_knows_the_engine_version_that_first_shipped_it() {
    let vanilla = TrimData::vanilla();
    let patterns: Vec<(&str, Option<SemVersion>)> = vanilla
        .patterns()
        .map(|pattern| (pattern.pattern_id.as_str(), pattern.since_engine()))
        .collect();
    assert_eq!(patterns.len(), 18);
    for (id, since) in &patterns[..16] {
        assert_eq!(*since, Some(SemVersion::new(1, 20, 0)), "{id}");
    }
    assert_eq!(
        patterns[16..],
        [
            ("flow", Some(SemVersion::new(1, 21, 0))),
            ("bolt", Some(SemVersion::new(1, 21, 0))),
        ]
    );

    let materials: Vec<(&str, Option<SemVersion>)> = vanilla
        .materials()
        .iter()
        .map(|material| (material.material_id.as_str(), material.since_engine()))
        .collect();
    assert_eq!(materials.len(), 11);
    for (id, since) in &materials[..10] {
        assert_eq!(*since, Some(SemVersion::new(1, 20, 0)), "{id}");
    }
    assert_eq!(materials[10], ("resin", Some(SemVersion::new(1, 21, 50))));
}

/// The version belongs to the vanilla entry of that name; an entry a server made up has none,
/// and a pattern is not looked up among the materials.
#[test]
fn an_entry_outside_the_vanilla_table_has_no_first_version() {
    let data = TrimData::from_packet(
        &[
            pattern_entry("demo:spiral_template", "spiral"),
            pattern_entry("demo:quartz_template", "quartz"),
            pattern_entry("demo:other_bolt_template", "bolt"),
        ],
        &[
            material_entry("obsidian", "§0", "minecraft:obsidian"),
            material_entry("ward", "§1", "demo:ward"),
            material_entry("resin", "§2", "demo:resin"),
        ],
    );
    assert_eq!(data.pattern("spiral").unwrap().since_engine(), None);
    assert_eq!(data.pattern("quartz").unwrap().since_engine(), None);
    assert_eq!(
        data.pattern("bolt").unwrap().since_engine(),
        Some(SemVersion::new(1, 21, 0))
    );
    assert_eq!(data.material("obsidian").unwrap().since_engine(), None);
    assert_eq!(data.material("ward").unwrap().since_engine(), None);
    assert_eq!(
        data.material("resin").unwrap().since_engine(),
        Some(SemVersion::new(1, 21, 50))
    );
}

/// The identifier is the whole name: the start of a vanilla name is another entry.
#[test]
fn the_first_version_is_looked_up_by_the_whole_identifier() {
    let data = TrimData::from_packet(
        &[
            pattern_entry("demo:a_template", "bol"),
            pattern_entry("demo:b_template", "bolts"),
            pattern_entry("demo:c_template", "Bolt"),
        ],
        &[
            material_entry("res", "§2", "demo:a"),
            material_entry("resins", "§2", "demo:b"),
            material_entry("Resin", "§2", "demo:c"),
        ],
    );
    assert_eq!(data.patterns().len(), 3);
    for pattern in data.patterns() {
        assert_eq!(pattern.since_engine(), None, "{}", pattern.pattern_id);
    }
    for material in data.materials() {
        assert_eq!(material.since_engine(), None, "{}", material.material_id);
    }
}

/// The build order from a second statement: both orders in words (`getPatterns`: "in the
/// order ward, snout, …"; `getMaterials`: "in the order quartz, iron, …"), apart from the
/// numbered tables that the fixture and the constants were transcribed from.
#[test]
fn the_build_order_is_the_one_the_readme_states_in_words() {
    const WORDED_PATTERNS: &str = "ward, snout, dune, spire, wild, rib, tide, sentry, coast, eye, \
        vex, silence, wayfinder, raiser, shaper, host, flow, bolt";
    const WORDED_MATERIALS: &str =
        "quartz, iron, netherite, redstone, copper, gold, emerald, diamond, lapis, amethyst, resin";
    let vanilla = TrimData::vanilla();
    let patterns: Vec<&str> = vanilla
        .patterns()
        .map(|pattern| pattern.pattern_id.as_str())
        .collect();
    assert_eq!(patterns.join(", "), WORDED_PATTERNS);
    let materials: Vec<&str> = vanilla
        .materials()
        .iter()
        .map(|material| material.material_id.as_str())
        .collect();
    assert_eq!(materials.join(", "), WORDED_MATERIALS);
}

/// The colours and ingredient items confirmed against the game: the colour
/// bytes "`§h §i §j §m §n §p §q §s §t §u §v`" and the vanilla item names the game
/// copies, in its order.
#[test]
fn the_colours_and_ingredients_are_the_ones_the_evidence_lists() {
    const EVIDENCE_COLOURS: &str = "§h §i §j §m §n §p §q §s §t §u §v";
    // Each `VanillaItemNames` symbol with the path of the item it names. The pairs are
    // written out: a rule such as "lower case, no underscores" would take `red_stone` for
    // `RedStone` (the item is `minecraft:redstone`).
    const EVIDENCE_ITEM_SYMBOLS: [(&str, &str); 11] = [
        ("Quartz", "quartz"),
        ("IronIngot", "iron_ingot"),
        ("NetheriteIngot", "netherite_ingot"),
        ("RedStone", "redstone"),
        ("CopperIngot", "copper_ingot"),
        ("GoldIngot", "gold_ingot"),
        ("Emerald", "emerald"),
        ("Diamond", "diamond"),
        ("LapisLazuli", "lapis_lazuli"),
        ("AmethystShard", "amethyst_shard"),
        ("ResinBrick", "resin_brick"),
    ];
    let vanilla = TrimData::vanilla();
    let colours: Vec<&str> = vanilla
        .materials()
        .iter()
        .map(|material| material.color.as_str())
        .collect();
    assert_eq!(colours.join(" "), EVIDENCE_COLOURS);
    let expected: Vec<&str> = EVIDENCE_ITEM_SYMBOLS
        .iter()
        .map(|(_, path)| *path)
        .collect();
    let paths: Vec<&str> = vanilla
        .materials()
        .iter()
        .map(|material| material.item_name.path())
        .collect();
    assert_eq!(paths, expected);
    for material in vanilla.materials() {
        assert_eq!(material.item_name.namespace(), Some("minecraft"));
    }
    // The comparison is exact: the spelling the old rule let through is told apart.
    let misspelt: Vec<&str> = paths
        .iter()
        .map(|path| {
            if *path == "redstone" {
                "red_stone"
            } else {
                path
            }
        })
        .collect();
    assert_ne!(misspelt, expected);
    assert_eq!(
        misspelt
            .iter()
            .map(|path| path.replace('_', ""))
            .collect::<Vec<_>>(),
        EVIDENCE_ITEM_SYMBOLS
            .iter()
            .map(|(symbol, _)| symbol.to_lowercase())
            .collect::<Vec<_>>(),
        "the old rule (lower case, underscores stripped) accepted it"
    );
}

/// The first versions from the table history (age check), which lists the entries of
/// each build by name (the vanilla packs that first ship them), apart from the
/// ranges by number.
#[test]
fn the_first_versions_are_the_ones_of_the_table_history() {
    // (build, patterns, materials), as the table's rows.
    let history: [(SemVersion, &str, &str); 3] = [
        (
            SemVersion::new(1, 20, 0),
            "coast, dune, eye, host, raiser, rib, sentry, shaper, silence, snout, spire, tide, vex, \
             ward, wayfinder, wild",
            "quartz, iron, netherite, redstone, copper, gold, emerald, diamond, lapis, amethyst",
        ),
        (SemVersion::new(1, 21, 0), "flow, bolt", ""),
        (SemVersion::new(1, 21, 50), "", "resin"),
    ];
    let names = |list: &'static str| list.split(", ").filter(|name| !name.is_empty());
    let vanilla = TrimData::vanilla();
    let mut patterns = 0;
    let mut materials = 0;
    for (build, pattern_names, material_names) in history {
        for name in names(pattern_names) {
            let pattern = vanilla.pattern(name).expect(name);
            assert_eq!(pattern.since_engine(), Some(build.clone()), "{name}");
            patterns += 1;
        }
        for name in names(material_names) {
            let material = vanilla.material(name).expect(name);
            assert_eq!(material.since_engine(), Some(build.clone()), "{name}");
            materials += 1;
        }
    }
    // Every entry of the table is in the history, once.
    assert_eq!(patterns, vanilla.patterns().len());
    assert_eq!(materials, vanilla.materials().len());
}
