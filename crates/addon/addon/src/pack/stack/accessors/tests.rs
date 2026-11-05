//! The accessor macro on toy collections: a test-only invocation over a host that wraps a
//! stack, expanding to a method that drives the loader once and caches the collection; and
//! (`forms`) every line form with the `Collection` enum and the `ENTRIES` table it expands to.

use std::sync::atomic::{AtomicUsize, Ordering};

use bedrock_addon_core::load::{
    Accept, AppData, AppDataPlace, CollectionLoader, ListDepth, LoadContext, LoadOptions, StackCx,
    Walk, WalkFile, WalkOrder,
};
use bedrock_addon_core::pack_identity::{PackPath, PackSide};

use crate::pack::{Behavior, PackStack, Side};

/// How often the toy collection was combined.
static COMBINED: AtomicUsize = AtomicUsize::new(0);

/// What the toy collection holds: each file's path, side and whether it parsed as base game,
/// and how many packs the stack had.
#[derive(Debug, PartialEq)]
struct Toys {
    files: Vec<(String, PackSide, bool)>,
    packs: usize,
}

/// The toy family's `load.rs`.
mod toy {
    use super::*;

    fn is_json(path: &PackPath) -> bool {
        path.as_str().to_ascii_lowercase().ends_with(".json")
    }

    fn parse(file: &WalkFile, cx: &LoadContext) -> (String, PackSide, bool) {
        (file.path.to_string(), file.side, cx.is_base_game_pack)
    }

    fn combine(parsed: Vec<(String, PackSide, bool)>, stack: &StackCx<'_>) -> Toys {
        COMBINED.fetch_add(1, Ordering::SeqCst);
        Toys {
            files: parsed,
            packs: stack.packs.len(),
        }
    }

    pub(super) const LOADER: CollectionLoader<Toys, (String, PackSide, bool)> = CollectionLoader {
        walk: Walk {
            directories: &["features"],
            depth: ListDepth::Recursive,
            accept: Accept::Fn(is_json),
            order: WalkOrder::BottomUp,
            directory_major: false,
            app_data: &["definitions/features"],
            app_data_place: AppDataPlace::After,
            in_pack: None,
            shadowed: None,
            read_error: None,
        },
        parse,
        combine,
        needs: &[],
        then: None,
    };
}

/// A host with its own caches around a stack, as `PackStack` is its own host.
struct ToyHost<S: Side> {
    stack: PackStack<S>,
    caches: ToyCaches,
}

impl<S: Side> ToyHost<S> {
    fn as_stack(&self) -> &PackStack<S> {
        &self.stack
    }
}

accessors! {
    @host ToyCaches for ToyHost;
    /// The toy collection.
    toys: Toys = toy::LOADER, collection = Toys, side = Behavior;
}

#[test]
fn an_accessor_line_expands_to_a_cached_method_that_drives_its_loader() {
    let options = LoadOptions {
        app_data: Some(AppData::from_files([
            (
                PackPath::from_static("definitions/features/b.json"),
                &b"{}"[..],
            ),
            (
                PackPath::from_static("definitions/features/a.json"),
                &b"{}"[..],
            ),
            (
                PackPath::from_static("definitions/features/notes.txt"),
                &b""[..],
            ),
            (
                PackPath::from_static("definitions/attachables/x.json"),
                &b"{}"[..],
            ),
        ])),
        ..LoadOptions::default()
    };
    let host = ToyHost {
        stack: PackStack::<Behavior>::of([], options).expect("an empty stack"),
        caches: ToyCaches::default(),
    };
    let first = host.toys();
    assert_eq!(
        *first,
        Toys {
            files: vec![
                (
                    "definitions/features/a.json".to_owned(),
                    PackSide::Behavior,
                    true
                ),
                (
                    "definitions/features/b.json".to_owned(),
                    PackSide::Behavior,
                    true
                ),
            ],
            packs: 0,
        }
    );
    let second = host.toys();
    assert!(
        std::ptr::eq(first, second),
        "the second call returns the cache"
    );
    assert_eq!(COMBINED.load(Ordering::SeqCst), 1);
}

mod forms;

#[test]
fn providers_name_their_loader_s_accessor() {
    for (variant, walk) in super::provider_walks() {
        let entry = super::ENTRIES
            .iter()
            .find(|entry| entry.collection() == Some(variant) && entry.side() == PackSide::Behavior)
            .unwrap_or_else(|| panic!("{variant:?} has an accessor line"));
        assert_eq!(
            format!("{:?}", entry.walk()),
            format!("{walk:?}"),
            "{variant:?}'s accessor walks what its provider line's loader walks"
        );
        assert!(entry.needs().is_empty(), "{variant:?}");
    }
    assert_eq!(super::provider_walks().len(), super::PROVIDERS.len());
}

/// What every `side = Both` line of the list does today: its two entries walk the same files,
/// need the same collections and run the same `then` on either stack, so its loader does not
/// depend on the side; the side of a file is the stack the walk runs on.
#[test]
fn a_both_sides_line_loads_the_same_way_on_both_sides() {
    for collection in super::Collection::ALL {
        let entries: Vec<_> = super::ENTRIES
            .iter()
            .filter(|entry| entry.collection() == Some(*collection))
            .collect();
        let [behavior, resource] = entries[..] else {
            continue;
        };
        assert_eq!(
            (behavior.side(), resource.side()),
            (PackSide::Behavior, PackSide::Resource),
            "{collection:?}"
        );
        assert_eq!(
            format!("{:?}", behavior.walk()),
            format!("{:?}", resource.walk()),
            "{collection:?} walks the same on both sides"
        );
        assert_eq!(behavior.needs(), resource.needs(), "{collection:?}");
        assert_eq!(behavior.has_then(), resource.has_then(), "{collection:?}");
    }
}
