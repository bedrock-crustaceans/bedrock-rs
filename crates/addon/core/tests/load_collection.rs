//! The per-collection loader signature the accessor macro calls: a toy family's
//! `LOADER` driven the way the loaders drive it, a walk's matching rules and an
//! accessor line of the shape the families use.

use bedrock_addon_core::__private::Sealed;
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, NoBeta, Origin, Registry,
    RegistryBuilder, Rejected, VersionInfo,
};
use bedrock_addon_core::id::{BlockId, Id, kind};
use bedrock_addon_core::json::{EncodeError, WriteStyle};
use bedrock_addon_core::load::{
    Accept, AppDataPlace, CollectionLoader, ListDepth, LoadContext, LoadOptions, Registered,
    Registration, StackCx, StackPack, Walk, WalkFile, WalkOrder,
};
use bedrock_addon_core::pack_identity::{
    PackIdVersion, PackOrigin, PackPath, PackRef, PackSide, PackType,
};
use bedrock_addon_core::version::{ENGINE_VERSION, FormatVersion, MinEngineVersion, SemVersion};
use bytes::Bytes;
use uuid::Uuid;

/// Whether `path` ends in `.ext`, the letter case compared exactly.
fn has_extension(path: &PackPath, ext: &str) -> bool {
    path.as_str()
        .rsplit_once('.')
        .is_some_and(|(_, found)| found == ext)
}

/// A toy block: its file holds its identifier.
#[derive(Clone, Debug, PartialEq)]
struct Ruby {
    pack: Option<usize>,
}

impl Sealed for Ruby {}

impl Content for Ruby {
    const NAME: &'static str = "ruby";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    fn parse(_: &PackPath, _: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        unimplemented!("the toy parses in `parse_file`")
    }

    fn write_json(&self, _: &mut Vec<u8>, _: WriteStyle) -> Result<(), EncodeError> {
        Ok(())
    }

    fn to_json(&self) -> Result<String, EncodeError> {
        Ok(String::new())
    }
}

impl Definition for Ruby {
    type Kind = kind::Block;
    type File = Ruby;
    type Beta = NoBeta<kind::Block>;
}

/// The toy family's module, as a family's `load.rs` would read.
mod ruby {
    use super::*;

    fn is_json(path: &PackPath) -> bool {
        has_extension(path, "json")
    }

    const WALK: Walk = Walk {
        directories: &["blocks"],
        depth: ListDepth::Recursive,
        accept: Accept::Fn(is_json),
        order: WalkOrder::BottomUp,
        directory_major: false,
        app_data: &[],
        app_data_place: AppDataPlace::After,
        in_pack: None,
        shadowed: None,
        read_error: None,
    };

    fn parse_file(file: &WalkFile, _: &LoadContext) -> Vec<Contribution<Ruby>> {
        let text = std::str::from_utf8(&file.bytes).unwrap_or_default();
        let v = FormatVersion::new(1, 21, 0);
        let origin = Origin::new(
            file.pack.clone(),
            file.side,
            file.path.clone(),
            None,
            Vec::new(),
        );
        vec![Contribution::Stable(
            Id::parse(text.trim()),
            Document::new(
                Ruby {
                    pack: file.pack.index(),
                },
                origin,
                VersionInfo::new(Some(v), false, v, v, false),
                Diagnostics::default(),
            ),
        )]
    }

    fn combine(parsed: Vec<Vec<Contribution<Ruby>>>, stack: &StackCx<'_>) -> Registry<Ruby> {
        let mut builder = RegistryBuilder::new(
            CombineRule::top_wins(),
            CombineRule::top_wins(),
            stack.options,
        );
        for contribution in parsed.into_iter().flatten() {
            builder.push(contribution);
        }
        builder.finish()
    }

    /// The loader an accessor line names.
    pub const LOADER: CollectionLoader<Registry<Ruby>, Vec<Contribution<Ruby>>> =
        CollectionLoader {
            walk: WALK,
            parse: parse_file,
            combine,
            needs: &[],
            then: None,
        };
}

/// A toy collection that only looks at the stack: its ceiling and how many packs it has.
mod ceiling {
    use super::*;

    fn none(_: &PackPath) -> bool {
        false
    }

    fn parse(_: &WalkFile, _: &LoadContext) {}

    fn combine(_: Vec<()>, stack: &StackCx<'_>) -> (SemVersion, usize) {
        (stack.full_stack_min_engine_version(), stack.packs.len())
    }

    pub const LOADER: CollectionLoader<(SemVersion, usize), ()> = CollectionLoader {
        walk: Walk {
            directories: &[],
            depth: ListDepth::Children,
            accept: Accept::Fn(none),
            order: WalkOrder::BottomUp,
            directory_major: false,
            app_data: &[],
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

#[test]
fn a_stack_pack_carries_the_header_description_when_given_one() {
    let entry = StackPack::new(pack(0), PackOrigin::Dev, LoadContext::detached());
    assert_eq!(entry.description, None);
    let described = entry.clone().with_description(Some("pack.description"));
    assert_eq!(described.description.as_deref(), Some("pack.description"));
    // the rest of the entry is unchanged, and an absent description stays absent
    assert_eq!(described.pack, entry.pack);
    assert_eq!(described.origin, entry.origin);
    assert_eq!(described.with_description(None).description, None);
    assert_eq!(
        entry.with_description(Some("")).description.as_deref(),
        Some("")
    );
}

#[test]
fn a_stack_pack_carries_the_manifest_capabilities_when_given_them() {
    let entry = StackPack::new(pack(0), PackOrigin::Dev, LoadContext::detached());
    assert!(entry.capabilities.is_empty());
    let capable = entry.clone().with_capabilities(["pbr", "raytraced"]);
    let names: Vec<&str> = capable.capabilities.iter().map(AsRef::as_ref).collect();
    assert_eq!(names, ["pbr", "raytraced"]);
    // the rest of the entry is unchanged, and no capability is none
    assert_eq!(capable.pack, entry.pack);
    assert_eq!(capable.origin, entry.origin);
    assert_eq!(capable.description, entry.description);
    let none: [&str; 0] = [];
    assert!(capable.with_capabilities(none).capabilities.is_empty());
}

#[test]
fn a_stack_pack_lists_its_files_by_name_without_their_bytes() {
    let entry = StackPack::new(pack(0), PackOrigin::Dev, LoadContext::detached());
    assert!(
        entry.files().is_empty(),
        "no listing unless the loader gives one"
    );
    let listed = |path: &'static str, key: &'static str| bedrock_addon_core::load::ListedFile {
        path: PackPath::from_static(path),
        key: PackPath::from_static(key),
    };
    let files = [
        listed("subpacks/hd/textures/a.png", "textures/a.png"),
        listed("textures/b.png", "textures/b.png"),
    ];
    let with = entry.clone().with_files(files.clone());
    assert_eq!(with.files(), files);
    // a subpack's file is listed at its key and keeps its own path
    assert_eq!(with.files()[0].key.as_str(), "textures/a.png");
    assert_eq!(with.files()[0].path.as_str(), "subpacks/hd/textures/a.png");
    // the rest of the entry is unchanged, and the listing is shown by its length only
    assert_eq!(with.pack, entry.pack);
    assert_eq!(with.description, entry.description);
    let debug = format!("{with:?}");
    assert!(debug.contains("2 files"), "{debug}");
    assert!(!debug.contains("a.png"), "{debug}");
}

// The files of one directory, at any depth, by bisection.
#[test]
fn a_stack_pack_lists_the_files_below_one_directory() {
    let listed = |path: &'static str, key: &'static str| bedrock_addon_core::load::ListedFile {
        path: PackPath::from_static(path),
        key: PackPath::from_static(key),
    };
    // Bytewise by key, as the loader lists a pack: `-`, `.` sort below `/`, `0`, `_` above.
    let files = [
        listed("manifest.json", "manifest.json"),
        listed("ui", "ui"),
        listed("ui-old/a.json", "ui-old/a.json"),
        listed("ui.json", "ui.json"),
        listed("ui/a.json", "ui/a.json"),
        listed("subpacks/hd/ui/deep/b.png", "ui/deep/b.png"),
        listed("ui/z.txt", "ui/z.txt"),
        listed("ui0/a.json", "ui0/a.json"),
        listed("ui_old/a.json", "ui_old/a.json"),
    ];
    let entry =
        StackPack::new(pack(0), PackOrigin::Dev, LoadContext::detached()).with_files(files.clone());
    let keys = |directory: &str| -> Vec<&str> {
        entry
            .files_below(directory)
            .iter()
            .map(|file| file.key.as_str())
            .collect()
    };
    assert_eq!(keys("ui"), ["ui/a.json", "ui/deep/b.png", "ui/z.txt"]);
    assert_eq!(keys("ui/deep"), ["ui/deep/b.png"]);
    assert_eq!(keys("ui0"), ["ui0/a.json"]);
    assert_eq!(keys("ui-old"), ["ui-old/a.json"]);
    // A file is below no directory of its own name, and a missing directory has no file.
    assert_eq!(keys("manifest.json"), [] as [&str; 0]);
    assert_eq!(keys("u"), [] as [&str; 0]);
    assert_eq!(keys("zz"), [] as [&str; 0]);
    assert_eq!(keys("").len(), files.len());
    // The file keeps its own path.
    assert_eq!(
        entry.files_below("ui/deep")[0].path.as_str(),
        "subpacks/hd/ui/deep/b.png"
    );
    let empty = StackPack::new(pack(0), PackOrigin::Dev, LoadContext::detached());
    assert!(empty.files_below("ui").is_empty());
}

#[test]
fn a_stack_pack_lists_the_files_its_active_subpack_shadows_apart() {
    let entry = StackPack::new(pack(0), PackOrigin::Dev, LoadContext::detached());
    assert!(entry.shadowed_files().is_empty());
    let listed = |path: &'static str, key: &'static str| bedrock_addon_core::load::ListedFile {
        path: PackPath::from_static(path),
        key: PackPath::from_static(key),
    };
    let read = [listed(
        "subpacks/hd/textures/list.json",
        "textures/list.json",
    )];
    let shadowed = [listed("textures/list.json", "textures/list.json")];
    let with = entry
        .with_files(read.clone())
        .with_shadowed_files(shadowed.clone());
    // The shadowed file is no file the stack reads, and the two listings do not mix.
    assert_eq!(with.files(), read);
    assert_eq!(with.shadowed_files(), shadowed);
    let debug = format!("{with:?}");
    assert!(!debug.contains("list.json"), "{debug}");
}

/// The stack of the tests: three packs, top first, with these `min_engine_version`s.
fn stack(versions: [Option<(u16, u16, u16)>; 3]) -> Vec<StackPack> {
    versions
        .into_iter()
        .enumerate()
        .map(|(index, version)| {
            let mut cx = LoadContext::detached();
            cx.min_engine_version = version.map(|(a, b, c)| MinEngineVersion::new(a, b, c));
            StackPack::new(pack(index), PackOrigin::Dev, cx)
        })
        .collect()
}

fn pack(index: usize) -> PackRef {
    PackRef::new(
        index,
        PackIdVersion::new(
            Uuid::from_u128(index as u128 + 1),
            SemVersion::new(1, 0, 0),
            PackType::Behavior,
        ),
        format!("pack {index}"),
    )
}

/// What a loader does with a collection loader, sequentially: pick the files the walk
/// matches, order them by the walk, parse each, combine.
fn drive<C, P>(
    loader: CollectionLoader<C, P>,
    packs: &[StackPack],
    files: &[(usize, &'static str, &'static str)],
) -> C {
    let options = LoadOptions::default();
    let mut walked: Vec<WalkFile> = files
        .iter()
        .filter(|(_, path, _)| loader.walk.matches(&PackPath::from_static(path)).is_some())
        .map(|(index, path, text)| WalkFile {
            pack: pack(*index),
            side: PackSide::Behavior,
            path: PackPath::from_static(path),
            key: PackPath::from_static(path),
            bytes: Bytes::from_static(text.as_bytes()),
        })
        .collect();
    assert_eq!(loader.walk.order, WalkOrder::BottomUp);
    walked.sort_by(|a, b| {
        b.pack
            .index()
            .cmp(&a.pack.index())
            .then_with(|| a.path.cmp(&b.path))
    });
    let parsed = walked
        .iter()
        .map(|file| {
            let cx = LoadContext::detached();
            (loader.parse)(file, &cx)
        })
        .collect();
    (loader.combine)(parsed, &StackCx::new(&options, packs))
}

/// The accessor macro's line shape, `name: Collection = family::LOADER;`, expanding to a function
/// that drives the named loader.
macro_rules! accessors {
    ($($name:ident : $collection:ty = $loader:path;)*) => {
        $(
            fn $name(packs: &[StackPack], files: &[(usize, &'static str, &'static str)]) -> $collection {
                drive($loader, packs, files)
            }
        )*
    };
}

accessors! {
    rubies: Registry<Ruby> = ruby::LOADER;
    stack_ceiling: (SemVersion, usize) = ceiling::LOADER;
}

fn id(text: &'static str) -> BlockId {
    Id::from_static(text)
}

#[test]
fn an_accessor_line_drives_the_family_loader() {
    let registry = rubies(
        &stack([None; 3]),
        &[
            (0, "blocks/ruby.json", "demo:ruby"),
            (1, "blocks/ruby.json", "demo:ruby"),
            (1, "blocks/nested/gem.json", "demo:gem"),
            (1, "blocks/notes.txt", "demo:ignored"),
            (1, "items/ruby.json", "demo:item"),
        ],
    );
    assert_eq!(registry.len(), 2);
    assert_eq!(registry.get(&id("demo:ruby")).unwrap().pack, Some(0));
    assert_eq!(registry.get(&id("demo:gem")).unwrap().pack, Some(1));
    assert!(registry.get(&id("demo:ignored")).is_none());
    // Walk order: pack 1 first, its files bytewise; pack 0 overrides in place.
    let order: Vec<&str> = registry.iter().map(|(id, _)| id.as_str()).collect();
    assert_eq!(order, ["demo:gem", "demo:ruby"]);
}

fn any(_: &PackPath) -> bool {
    true
}

fn walk(directories: &'static [&'static str], depth: ListDepth) -> Walk {
    Walk {
        directories,
        depth,
        accept: Accept::Fn(any),
        order: WalkOrder::BottomUp,
        directory_major: true,
        app_data: &["definitions/attachables"],
        app_data_place: AppDataPlace::Before,
        in_pack: None,
        shadowed: None,
        read_error: None,
    }
}

#[test]
fn a_walk_matches_its_directories_at_its_depth() {
    let children = walk(&["entity", "attachables"], ListDepth::Children);
    let path = PackPath::from_static;
    assert_eq!(children.matches(&path("entity/pig.json")), Some(0));
    assert_eq!(children.matches(&path("attachables/bow.json")), Some(1));
    assert_eq!(children.matches(&path("entity/sub/pig.json")), None);
    assert_eq!(children.matches(&path("entity_extra/pig.json")), None);
    assert_eq!(children.matches(&path("entity")), None);
    let recursive = walk(&["entity"], ListDepth::Recursive);
    assert_eq!(recursive.matches(&path("entity/sub/pig.json")), Some(0));
    let root = walk(&[""], ListDepth::Children);
    assert_eq!(root.matches(&path("manifest.json")), Some(0));
    assert_eq!(root.matches(&path("texts/en_US.lang")), None);
}

#[test]
fn a_walk_matches_app_data_in_its_app_data_directories() {
    let walk = walk(&["attachables"], ListDepth::Recursive);
    assert!(walk.matches_app_data(&PackPath::from_static("definitions/attachables/bow.json")));
    assert!(!walk.matches_app_data(&PackPath::from_static("definitions/features/a.json")));
}

#[test]
fn the_accept_rule_filters_before_the_directories() {
    fn lang(path: &PackPath) -> bool {
        has_extension(path, "lang")
    }
    let texts = Walk {
        accept: Accept::Fn(lang),
        ..walk(&["texts"], ListDepth::Children)
    };
    assert_eq!(
        texts.matches(&PackPath::from_static("texts/en_US.lang")),
        Some(0)
    );
    assert_eq!(
        texts.matches(&PackPath::from_static("texts/languages.json")),
        None
    );
}

#[test]
fn combine_sees_every_pack_of_the_stack_and_its_ceiling() {
    // No pack has a file the walk matches; the stack is still in view.
    let (ceiling, packs) = stack_ceiling(&stack([Some((1, 20, 0)), None, Some((1, 21, 10))]), &[]);
    assert_eq!(packs, 3);
    assert_eq!(ceiling, SemVersion::new(1, 21, 10));
}

#[test]
fn the_ceiling_of_a_stack_without_versions_is_the_engine_version() {
    let engine = SemVersion::new(
        ENGINE_VERSION.major,
        ENGINE_VERSION.minor,
        ENGINE_VERSION.patch,
    );
    assert_eq!(stack_ceiling(&stack([None; 3]), &[]).0, engine);
    assert_eq!(stack_ceiling(&[], &[]), (engine, 0));
}

// ---- one collection's combine asks about another's identifiers ----

/// A toy collection that places rubies: for each line of its files, which pack of the stack
/// registered the block of that name, as far as the stack lets it see.
mod setting {
    use super::*;
    use bedrock_addon_core::load::Registration;

    fn is_list(path: &PackPath) -> bool {
        has_extension(path, "txt")
    }

    fn parse(file: &WalkFile, _: &LoadContext) -> Vec<String> {
        std::str::from_utf8(&file.bytes)
            .unwrap_or_default()
            .lines()
            .map(str::to_owned)
            .collect()
    }

    /// `Some(Some(index))` registered by that pack, `Some(None)` by none, `None` not in view.
    pub type Answer = Option<Option<usize>>;

    fn combine(parsed: Vec<Vec<String>>, stack: &StackCx<'_>) -> Vec<(String, Answer, Answer)> {
        let answer = |registration: Registration<'_>| match registration {
            Registration::NotInView => None,
            Registration::Absent => Some(None),
            Registration::By { pack, .. } => Some(pack.index()),
        };
        parsed
            .into_iter()
            .flatten()
            .map(|name| {
                let block = stack.registered(&BlockId::parse(name.as_str()));
                let item = stack.registered(&bedrock_addon_core::id::ItemId::parse(name.as_str()));
                (name, answer(block), answer(item))
            })
            .collect()
    }

    pub const LOADER: CollectionLoader<Vec<(String, Answer, Answer)>, Vec<String>> =
        CollectionLoader {
            walk: Walk {
                directories: &["settings"],
                depth: ListDepth::Recursive,
                accept: Accept::Fn(is_list),
                order: WalkOrder::BottomUp,
                directory_major: false,
                app_data: &[],
                app_data_place: AppDataPlace::After,
                in_pack: None,
                shadowed: None,
                read_error: None,
            },
            parse,
            combine,
            needs: &[<kind::Block as bedrock_addon_core::id::Kind>::NAME],
            then: None,
        };
}

/// A collection that registers a kind for the others needs none itself: that is what keeps
/// the needs of a stack free of cycles, and it is a compile-time fact of each loader.
const _: () = assert!(ruby::LOADER.needs.is_empty());

#[test]
fn a_combine_asks_the_stack_which_pack_registered_an_identifier() {
    use std::sync::Arc;
    let packs = stack([None; 3]);
    let files = [
        (0, "blocks/ruby.json", "demo:ruby"),
        (1, "blocks/ruby.json", "demo:ruby"),
        (2, "blocks/gem.json", "demo:gem"),
        (
            1,
            "settings/ring.txt",
            "demo:ruby\ndemo:gem\ndemo:glass\ngem",
        ),
    ];
    let loader = setting::LOADER;
    assert_eq!(loader.needs, ["block"]);
    // The loaders combine the collection that registers the kind first …
    let rubies = rubies(&packs, &files);
    let view = [Arc::new(rubies.registered())];
    // … and hand its identifiers to the combine that needs them.
    let options = LoadOptions::default();
    let cx = LoadContext::detached();
    let file = WalkFile {
        pack: pack(1),
        side: PackSide::Behavior,
        path: PackPath::from_static("settings/ring.txt"),
        key: PackPath::from_static("settings/ring.txt"),
        bytes: Bytes::from_static(files[3].2.as_bytes()),
    };
    let parsed = vec![(loader.parse)(&file, &cx)];
    let placed = (loader.combine)(
        parsed.clone(),
        &StackCx::new(&options, &packs).with_registered(&view),
    );
    assert_eq!(
        placed,
        [
            ("demo:ruby".to_owned(), Some(Some(0)), None),
            ("demo:gem".to_owned(), Some(Some(2)), None),
            ("demo:glass".to_owned(), Some(None), None),
            // `gem` is `minecraft:gem` as a block identifier: not `demo:gem`.
            ("gem".to_owned(), Some(None), None),
        ],
        "blocks are in view (by pack, or absent); items are not"
    );
    // The answer carries the identifier as its definition spells it: `ruby` and
    // `minecraft:ruby` are one block identifier, written `minecraft:ruby` by the pack.
    let spelled = [Arc::new(Registered::new::<kind::Block>([(
        id("minecraft:ruby"),
        pack(2),
    )]))];
    let cx = StackCx::new(&options, &packs).with_registered(&spelled);
    let pack_2 = pack(2);
    assert_eq!(
        cx.registered(&id("ruby")),
        Registration::By {
            pack: &pack_2,
            written: "minecraft:ruby"
        }
    );
    assert_eq!(cx.registered(&id("demo:ruby")), Registration::Absent);
    // Without the view every kind is out of view: `StackCx::new` shows no other collection.
    let blind = (loader.combine)(parsed, &StackCx::new(&options, &packs));
    assert!(
        blind
            .iter()
            .all(|(_, block, item)| block.is_none() && item.is_none())
    );
}

#[test]
fn a_registration_says_whether_a_pack_registered_the_identifier() {
    use bedrock_addon_core::load::Registration;
    let pack = pack(0);
    let by = Registration::By {
        pack: &pack,
        written: "demo:ruby",
    };
    assert!(by.is_registered());
    assert!(!Registration::Absent.is_registered());
    assert!(!Registration::NotInView.is_registered());
}

// ---- a walk that leaves files out by their pack ----

fn not_templates_of_the_base_game(path: &PackPath, cx: &LoadContext) -> bool {
    !(cx.is_base_game_pack && has_extension(path, "nbt"))
}

#[test]
fn a_walk_can_leave_a_file_out_by_its_pack_s_context() {
    let mut walk = walk(&["structures"], ListDepth::Recursive);
    let template = PackPath::from_static("structures/ruin.nbt");
    let structure = PackPath::from_static("structures/ruin.mcstructure");
    let add_on = LoadContext::detached();
    let mut base_game = LoadContext::detached();
    base_game.is_base_game_pack = true;
    // Without the rule every pack's matching file is read.
    assert_eq!(walk.matches_in(&template, &base_game), Some(0));
    walk.in_pack = Some(not_templates_of_the_base_game);
    assert_eq!(walk.matches_in(&template, &base_game), None);
    assert_eq!(walk.matches_in(&template, &add_on), Some(0));
    assert_eq!(walk.matches_in(&structure, &base_game), Some(0));
    // The rule narrows `matches`, it never widens it; `matches` itself does not ask it.
    assert_eq!(
        walk.matches_in(&PackPath::from_static("blocks/ruin.nbt"), &add_on),
        None
    );
    assert_eq!(walk.matches(&template), Some(0));
}

// ---- a walk that also reads a shadowed file ----

fn the_list(path: &PackPath) -> bool {
    path.as_str() == "textures/list.json"
}

#[test]
fn a_walk_reads_a_shadowed_file_only_when_it_asks_and_would_read_it_unshadowed() {
    let mut walk = walk(&["textures"], ListDepth::Recursive);
    let list = PackPath::from_static("textures/list.json");
    let other = PackPath::from_static("textures/other.json");
    let add_on = LoadContext::detached();
    let mut base_game = LoadContext::detached();
    base_game.is_base_game_pack = true;
    // Without the rule no shadowed file is read: one file per key.
    assert_eq!(walk.matches_shadowed(&list, &add_on), None);
    walk.shadowed = Some(the_list);
    assert_eq!(walk.matches_shadowed(&list, &add_on), Some(0));
    assert_eq!(walk.matches_shadowed(&other, &add_on), None);
    // The rule never widens the walk: another directory, or a file the pack rule leaves out.
    walk.shadowed = Some(any);
    assert_eq!(
        walk.matches_shadowed(&PackPath::from_static("blocks/list.json"), &add_on),
        None
    );
    walk.in_pack = Some(|_, cx| !cx.is_base_game_pack);
    assert_eq!(walk.matches_shadowed(&list, &base_game), None);
    assert_eq!(walk.matches_shadowed(&list, &add_on), Some(0));
    // What the walk reads unshadowed does not depend on the rule.
    assert_eq!(walk.matches_in(&other, &add_on), Some(0));
}

/// `Accept`, the presets, and the modifiers, against the literals they stand for.
mod presets {
    use super::*;

    fn path(text: &str) -> PackPath {
        PackPath::new(text).unwrap()
    }

    const PATHS: &[&str] = &[
        "blocks/a.json",
        "blocks/a.JSON",
        "blocks/x/.json",
        "blocks/a.json/b",
        "blocks/a",
        "blocks/a.b.json",
        "blocks/deep/er/c.json",
        "blocks.json",
        "items/a.json",
        "credits/credits.json",
        "a.json",
    ];

    /// Every field but `accept` of two walks, and their answers over `PATHS`.
    fn assert_same(new: &Walk, old: &Walk) {
        assert_eq!(new.directories, old.directories);
        assert_eq!(new.depth, old.depth);
        assert_eq!(new.order, old.order);
        assert_eq!(new.directory_major, old.directory_major);
        assert_eq!(new.app_data, old.app_data);
        assert_eq!(new.app_data_place, old.app_data_place);
        assert_eq!(new.in_pack.is_some(), old.in_pack.is_some());
        assert_eq!(new.shadowed.is_some(), old.shadowed.is_some());
        assert_eq!(new.read_error, old.read_error);
        for text in PATHS {
            let p = path(text);
            assert_eq!(new.matches(&p), old.matches(&p), "{text}");
            assert_eq!(new.matches_app_data(&p), old.matches_app_data(&p), "{text}");
        }
    }

    fn json(path: &PackPath) -> bool {
        has_extension(path, "json")
    }

    /// The ten-field literal of a JSON tree below `blocks/`, as 28 families spell it.
    const BLOCKS: Walk = Walk {
        directories: &["blocks"],
        depth: ListDepth::Recursive,
        accept: Accept::Fn(json),
        order: WalkOrder::BottomUp,
        directory_major: false,
        app_data: &[],
        app_data_place: AppDataPlace::After,
        in_pack: None,
        shadowed: None,
        read_error: None,
    };

    #[test]
    fn json_tree_in_is_the_literal_of_a_json_tree() {
        const WALK: Walk = Walk::json_tree_in(&["blocks"]);
        assert_same(&WALK, &BLOCKS);
        assert!(matches!(WALK.accept, Accept::Ext("json")));
    }

    #[test]
    fn json_children_in_reads_direct_children_only() {
        const WALK: Walk = Walk::json_children_in(&["blocks"]);
        assert_same(
            &WALK,
            &Walk {
                depth: ListDepth::Children,
                ..BLOCKS
            },
        );
        assert_eq!(WALK.matches(&path("blocks/a.json")), Some(0));
        assert_eq!(WALK.matches(&path("blocks/deep/er/c.json")), None);
    }

    #[test]
    fn exact_files_of_one_path_reads_that_path_and_nothing_else() {
        const WALK: Walk = Walk::exact_files(&["credits/credits.json"]);
        for text in PATHS {
            let p = path(text);
            let expected = (*text == "credits/credits.json").then_some(0);
            assert_eq!(WALK.matches(&p), expected, "{text}");
        }
        assert_eq!(WALK.directories, &[""]);
        assert!(WALK.app_data.is_empty());
    }

    #[test]
    fn exact_files_reads_each_listed_path_and_nothing_else() {
        // The shape of font (`font/default8.json`, `font/emoticons.json`) and json_ui
        // (`ui/_ui_defs.json`, `ui/_global_variables.json`): two fixed files of the pack.
        const LISTED: &[&str] = &["ui/_ui_defs.json", "ui/_global_variables.json"];
        const WALK: Walk = Walk::exact_files(LISTED);
        let others = [
            "ui/_ui_defs.JSON",
            "ui/hud_screen.json",
            "ui/deep/_ui_defs.json",
            "_ui_defs.json",
            "font/default8.json",
        ];
        for text in LISTED {
            assert_eq!(WALK.matches(&path(text)), Some(0), "{text}");
            assert!(WALK.accept.takes(&path(text)), "{text}");
        }
        for text in others.iter().chain(PATHS) {
            assert_eq!(WALK.matches(&path(text)), None, "{text}");
            assert!(!WALK.accept.takes(&path(text)), "{text}");
        }
        assert_eq!(WALK.directories, &[""]);
        assert_eq!(WALK.depth, ListDepth::Recursive);
        assert!(WALK.app_data.is_empty());
    }

    #[test]
    fn none_reads_nothing() {
        const WALK: Walk = Walk::none();
        assert!(WALK.directories.is_empty());
        for text in PATHS {
            assert_eq!(WALK.matches(&path(text)), None, "{text}");
            assert!(!WALK.matches_app_data(&path(text)), "{text}");
        }
    }

    #[test]
    fn each_accept_arm_takes_its_paths() {
        let ext = Accept::Ext("json");
        let exact = Accept::Exact(&["credits/credits.json"]);
        let custom = Accept::Fn(|path| path.as_str().starts_with("items/"));
        for text in PATHS {
            let p = path(text);
            assert_eq!(ext.takes(&p), p.is_json(), "{text}");
            assert_eq!(exact.takes(&p), *text == "credits/credits.json", "{text}");
            assert_eq!(custom.takes(&p), text.starts_with("items/"), "{text}");
        }
        // `Ext` agrees with the old `rsplit_once` body over the table.
        for text in PATHS {
            assert_eq!(ext.takes(&path(text)), json(&path(text)), "{text}");
        }
    }

    fn in_pack(_: &PackPath, _: &LoadContext) -> bool {
        true
    }

    fn shadowed(_: &PackPath) -> bool {
        true
    }

    #[test]
    fn the_modifiers_set_one_field_each() {
        const WALK: Walk = Walk::json_tree_in(&["blocks", "items"])
            .top_down()
            .directory_major()
            .app_data(&["blocks"], AppDataPlace::Before)
            .in_pack(in_pack)
            .shadowed(shadowed)
            .read_error("could not read {}");
        assert_same(
            &WALK,
            &Walk {
                directories: &["blocks", "items"],
                order: WalkOrder::TopDown,
                directory_major: true,
                app_data: &["blocks"],
                app_data_place: AppDataPlace::Before,
                in_pack: Some(in_pack),
                shadowed: Some(shadowed),
                read_error: Some("could not read {}"),
                ..BLOCKS
            },
        );
    }
}
