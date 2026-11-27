//! What a load of the sound families asks the host for: the three documents, and the bytes of no
//! audio file. The audio a sound name resolves to is read from the pack's file listing.

use bedrock_addon::diagnostic::Kind;
use bedrock_addon::families::sound_file::AudioExt;
use bedrock_addon::load::{
    Collection, CollectionSet, LoadOptions, RepositoryLoader, Request, Residency, RootId, Source,
    StackLoader, StackOptions,
};
use bedrock_addon::pack::{OpenOptions, PackOrigin, WorldPacks};

use super::responder::{Order, Responder, manifest};

/// The paths the load read, in request order.
fn read_paths(responder: &Responder) -> Vec<String> {
    responder
        .log
        .requests
        .iter()
        .filter_map(|request| match request {
            Request::ReadFile { location, .. } => Some(location.path.as_str().to_owned()),
            Request::ListDir { .. } => None,
        })
        .collect()
}

#[test]
fn a_stack_with_listed_sound_files_requests_the_bytes_of_none_of_them() {
    let definitions = br#"{"format_version": "1.20.20", "sound_definitions": {
        "a": {"sounds": ["sounds/x", "sounds/deep/y", "sounds/z", "sounds/missing"]}}}"#;
    let mut responder = Responder::new(Order::Issue);
    responder
        .file(1, "rp/manifest.json", manifest(2, "resources"))
        .file(1, "rp/sounds/sound_definitions.json", &definitions[..])
        .file(1, "rp/sounds.json", &b"{}"[..])
        .file(1, "rp/sounds/music_definitions.json", &b"{}"[..])
        .file(1, "rp/sounds/x.ogg", &b"not read"[..])
        .file(1, "rp/sounds/x.wav", &b"not read"[..])
        .file(1, "rp/sounds/deep/y.fsb", &b"not read"[..])
        .file(1, "rp/sounds/z.wav", &b"not read"[..])
        .file(1, "rp/sounds/w.mp3", &b"not read"[..]);
    let mut repository = RepositoryLoader::new(
        vec![Source::Directory {
            root: RootId(1),
            origin: PackOrigin::World,
        }],
        OpenOptions::default(),
    );
    let repository = responder.drive(&mut repository);
    let world = format!(
        r#"[{{"pack_id": "{}", "version": [1, 0, 0]}}]"#,
        uuid::Uuid::from_u128(2)
    );
    let world = WorldPacks::parse(Some(b"[]"), Some(world.as_bytes())).expect("a world list");
    let set = repository.stacks(&world, LoadOptions::default());
    for residency in [Residency::Transient, Residency::All] {
        let before = responder.log.requests.len();
        let options = StackOptions {
            collections: CollectionSet::only([
                Collection::SoundDefinitions,
                Collection::SoundsJson,
                Collection::Music,
                Collection::SoundFiles,
            ]),
            resource: Residency::Transient,
            behavior: residency,
            ..StackOptions::default()
        };
        let mut loader =
            StackLoader::new(repository.stacks(&world, LoadOptions::default()), options);
        let stack = responder.drive(&mut loader);
        let read: Vec<String> = read_paths(&responder).split_off(0);
        let audio: Vec<&String> = read
            .iter()
            .filter(|path| {
                [".ogg", ".fsb", ".wav", ".mp3"]
                    .iter()
                    .any(|ext| path.ends_with(ext))
            })
            .collect();
        assert!(audio.is_empty(), "audio bytes were requested: {audio:?}");
        assert!(
            responder.log.requests.len() > before,
            "the documents were read"
        );
        for document in [
            "sounds/sound_definitions.json",
            "sounds.json",
            "sounds/music_definitions.json",
        ] {
            assert!(
                read.iter().any(|path| path.ends_with(document)),
                "{document}: {read:?}"
            );
        }
        // The listing alone resolves the names, in probe order.
        let sounds = stack.resource.sound_files();
        assert_eq!(sounds.get("sounds/x").unwrap().ext, AudioExt::Ogg);
        assert_eq!(sounds.get("sounds/deep/y").unwrap().ext, AudioExt::Fsb);
        assert_eq!(sounds.get("sounds/z").unwrap().ext, AudioExt::Wav);
        assert_eq!(sounds.by_name.len(), 3);
        assert_eq!(sounds.shadowed.len(), 1);
        assert_eq!(sounds.ignored.len(), 1);
        let codes: Vec<&str> = sounds.findings().map(|d| d.code.id()).collect();
        // A world pack is no base-game pack: its dangling name is not judged.
        assert_eq!(codes, ["core/ignored_file", "core/shadowed_definition"]);
        assert!(
            stack
                .link()
                .iter()
                .all(|d| d.kind() != Kind::UnresolvedReference),
            "the dangling name of a world pack is not judged by the link pass either"
        );
        // Nothing was skipped for want of bytes.
        assert!(
            stack
                .resource
                .report()
                .iter()
                .all(|d| d.code.id() != "core/file_not_loaded")
        );
    }
    // The lazy accessor on a stack whose files are only indexed asks for nothing at all and
    // still lists the audio.
    assert_eq!(set.resource.sound_files().by_name.len(), 3);
}

/// A resource pack that declares the subpacks `hd` and `off`.
fn manifest_with_subpacks(n: u128) -> String {
    format!(
        r#"{{"format_version": 2,
  "header": {{"name": "pack {n}", "description": "", "uuid": "{}", "version": [1, 0, 0],
              "min_engine_version": [1, 21, 0]}},
  "modules": [{{"type": "resources", "uuid": "{}", "version": [1, 0, 0]}}],
  "subpacks": [{{"folder_name": "hd", "name": "hd", "memory_tier": 0}},
               {{"folder_name": "off", "name": "off", "memory_tier": 0}}]}}"#,
        uuid::Uuid::from_u128(n),
        uuid::Uuid::from_u128(n | 0xffff_0000_0000_0000_0000),
    )
}

// `StackPack::files()`: a file is named by its key (the active subpack's file without its
// folder) and found at its path.
#[test]
fn an_active_subpack_s_audio_is_named_without_its_folder_and_found_inside_it() {
    let mut responder = Responder::new(Order::Issue);
    responder
        .file(1, "rp/manifest.json", manifest_with_subpacks(2))
        .file(1, "rp/sounds/x.ogg", &b"not read"[..])
        .file(1, "rp/sounds/own.ogg", &b"not read"[..])
        .file(1, "rp/subpacks/hd/sounds/x.ogg", &b"not read"[..])
        .file(1, "rp/subpacks/hd/sounds/y.wav", &b"not read"[..])
        .file(1, "rp/subpacks/hd/sounds/notes.txt", &b"not read"[..])
        .file(1, "rp/subpacks/off/sounds/z.ogg", &b"not read"[..]);
    let mut repository = RepositoryLoader::new(
        vec![Source::Directory {
            root: RootId(1),
            origin: PackOrigin::World,
        }],
        OpenOptions::default(),
    );
    let repository = responder.drive(&mut repository);
    let world = format!(
        r#"[{{"pack_id": "{}", "version": [1, 0, 0], "subpack": "hd"}}]"#,
        uuid::Uuid::from_u128(2)
    );
    let world = WorldPacks::parse(Some(b"[]"), Some(world.as_bytes())).expect("a world list");
    let set = repository.stacks(&world, LoadOptions::default());
    assert_eq!(set.resource.packs().count(), 1, "the pack is on the stack");
    let files = set.resource.sound_files();
    let mut names: Vec<(&str, &str)> = files
        .by_name
        .iter()
        .map(|(name, asset)| (name.as_str(), asset.path.as_str()))
        .collect();
    names.sort_unstable();
    assert_eq!(
        names,
        [
            ("sounds/own", "sounds/own.ogg"),
            ("sounds/x", "subpacks/hd/sounds/x.ogg"),
            ("sounds/y", "subpacks/hd/sounds/y.wav"),
        ],
        "each name with the path its pack is asked for"
    );
    // The pack's own `sounds/x.ogg` is no second file of the name.
    assert!(files.shadowed.is_empty(), "{:?}", files.shadowed);
    let ignored: Vec<&str> = files
        .ignored
        .iter()
        .map(|(_, path)| path.as_str())
        .collect();
    assert_eq!(ignored, ["subpacks/hd/sounds/notes.txt"]);
}
