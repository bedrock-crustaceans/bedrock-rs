//! The six members of an entry.

use bedrock_addon::families::blocks_json::{
    BLOCK_SHAPES, FaceBools3, FaceBools6, FaceTextures, Isotropic, block_shape_value,
    default_block_shape,
};

use super::common::{E, Found, I, W, base, faces, file, load, load_one, ours, six, user, vanilla};

/// The block `obsidian` of a file whose one entry is `obsidian: {members}`.
fn obsidian(members: &str) -> (super::common::Loaded, Vec<Found>) {
    let loaded = load_one(&file(&format!(r#""obsidian": {{{members}}}"#)));
    let found = loaded.found();
    (loaded, found)
}

fn bad_textures(key: &str) -> Found {
    vanilla(
        "blocks_json/bad_textures",
        W,
        &format!("{key}/textures"),
        &format!("'minecraft:{key}' | Invalid or incomplete texture list."),
    )
}

#[test]
fn the_three_texture_forms_are_applied() {
    let (loaded, found) = obsidian(r#""textures": "t""#);
    assert_eq!(loaded.block("obsidian").textures, Some(faces(["t"; 6])));
    assert_eq!(found, []);
    let (loaded, found) = obsidian(r#""textures": {"up": "u", "down": "d", "side": "s"}"#);
    assert_eq!(
        loaded.block("obsidian").textures,
        Some(faces(["u", "d", "s", "s", "s", "s"]))
    );
    assert_eq!(found, []);
    let (loaded, found) = obsidian(
        r#""textures": {"east": "e", "west": "w", "south": "s", "north": "n", "down": "d", "up": "u"}"#,
    );
    assert_eq!(
        loaded.block("obsidian").textures,
        Some(faces(["u", "d", "n", "s", "w", "e"]))
    );
    assert_eq!(found, []);
    // An empty name is a name.
    let (loaded, found) = obsidian(r#""textures": """#);
    assert_eq!(loaded.textures("obsidian"), six(""));
    assert_eq!(found, []);
}

#[test]
fn any_other_texture_shape_is_refused_with_vanilla_s_warning() {
    for textures in [
        // Seven members (vanilla's own `azalea`).
        r#"{"up": "a", "down": "a", "side": "a", "north": "a", "south": "a", "east": "a", "west": "a"}"#,
        // Three members that are not up, down and side.
        r#"{"up": "a", "down": "a", "north": "a"}"#,
        // A face that is not a string.
        r#"{"up": "a", "down": "a", "side": 1}"#,
        r#"{"up": "a", "down": "a", "side": null}"#,
        // Six members, one of them not a face.
        r#"{"up": "a", "down": "a", "north": "a", "south": "a", "west": "a", "side": "a"}"#,
        r#"{"up": "a", "down": "a", "north": "a", "south": "a", "west": "a", "east": 5}"#,
        // Other member counts.
        "{}",
        r#"{"up": "a"}"#,
        r#"{"up": "a", "down": "a"}"#,
        r#"{"up": "a", "down": "a", "side": "a", "north": "a"}"#,
        r#"{"up": "a", "down": "a", "north": "a", "south": "a", "west": "a"}"#,
        // Other JSON types.
        r#"["a"]"#,
        "5",
        "true",
    ] {
        let (loaded, found) = obsidian(&format!(r#""textures": {textures}, "sound": "glass""#));
        assert_eq!(loaded.block("obsidian").textures, None, "{textures}");
        assert_eq!(
            loaded.block("obsidian").sound(),
            "glass",
            "the rest of the entry stays"
        );
        assert_eq!(found, [bad_textures("obsidian")], "{textures}");
    }
    // `null` is "not set": no warning.
    let (loaded, found) = obsidian(r#""textures": null"#);
    assert_eq!(loaded.block("obsidian").textures, None);
    assert_eq!(found, []);
}

#[test]
fn carried_textures_take_the_same_forms_with_their_own_text() {
    let (loaded, found) =
        obsidian(r#""textures": "t", "carried_textures": {"up": "u", "down": "d", "side": "s"}"#);
    let block = loaded.block("obsidian");
    assert_eq!(
        block.carried_textures,
        Some(faces(["u", "d", "s", "s", "s", "s"]))
    );
    assert_eq!(
        block.carried_face_textures(),
        ["u", "d", "s", "s", "s", "s"]
    );
    assert_eq!(found, []);
    let (loaded, found) = obsidian(r#""textures": "t", "carried_textures": 5"#);
    // Unset carried textures are the block's own.
    assert_eq!(loaded.block("obsidian").carried_textures, None);
    assert_eq!(loaded.block("obsidian").carried_face_textures(), ["t"; 6]);
    assert_eq!(
        found,
        [vanilla(
            "blocks_json/bad_carried_textures",
            W,
            "obsidian/carried_textures",
            "'minecraft:obsidian' | Invalid or incomplete carried_textures list."
        )]
    );
    // Neither set: the missing texture, carried too.
    let (loaded, _) = obsidian(r#""sound": "x""#);
    assert_eq!(
        loaded.block("obsidian").face_textures(),
        ["missing_tile"; 6]
    );
    assert_eq!(
        loaded.block("obsidian").carried_face_textures(),
        ["missing_tile"; 6]
    );
}

#[test]
fn the_texts_name_the_block_not_the_key() {
    let loaded = load_one(&file(
        r#""Obsidian": {"textures": 5}, "demo:Lamp": {"textures": 5}"#,
    ));
    let texts: Vec<(String, String)> = loaded.found().into_iter().map(|f| (f.at, f.text)).collect();
    assert_eq!(
        texts,
        [
            (
                "Obsidian/textures".to_owned(),
                "'minecraft:obsidian' | Invalid or incomplete texture list.".to_owned()
            ),
            (
                "demo:Lamp/textures".to_owned(),
                "'demo:Lamp' | Invalid or incomplete texture list.".to_owned()
            ),
        ]
    );
}

#[test]
fn the_isotropic_forms_and_their_masks() {
    let l5 = || ours("core/invalid_value", I, "obsidian/isotropic");
    let cases: [(&str, Option<u8>, bool); 16] = [
        ("true", Some(0xff), false),
        ("false", Some(0), false),
        (r#"{"up": true, "down": true}"#, Some(0x03), false),
        (r#"{"side": true}"#, Some(0x3c), false),
        (r#"{"north": true, "east": true}"#, Some(0x24), false),
        // `side` next to a face of the six-face map: the three-face map is read, `north` is not.
        (r#"{"side": true, "north": true}"#, Some(0x3c), true),
        (
            r#"{"up": true, "down": false, "side": true}"#,
            Some(0x3e),
            false,
        ),
        (r#"{"side": false, "up": true}"#, Some(0x02), false),
        (
            r#"{"down": true, "up": true, "north": true, "south": true, "west": true, "east": true}"#,
            Some(0x3f),
            false,
        ),
        (r#"{"south": true, "west": true}"#, Some(0x18), false),
        // An unknown face, a face that is not a boolean: the mask is still set from the rest.
        (r#"{"up": true, "top": true}"#, Some(0x02), true),
        (r#"{"up": true, "down": "yes"}"#, Some(0x02), true),
        (r#"{"up": 1}"#, Some(0), true),
        // `"side": null` selects the six-face map, and is a member it does not match.
        (r#"{"side": null, "north": true}"#, Some(0x04), true),
        ("{}", Some(0), false),
        ("null", None, false),
    ];
    for (isotropic, mask, warned) in cases {
        let (loaded, found) = obsidian(&format!(r#""isotropic": {isotropic}"#));
        let block = loaded.block("obsidian");
        assert_eq!(block.isotropic.map(Isotropic::mask), mask, "{isotropic}");
        assert_eq!(block.isotropic_mask(), mask.unwrap_or(0), "{isotropic}");
        assert_eq!(
            found,
            if warned { vec![l5()] } else { vec![] },
            "{isotropic}"
        );
    }
}

#[test]
fn an_isotropic_of_another_type_changes_nothing_deviation() {
    // `vanilla = "silent"`.
    for isotropic in [r#""yes""#, "1", "0", "[true]", "1.5"] {
        let (loaded, found) = obsidian(&format!(r#""isotropic": {isotropic}"#));
        assert_eq!(loaded.block("obsidian").isotropic, None, "{isotropic}");
        assert_eq!(
            found,
            [ours("core/invalid_type", I, "obsidian/isotropic")],
            "{isotropic}"
        );
    }
}

#[test]
fn the_isotropic_value_shows_which_map_was_read() {
    let (loaded, _) = obsidian(r#""isotropic": {"side": true, "up": true}"#);
    // The merged block holds the six faces.
    assert_eq!(
        loaded.block("obsidian").isotropic,
        Some(Isotropic::SixFaces(FaceBools6 {
            up: true,
            down: false,
            north: true,
            south: true,
            west: true,
            east: true,
        }))
    );
    assert_eq!(
        Isotropic::UpDownSide(FaceBools3 {
            up: true,
            down: false,
            side: true
        })
        .mask(),
        0x3e
    );
    assert_eq!(
        Isotropic::All(true).to_six_faces(),
        Isotropic::All(true),
        "0xff is more than six faces"
    );
    // Each bit of the six-face map.
    let bit = |faces: FaceBools6| Isotropic::SixFaces(faces).mask();
    let none = FaceBools6::default();
    assert_eq!(bit(FaceBools6 { down: true, ..none }), 0x01);
    assert_eq!(bit(FaceBools6 { up: true, ..none }), 0x02);
    assert_eq!(
        bit(FaceBools6 {
            north: true,
            ..none
        }),
        0x04
    );
    assert_eq!(
        bit(FaceBools6 {
            south: true,
            ..none
        }),
        0x08
    );
    assert_eq!(bit(FaceBools6 { west: true, ..none }), 0x10);
    assert_eq!(bit(FaceBools6 { east: true, ..none }), 0x20);
    let bit3 = |faces: FaceBools3| Isotropic::UpDownSide(faces).mask();
    let none = FaceBools3::default();
    assert_eq!(bit3(FaceBools3 { down: true, ..none }), 0x01);
    assert_eq!(bit3(FaceBools3 { up: true, ..none }), 0x02);
    assert_eq!(bit3(FaceBools3 { side: true, ..none }), 0x3c);
    assert_eq!(Isotropic::All(false).mask(), 0);
    assert_eq!(
        FaceTextures::All("t".to_owned()).to_six_faces(),
        faces(["t"; 6])
    );
}

fn bad_shape(key: &str, block: &str) -> Found {
    vanilla(
        "blocks_json/bad_blockshape",
        W,
        &format!("{key}/blockshape"),
        &format!("'{block}' | Invalid blockshape name."),
    )
}

#[test]
fn blockshape_is_a_creator_pack_s_to_set_on_a_vanilla_block() {
    let loaded = load(&[
        base(&file(
            r#""dirt": {"blockshape": "block", "textures": "d"}, "stone": {"textures": "s"}"#,
        )),
        user(&file(
            r#""glass": {"blockshape": "cross_texture"},
               "obsidian": {"blockshape": "tree"},
               "x": {"blockshape": "nope"},
               "ns:foo": {"blockshape": "block", "textures": "f"}"#,
        )),
    ]);
    assert_eq!(
        loaded.block("glass").blockshape.as_deref(),
        Some("cross_texture")
    );
    assert_eq!(loaded.block("glass").blockshape_value(), Some(1));
    // `obsidian` is registered with `block` (0): `tree` is a refused conversion.
    assert_eq!(loaded.block("obsidian").blockshape, None);
    assert_eq!(loaded.block("x").blockshape, None);
    // The base-game entry with a `blockshape` is skipped whole: `dirt` has no textures.
    assert!(loaded.get("dirt").is_none());
    assert_eq!(loaded.textures("stone"), six("s"));
    // The custom block keeps its other fields.
    assert_eq!(loaded.block("ns:foo").blockshape, None);
    assert_eq!(loaded.textures("ns:foo"), six("f"));
    assert_eq!(
        loaded.found(),
        [
            vanilla(
                "blocks_json/base_game_blockshape",
                E,
                "dirt",
                "Unexpected: blocks.json in a base game pack specifies a blockshape. Json entry: 'dirt'"
            ),
            vanilla(
                "blocks_json/custom_blockshape",
                E,
                "ns:foo/blockshape",
                "'ns:foo' | Cannot set 'blockshape' on custom blocks."
            )
            .in_pack(1),
            bad_shape("obsidian", "minecraft:obsidian").in_pack(1),
            bad_shape("x", "minecraft:x").in_pack(1),
        ]
    );
}

#[test]
fn a_blockshape_that_is_no_name_is_refused() {
    for shape in ["5", "true", "[]", "{}", r#""Block""#, r#""""#, r#""tree ""#] {
        let (loaded, found) = obsidian(&format!(r#""blockshape": {shape}"#));
        assert_eq!(loaded.block("obsidian").blockshape, None, "{shape}");
        assert_eq!(
            found,
            [bad_shape("obsidian", "minecraft:obsidian")],
            "{shape}"
        );
    }
    // `null` on a vanilla block is "not set".
    let (loaded, found) = obsidian(r#""blockshape": null"#);
    assert_eq!(loaded.block("obsidian").blockshape, None);
    assert_eq!(found, []);
    // Every one of the 90 names is taken by a block whose shape does not refuse it.
    assert_eq!(BLOCK_SHAPES.len(), 90);
    for (name, value) in BLOCK_SHAPES {
        let loaded = load_one(&file(&format!(r#""oak_log": {{"blockshape": "{name}"}}"#)));
        assert_eq!(loaded.block("oak_log").blockshape.as_deref(), Some(*name));
        assert_eq!(loaded.block("oak_log").blockshape_value(), Some(*value));
        assert_eq!(block_shape_value(name), Some(*value));
        assert_eq!(loaded.found(), [], "{name}");
    }
}

#[test]
fn the_three_refused_conversions_compare_shape_numbers() {
    let loaded = load_one(&file(
        r#""barrier": {"blockshape": "camera_facing_sprite"},
           "white_glazed_terracotta": {"blockshape": "tree"},
           "oak_log": {"blockshape": "tree"},
           "white_shulker_box": {"blockshape": "command_block"}"#,
    ));
    // 106 → 106: a name of the same number.
    assert_eq!(
        loaded.block("barrier").blockshape.as_deref(),
        Some("camera_facing_sprite")
    );
    assert_eq!(default_block_shape("barrier"), 106);
    // 86 → 31 refused.
    assert_eq!(default_block_shape("white_glazed_terracotta"), 86);
    assert_eq!(loaded.block("white_glazed_terracotta").blockshape, None);
    // 31 → 31 applied.
    assert_eq!(loaded.block("oak_log").blockshape.as_deref(), Some("tree"));
    // 89 → 85 refused.
    assert_eq!(default_block_shape("white_shulker_box"), 89);
    assert_eq!(loaded.block("white_shulker_box").blockshape, None);
    assert_eq!(
        loaded.found(),
        [
            bad_shape(
                "white_glazed_terracotta",
                "minecraft:white_glazed_terracotta"
            ),
            bad_shape("white_shulker_box", "minecraft:white_shulker_box"),
        ]
    );
}

#[test]
fn each_refused_conversion_is_refused_only_from_its_own_shape() {
    let shape = |block: &str, requested: &str| {
        let loaded = load_one(&file(&format!(
            r#""{block}": {{"blockshape": "{requested}"}}"#
        )));
        loaded.block(block).blockshape.clone()
    };
    // `tree` (31): refused from 0 and from 86, taken from 89 and from any other shape.
    assert_eq!(shape("obsidian", "tree"), None);
    assert_eq!(shape("white_glazed_terracotta", "tree"), None);
    assert_eq!(shape("white_shulker_box", "tree").as_deref(), Some("tree"));
    assert_eq!(
        shape("glass_pane", "tree").is_some(),
        default_block_shape("glass_pane") != 0
    );
    // `command_block` (85): refused from 89 only.
    assert_eq!(shape("white_shulker_box", "command_block"), None);
    assert_eq!(
        shape("obsidian", "command_block").as_deref(),
        Some("command_block")
    );
    assert_eq!(
        shape("white_glazed_terracotta", "command_block").as_deref(),
        Some("command_block")
    );
    // Another shape is taken from each of the three.
    for block in ["obsidian", "white_glazed_terracotta", "white_shulker_box"] {
        assert_eq!(
            shape(block, "cross_texture").as_deref(),
            Some("cross_texture"),
            "{block}"
        );
    }
}

#[test]
fn the_current_shape_is_what_a_lower_entry_made_it() {
    // "Current" is the registration default or a lower pack's value.
    let loaded = load(&[
        user(&file(r#""obsidian": {"blockshape": "cross_texture"}"#)),
        user(&file(r#""obsidian": {"blockshape": "tree"}"#)),
    ]);
    assert_eq!(loaded.block("obsidian").blockshape.as_deref(), Some("tree"));
    assert_eq!(loaded.found(), [], "1 → 31 is no refused conversion");
    // The other way round: the lower `tree` is refused (0 → 31), the higher shape is taken.
    let loaded = load(&[
        user(&file(r#""obsidian": {"blockshape": "tree"}"#)),
        user(&file(r#""obsidian": {"blockshape": "cross_texture"}"#)),
    ]);
    assert_eq!(
        loaded.block("obsidian").blockshape.as_deref(),
        Some("cross_texture")
    );
    assert_eq!(
        loaded.found(),
        [bad_shape("obsidian", "minecraft:obsidian")]
    );
    // A lower pack made it a shulker box (89): `command_block` is refused in the pack above,
    // and the lower shape stays.
    let loaded = load(&[
        user(&file(r#""obsidian": {"blockshape": "shulker_box"}"#)),
        user(&file(r#""obsidian": {"blockshape": "command_block"}"#)),
        user(&file(r#""obsidian": {"sound": "x"}"#)),
    ]);
    assert_eq!(
        loaded.block("obsidian").blockshape.as_deref(),
        Some("shulker_box")
    );
    assert_eq!(
        loaded.found(),
        [bad_shape("obsidian", "minecraft:obsidian").in_pack(1)]
    );
    // Within one file, in key order: `Obsidian` makes it terracotta (86), then `obsidian`
    // asks for `tree`.
    let loaded = load_one(&file(
        r#""obsidian": {"blockshape": "tree"}, "Obsidian": {"blockshape": "terracotta"}"#,
    ));
    assert_eq!(
        loaded.block("obsidian").blockshape.as_deref(),
        Some("terracotta")
    );
    assert_eq!(
        loaded.found(),
        [bad_shape("obsidian", "minecraft:obsidian")]
    );
}

#[test]
fn members_the_client_never_reads_deviation() {
    // `vanilla = "silent"`: an information for every member, the two with a fix too
    // (the client reads neither and says nothing).
    let (loaded, found) = obsidian(
        r#""brightness_gamma": 0.7, "sounds": "stone", "client_entity": {"a": 1}, "textures": "t""#,
    );
    let block = loaded.block("obsidian");
    assert_eq!(
        block.face_textures(),
        ["t"; 6],
        "the merged result is unaffected"
    );
    assert_eq!(block.sound(), "normal", "`sounds` is not `sound`");
    assert_eq!(block.ambient_occlusion_exponent(), 1.0);
    assert_eq!(
        found,
        [
            ours("core/ignored_member", I, "obsidian/brightness_gamma"),
            ours("core/ignored_member", I, "obsidian/sounds"),
            ours("core/ignored_member", I, "obsidian/client_entity"),
        ]
    );
    // They are kept, as written.
    let extra: Vec<&str> = block.extra.iter().map(|(name, _)| name).collect();
    assert_eq!(extra, ["brightness_gamma", "sounds", "client_entity"]);
    // A member named like a read one in another case is unread too.
    let (_, found) = obsidian(r#""Textures": "t", "extra": 1"#);
    assert_eq!(
        found,
        [
            ours("core/ignored_member", I, "obsidian/Textures"),
            ours("core/ignored_member", I, "obsidian/extra"),
        ]
    );
    // Once per entry, however many blocks the entry is split onto, and for a skipped entry.
    let loaded = load_one(&file(r#""wool": {"textures": "w", "sounds": "cloth"}"#));
    let found = loaded.found();
    assert_eq!(found.len(), 2);
    assert_eq!(found[0], ours("core/ignored_member", I, "wool/sounds"));
    let loaded = load_one(&super::common::file_at(
        "[1, 1, 0]",
        r#""wool": {"textures": "w", "sounds": "cloth"}"#,
    ));
    assert_eq!(
        loaded.found(),
        [ours("core/ignored_member", I, "wool/sounds")]
    );
    assert_eq!(loaded.registry().len(), 16);
}

#[test]
fn values_of_odd_types() {
    let loaded = load(&[
        base(&file(r#""dirt": {"blockshape": null, "textures": "d"}"#)),
        user(&file(
            r#""obsidian": {"sound": 5, "ambient_occlusion_exponent": true},
               "glass": {"sound": false, "isotropic": {"side": null, "north": true}},
               "ns:foo": {"blockshape": null}"#,
        )),
    ]);
    let obsidian = loaded.block("obsidian");
    assert_eq!(
        obsidian.sound.as_deref(),
        Some(""),
        "a number is the empty string"
    );
    assert_eq!(obsidian.ambient_occlusion_exponent, Some(1.0));
    let glass = loaded.block("glass");
    assert_eq!(glass.sound.as_deref(), Some("false"));
    assert_eq!(glass.isotropic_mask(), 0x04);
    // A `null` blockshape is a member all the same: reported in the base-game pack and on the
    // custom block.
    assert!(loaded.get("dirt").is_none());
    assert_eq!(
        loaded.found(),
        [
            vanilla(
                "blocks_json/base_game_blockshape",
                E,
                "dirt",
                "Unexpected: blocks.json in a base game pack specifies a blockshape. Json entry: 'dirt'"
            ),
            ours("core/invalid_value", I, "glass/isotropic").in_pack(1),
            ours("core/invalid_type", I, "glass/sound").in_pack(1),
            vanilla(
                "blocks_json/custom_blockshape",
                E,
                "ns:foo/blockshape",
                "'ns:foo' | Cannot set 'blockshape' on custom blocks."
            )
            .in_pack(1),
            ours(
                "core/invalid_type",
                I,
                "obsidian/ambient_occlusion_exponent"
            )
            .in_pack(1),
            ours("core/invalid_type", I, "obsidian/sound").in_pack(1),
        ]
    );
}

#[test]
fn the_base_game_check_runs_before_the_key_is_looked_at() {
    // It fires for an unknown key, `chalkboard` and `lava_cauldron` too, and not in a user pack.
    for key in [
        "not_a_block",
        "chalkboard",
        "lava_cauldron",
        "wool",
        "demo:x",
    ] {
        let entry = format!(r#""{key}": {{"blockshape": "block"}}"#);
        let loaded = load(&[base(&file(&entry))]);
        let found = loaded.found();
        assert_eq!(found.len(), 1, "{key}: {found:#?}");
        assert_eq!(
            found[0].text,
            format!(
                "Unexpected: blocks.json in a base game pack specifies a blockshape. Json entry: '{key}'"
            )
        );
        assert_eq!(
            (found[0].code, found[0].severity),
            ("blocks_json/base_game_blockshape", E)
        );
        assert!(loaded.registry().is_empty(), "{key}");
    }
    // Without the member a base-game entry is an entry like any other.
    let loaded = load(&[base(&file(r#""dirt": {"textures": "d"}"#))]);
    assert_eq!(loaded.textures("dirt"), six("d"));
    assert_eq!(loaded.found(), []);
}

#[test]
fn ambient_occlusion_takes_a_number() {
    for (value, read) in [
        ("0.5", 0.5f32),
        ("2", 2.0),
        ("0", 0.0),
        ("-3.25", -3.25),
        ("1e2", 100.0),
    ] {
        let (loaded, found) = obsidian(&format!(r#""ambient_occlusion_exponent": {value}"#));
        assert_eq!(
            loaded.block("obsidian").ambient_occlusion_exponent,
            Some(read),
            "{value}"
        );
        assert_eq!(loaded.block("obsidian").ambient_occlusion_exponent(), read);
        assert_eq!(found, [], "{value}: there is no range check");
    }
    let (loaded, found) = obsidian(r#""ambient_occlusion_exponent": null"#);
    assert_eq!(loaded.block("obsidian").ambient_occlusion_exponent, None);
    assert_eq!(
        loaded.block("obsidian").ambient_occlusion_exponent(),
        1.0,
        "the client's default"
    );
    assert_eq!(found, []);
}

#[test]
fn an_ambient_occlusion_that_is_no_number_deviation() {
    // `vanilla = "silent"` for a string, array or object; `"applied silently"` for a boolean.
    let l6 = || {
        ours(
            "core/invalid_type",
            I,
            "obsidian/ambient_occlusion_exponent",
        )
    };
    for value in [r#""high""#, "[0.5]", r#"{"value": 0.5}"#] {
        let (loaded, found) = obsidian(&format!(r#""ambient_occlusion_exponent": {value}"#));
        assert_eq!(
            loaded.block("obsidian").ambient_occlusion_exponent,
            None,
            "{value}"
        );
        assert_eq!(found, [l6()], "{value}");
    }
    for (value, read) in [("true", 1.0f32), ("false", 0.0)] {
        let (loaded, found) = obsidian(&format!(r#""ambient_occlusion_exponent": {value}"#));
        assert_eq!(
            loaded.block("obsidian").ambient_occlusion_exponent,
            Some(read)
        );
        assert_eq!(found, [l6()], "{value}");
    }
}

#[test]
fn an_ambient_occlusion_beyond_f32_is_the_largest_float_deviation() {
    // Ours: never an infinity the writer would refuse.
    for (value, read) in [
        ("1e39", f32::MAX),
        ("-1e39", f32::MIN),
        ("1e999", f32::MAX),
        ("-1e999", f32::MIN),
    ] {
        let (loaded, found) = obsidian(&format!(r#""ambient_occlusion_exponent": {value}"#));
        assert_eq!(
            loaded.block("obsidian").ambient_occlusion_exponent,
            Some(read),
            "{value}"
        );
        // A number that is no double is first held as the largest finite one.
        let at = "obsidian/ambient_occlusion_exponent";
        let mut expected = vec![ours("core/invalid_value", I, at)];
        if value.ends_with("e999") {
            expected.insert(0, ours("core/clamped_value", I, at));
        }
        assert_eq!(found, expected, "{value}");
    }
    // The largest float itself is a float.
    let (loaded, found) = obsidian(r#""ambient_occlusion_exponent": 3.4028234e38"#);
    assert_eq!(
        loaded.block("obsidian").ambient_occlusion_exponent,
        Some(f32::MAX)
    );
    assert_eq!(found, []);
}

#[test]
fn sound_takes_any_string_and_reads_other_values_as_jsoncpp_does_deviation() {
    // `vanilla = "silent"`.
    for sound in ["stone", "", "Stone", "not a sound"] {
        let (loaded, found) = obsidian(&format!(r#""sound": "{sound}""#));
        assert_eq!(loaded.block("obsidian").sound.as_deref(), Some(sound));
        assert_eq!(loaded.block("obsidian").sound(), sound);
        assert_eq!(found, [], "values are not validated");
    }
    let l7 = || ours("core/invalid_type", I, "obsidian/sound");
    for (value, read) in [
        ("5", ""),
        ("1.5", ""),
        ("[]", ""),
        (r#"["stone"]"#, ""),
        ("{}", ""),
        ("true", "true"),
        ("false", "false"),
    ] {
        let (loaded, found) = obsidian(&format!(r#""sound": {value}"#));
        assert_eq!(
            loaded.block("obsidian").sound.as_deref(),
            Some(read),
            "{value}"
        );
        assert_eq!(found, [l7()], "{value}");
    }
    let (loaded, found) = obsidian(r#""sound": null"#);
    assert_eq!(loaded.block("obsidian").sound, None);
    assert_eq!(loaded.block("obsidian").sound(), "normal");
    assert_eq!(found, []);
}

#[test]
fn findings_carry_the_span_of_the_value_their_path_names() {
    // The file is read from a parsed value; the span is found in the text by the finding's
    // path afterwards (core's `json::span_at`).
    let text = file(
        r#""obsidian": {"textures": 5, "sound": 5, "brightness_gamma": 1e999}, "dirt": {"sound": true, "sound": [7]}"#,
    );
    let loaded = load_one(&text);
    let found: Vec<_> = loaded.registry().diagnostics().collect();
    assert_eq!(found.len(), 5, "{found:#?}");
    for finding in found {
        assert!(finding.location.path.is_some() && finding.location.pack.is_some());
        let at = finding.location.json_path.to_string();
        let span = finding
            .location
            .span
            .unwrap_or_else(|| panic!("no span at {at}"));
        let covered = &text[span.offset as usize..(span.offset + span.len) as usize];
        let expected = match at.as_str() {
            "obsidian/textures" | "obsidian/sound" => "5",
            "obsidian/brightness_gamma" => "1e999",
            // A member written twice: the last, the one the client reads.
            "dirt/sound" => "[7]",
            other => panic!("a finding at {other}"),
        };
        assert_eq!(covered, expected, "{at}");
    }
}
