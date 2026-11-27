//! What a stack load of chosen collections asks its host for: packs in a directory source,
//! opened by the `RepositoryLoader`, stacked by a world list, and loaded by a `StackLoader`
//! whose every read request is recorded. The way a family test shows that a file it only
//! reports (a picture, a note, a file of another extension) is never read.

use bedrock_addon::load::{
    Collection, CollectionSet, LoadOptions, RepositoryLoader, Request, RootId, Source, StackLoader,
    StackOptions,
};
use bedrock_addon::pack::{OpenOptions, PackOrigin, StackSet, WorldPacks};

use super::responder::{Order, Responder, manifest};

/// The stacks of `packs` (the first on top; each a folder `p<n>` of `module` type, `data` or
/// `resources`, with its files) after a stack load of `collections` answered in `order`, and
/// the path of every file that load asked for, in issue order (`p0/entities/pig.json`).
pub fn stack_load(
    module: &str,
    packs: &[&[(&str, &[u8])]],
    collections: &[Collection],
    order: Order,
) -> (StackSet, Vec<String>) {
    let mut responder = Responder::new(Order::Issue);
    let mut list = Vec::new();
    for (index, files) in packs.iter().enumerate() {
        let n = index as u128 + 1;
        responder.file(1, &format!("p{index}/manifest.json"), manifest(n, module));
        for (path, bytes) in *files {
            responder.file(1, &format!("p{index}/{path}"), bytes.to_vec());
        }
        list.push(format!(
            r#"{{"pack_id": "{}", "version": [1, 0, 0]}}"#,
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
    let list = format!("[{}]", list.join(", "));
    let (behavior, resource) = if module == "data" {
        (list.as_str(), "[]")
    } else {
        ("[]", list.as_str())
    };
    let world = WorldPacks::parse(Some(behavior.as_bytes()), Some(resource.as_bytes())).unwrap();
    let set = repository.stacks(&world, LoadOptions::default());
    let stacked = if module == "data" {
        set.behavior.packs().count()
    } else {
        set.resource.packs().count()
    };
    assert_eq!(stacked, packs.len(), "every pack is on the stack");
    let mut responder = responder.reordered(order);
    let options = StackOptions {
        collections: CollectionSet::only(collections.iter().copied()),
        ..StackOptions::default()
    };
    let mut loader = StackLoader::new(set, options);
    let set = responder.drive(&mut loader);
    let asked = responder
        .log
        .requests
        .iter()
        .filter_map(|request| match request {
            Request::ReadFile { location, .. } => Some(location.path.as_str().to_owned()),
            Request::ListDir { .. } => None,
        })
        .collect();
    (set, asked)
}
