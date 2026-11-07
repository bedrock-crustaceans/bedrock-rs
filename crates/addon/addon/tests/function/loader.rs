//! `bp.tick_functions()` through a `StackLoader`: the tick list asks
//! the stack's function registry which names exist, so the function files are read once and a
//! loader finding on one of them is listed once.

use std::io::ErrorKind;

use bedrock_addon::load::{
    Collection, CollectionSet, LoadOptions, RepositoryLoader, RootId, Source, StackLoader,
    StackOptions,
};
use bedrock_addon::pack::{OpenOptions, PackOrigin, StackSet, WorldPacks};

use super::responder::{Order, Responder, manifest};

/// One behaviour pack on disk: a function that loads, one whose file cannot be read, and a
/// tick list naming both.
fn world() -> Responder {
    let mut responder = Responder::new(Order::Issue);
    responder
        .file(1, "bp/manifest.json", manifest(1, "data"))
        .file(1, "bp/functions/ok.mcfunction", &b"say hi\n"[..])
        .file(1, "bp/functions/broken.mcfunction", &b"say hi\n"[..])
        .file(
            1,
            "bp/functions/tick.json",
            &br#"{"values": ["ok", "broken"]}"#[..],
        );
    responder
}

/// The stacks after a stack load of `collections`, completed in `order`.
fn load(order: Order, collections: CollectionSet) -> StackSet {
    let mut responder = world();
    let mut repository = RepositoryLoader::new(
        vec![Source::Directory {
            root: RootId(1),
            origin: PackOrigin::World,
        }],
        OpenOptions::default(),
    );
    let repository = responder.drive(&mut repository);
    let entry = format!(
        r#"[{{"pack_id": "{}", "version": [1, 0, 0]}}]"#,
        uuid::Uuid::from_u128(1)
    );
    let world = WorldPacks::parse(Some(entry.as_bytes()), None).unwrap();
    let set = repository.stacks(&world, LoadOptions::default());
    let mut responder = responder.reordered(order);
    responder.failed(
        1,
        "bp/functions/broken.mcfunction",
        ErrorKind::PermissionDenied,
    );
    let options = StackOptions {
        collections,
        ..StackOptions::default()
    };
    let mut loader = StackLoader::new(set, options);
    responder.drive(&mut loader)
}

/// How many findings of the stack's report are about the unreadable function file.
fn about_the_broken_file(set: &StackSet) -> usize {
    set.behavior
        .report()
        .iter()
        .filter(|f| {
            f.location
                .path
                .as_ref()
                .is_some_and(|path| path.as_str() == "functions/broken.mcfunction")
        })
        // By legacy name: `READ_ERROR` and `READ_ERROR_WORDED` (the walk's sentence).
        .inspect(|f| assert!(f.code.name() == "ReadError", "{f:?}"))
        .count()
}

#[test]
fn a_stack_load_of_the_tick_list_resolves_against_the_stack_s_functions() {
    for collections in [
        CollectionSet::only([Collection::TickFunctions]),
        CollectionSet::ALL,
    ] {
        for order in Order::ALL {
            let set = load(order, collections);
            let tick = set.behavior.tick_functions();
            let names: Vec<&str> = tick.values.iter().map(|e| e.function.as_str()).collect();
            assert_eq!(names, ["ok"], "{order:?}");
            // `broken` exists in no pack's registry: the M10 warning, and nothing else.
            let found: Vec<&'static str> = tick.diagnostics().map(|f| f.code.name()).collect();
            assert_eq!(found, ["UnresolvedReference"], "{order:?}");
            // The function registry was loaded for it, selected or not.
            assert_eq!(set.behavior.functions().len(), 1, "{order:?}");
            // One read of the file, one finding: the function registry's.
            assert_eq!(about_the_broken_file(&set), 1, "{order:?} {collections:?}");
        }
    }
}
