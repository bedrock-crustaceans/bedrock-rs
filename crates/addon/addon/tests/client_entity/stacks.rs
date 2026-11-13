//! The in-memory resource stacks of the client entity tests: packs at a `min_engine_version`,
//! loaded through the repository and stack loaders.

#![allow(dead_code, reason = "each test file uses part of the helpers")]

use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::StackSet;
use uuid::Uuid;

/// One resource pack of an in-memory stack: its manifest `min_engine_version` and its files.
pub struct TestPack<'a> {
    pub min_engine_version: (u16, u16, u16),
    pub files: &'a [(&'a str, &'a str)],
}

/// A pack at `min_engine_version` 1.21.0.
pub fn pack<'a>(files: &'a [(&'a str, &'a str)]) -> TestPack<'a> {
    TestPack {
        min_engine_version: (1, 21, 0),
        files,
    }
}

/// A stack set whose resource stack is `packs`, highest priority first.
pub fn stack(packs: &[TestPack<'_>]) -> StackSet {
    stack_with_unread(packs, &[])
}

/// [`stack`], with the files at `unread` (`p<n>/<path>`, `n` counted from 1) listed and their
/// reads failing.
pub fn stack_with_unread(packs: &[TestPack<'_>], unread: &[&str]) -> StackSet {
    use crate::responder::{Order, Responder};
    use bedrock_addon::load::{RepositoryLoader, RootId, Source, StackLoader, StackOptions};
    use bedrock_addon::pack::{OpenOptions, PackOrigin, WorldPacks};

    let mut responder = Responder::new(Order::Issue);
    let mut list = Vec::new();
    for (index, pack) in packs.iter().enumerate() {
        let n = index as u128 + 1;
        let (a, b, c) = pack.min_engine_version;
        let manifest = format!(
            r#"{{"format_version": 2, "header": {{"name": "pack {n}", "description": "", "uuid": "{}", "version": [1, 0, 0], "min_engine_version": [{a}, {b}, {c}]}}, "modules": [{{"type": "resources", "uuid": "{}", "version": [1, 0, 0]}}]}}"#,
            Uuid::from_u128(n),
            Uuid::from_u128(n | 0xffff_0000_0000_0000_0000),
        );
        responder.file(1, &format!("p{n}/manifest.json"), manifest);
        for (path, text) in pack.files {
            responder.file(1, &format!("p{n}/{path}"), (*text).to_string());
        }
        list.push(format!(
            r#"{{"pack_id": "{}", "version": [1, 0, 0]}}"#,
            Uuid::from_u128(n)
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
    let resource = format!("[{}]", list.join(", "));
    let world = WorldPacks::parse(None, Some(resource.as_bytes())).unwrap();
    let set = repository.stacks(&world, LoadOptions::default());
    for path in unread {
        responder.failed(1, path, std::io::ErrorKind::PermissionDenied);
    }
    let mut loader = StackLoader::new(set, StackOptions::default());
    responder.drive(&mut loader)
}
