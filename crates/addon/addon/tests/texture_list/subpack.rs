//! The rules of a pack with an active subpack, through a stack: the world list selects the
//! subpack `hi`, and the texture group is loaded by a `StackLoader`.

use bedrock_addon::families::texture_list::TextureListMode;
use bedrock_addon::load::{
    Collection, CollectionSet, LoadOptions, RepositoryLoader, RootId, Source, StackLoader,
    StackOptions,
};
use bedrock_addon::pack::{OpenOptions, PackOrigin, StackSet, WorldPacks};

use super::responder::{Order, Responder};

/// The stacks of one resource pack with `files` and the declared subpack `hi` active, after a
/// stack load of the texture group.
fn load(files: &[(&str, &[u8])]) -> StackSet {
    let mut responder = Responder::new(Order::Issue);
    let manifest = super::common::manifest(1, "resources", &["hi"]);
    responder.file(1, "rp/manifest.json", manifest.into_bytes());
    for (path, bytes) in files {
        responder.file(1, &format!("rp/{path}"), bytes.to_vec());
    }
    let mut repository = RepositoryLoader::new(
        vec![Source::Directory {
            root: RootId(1),
            origin: PackOrigin::World,
        }],
        OpenOptions::default(),
    );
    let repository = responder.drive(&mut repository);
    let entries = format!(
        r#"[{{"pack_id": "{}", "version": [1, 0, 0], "subpack": "hi"}}]"#,
        uuid::Uuid::from_u128(1)
    );
    let world = WorldPacks::parse(None, Some(entries.as_bytes())).unwrap();
    let set = repository.stacks(&world, LoadOptions::default());
    assert_eq!(set.resource.packs().count(), 1, "the pack is on the stack");
    let options = StackOptions {
        collections: CollectionSet::only([Collection::TextureLists]),
        ..StackOptions::default()
    };
    let mut loader = StackLoader::new(set, options);
    responder.drive(&mut loader)
}

/// The claims of the stack, in order.
fn claims(set: &StackSet) -> Vec<&str> {
    set.resource
        .texture_lists()
        .claims()
        .keys()
        .map(String::as_str)
        .collect()
}

// The subpack's elements are appended to the
// pack's own list. The walk reads the pack's own list beside the subpack's (`Walk::shadowed`).
#[test]
fn tl17_through_a_stack_the_subpack_s_entries_are_appended_to_the_pack_s() {
    let set = load(&[
        ("textures/textures_list.json", br#"["textures/a"]"#),
        (
            "subpacks/hi/textures/textures_list.json",
            br#"["textures/b"]"#,
        ),
        ("textures/a.png", b""),
        ("textures/b.png", b""),
    ]);
    let textures = set.resource.texture_lists();
    assert_eq!(
        textures.list_mode(0),
        Some(&TextureListMode::List {
            entries: vec!["textures/a".to_owned(), "textures/b".to_owned()]
        })
    );
    assert_eq!(claims(&set), ["textures/a", "textures/b"]);
    // The list is complete: no V9, and no other finding.
    let found: Vec<String> = textures.diagnostics().map(ToString::to_string).collect();
    assert_eq!(found, [] as [String; 0]);
    // Both lists are documents of the group: the pack's own, then the subpack's.
    let lists: Vec<&str> = textures
        .lists
        .iter()
        .map(|list| list.origin().path.as_str())
        .collect();
    assert_eq!(
        lists,
        [
            "textures/textures_list.json",
            "subpacks/hi/textures/textures_list.json"
        ]
    );
}

// When the subpack contributes nothing, the base value is
// returned unchanged: an empty subpack list, and one that is no JSON, leave the pack's list.
#[test]
fn tl17_an_empty_or_invalid_subpack_list_keeps_the_pack_s_list() {
    for subpack in [&b"[]"[..], b"}", b"null"] {
        let set = load(&[
            ("textures/textures_list.json", br#"["textures/a"]"#),
            ("subpacks/hi/textures/textures_list.json", subpack),
            ("textures/a.png", b""),
        ]);
        assert_eq!(
            set.resource.texture_lists().list_mode(0),
            Some(&TextureListMode::List {
                entries: vec!["textures/a".to_owned()]
            }),
            "subpack list {:?}",
            String::from_utf8_lossy(subpack)
        );
        assert_eq!(claims(&set), ["textures/a"]);
    }
}

// V9 is about the list the client uses: the pack's entries and the subpack's together. An
// image neither names is omitted, and the finding is on the subpack's list (the last one
// appended); an image the pack's own list names is not.
#[test]
fn tl17_the_appended_list_is_incomplete_only_for_an_image_neither_list_names() {
    let set = load(&[
        ("textures/textures_list.json", br#"["textures/a"]"#),
        (
            "subpacks/hi/textures/textures_list.json",
            br#"["textures/b"]"#,
        ),
        ("textures/a.png", b""),
        ("textures/b.png", b""),
        ("textures/c.png", b""),
    ]);
    let textures = set.resource.texture_lists();
    let found: Vec<(String, &str)> = textures
        .diagnostics()
        .map(|d| {
            let path = d.location.path.as_ref().map_or("", |p| p.as_str());
            (format!("{:?}", d.code), path)
        })
        .collect();
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].0.starts_with("InvalidValue"), "{found:?}");
    assert_eq!(found[0].1, "subpacks/hi/textures/textures_list.json");
}

// Fixture tl26 through a stack: a pack without a list is scanned, the active subpack's
// directory first, then the pack's own; a path both have is claimed once, at its first place.
#[test]
fn tl26_through_a_stack_the_subpack_s_files_are_claimed_first() {
    let set = load(&[
        ("textures/a.png", b""),
        ("textures/c.png", b""),
        ("subpacks/hi/textures/b.png", b""),
        ("subpacks/hi/textures/c.png", b""),
        ("subpacks/hi/textures/z.png", b""),
    ]);
    assert_eq!(
        set.resource.texture_lists().list_mode(0),
        Some(&TextureListMode::Scan)
    );
    assert_eq!(
        claims(&set),
        ["textures/b", "textures/c", "textures/z", "textures/a"]
    );
}
