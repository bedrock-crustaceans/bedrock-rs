//! What a load of the materials asks the host for: the list files and every file whose path
//! ends in `.material`, and the bytes of nothing else. A compiled `x.material.bin` beside them
//! is no file of this family.

use bedrock_addon::load::{
    Collection, CollectionSet, LoadOptions, RepositoryLoader, Request, RootId, Source, StackLoader,
    StackOptions,
};
use bedrock_addon::pack::{OpenOptions, PackOrigin, WorldPacks};

use crate::responder::{Order, Responder, manifest};

/// The paths the load read, in request order.
fn read_paths(responder: &Responder) -> Vec<String> {
    responder
        .log
        .requests
        .iter()
        .filter_map(|request| match request {
            Request::ReadFile { location, .. } => Some(location.path.as_str().to_owned()),
            _ => None,
        })
        .collect()
}

#[test]
fn a_load_reads_the_lists_and_the_material_files_and_nothing_that_only_looks_like_one() {
    let file = br#"{"materials": {"version": "1.0.0", "a": {}}}"#;
    let listing = br#"[{"path": "materials/a.material"}, {"path": "materials/x.material.bin"}]"#;
    let mut responder = Responder::new(Order::Issue);
    responder
        .file(1, "rp/manifest.json", manifest(2, "resources"))
        .file(1, "rp/materials/common.json", &listing[..])
        .file(1, "rp/materials/gameface.json", &b"[]"[..])
        .file(1, "rp/materials/a.material", &file[..])
        .file(1, "rp/deep/dir/b.material", &file[..])
        // None of these is a material file: the extension is the end of the path.
        .file(1, "rp/materials/x.material.bin", &b"\x00compiled"[..])
        .file(1, "rp/materials/x.material.json", &file[..])
        .file(1, "rp/materials/a.materials", &file[..])
        .file(1, "rp/materials/material", &file[..])
        .file(1, "rp/materials/other.json", &b"[]"[..])
        .file(
            1,
            "rp/renderer/materials/Actor.material.bin",
            &b"\x00compiled"[..],
        )
        .file(1, "rp/textures/a.material.png", &b"not read"[..]);
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
    let before = responder.log.requests.len();
    let options = StackOptions {
        collections: CollectionSet::only([Collection::Materials]),
        ..StackOptions::default()
    };
    let mut loader = StackLoader::new(repository.stacks(&world, LoadOptions::default()), options);
    let loaded = responder.drive(&mut loader);
    let mut read: Vec<String> = read_paths(&responder).split_off(0);
    assert!(responder.log.requests.len() > before, "the files were read");
    read.retain(|path| !path.ends_with("manifest.json"));
    read.sort();
    read.dedup();
    assert_eq!(
        read,
        [
            "rp/deep/dir/b.material",
            "rp/materials/a.material",
            "rp/materials/common.json",
            "rp/materials/gameface.json",
        ]
    );
    let materials = loaded.resource.materials();
    let files: Vec<&str> = materials.files.keys().map(|path| path.as_str()).collect();
    assert_eq!(files, ["materials/a.material", "deep/dir/b.material"]);
    assert!(materials.common.contains("a"));
    // The listed `.material.bin` exists and is not read: the entry says so.
    let found: Vec<_> = materials
        .diagnostics()
        .map(|f| (f.code.name(), f.location.json_path.to_string()))
        .collect();
    assert_eq!(found, [("IgnoredFile", "1/path".to_owned())]);
}
