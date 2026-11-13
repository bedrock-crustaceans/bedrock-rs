//! The in-memory resource stacks of the geometry stack tests.

use bedrock_addon::load::LoadOptions;
use uuid::Uuid;

/// A resource stack read from in-memory packs, highest priority first; each pack is a list of
/// `(path, text)` files and gets a manifest.
pub fn stack(resource: &[&[(&str, &str)]]) -> bedrock_addon::pack::StackSet {
    use crate::responder::{Order, Responder, manifest};
    use bedrock_addon::load::{RepositoryLoader, RootId, Source, StackLoader, StackOptions};
    use bedrock_addon::pack::{OpenOptions, PackOrigin, WorldPacks};

    let mut responder = Responder::new(Order::Issue);
    let mut list = Vec::new();
    for (n, files) in (1_u128..).zip(resource) {
        responder.file(1, &format!("p{n}/manifest.json"), manifest(n, "resources"));
        for (path, text) in *files {
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
    let world = WorldPacks::parse(Some(b"[]"), Some(resource.as_bytes())).unwrap();
    let set = repository.stacks(&world, LoadOptions::default());
    let mut loader = StackLoader::new(set, StackOptions::default());
    responder.drive(&mut loader)
}
