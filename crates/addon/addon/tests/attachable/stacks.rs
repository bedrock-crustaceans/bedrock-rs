//! The in-memory stacks of the attachable tests: packs at a `min_engine_version`, on the
//! resource stack or on both, loaded through the repository and stack loaders.

use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::StackSet;
use bedrock_addon::storage::PackPath;
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
    stack_with(packs, LoadOptions::default())
}

/// [`stack`] over the app-data files `app_data` (paths below the app's own directory).
pub fn stack_over(packs: &[TestPack<'_>], app_data: &[(&str, &str)]) -> StackSet {
    let files = app_data
        .iter()
        .map(|(path, text)| (PackPath::new(path).unwrap(), text.as_bytes().to_vec()));
    stack_with(
        packs,
        LoadOptions {
            app_data: Some(bedrock_addon::load::AppData::from_files(files)),
            ..LoadOptions::default()
        },
    )
}

/// [`stack`] loaded with `options`.
pub fn stack_with(packs: &[TestPack<'_>], options: LoadOptions) -> StackSet {
    stacks(&[], packs, options)
}

/// A stack set whose behaviour stack is `behavior` and whose resource stack is `resource`,
/// each highest priority first.
pub fn stacks(
    behavior: &[TestPack<'_>],
    resource: &[TestPack<'_>],
    options: LoadOptions,
) -> StackSet {
    use crate::responder::{Order, Responder};
    use bedrock_addon::load::{RepositoryLoader, RootId, Source, StackLoader, StackOptions};
    use bedrock_addon::pack::{OpenOptions, PackOrigin, WorldPacks};

    let mut responder = Responder::new(Order::Issue);
    let mut lists = Vec::new();
    for (side, module, packs, first) in [
        ("b", "data", behavior, 100_u128),
        ("p", "resources", resource, 0),
    ] {
        let mut list = Vec::new();
        for (index, pack) in packs.iter().enumerate() {
            let n = first + index as u128 + 1;
            let (a, b, c) = pack.min_engine_version;
            let manifest = format!(
                r#"{{"format_version": 2, "header": {{"name": "pack {n}", "description": "", "uuid": "{}", "version": [1, 0, 0], "min_engine_version": [{a}, {b}, {c}]}}, "modules": [{{"type": "{module}", "uuid": "{}", "version": [1, 0, 0]}}]}}"#,
                Uuid::from_u128(n),
                Uuid::from_u128(n | 0xffff_0000_0000_0000_0000),
            );
            responder.file(1, &format!("{side}{n}/manifest.json"), manifest);
            for (path, text) in pack.files {
                responder.file(1, &format!("{side}{n}/{path}"), (*text).to_string());
            }
            list.push(format!(
                r#"{{"pack_id": "{}", "version": [1, 0, 0]}}"#,
                Uuid::from_u128(n)
            ));
        }
        lists.push(format!("[{}]", list.join(", ")));
    }
    let mut repository = RepositoryLoader::new(
        vec![Source::Directory {
            root: RootId(1),
            origin: PackOrigin::World,
        }],
        OpenOptions::default(),
    );
    let repository = responder.drive(&mut repository);
    let world = WorldPacks::parse(Some(lists[0].as_bytes()), Some(lists[1].as_bytes())).unwrap();
    let set = repository.stacks(&world, options);
    let mut loader = StackLoader::new(set, StackOptions::default());
    responder.drive(&mut loader)
}
