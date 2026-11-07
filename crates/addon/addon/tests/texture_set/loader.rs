//! The texture group reads the JSON files it parses and no image:
//! a stack load never asks for an image's bytes, and the lookups still find every image by
//! the pack's listing.

use bedrock_addon::id::TexturePath;
use bedrock_addon::load::{
    Collection, CollectionSet, LoadOptions, RepositoryLoader, RootId, Source, StackLoader,
    StackOptions,
};
use bedrock_addon::pack::{OpenOptions, PackOrigin, StackSet, WorldPacks};

use super::responder::{Order, Responder, manifest};

/// The collections of the texture families.
const GROUP: [Collection; 5] = [
    Collection::TextureAtlases,
    Collection::Flipbooks,
    Collection::TextureLists,
    Collection::TextureSets,
    Collection::TextureFiles,
];

/// How many images the pack lists.
const IMAGES: usize = 1_000;

const ATLAS: &[u8] = br#"{ "texture_name": "atlas.terrain", "texture_data": {
    "first": { "textures": "textures/blocks/i0000" },
    "gone": { "textures": "textures/blocks/gone" } } }"#;
const SET: &[u8] =
    br#"{ "format_version": "1.21.30", "minecraft:texture_set": { "color": "i0001" } }"#;

/// One resource pack with an atlas, a flipbook file, a list, a texture set and `IMAGES`
/// images, none of them a real image.
fn world() -> Responder {
    let mut responder = Responder::new(Order::Issue);
    responder
        .file(1, "rp/manifest.json", manifest(1, "resources"))
        .file(1, "rp/textures/terrain_texture.json", ATLAS)
        .file(1, "rp/textures/flipbook_textures.json", &b"[]"[..])
        .file(
            1,
            "rp/textures/textures_list.json",
            &br#"["textures/blocks/i0000"]"#[..],
        )
        .file(1, "rp/textures/blocks/i0001.texture_set.json", SET);
    for n in 0..IMAGES {
        responder.file(
            1,
            &format!("rp/textures/blocks/i{n:04}.png"),
            &b"never read"[..],
        );
    }
    responder
}

/// The stacks of the one resource pack of `responder` (UUID 1, root 1) after a stack load of
/// the texture group in `order`, with the subpack `subpack` active; what the responder was
/// asked during the load; and how long building the stacks took.
fn load_world(
    mut responder: Responder,
    subpack: Option<&str>,
    order: Order,
) -> (StackSet, Vec<String>, std::time::Duration) {
    let mut repository = RepositoryLoader::new(
        vec![Source::Directory {
            root: RootId(1),
            origin: PackOrigin::World,
        }],
        OpenOptions::default(),
    );
    let repository = responder.drive(&mut repository);
    let subpack = subpack.map_or(String::new(), |name| format!(r#", "subpack": "{name}""#));
    let entries = format!(
        r#"[{{"pack_id": "{}", "version": [1, 0, 0]{subpack}}}]"#,
        uuid::Uuid::from_u128(1)
    );
    let world = WorldPacks::parse(None, Some(entries.as_bytes())).unwrap();
    let (set, built) = crate::cost::timed(|| repository.stacks(&world, LoadOptions::default()));
    assert_eq!(set.resource.packs().count(), 1, "the pack is on the stack");
    let mut responder = responder.reordered(order);
    let before = responder.log.requests.len();
    let options = StackOptions {
        collections: CollectionSet::only(GROUP),
        ..StackOptions::default()
    };
    let mut loader = StackLoader::new(set, options);
    let set = responder.drive(&mut loader);
    let asked = responder.log.requests[before..]
        .iter()
        .map(|request| format!("{request:?}"))
        .collect();
    (set, asked, built)
}

/// The stacks after a stack load of the texture group in `order`, and what the responder was
/// asked during it.
fn load(order: Order) -> (StackSet, Vec<String>) {
    let (set, asked, _) = load_world(world(), None, order);
    (set, asked)
}

#[test]
fn a_stack_with_a_thousand_listed_images_asks_for_the_bytes_of_none() {
    for order in Order::ALL {
        let (set, asked) = load(order);
        let images: Vec<&String> = asked.iter().filter(|r| r.contains(".png")).collect();
        assert_eq!(images.len(), 0, "{order:?}: {images:#?}");
        // The JSON files of the group are what the load reads.
        let asked_for = |name: &str| asked.iter().filter(|r| r.contains(name)).count();
        assert_eq!(
            asked_for("terrain_texture.json"),
            1,
            "{order:?}: {asked:#?}"
        );
        assert_eq!(asked_for("flipbook_textures.json"), 1, "{order:?}");
        assert_eq!(asked_for("textures_list.json"), 1, "{order:?}");
        assert_eq!(asked_for("i0001.texture_set.json"), 1, "{order:?}");
        assert_eq!(asked.len(), 4, "{order:?}: {asked:#?}");

        // Every image is known by name all the same.
        let rp = &set.resource;
        for n in [0, 1, 499, IMAGES - 1] {
            let key = format!("textures/blocks/i{n:04}.png");
            assert_eq!(
                rp.texture_files().packs_with(&key).count(),
                1,
                "{order:?}: {key}"
            );
        }
        assert_eq!(
            rp.texture_files()
                .packs_with("textures/blocks/i1000.png")
                .count(),
            0
        );
        // The atlas rule that needs a file's existence: one path has an image, one has none.
        let missing: Vec<String> = rp
            .texture_atlases()
            .terrain_atlas
            .diagnostics()
            .filter(|f| f.code.id() == "core/texture_not_found")
            .map(|f| f.location.json_path.to_string())
            .collect();
        assert_eq!(missing.len(), 1, "{order:?}: {missing:?}");
        assert!(missing[0].contains("gone"), "{missing:?}");
        // The texture set's layer image is found in its pack, unread.
        let file = rp
            .resolve_texture(&TexturePath::parse("textures/blocks/i0001"))
            .expect("the texture");
        assert!(file.set.is_some(), "{order:?}");
        assert_eq!(
            file.image.expect("the image").key,
            "textures/blocks/i0001.png"
        );
        assert_eq!(rp.layer_images().len(), 1);
        // The scan of a pack in list mode, and the claims, come from the names.
        assert!(
            rp.texture_lists()
                .claims()
                .contains_key("textures/blocks/i0000")
        );
        // Nothing is reported as not loaded: an image is no file of the walk.
        assert!(
            !set.resource
                .report()
                .iter()
                .any(|f| f.code.id() == "core/file_not_loaded"),
            "{order:?}"
        );
    }
}

/// A resource pack (UUID 1, root 1) with the subpack `hd` declared and `files` below `rp/`.
fn with_subpack(files: &[(String, Vec<u8>)]) -> Responder {
    let mut responder = Responder::new(Order::Issue);
    let manifest = super::common::manifest(1, "resources", &["hd"]);
    responder.file(1, "rp/manifest.json", manifest.into_bytes());
    for (path, bytes) in files {
        responder.file(1, &format!("rp/{path}"), bytes.clone());
    }
    responder
}

// `entry_files` looks the overlay's keys up in a set and runs once per pack: 20 000 files each
// shadowed by a subpack file took 6.9 s per run, and it ran for the stack and for each walk. Four times the files may take about four times the time, for
// building the stacks and for the whole load (5 000 against 20 000).
#[test]
fn twenty_thousand_shadowed_files_are_listed_in_linear_time() {
    let load = |count: usize| {
        let mut files = Vec::new();
        for n in 0..count {
            files.push((format!("textures/blocks/i{n:05}.png"), b"own".to_vec()));
            files.push((
                format!("subpacks/hd/textures/blocks/i{n:05}.png"),
                b"hd".to_vec(),
            ));
        }
        let responder = with_subpack(&files);
        let ((set, asked, built), whole) =
            crate::cost::timed(|| load_world(responder, Some("hd"), Order::Issue));
        assert_eq!(asked.iter().filter(|r| r.contains(".png")).count(), 0);
        // Each image is the subpack's, at the pack's key.
        let rp = &set.resource;
        let last = count - 1;
        let file = rp
            .resolve_texture(&TexturePath::parse(format!("textures/blocks/i{last:05}")))
            .expect("the image");
        let image = file.image.expect("the image");
        assert_eq!(image.key, format!("textures/blocks/i{last:05}.png"));
        assert_eq!(
            image.path.as_str(),
            format!("subpacks/hd/textures/blocks/i{last:05}.png")
        );
        assert_eq!(rp.texture_lists().claims().len(), count);
        [built, whole]
    };
    crate::cost::assert_each_linear_timed_at(
        ["building the stacks", "the whole load"],
        5_000,
        load,
    );
}

/// The findings of the texture group of `set`, as code and file.
fn found(set: &StackSet) -> Vec<(&'static str, String)> {
    crate::group(&set.resource)
        .map(|f| {
            let file = f.location.path.as_ref().map_or("", |path| path.as_str());
            (f.code.id(), file.to_owned())
        })
        .collect()
}

// with an active subpack that has its own list, the walk reads the
// pack's own list too and the client's list is the two together, so V9 (the list omits an
// image) is said of both lists' entries: `[a]` and `[b]` beside the images a and b are whole.
#[test]
fn a_subpack_list_over_the_pack_s_list_is_not_called_incomplete() {
    let files = [
        ("textures/textures_list.json", &br#"["textures/a"]"#[..]),
        (
            "subpacks/hd/textures/textures_list.json",
            br#"["textures/b"]"#,
        ),
        ("textures/a.png", b""),
        ("textures/b.png", b""),
    ]
    .map(|(path, bytes)| (path.to_owned(), bytes.to_vec()));
    let (set, _, _) = load_world(with_subpack(&files), Some("hd"), Order::Issue);
    assert_eq!(
        found(&set),
        [],
        "no V9 for the list `[b]` beside the image a"
    );
    // Without the pack's own list the subpack's list is the whole list, and V9 stands.
    let (set, _, _) = load_world(with_subpack(&files[1..]), Some("hd"), Order::Issue);
    assert_eq!(
        found(&set),
        [(
            "core/invalid_value",
            "subpacks/hd/textures/textures_list.json".to_owned()
        )]
    );
}

// The capabilities are the pack's manifest's (`StackPack::capabilities`): a
// `manifest.json` in the active subpack does not stand in for it and takes nothing from it.
#[test]
fn a_subpack_s_manifest_json_does_not_declare_the_pack_s_capabilities() {
    const SET: &[u8] =
        br#"{ "format_version": "1.21.30", "minecraft:texture_set": { "color": [1, 2, 3, 4] } }"#;
    let files = [
        ("textures/x.texture_set.json", SET),
        (
            "subpacks/hd/manifest.json",
            &br#"{ "capabilities": ["pbr"] }"#[..],
        ),
        ("subpacks/hd/textures/y.texture_set.json", SET),
    ]
    .map(|(path, bytes)| (path.to_owned(), bytes.to_vec()));
    // Without the subpack's file the pack ships sets and declares nothing: the hint.
    let without: Vec<_> = files
        .iter()
        .filter(|f| !f.0.ends_with("hd/manifest.json"))
        .cloned()
        .collect();
    let (set, _, _) = load_world(with_subpack(&without), Some("hd"), Order::Issue);
    assert_eq!(
        found(&set),
        [("core/capability_hint", "manifest.json".to_owned())]
    );
    // With it, the same: the subpack's file is no manifest, so its `pbr` is not the pack's,
    // and the pack's own capabilities are known all the same (the sets still load).
    let (set, _, _) = load_world(with_subpack(&files), Some("hd"), Order::Issue);
    assert_eq!(
        found(&set),
        [("core/capability_hint", "manifest.json".to_owned())]
    );
    assert_eq!(set.resource.texture_sets().sets().count(), 2);
}

/// One resource pack of a world: its UUID number, its `min_engine_version`, and its files. Each
/// declares the subpack `hd`, which the world selects.
struct WorldPack {
    n: u128,
    min_engine: [u16; 3],
    files: Vec<(&'static str, &'static [u8])>,
}

/// The stacks of `packs` (the first on top, each with its subpack `hd` active) after a stack
/// load of the texture group.
fn load_packs(packs: &[WorldPack]) -> StackSet {
    let mut responder = Responder::new(Order::Issue);
    let mut entries = Vec::new();
    for pack in packs {
        let n = pack.n;
        let [major, minor, patch] = pack.min_engine;
        let manifest = format!(
            r#"{{ "format_version": 2,
  "header": {{ "name": "pack {n}", "description": "", "uuid": "{}", "version": [1, 0, 0],
              "min_engine_version": [{major}, {minor}, {patch}] }},
  "modules": [{{ "type": "resources", "uuid": "{}", "version": [1, 0, 0] }}],
  "subpacks": [{{ "folder_name": "hd", "name": "hd", "memory_tier": 0 }}] }}"#,
            uuid::Uuid::from_u128(n),
            uuid::Uuid::from_u128(n | 0xffff_0000_0000_0000_0000),
        );
        responder.file(1, &format!("p{n}/manifest.json"), manifest.into_bytes());
        for (path, bytes) in &pack.files {
            responder.file(1, &format!("p{n}/{path}"), bytes.to_vec());
        }
        entries.push(format!(
            r#"{{"pack_id": "{}", "version": [1, 0, 0], "subpack": "hd"}}"#,
            uuid::Uuid::from_u128(n)
        ));
    }
    let mut repository = RepositoryLoader::new(
        vec![Source::Directory {
            root: RootId(1),
            origin: PackOrigin::World,
        }],
        OpenOptions::default(),
    );
    let repository = responder.drive(&mut repository);
    let world = format!("[{}]", entries.join(", "));
    let world = WorldPacks::parse(None, Some(world.as_bytes())).unwrap();
    let set = repository.stacks(&world, LoadOptions::default());
    assert_eq!(
        set.resource.packs().count(),
        packs.len(),
        "every pack is on the stack"
    );
    let options = StackOptions {
        collections: CollectionSet::only(GROUP),
        ..StackOptions::default()
    };
    let mut loader = StackLoader::new(set, options);
    responder.drive(&mut loader)
}

/// The lints of the texture group of `set`: the code and the pack's place in the stack.
fn lints(set: &StackSet) -> Vec<(&'static str, Option<usize>)> {
    crate::group(&set.resource)
        .map(|f| {
            let pack = f
                .location
                .pack
                .as_ref()
                .and_then(bedrock_addon::pack::PackRef::index);
            (f.code.id(), pack)
        })
        .collect()
}

const SUBPACK_MANIFEST: (&str, &[u8]) = ("subpacks/hd/manifest.json", b"{}");
const A_SET: &[u8] =
    br#"{ "format_version": "1.21.30", "minecraft:texture_set": { "color": [1, 2, 3, 4] } }"#;

// R1 needs the pack's capabilities, which are the opened manifest's: a pack at 1.21.110 that
// declares nothing turns Vibrant Visuals off, with or without a `manifest.json` in its active
// subpack (the shadowed manifest does not make the pack's capabilities unknown).
#[test]
fn a_manifest_json_in_the_subpack_does_not_hide_that_the_pack_turns_vibrant_visuals_off() {
    let pack = |files| WorldPack {
        n: 1,
        min_engine: [1, 21, 110],
        files,
    };
    let plain = load_packs(&[pack(vec![])]);
    assert_eq!(
        lints(&plain),
        [("core/vibrant_visuals_disabled_by_pack", Some(0))]
    );
    let shadowed = load_packs(&[pack(vec![SUBPACK_MANIFEST])]);
    assert_eq!(
        lints(&shadowed),
        [("core/vibrant_visuals_disabled_by_pack", Some(0))]
    );
    // Below the version the rule starts at, neither says anything.
    let older = load_packs(&[WorldPack {
        n: 1,
        min_engine: [1, 21, 100],
        files: vec![SUBPACK_MANIFEST],
    }]);
    assert_eq!(lints(&older), []);
}

// R3 asks whether any pack of the stack declares `pbr` or `raytraced`. A pack with a
// `manifest.json` in its active subpack declares what its own manifest does: nothing, so the
// pack below it that ships texture sets gets the hint either way.
#[test]
fn a_manifest_json_in_a_subpack_does_not_silence_the_hint_for_the_stack() {
    let stack = |upper: Vec<(&'static str, &'static [u8])>| {
        load_packs(&[
            WorldPack {
                n: 1,
                min_engine: [1, 21, 0],
                files: upper,
            },
            WorldPack {
                n: 2,
                min_engine: [1, 21, 0],
                files: vec![("textures/x.texture_set.json", A_SET)],
            },
        ])
    };
    assert_eq!(lints(&stack(vec![])), [("core/capability_hint", Some(1))]);
    assert_eq!(
        lints(&stack(vec![SUBPACK_MANIFEST])),
        [("core/capability_hint", Some(1))]
    );
}
