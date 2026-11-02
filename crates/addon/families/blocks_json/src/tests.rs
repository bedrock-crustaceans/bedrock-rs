//! Unit tests of the tables and scans the public API does not show.

use super::legacy::{alias, aliases};
use super::resolve::duplicate_keys;
use super::shape::{BLOCK_SHAPES, REFUSED_CONVERSIONS, block_shape_value, default_block_shape};
use super::texts::TEMPLATES;
use super::version::{Effective, Match, from_string};
use bedrock_addon_core::version::FormatVersion;

#[test]
fn every_template_has_the_number_of_arguments_its_rule_gives() {
    for (template, arguments) in TEMPLATES {
        assert_eq!(template.matches("{}").count(), *arguments, "{template}");
    }
}

#[test]
fn the_shape_table_has_vanilla_s_90_names_and_three_shared_numbers() {
    assert_eq!(BLOCK_SHAPES.len(), 90);
    let mut names: Vec<&str> = BLOCK_SHAPES.iter().map(|(name, _)| *name).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), 90, "no name twice");
    let shared = |a: &str, b: &str| block_shape_value(a) == block_shape_value(b);
    assert!(shared("barrier", "camera_facing_sprite"));
    assert!(shared("azalea", "flowering_azalea"));
    assert!(shared("glow_lichen", "sculk_vein"));
    let mut numbers: Vec<i16> = BLOCK_SHAPES.iter().map(|(_, number)| *number).collect();
    numbers.sort_unstable();
    numbers.dedup();
    assert_eq!(numbers.len(), 87);
    assert_eq!(block_shape_value("invisible"), Some(-1));
    assert_eq!(block_shape_value("sculk_shrieker"), Some(138));
    assert_eq!(block_shape_value("Block"), None, "names compare as written");
}

#[test]
fn the_refused_conversions_are_the_clients_three_pairs() {
    assert_eq!(REFUSED_CONVERSIONS, [(86, 31), (0, 31), (89, 85)]);
}

#[test]
fn registration_defaults_come_from_the_call_sites_and_the_groups() {
    // One call site each.
    assert_eq!(default_block_shape("oak_log"), 31);
    assert_eq!(default_block_shape("white_glazed_terracotta"), 86);
    assert_eq!(default_block_shape("air"), -1);
    assert_eq!(default_block_shape("bamboo_sapling"), 109);
    // Groups.
    assert_eq!(default_block_shape("white_shulker_box"), 89);
    // Not one of the legacy name's sixteen: in the group by its name alone.
    assert_eq!(default_block_shape("undyed_shulker_box"), 89);
    assert_eq!(default_block_shape("oak_sapling"), 155);
    assert_eq!(default_block_shape("element_0"), 93);
    assert_eq!(default_block_shape("element_118"), 93);
    assert_eq!(default_block_shape("smooth_stone_slab"), 67);
    assert_eq!(default_block_shape("stripped_oak_wood"), 31);
    assert_eq!(default_block_shape("light_block_7"), 106);
    assert_eq!(default_block_shape("black_candle_cake"), 137);
    assert_eq!(default_block_shape("black_candle"), 136);
    // Named by no call site: `block`.
    assert_eq!(default_block_shape("obsidian"), 0);
    assert_eq!(default_block_shape("white_carpet"), 0);
    assert_eq!(default_block_shape("element_"), 0);
    // `_shelf` is the shelves' suffix, not the bookshelf's, which has a call site of its own.
    assert_eq!(default_block_shape("chiseled_bookshelf"), 144);
    assert_eq!(default_block_shape("bookshelf"), 0);
}

#[test]
fn the_alias_table_has_the_58_gated_names_and_lava_cauldron() {
    assert_eq!(aliases().len(), 59);
    let gate = |major, minor, patch| {
        let wanted = Some(FormatVersion::new(major, minor, patch));
        let mut names: Vec<&str> = aliases()
            .iter()
            .filter(|alias| alias.gate == wanted)
            .map(|alias| alias.name)
            .collect();
        names.sort_unstable();
        names
    };
    assert_eq!(gate(1, 20, 50), ["stone"]);
    assert_eq!(gate(1, 21, 10), ["prismarine"]);
    assert_eq!(
        gate(1, 21, 20),
        [
            "anvil",
            "dirt",
            "quartz_block",
            "red_sandstone",
            "sand",
            "sandstone"
        ]
    );
    assert_eq!(gate(1, 21, 30).len(), 49);
    assert_eq!(gate(1, 21, 40), ["skull"]);
    let never: Vec<&str> = aliases()
        .iter()
        .filter(|alias| alias.gate.is_none())
        .map(|alias| alias.name)
        .collect();
    assert_eq!(never, ["lava_cauldron"]);
    assert!(alias("lava_cauldron").unwrap().removed);
    assert_eq!(aliases().iter().filter(|alias| alias.removed).count(), 1);
}

#[test]
fn thirteen_legacy_names_are_still_their_own_block() {
    let mut still: Vec<&str> = aliases()
        .iter()
        .filter(|alias| alias.is_still_a_block())
        .map(|alias| alias.name)
        .collect();
    still.sort_unstable();
    assert_eq!(
        still,
        [
            "anvil",
            "cobblestone_wall",
            "dirt",
            "prismarine",
            "purpur_block",
            "quartz_block",
            "red_sandstone",
            "sand",
            "sandstone",
            "sponge",
            "stone",
            "structure_void",
            "tnt"
        ]
    );
}

#[test]
fn post_split_names_are_in_the_client_s_listing_order() {
    let wool = alias("wool").unwrap();
    assert_eq!(wool.post_split.len(), 16);
    assert_eq!(wool.post_split[0], (0, "white_wool"));
    assert_eq!(wool.post_split[15], (15, "black_wool"));
    assert_eq!(wool.aux0, "white_wool");
    // Stripped next to plain, as vanilla's atlas item `wood` is ordered; not the six woods and
    // then the six stripped ones (the order of the data values).
    let logs = alias("wood").unwrap();
    assert_eq!(logs.post_split[0], (0, "oak_wood"));
    assert_eq!(logs.post_split[1], (1, "stripped_oak_wood"));
    assert_eq!(logs.post_split[2], (2, "spruce_wood"));
    assert_eq!(logs.post_split[11], (11, "stripped_dark_oak_wood"));
    // Infested cobblestone first, infested stone last; the data-0 block is infested stone.
    let egg = alias("monster_egg").unwrap();
    assert_eq!(egg.post_split[0], (0, "infested_cobblestone"));
    assert_eq!(egg.post_split[5], (5, "infested_stone"));
    assert_eq!(egg.default_name, "infested_cobblestone");
    assert_eq!(egg.aux0, "infested_stone");
    // Vanilla's chemistry data shows `tnt` starting at variant 0, not 2.
    assert_eq!(alias("tnt").unwrap().start_variant, 0);
    assert_eq!(alias("colored_torch_bp").unwrap().start_variant, 2);
    assert_eq!(alias("wool").unwrap().start_variant, 0);
}

#[test]
fn no_block_is_listed_twice_and_a_repeated_one_keeps_its_first_position() {
    for alias in aliases() {
        let mut names: Vec<&str> = alias.post_split.iter().map(|(_, name)| *name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), alias.post_split.len(), "{}", alias.name);
        assert!(
            alias
                .post_split
                .windows(2)
                .all(|pair| pair[0].0 < pair[1].0),
            "{}: positions rise",
            alias.name
        );
    }
    // The client names both blocks twice: positions 1 and 3 are the repeats.
    assert_eq!(
        alias("tallgrass").unwrap().post_split,
        [(0, "short_grass"), (2, "fern")]
    );
    // Named at 0 and 1; vanilla's flattened block draws element 0 (`structure_void`), not 1
    // (`structure_air`).
    assert_eq!(
        alias("structure_void").unwrap().post_split,
        [(0, "structure_void")]
    );
    // Named again at 4, past the atlas item's four elements.
    assert_eq!(alias("stonebrick").unwrap().post_split.len(), 4);
}

#[test]
fn the_gate_is_open_below_it_and_for_a_pre_release_of_it() {
    let skull = alias("skull").unwrap();
    let at = |text: &str| match from_string(text) {
        Match::Full(version) | Match::Partial(version) => version,
        Match::None { .. } => panic!("{text}"),
    };
    assert!(!skull.is_flattened_at(&at("1.21.39")));
    assert!(skull.is_flattened_at(&at("1.21.40")));
    assert!(!skull.is_flattened_at(&at("1.21.40-beta.1")));
    assert!(skull.is_flattened_at(&at("1.21.41-beta.1")));
    let never = alias("lava_cauldron").unwrap();
    assert!(!never.is_flattened_at(&Effective::release(FormatVersion::new(9999, 0, 0))));
}

#[test]
fn version_strings_match_as_vanilla_s_regex_does() {
    let full = |major, minor, patch| {
        Match::Full(Effective::release(FormatVersion::new(major, minor, patch)))
    };
    let partial =
        |major, minor| Match::Partial(Effective::release(FormatVersion::new(major, minor, 0)));
    let none = Match::None { zeroed: false };
    assert_eq!(from_string("1.21.40"), full(1, 21, 40));
    assert_eq!(from_string("0.0.0"), full(0, 0, 0));
    assert_eq!(from_string("65535.65535.65535"), full(65535, 65535, 65535));
    assert_eq!(from_string("65536.0.0"), none, "a part beyond u16");
    assert_eq!(from_string("1.21"), partial(1, 21));
    assert_eq!(from_string("2"), partial(2, 0));
    assert_eq!(from_string("*"), Match::None { zeroed: true });
    for text in [
        "",
        "1.",
        "1.21.",
        "1.21.40.1",
        "1.21.40 ",
        " 1.21.40",
        "01.2.3",
        "1.02.3",
        "1.2.03",
        "v1.2.3",
        "1.2-beta",
        "1+build",
        "1.2.3-",
        "1.2.3+",
        "1.2.3-a..b",
        "1.2.3-a b",
        "123456.0.0",
        "-1.2.3",
        "1,2,3",
    ] {
        assert_eq!(from_string(text), none, "{text:?}");
    }
    assert_eq!(
        from_string("1.21.40-beta.1+build.5"),
        Match::Full(Effective {
            version: FormatVersion::new(1, 21, 40),
            pre_release: "beta.1".to_owned(),
        })
    );
    assert_eq!(from_string("1.21.40+a-b"), full(1, 21, 40));
    assert_eq!(
        from_string("beta"),
        Match::Full(Effective {
            version: FormatVersion::new(9999, 9999, 9999),
            pre_release: "beta".to_owned(),
        })
    );
}

#[test]
fn the_key_scan_counts_root_keys_only() {
    let text = r#"{
        // "dirt": a comment is no key
        "dirt": {"dirt": 1, "x": {"dirt": 2}},
        "stone": {}, /* "stone": {} */
        "dirt": {"textures": "b"},
        "ab": 1, "ab": [{"ab": 1}, "ab"],
        "dirt": "\"dirt\": 3",
        "glass": "glass"
    }"#;
    assert_eq!(
        duplicate_keys(text),
        [("dirt".to_owned(), 3), ("ab".to_owned(), 2)]
    );
    assert_eq!(duplicate_keys(r#"{"a": 1, "b": 2}"#), []);
    assert_eq!(duplicate_keys(r#"{"a": 1, "a""#), [("a".to_owned(), 2)]);
    assert_eq!(duplicate_keys(""), []);
    // The scan ends with the root: a second object is not the file's.
    assert_eq!(duplicate_keys(r#"{"a": 1} {"a": 2}"#), []);
    assert_eq!(duplicate_keys(r#"{"a": {"b": []}} {"a": 2, "a": 3}"#), []);
    assert_eq!(
        duplicate_keys(r#"{"a": 1, "a": 2} {"a": 3}"#),
        [("a".to_owned(), 2)]
    );
}

#[test]
fn the_key_scan_finds_no_key_in_a_root_that_is_no_object() {
    // The elements of a root array are no keys, although a comma at the root's depth stands
    // before each; neither are those of an object inside it or after it.
    assert_eq!(duplicate_keys(r#"["a", "a", "a"]"#), []);
    assert_eq!(duplicate_keys(r#"[1, "a", "a"]"#), []);
    assert_eq!(duplicate_keys(r#"[{"a": 1, "a": 2}, "b", "b"]"#), []);
    assert_eq!(duplicate_keys(r#"["b", "b"] {"a": 1, "a": 2}"#), []);
    assert_eq!(duplicate_keys(r#""a""#), []);
}
