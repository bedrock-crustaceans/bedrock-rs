//! How a root key
//! becomes the blocks its entry applies to.

use super::common::{E, Found, I, file, file_at, load_one, ours, six, vanilla};
use std::fmt::Write as _;

/// The legacy-name warning for the lower-cased legacy name `name` with its post-split `blocks`.
fn old_name(key: &str, name: &str, blocks: &[&str]) -> Found {
    let names: String = blocks.iter().fold(String::new(), |mut out, block| {
        write!(out, "\n - minecraft:{block}").unwrap();
        out
    });
    vanilla(
        "blocks_json/old_name",
        E,
        key,
        &format!(
            "blocks.json file contains a block using an old name that was used for multiple blocks: minecraft:{name}. \nPlease use their new names instead: {names}"
        ),
    )
}

const SKULLS: [&str; 7] = [
    "skeleton_skull",
    "wither_skeleton_skull",
    "zombie_head",
    "player_head",
    "creeper_head",
    "dragon_head",
    "piglin_head",
];

#[test]
fn a_legacy_name_is_split_below_its_gate_and_an_error_from_it() {
    // (key, the last version below the gate, the gate, the first and last post-split block).
    let gates = [
        ("wool", "1.21.29", "1.21.30", "white_wool", "black_wool", 15),
        (
            "skull",
            "1.21.39",
            "1.21.40",
            "skeleton_skull",
            "piglin_head",
            6,
        ),
        (
            "prismarine",
            "1.21.9",
            "1.21.10",
            "prismarine",
            "prismarine_bricks",
            2,
        ),
        ("log", "1.21.29", "1.21.30", "oak_log", "jungle_log", 3),
    ];
    for (key, below, gate, first, last, last_variant) in gates {
        let entry = format!(r#""{key}": {{"textures": "t"}}"#);
        let split = load_one(&file_at(&format!("\"{below}\""), &entry));
        assert_eq!(split.textures(first), six("t:0"), "{key} at {below}");
        assert_eq!(
            split.textures(last),
            six(&format!("t:{last_variant}")),
            "{key} at {below}"
        );
        assert_eq!(
            split.problems(),
            [],
            "a version of its own: no legacy-name finding"
        );
        let flattened = load_one(&file_at(&format!("\"{gate}\""), &entry));
        if key == "prismarine" {
            // Still the name of its own block: resolved directly, no message.
            assert_eq!(flattened.textures("prismarine"), six("t"));
            assert_eq!(flattened.registry().len(), 1);
            assert_eq!(flattened.found(), []);
        } else {
            assert!(flattened.registry().is_empty(), "{key} at {gate}");
            let found = flattened.problems();
            assert_eq!(found.len(), 1, "{key} at {gate}");
            assert_eq!(
                (found[0].code, found[0].severity),
                ("blocks_json/old_name", E)
            );
        }
    }
}

#[test]
fn the_rows_of_the_fixture() {
    let entry = |key: &str| format!(r#""{key}": {{"textures": "t"}}"#);
    // `sand` at 1.21.10 and 1.21.20: split, then resolved directly (it is still a block).
    let split = load_one(&file_at(r#""1.21.10""#, &entry("sand")));
    assert_eq!(split.textures("sand"), six("t:0"));
    assert_eq!(split.textures("red_sand"), six("t:1"));
    let direct = load_one(&file_at(r#""1.21.20""#, &entry("sand")));
    assert_eq!(direct.textures("sand"), six("t"));
    assert!(direct.get("red_sand").is_none());
    assert_eq!(direct.found(), []);
    // `stone` at 1.20.40 and 1.20.50.
    let split = load_one(&file_at(r#""1.20.40""#, &entry("stone")));
    assert_eq!(split.registry().len(), 7);
    let direct = load_one(&file_at(r#""1.20.50""#, &entry("stone")));
    assert_eq!(direct.registry().len(), 1);
    assert_eq!(direct.found(), []);
    // `wool` at 1.21.30: the legacy-name warning with `minecraft:wool` and the sixteen colours.
    let wool = load_one(&file_at(r#""1.21.30""#, &entry("wool")));
    let found = wool.found();
    assert_eq!(found.len(), 1);
    assert!(found[0].text.starts_with("blocks.json file contains a block using an old name that was used for multiple blocks: minecraft:wool. \nPlease use their new names instead: \n - minecraft:white_wool\n - minecraft:orange_wool"));
    assert!(
        found[0]
            .text
            .ends_with("\n - minecraft:red_wool\n - minecraft:black_wool")
    );
    // `skull` at 1.21.30 (split) and 1.21.40 (the legacy-name warning).
    let split = load_one(&file_at(r#""1.21.30""#, &entry("skull")));
    assert_eq!(split.registry().len(), 7);
    let skull = load_one(&file_at(r#""1.21.40""#, &entry("skull")));
    assert_eq!(skull.found(), [old_name("skull", "skull", &SKULLS)]);
    // `Wool` at 1.21.30: the alias lookup is lower-cased, and the text prints the lower case.
    let upper = load_one(&file_at(r#""1.21.30""#, &entry("Wool")));
    let found = upper.found();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].at, "Wool", "the finding is at the key as written");
    assert!(
        found[0]
            .text
            .contains("multiple blocks: minecraft:wool. \n")
    );
    // And below the gate `WOOL` splits like `wool`.
    let split = load_one(&file_at(r#""1.21.29""#, &entry("WOOL")));
    assert_eq!(split.textures("white_wool"), six("t:0"));
}

#[test]
fn the_old_name_text_lists_a_block_once() {
    // The client names `short_grass` and `fern` twice each for `tallgrass`; the text names each
    // once.
    let loaded = load_one(&file(r#""tallgrass": {"textures": "t"}"#));
    assert_eq!(
        loaded.found(),
        [old_name("tallgrass", "tallgrass", &["short_grass", "fern"])]
    );
}

#[test]
fn the_split_gives_each_block_its_variant() {
    // The variant is the block's position in the client's list, from the alias's first variant.
    let loaded = load_one(&file_at(
        "[1, 1, 0]",
        r#""tallgrass": {"textures": "g"}, "tnt": {"textures": "t"}, "monster_egg": {"textures": "m"}, "wood": {"textures": "w"}, "coral": {"textures": "c"}, "structure_void": {"textures": "v"}"#,
    ));
    // The client names `short_grass` at 0 and 1 and `fern` at 2 and 3: the first one counts,
    // as vanilla's flattened `structure_void` shows (element 0, not 1).
    assert_eq!(loaded.textures("short_grass"), six("g:0"));
    assert_eq!(loaded.textures("fern"), six("g:2"));
    assert_eq!(loaded.textures("structure_void"), six("v:0"));
    // `tnt` starts at variant 0, as vanilla's chemistry data shows.
    assert_eq!(loaded.textures("tnt"), six("t:0"));
    assert_eq!(loaded.textures("underwater_tnt"), six("t:1"));
    // The order the client lists the names in (vanilla's atlas order), not data-value order:
    // infested cobblestone first, the stripped wood next to the plain one.
    assert_eq!(loaded.textures("infested_cobblestone"), six("m:0"));
    assert_eq!(loaded.textures("infested_stone_bricks"), six("m:1"));
    assert_eq!(
        loaded.textures("infested_chiseled_stone_bricks"),
        six("m:4")
    );
    assert_eq!(loaded.textures("infested_stone"), six("m:5"));
    assert_eq!(loaded.textures("oak_wood"), six("w:0"));
    assert_eq!(loaded.textures("stripped_oak_wood"), six("w:1"));
    assert_eq!(loaded.textures("spruce_wood"), six("w:2"));
    assert_eq!(loaded.textures("dark_oak_wood"), six("w:10"));
    assert_eq!(loaded.textures("stripped_dark_oak_wood"), six("w:11"));
    assert_eq!(loaded.textures("horn_coral"), six("c:4"));
    assert_eq!(loaded.textures("dead_tube_coral"), six("c:5"));
    assert_eq!(loaded.problems(), []);
}

#[test]
fn the_split_rewrites_texture_names_and_keeps_every_other_field() {
    let loaded = load_one(&file_at(
        "[1, 1, 0]",
        r#""dirt": {
            "textures": {"up": "top", "down": "bottom", "side": "side"},
            "carried_textures": "held",
            "sound": "gravel", "isotropic": true, "ambient_occlusion_exponent": 0.5
        }"#,
    ));
    for (block, variant) in [("dirt", 0), ("coarse_dirt", 1)] {
        let document = loaded.block(block);
        let v = variant;
        assert_eq!(
            document.face_textures().map(str::to_owned),
            [
                format!("top:{v}"),
                format!("bottom:{v}"),
                format!("side:{v}"),
                format!("side:{v}"),
                format!("side:{v}"),
                format!("side:{v}")
            ]
        );
        assert_eq!(
            document.carried_face_textures(),
            [format!("held:{v}").as_str(); 6]
        );
        assert_eq!(document.sound(), "gravel");
        assert_eq!(document.isotropic_mask(), 0xff);
        assert_eq!(document.ambient_occlusion_exponent, Some(0.5));
    }
    // A face that is not a string is left alone, and the texture list is then refused for
    // every post-split block (once per block, as the client logs it).
    let loaded = load_one(&file_at(
        "[1, 1, 0]",
        r#""sand": {"textures": {"up": "a", "down": 5, "side": "b"}, "carried_textures": ""}"#,
    ));
    assert_eq!(loaded.block("sand").textures, None);
    assert_eq!(loaded.block("red_sand").carried_face_textures(), [":1"; 6]);
    let found = loaded.problems();
    let texts: Vec<&str> = found.iter().map(|f| f.text.as_str()).collect();
    assert_eq!(
        texts,
        [
            "'minecraft:sand' | Invalid or incomplete texture list.",
            "'minecraft:red_sand' | Invalid or incomplete texture list."
        ]
    );
    // `null` and absent textures are not rewritten.
    let loaded = load_one(&file_at(
        "[1, 1, 0]",
        r#""sponge": {"textures": null, "sound": "x"}"#,
    ));
    assert_eq!(loaded.block("wet_sponge").textures, None);
    assert_eq!(loaded.block("wet_sponge").sound(), "x");
    assert_eq!(loaded.found(), []);
}

#[test]
fn lava_cauldron_is_dropped_at_every_version_deviation() {
    // `vanilla = "silent"`: never flattened, and removed, so the entry applies to nothing.
    for version in [r#""1.1.0""#, r#""1.21.40""#, "[1, 1, 0]", r#""1.20.0""#] {
        for key in ["lava_cauldron", "Lava_Cauldron"] {
            let loaded = load_one(&file_at(
                version,
                &format!(r#""{key}": {{"textures": "c"}}"#),
            ));
            assert!(loaded.registry().is_empty(), "{version}");
            assert_eq!(
                loaded.found(),
                [ours("core/ignored_member", I, key)],
                "{version}"
            );
        }
    }
    // The namespaced key bypasses the legacy handling: an ordinary name.
    let loaded = load_one(&file(r#""minecraft:lava_cauldron": {"textures": "d"}"#));
    assert_eq!(loaded.textures("lava_cauldron"), six("d"));
    assert_eq!(loaded.found(), []);
}

#[test]
fn the_quirks_of_key_resolution() {
    let loaded = load_one(&file(
        r#""chalkboard": {"textures": "c"},
           "red_mushroom_block": {"textures": "r"},
           "minecraft:wool": {"textures": "w"},
           "x": 3"#,
    ));
    // `chalkboard` is skipped; the mushroom block also feeds the stem; the
    // namespaced legacy key applies to the data-0 block only; `x` is not an object.
    assert_eq!(loaded.registry().len(), 3);
    assert_eq!(loaded.textures("red_mushroom_block"), six("r"));
    assert_eq!(loaded.textures("mushroom_stem"), six("r"));
    assert_eq!(loaded.textures("white_wool"), six("w"));
    assert!(loaded.get("orange_wool").is_none());
    assert!(loaded.get("chalkboard").is_none());
    assert_eq!(
        loaded.found(),
        [
            ours("core/ignored_member", I, "chalkboard"),
            ours("core/identifier_namespaced", I, "minecraft:wool"),
            ours("core/shadowed_definition", I, "red_mushroom_block"),
            ours("core/invalid_type", I, "x"),
        ]
    );
}

#[test]
fn a_member_that_is_not_an_object_is_skipped_deviation() {
    // `vanilla = "silent"`, for every JSON type but an object.
    for value in ["3", r#""stone""#, "true", "null", "[]", "[{}]", "1.5"] {
        let loaded = load_one(&file(&format!(r#""obsidian": {value}"#)));
        assert!(loaded.registry().is_empty(), "{value}");
        assert_eq!(
            loaded.found(),
            [ours("core/invalid_type", I, "obsidian")],
            "{value}"
        );
    }
    // An empty object is an entry: the block is registered with nothing set.
    let loaded = load_one(&file(r#""obsidian": {}"#));
    assert_eq!(loaded.block("obsidian").textures, None);
    assert_eq!(
        loaded.block("obsidian").face_textures(),
        ["missing_tile"; 6]
    );
    assert_eq!(loaded.found(), []);
}

#[test]
fn both_mushroom_blocks_feed_the_stem_in_key_order_deviation() {
    // `vanilla = "applied silently"`. `brown…` < `mushroom_stem` < `red…`: the stem's own
    // entry is overridden by the red block's.
    let loaded = load_one(&file(
        r#""red_mushroom_block": {"textures": "r", "sound": "red"},
           "mushroom_stem": {"textures": "s", "carried_textures": "stem"},
           "brown_mushroom_block": {"textures": "b", "isotropic": true}"#,
    ));
    let stem = loaded.block("mushroom_stem");
    assert_eq!(stem.face_textures(), ["r"; 6]);
    assert_eq!(
        stem.carried_face_textures(),
        ["stem"; 6],
        "a field only the stem sets"
    );
    assert_eq!(stem.isotropic_mask(), 0xff, "from the brown block");
    assert_eq!(stem.sound(), "red");
    assert_eq!(loaded.textures("brown_mushroom_block"), six("b"));
    assert_eq!(loaded.block("brown_mushroom_block").sound(), "normal");
    assert_eq!(
        loaded.found(),
        [
            ours("core/shadowed_definition", I, "brown_mushroom_block"),
            ours("core/shadowed_definition", I, "red_mushroom_block"),
        ]
    );
    // Case does not matter, at any version.
    let loaded = load_one(&file_at(
        "[1, 1, 0]",
        r#""Brown_Mushroom_Block": {"textures": "b"}"#,
    ));
    assert_eq!(loaded.textures("mushroom_stem"), six("b"));
}

#[test]
fn the_entries_of_a_file_are_applied_in_bytewise_key_order() {
    // `Stone` < `minecraft:stone` < `stone` (upper case sorts first): the last one wins.
    let loaded = load_one(&file(
        r#""stone": {"textures": "lower"},
           "Stone": {"textures": "upper", "sound": "upper"},
           "minecraft:stone": {"textures": "namespaced", "carried_textures": "ns"}"#,
    ));
    let stone = loaded.block("stone");
    assert_eq!(stone.face_textures(), ["lower"; 6]);
    assert_eq!(stone.sound(), "upper");
    assert_eq!(stone.carried_face_textures(), ["ns"; 6]);
    assert_eq!(loaded.registry().len(), 1);
    assert_eq!(
        stone.origin().key.as_deref(),
        Some("stone"),
        "the last key applied"
    );
    assert_eq!(loaded.found(), [], "two keys of one block are no duplicate");
    // `_` sorts between the upper and the lower case: `aAb` < `a_b` < `aab`, and the two
    // spellings of `aab` are one block, the lower-case key applied last.
    let loaded = load_one(&file(
        r#""a_b": {"textures": "underscore"}, "aab": {"textures": "lower"}, "aAb": {"textures": "upper", "sound": "upper"}"#,
    ));
    assert_eq!(loaded.registry().len(), 2);
    assert_eq!(loaded.textures("aab"), six("lower"));
    assert_eq!(loaded.block("aab").sound(), "upper");
}

#[test]
fn a_bare_key_is_a_vanilla_name_and_a_namespaced_key_is_taken_as_written() {
    let loaded = load_one(&file(
        r#""seaLantern": {"textures": "a"},
           "demo:Lamp": {"textures": "b"},
           "demo:lamp": {"textures": "c"},
           "Minecraft:Glass": {"textures": "d"}"#,
    ));
    // The registry lower-cases a bare name and gives it the vanilla namespace.
    assert_eq!(loaded.textures("minecraft:sealantern"), six("a"));
    assert_eq!(
        loaded.block("minecraft:sealantern").origin().key.as_deref(),
        Some("seaLantern"),
        "the key as written"
    );
    // A custom block's name keeps its case.
    assert_eq!(loaded.textures("demo:Lamp"), six("b"));
    assert_eq!(loaded.textures("demo:lamp"), six("c"));
    assert_eq!(loaded.textures("minecraft:glass"), six("d"));
    assert_eq!(loaded.registry().len(), 4);
    assert_eq!(loaded.found(), []);
}

#[test]
fn a_namespaced_legacy_key_names_the_data_0_block_deviation() {
    // `vanilla = "applied silently"` (the block is inferred); an information.
    let loaded = load_one(&file(
        r#""minecraft:monster_egg": {"textures": "m"}, "MINECRAFT:Log": {"textures": "l"}, "minecraft:stone": {"textures": "s"}, "demo:wool": {"textures": "w"}"#,
    ));
    // Data value 0, which is not always the first name the client lists.
    assert_eq!(loaded.textures("infested_stone"), six("m"));
    assert_eq!(loaded.textures("oak_log"), six("l"));
    // A legacy name that is still a block, and another namespace, are ordinary names.
    assert_eq!(loaded.textures("stone"), six("s"));
    assert_eq!(loaded.textures("demo:wool"), six("w"));
    assert_eq!(loaded.registry().len(), 4);
    assert_eq!(
        loaded.found(),
        [
            ours("core/identifier_namespaced", I, "MINECRAFT:Log"),
            ours("core/identifier_namespaced", I, "minecraft:monster_egg"),
        ]
    );
    // At 1.1.0 too: the namespaced key is never split.
    let loaded = load_one(&file_at(
        "[1, 1, 0]",
        r#""minecraft:wool": {"textures": "w"}"#,
    ));
    assert_eq!(loaded.registry().len(), 1);
    assert_eq!(loaded.textures("white_wool"), six("w"));
}

#[test]
fn a_repeated_key_keeps_its_last_value_with_a_warning_per_earlier_one() {
    let loaded = load_one(
        r#"{"format_version": "1.21.40",
            "dirt": {"textures": "a", "sound": "first"},
            "glass": {"textures": "g"},
            "dirt": {"textures": "b"}}"#,
    );
    // The last value whole: the first one's `sound` is gone with it.
    assert_eq!(loaded.textures("dirt"), six("b"));
    assert_eq!(loaded.block("dirt").sound(), "normal");
    assert_eq!(
        loaded.found(),
        [ours("core/duplicate_identifier", I, "dirt")]
    );
    // Three times: two earlier values, two warnings.
    let loaded = load_one(
        r#"{"format_version": "1.21.40", "dirt": {"textures": "a"}, "dirt": {"textures": "b"},
            "glass": {"textures": "g"}, "glass": 5, "dirt": {"textures": "c"}}"#,
    );
    assert_eq!(loaded.textures("dirt"), six("c"));
    assert!(
        loaded.get("glass").is_none(),
        "the last `glass` is not an object"
    );
    assert_eq!(
        loaded.found(),
        [
            ours("core/duplicate_identifier", I, "dirt"),
            ours("core/duplicate_identifier", I, "dirt"),
            ours("core/duplicate_identifier", I, "glass"),
            ours("core/invalid_type", I, "glass"),
        ]
    );
    // A repeated `format_version`: the last one is the file's.
    let loaded = load_one(
        r#"{"format_version": "1.26.0", "wool": {"textures": "w"}, "format_version": "1.21.29"}"#,
    );
    assert_eq!(loaded.textures("white_wool"), six("w:0"));
    assert_eq!(
        loaded.found(),
        [ours("core/duplicate_identifier", I, "format_version")]
    );
    // A repeated member inside an entry is not a repeated key of the file.
    let loaded = load_one(&file(r#""dirt": {"textures": "a", "textures": "b"}"#));
    assert_eq!(loaded.textures("dirt"), six("b"));
    assert_eq!(loaded.found(), []);
}
