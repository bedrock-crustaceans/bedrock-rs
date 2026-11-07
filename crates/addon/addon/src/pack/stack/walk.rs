//! The lazy accessor path: a collection's walk over a stack's resident files, one `parse` per
//! file and one `combine`, on the calling thread; the order is the walk's, never a completion
//! order.

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics};
use std::sync::Arc;

use bedrock_addon_core::load::{
    AppData, AppDataPlace, CollectionLoader, DocumentSource, ListedFile, LoadContext, Registered,
    StackCx, StackPack, Walk, WalkFile, WalkOrder,
};
use bedrock_addon_core::molang;
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::{ENGINE_VERSION, MinEngineVersion};
use bytes::Bytes;
use uuid::Uuid;

use super::{PackStack, Side};
use crate::pack::open::storage_finding;
use crate::pack::{Pack, SubpackFacts};
use crate::storage::magic::MCB_MAGIC;
use crate::storage::mcb;

/// Where a walked file comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WalkSource {
    /// The stack entry at this index (0 = top).
    Pack(usize),
    /// `LoadOptions::app_data`.
    AppData,
}

/// One file a walk reaches, in walk order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WalkItem {
    /// Its pack or the app data.
    pub(crate) source: WalkSource,
    /// The file's path in its pack, subpack folder included.
    pub(crate) path: PackPath,
    /// The path the collection keys by: `path` without the subpack folder.
    pub(crate) key: PackPath,
}

/// The `subpacks/` folder.
const SUBPACKS: &str = "subpacks/";

/// The files of one stack entry as `(path, key)`, bytewise by key: the pack's own files, with
/// the active subpack's files (`subpacks/<folder_name>/…`, keyed without the prefix) replacing
/// the pack's file at the same key. Files of the declared subpack folders are no file of the
/// pack itself.
// The game looks a resource up in the active subpack before its parent pack and stops at the
// first hit, so the parent's file at a key the subpack has is never read.
#[cfg_attr(
    not(test),
    allow(
        dead_code,
        reason = "the walk tests name it; the stack takes `entry_listing`"
    )
)]
pub(crate) fn entry_files<'a>(
    paths: impl IntoIterator<Item = &'a PackPath>,
    subpacks: &[SubpackFacts],
    active: Option<usize>,
) -> Vec<ListedFile> {
    entry_listing(paths, subpacks, active).0
}

/// [`entry_files`], and beside it the pack's own files that a file of the active subpack
/// stands in for, bytewise: no file of the walk, and still a file of the pack (a rule that
/// reads both copies needs to know it is there). One pass over the paths.
pub(crate) fn entry_listing<'a>(
    paths: impl IntoIterator<Item = &'a PackPath>,
    subpacks: &[SubpackFacts],
    active: Option<usize>,
) -> (Vec<ListedFile>, Vec<PackPath>) {
    let active = active.and_then(|index| subpacks.get(index));
    let declared = |path: &str| {
        subpacks.iter().any(|subpack| {
            path.strip_prefix(SUBPACKS)
                .and_then(|rest| rest.strip_prefix(&*subpack.folder_name))
                .is_some_and(|rest| rest.starts_with('/'))
        })
    };
    let mut own = Vec::new();
    let mut overlay = Vec::new();
    for path in paths {
        let text = path.as_str();
        let in_active = active.and_then(|subpack| {
            text.strip_prefix(SUBPACKS)?
                .strip_prefix(&*subpack.folder_name)?
                .strip_prefix('/')
        });
        if let Some(rest) = in_active {
            if let Ok(key) = PackPath::new(rest) {
                overlay.push(ListedFile {
                    path: path.clone(),
                    key,
                });
            }
        } else if !declared(text) {
            own.push(ListedFile {
                path: path.clone(),
                key: path.clone(),
            });
        }
    }
    let shadows: std::collections::HashSet<&PackPath> =
        overlay.iter().map(|file| &file.key).collect();
    let (shadowed, mut own): (Vec<_>, Vec<_>) = own
        .into_iter()
        .partition(|file| shadows.contains(&file.key));
    let mut shadowed: Vec<PackPath> = shadowed.into_iter().map(|file| file.path).collect();
    shadowed.sort();
    own.extend(overlay);
    own.sort_by(|a, b| a.key.cmp(&b.key).then_with(|| a.path.cmp(&b.path)));
    (own, shadowed)
}

/// The files `walk` reads, in the order its `combine` sees them: packs in the walk's order
/// (entries are given top first), each pack's files bytewise by key, or each directory over
/// every pack first when the walk is directory-major; app-data files before or after. `packs`
/// are the stack's entries, for the walk's `in_pack` rule: a file the rule leaves out
/// of its pack is no item, so nothing reads it.
pub(crate) fn walk_items(
    walk: &Walk,
    entries: &[Vec<ListedFile>],
    packs: &[StackPack],
    app_data: Option<&AppData>,
) -> Vec<WalkItem> {
    let count = entries.len();
    let rank = |index: usize| match walk.order {
        WalkOrder::BottomUp => count - 1 - index,
        WalkOrder::TopDown => index,
    };
    let mut items: Vec<(usize, usize, WalkItem)> = Vec::new();
    for (index, files) in entries.iter().enumerate() {
        for (directory, path, key) in pack_files(walk, files, packs.get(index)) {
            items.push((
                rank(index),
                directory,
                WalkItem {
                    source: WalkSource::Pack(index),
                    path: path.clone(),
                    key: key.clone(),
                },
            ));
        }
    }
    // Stable: within a pack, files keep their bytewise order.
    if walk.directory_major {
        items.sort_by_key(|(rank, directory, _)| (*directory, *rank));
    } else {
        items.sort_by_key(|(rank, _, _)| *rank);
    }
    let packs = items.into_iter().map(|(_, _, item)| item);
    let app = app_data
        .into_iter()
        .flat_map(AppData::iter)
        .filter(|(path, _)| walk.matches_app_data(path))
        .map(|(path, _)| WalkItem {
            source: WalkSource::AppData,
            path: path.clone(),
            key: path.clone(),
        });
    match walk.app_data_place {
        AppDataPlace::Before => app.chain(packs).collect(),
        AppDataPlace::After => packs.chain(app).collect(),
    }
}

/// The files `walk` reads of one stack entry, as `(directory, path, key)` bytewise by key:
/// those of `files` (the entry's listing) the walk matches and, where the walk asks for them
/// (`Walk::shadowed`), the pack's own files its active subpack shadows, each just
/// before the subpack's file of its key. What the lazy walk and the `StackLoader` both read.
pub(crate) fn pack_files<'a>(
    walk: &Walk,
    files: &'a [ListedFile],
    pack: Option<&'a StackPack>,
) -> Vec<(usize, &'a PackPath, &'a PackPath)> {
    let read: Vec<_> = files
        .iter()
        .filter_map(|file| matches_in(walk, &file.key, pack).map(|at| (at, &file.path, &file.key)))
        .collect();
    let Some(pack) = pack.filter(|_| walk.shadowed.is_some()) else {
        return read;
    };
    let mut own = pack
        .shadowed_files()
        .iter()
        .filter_map(|file| {
            let at = walk.matches_shadowed(&file.path, &pack.context)?;
            Some((at, &file.path, &file.path))
        })
        .peekable();
    if own.peek().is_none() {
        return read;
    }
    // Both are bytewise by key: one merge, the pack's own copy first at a key both have.
    let mut merged = Vec::with_capacity(read.len() + 1);
    for item in read {
        while let Some(own) = own.next_if(|own| own.2 <= item.2) {
            merged.push(own);
        }
        merged.push(item);
    }
    merged.extend(own);
    merged
}

/// The files of a walk's second stage (`CollectionLoader::then`), in the order its
/// `combine` sees them after the walk's own: for each pack in the walk's order, the file at
/// each of `keys` the pack has, bytewise by key and each once. Never a file the walk itself
/// reads (it is an item of the first stage), nor one the walk's `in_pack` leaves out of its
/// pack; a key no pack has is no item.
pub(crate) fn then_items(
    walk: &Walk,
    entries: &[Vec<ListedFile>],
    packs: &[StackPack],
    mut keys: Vec<PackPath>,
) -> Vec<WalkItem> {
    keys.sort();
    keys.dedup();
    let mut order: Vec<usize> = (0..entries.len()).collect();
    if walk.order == WalkOrder::BottomUp {
        order.reverse();
    }
    let mut items = Vec::new();
    for index in order {
        let files = &entries[index];
        let pack = packs.get(index);
        for key in &keys {
            // The listing is bytewise by key, each key once.
            let Ok(at) = files.binary_search_by(|listed| listed.key.cmp(key)) else {
                continue;
            };
            let left_out = walk
                .in_pack
                .zip(pack)
                .is_some_and(|(in_pack, pack)| !in_pack(key, &pack.context));
            if left_out || matches_in(walk, key, pack).is_some() {
                continue;
            }
            items.push(WalkItem {
                source: WalkSource::Pack(index),
                path: files[at].path.clone(),
                key: key.clone(),
            });
        }
    }
    items
}

/// `Walk::matches_in` for a file of `pack`; `Walk::matches` where the pack is not known.
pub(crate) fn matches_in(walk: &Walk, key: &PackPath, pack: Option<&StackPack>) -> Option<usize> {
    match pack {
        Some(pack) => walk.matches_in(key, &pack.context),
        None => walk.matches(key),
    }
}

/// What a `combine` that needs the identifier kinds `needs` sees of `stack`'s other
/// collections: for each kind, the identifiers of the accessor that registers it, loaded now
/// if it is not yet. A kind no accessor of this side registers is left out.
// No cycle: the accessor that registers a kind needs none itself (`accessors::PROVIDERS`).
fn registered_for<S: Side>(needs: &[&str], stack: &PackStack<S>) -> Vec<Arc<Registered>> {
    needs
        .iter()
        .filter_map(|kind| {
            let entry = super::accessors::provider(kind, S::SIDE, super::accessors::ENTRIES)?;
            entry.load(stack);
            entry.registered(stack).map(Arc::new)
        })
        .collect()
}

impl<S: Side> PackStack<S> {
    /// The files of every loaded pack, top first, with its active subpack applied.
    // computed once when the stack is built and again when a pack is replaced.
    pub(crate) fn listed_files(&self) -> Vec<Vec<ListedFile>> {
        self.stack_packs()
            .iter()
            .map(|entry| entry.files().to_vec())
            .collect()
    }

    /// Loads one collection of the stack on the calling thread: walks the resident files, parses each
    /// with its pack's context and combines them with the whole stack in view. A file that is
    /// indexed but not resident is skipped with `FileNotLoaded`, one that cannot be read or decoded
    /// with its read finding; those findings come back beside the collection.
    // accessor caches them with the collection and `StackReport` borrows them.
    #[allow(
        dead_code,
        reason = "the accessor lines call it; the list fills as the families land"
    )]
    pub(crate) fn drive<C, P>(&self, loader: CollectionLoader<C, P>) -> (C, Diagnostics) {
        let entries = self.listed_files();
        let app_data = self.options().app_data.as_ref();
        let items = walk_items(&loader.walk, &entries, self.stack_packs(), app_data);
        let app_data_context = self.app_data_context();
        let mut parsed = Vec::with_capacity(items.len());
        let mut findings = Diagnostics::default();
        let mut read = |item: WalkItem, parsed: &mut Vec<P>| match read_item(
            self,
            item,
            app_data,
            &app_data_context,
        ) {
            Ok(Some((file, context))) => parsed.push((loader.parse)(&file, &context)),
            Ok(None) => {}
            Err(finding) => findings.push(*finding),
        };
        for item in items {
            read(item, &mut parsed);
        }
        // The second stage: the files the first stage's files name.
        if let Some(then) = loader.then {
            let keys = then(&parsed.iter().collect::<Vec<&P>>());
            for item in then_items(&loader.walk, &entries, self.stack_packs(), keys) {
                read(item, &mut parsed);
            }
        }
        let registered = registered_for(loader.needs, self);
        let cx = StackCx::new(self.options(), self.stack_packs()).with_registered(&registered);
        ((loader.combine)(parsed, &cx), findings)
    }

    /// The context app-data files parse with: the game's own data, so a base-game "pack" of the
    /// stack's type at the engine version.
    // App data is not a pack: the game reads it with a context of its own, so the crate parses it
    // with the game's own context for the stack's type at the engine version, not with any
    // pack's.
    pub(crate) fn app_data_context(&self) -> LoadContext {
        let engine = MinEngineVersion::new(
            ENGINE_VERSION.major,
            ENGINE_VERSION.minor,
            ENGINE_VERSION.patch,
        );
        let mut context = LoadContext::for_pack(
            self.options(),
            Uuid::nil(),
            S::PACK_TYPE,
            Some(engine.clone()),
            true,
        );
        context.molang_version = molang::pack_molang_version(Some(&engine));
        context
    }
}

/// One walked item read for its collection's `parse`: the file and the context it parses
/// with (its pack's, or the app data's), `None` for an app-data file the options do not hold
/// (skipped without a finding), or the read finding with the item's pack on it.
///
/// The one read-and-wrap body of the two synchronous drivers, [`PackStack::drive`] and the
/// keyed accessors' visit; the sans-IO `StackLoader` reads outside and shares only
/// [`walk_file`].
pub(crate) fn read_item<'a, S: Side>(
    stack: &'a PackStack<S>,
    item: WalkItem,
    app_data: Option<&'a AppData>,
    app_data_context: &'a LoadContext,
) -> Result<Option<(WalkFile, std::borrow::Cow<'a, LoadContext>)>, Box<Diagnostic>> {
    let (pack, bytes, context) = match item.source {
        WalkSource::Pack(index) => {
            let entry = &stack.stack_packs()[index];
            let (bytes, source) =
                prepare(&stack.instances()[index].pack, &item.path).map_err(|mut finding| {
                    finding.location.pack = Some(entry.pack.clone());
                    finding
                })?;
            (
                entry.pack.clone(),
                bytes,
                context_for(&entry.context, source),
            )
        }
        WalkSource::AppData => {
            let Some(bytes) = app_data.and_then(|files| files.get(&item.path)) else {
                return Ok(None);
            };
            (
                PackRef::app_data(),
                bytes.clone(),
                std::borrow::Cow::Borrowed(app_data_context),
            )
        }
    };
    Ok(Some((walk_file(item, pack, S::SIDE, bytes), context)))
}

/// The [`WalkFile`] a collection's `parse` gets for `item`, read from `pack` on `side`.
pub(crate) fn walk_file(item: WalkItem, pack: PackRef, side: PackSide, bytes: Bytes) -> WalkFile {
    WalkFile {
        pack,
        side,
        path: item.path,
        key: item.key,
        bytes,
    }
}

/// The bytes a collection's `parse` gets for `path` of `pack`: inflated, decrypted, and a
/// binary (`\x7FMCB`) document turned into JSON text. A file that is not resident, cannot be
/// read or cannot be decoded is the finding about it.
pub(crate) fn prepare(
    pack: &Pack,
    path: &PackPath,
) -> Result<(Bytes, DocumentSource), Box<Diagnostic>> {
    let bytes = pack
        .resident(path)
        .map_err(|error| Box::new(storage_finding(path, &error)))?;
    mcb_source(path, bytes)
}

/// `bytes` as JSON text: an MCB document converted, anything else unchanged.
#[cfg(test)]
pub(crate) fn mcb_text(path: &PackPath, bytes: Bytes) -> Result<Bytes, Box<Diagnostic>> {
    mcb_source(path, bytes).map(|(text, _)| text)
}

/// [`mcb_text`] and what the bytes were: an MCB document is `DocumentSource::Mcb`, which the
/// stack records on the context the file is read with.
pub(crate) fn mcb_source(
    path: &PackPath,
    bytes: Bytes,
) -> Result<(Bytes, DocumentSource), Box<Diagnostic>> {
    if !bytes.starts_with(&MCB_MAGIC) {
        return Ok((bytes, DocumentSource::Json));
    }
    mcb::to_json_text(&bytes)
        .map(|text| (Bytes::from(text), DocumentSource::Mcb))
        .map_err(|error| Box::new(storage_finding(path, &error)))
}

/// The context a file of `source` is read with: the pack's own for JSON, a copy that says MCB
/// for a binary document.
pub(crate) fn context_for(
    context: &LoadContext,
    source: DocumentSource,
) -> std::borrow::Cow<'_, LoadContext> {
    if context.source == source {
        std::borrow::Cow::Borrowed(context)
    } else {
        let mut copy = context.clone();
        copy.source = source;
        std::borrow::Cow::Owned(copy)
    }
}

#[cfg(test)]
mod tests {
    use bedrock_addon_core::load::ListDepth;

    use super::*;

    fn paths(list: &[&'static str]) -> Vec<PackPath> {
        list.iter()
            .map(|path| PackPath::from_static(path))
            .collect()
    }

    fn subpacks(folders: &[&str]) -> Vec<SubpackFacts> {
        folders
            .iter()
            .map(|folder| SubpackFacts {
                folder_name: (*folder).into(),
                name: (*folder).into(),
                memory_tier: 0,
            })
            .collect()
    }

    fn texts(files: &[ListedFile]) -> Vec<(&str, &str)> {
        files
            .iter()
            .map(|file| (file.path.as_str(), file.key.as_str()))
            .collect()
    }

    #[test]
    fn the_active_subpack_replaces_the_packs_file_at_its_key() {
        let all = paths(&[
            "textures/a.png",
            "textures/b.png",
            "subpacks/low/textures/a.png",
            "subpacks/low/textures/c.png",
            "subpacks/high/textures/b.png",
            "subpacks/notes.txt",
        ]);
        let files = entry_files(&all, &subpacks(&["high", "low"]), Some(1));
        assert_eq!(
            texts(&files),
            [
                ("subpacks/notes.txt", "subpacks/notes.txt"),
                ("subpacks/low/textures/a.png", "textures/a.png"),
                ("textures/b.png", "textures/b.png"),
                ("subpacks/low/textures/c.png", "textures/c.png"),
            ]
        );
    }

    #[test]
    fn without_an_active_subpack_the_subpack_folders_are_not_read() {
        let all = paths(&["textures/a.png", "subpacks/low/textures/a.png"]);
        let files = entry_files(&all, &subpacks(&["low"]), None);
        assert_eq!(texts(&files), [("textures/a.png", "textures/a.png")]);
    }

    fn json(path: &PackPath) -> bool {
        path.as_str().to_ascii_lowercase().ends_with(".json")
    }

    fn walk(order: WalkOrder, directory_major: bool, app_data_place: AppDataPlace) -> Walk {
        Walk {
            directories: &["entity", "attachables"],
            depth: ListDepth::Recursive,
            accept: bedrock_addon_core::load::Accept::Fn(json),
            order,
            directory_major,
            app_data: &["definitions/attachables"],
            app_data_place,
            in_pack: None,
            shadowed: None,
            read_error: None,
        }
    }

    fn own(list: &[&'static str]) -> Vec<ListedFile> {
        entry_files(&paths(list), &[], None)
    }

    fn order(items: &[WalkItem]) -> Vec<(WalkSource, &str)> {
        items
            .iter()
            .map(|item| (item.source, item.path.as_str()))
            .collect()
    }

    fn two_packs() -> Vec<Vec<ListedFile>> {
        vec![
            own(&[
                "entity/pig.json",
                "attachables/bow.json",
                "entity/notes.txt",
            ]),
            own(&["entity/cow.json", "attachables/axe.json", "items/x.json"]),
        ]
    }

    fn app_data() -> AppData {
        AppData::from_files([
            (
                PackPath::from_static("definitions/attachables/shield.json"),
                &b"{}"[..],
            ),
            (
                PackPath::from_static("definitions/features/f.json"),
                &b"{}"[..],
            ),
        ])
    }

    #[test]
    fn a_bottom_up_walk_reads_the_lowest_pack_first_each_bytewise() {
        let items = walk_items(
            &walk(WalkOrder::BottomUp, false, AppDataPlace::After),
            &two_packs(),
            &[],
            None,
        );
        assert_eq!(
            order(&items),
            [
                (WalkSource::Pack(1), "attachables/axe.json"),
                (WalkSource::Pack(1), "entity/cow.json"),
                (WalkSource::Pack(0), "attachables/bow.json"),
                (WalkSource::Pack(0), "entity/pig.json"),
            ]
        );
    }

    #[test]
    fn a_directory_major_top_down_walk_reads_each_directory_over_every_pack() {
        let items = walk_items(
            &walk(WalkOrder::TopDown, true, AppDataPlace::After),
            &two_packs(),
            &[],
            None,
        );
        assert_eq!(
            order(&items),
            [
                (WalkSource::Pack(0), "entity/pig.json"),
                (WalkSource::Pack(1), "entity/cow.json"),
                (WalkSource::Pack(0), "attachables/bow.json"),
                (WalkSource::Pack(1), "attachables/axe.json"),
            ]
        );
    }

    #[test]
    fn app_data_comes_before_or_after_the_packs() {
        let data = app_data();
        let before = walk_items(
            &walk(WalkOrder::BottomUp, false, AppDataPlace::Before),
            &two_packs(),
            &[],
            Some(&data),
        );
        assert_eq!(
            order(&before)[0],
            (WalkSource::AppData, "definitions/attachables/shield.json")
        );
        assert_eq!(before.len(), 5);
        let after = walk_items(
            &walk(WalkOrder::BottomUp, false, AppDataPlace::After),
            &two_packs(),
            &[],
            Some(&data),
        );
        assert_eq!(
            order(&after)[4],
            (WalkSource::AppData, "definitions/attachables/shield.json")
        );
    }

    #[test]
    fn a_subpack_file_is_walked_by_its_key() {
        let entries = vec![entry_files(
            &paths(&["entity/pig.json", "subpacks/hd/entity/pig.json"]),
            &subpacks(&["hd"]),
            Some(0),
        )];
        let items = walk_items(
            &walk(WalkOrder::BottomUp, false, AppDataPlace::After),
            &entries,
            &[],
            None,
        );
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].path.as_str(), "subpacks/hd/entity/pig.json");
        assert_eq!(items[0].key.as_str(), "entity/pig.json");
    }

    fn pig(path: &PackPath) -> bool {
        path.as_str() == "entity/pig.json"
    }

    /// One stack entry whose active subpack `hd` shadows `shadowed`.
    fn entry_with_shadowed(shadowed: &[&'static str]) -> StackPack {
        use bedrock_addon_core::load::{ListedFile, LoadOptions};
        use bedrock_addon_core::pack_identity::{PackIdVersion, PackOrigin, PackType};
        use bedrock_addon_core::version::SemVersion;

        let id = PackIdVersion::new(
            Uuid::from_u128(1),
            SemVersion::new(1, 0, 0),
            PackType::Resource,
        );
        let context =
            LoadContext::for_pack(&LoadOptions::default(), id.uuid, id.pack_type, None, false);
        StackPack::new(PackRef::new(0, id, "p"), PackOrigin::World, context).with_shadowed_files(
            paths(shadowed).into_iter().map(|path| ListedFile {
                key: path.clone(),
                path,
            }),
        )
    }

    // A walk that asks for a shadowed file reads the pack's own copy just before the
    // subpack's file of that key, at its own path; a shadowed file it does not ask for, or
    // would not read unshadowed (another directory, another extension), stays unread.
    #[test]
    fn a_walk_that_asks_reads_the_pack_s_own_file_before_the_subpack_s() {
        let all = [
            "entity/cow.json",
            "entity/pig.json",
            "entity/zebra.json",
            "items/pig.json",
            "subpacks/hd/entity/cow.json",
            "subpacks/hd/entity/pig.json",
            "subpacks/hd/items/pig.json",
        ];
        let entries = vec![entry_files(&paths(&all), &subpacks(&["hd"]), Some(0))];
        let packs = [entry_with_shadowed(&[
            "entity/cow.json",
            "entity/pig.json",
            "items/pig.json",
        ])];
        let mut walk = walk(WalkOrder::BottomUp, false, AppDataPlace::After);
        let unasked = walk_items(&walk, &entries, &packs, None);
        assert_eq!(
            order(&unasked),
            [
                (WalkSource::Pack(0), "subpacks/hd/entity/cow.json"),
                (WalkSource::Pack(0), "subpacks/hd/entity/pig.json"),
                (WalkSource::Pack(0), "entity/zebra.json"),
            ]
        );
        walk.shadowed = Some(pig);
        let asked = walk_items(&walk, &entries, &packs, None);
        assert_eq!(
            order(&asked),
            [
                (WalkSource::Pack(0), "subpacks/hd/entity/cow.json"),
                (WalkSource::Pack(0), "entity/pig.json"),
                (WalkSource::Pack(0), "subpacks/hd/entity/pig.json"),
                (WalkSource::Pack(0), "entity/zebra.json"),
            ]
        );
        let keys: Vec<&str> = asked.iter().map(|item| item.key.as_str()).collect();
        assert_eq!(
            keys,
            [
                "entity/cow.json",
                "entity/pig.json",
                "entity/pig.json",
                "entity/zebra.json"
            ]
        );
        // The rule alone does not make a file one of the walk: `items/` is no directory of it.
        walk.shadowed = Some(|path| path.as_str() == "items/pig.json");
        assert_eq!(walk_items(&walk, &entries, &packs, None), unasked);
    }

    // A shadowed file whose key sorts after every file the walk reads still comes, last.
    #[test]
    fn a_shadowed_file_after_the_last_read_file_is_read_last() {
        let entries = vec![own(&["entity/cow.json"])];
        let packs = [entry_with_shadowed(&["entity/pig.json"])];
        let mut walk = walk(WalkOrder::BottomUp, false, AppDataPlace::After);
        walk.shadowed = Some(pig);
        let items = walk_items(&walk, &entries, &packs, None);
        assert_eq!(
            order(&items),
            [
                (WalkSource::Pack(0), "entity/cow.json"),
                (WalkSource::Pack(0), "entity/pig.json"),
            ]
        );
    }

    // The second stage reads, pack by pack in the walk's order, the file at each named
    // key the pack has: the subpack's file at its key, never a file the walk reads itself,
    // never one `in_pack` leaves out, and a key no pack has is no item.
    #[test]
    fn a_second_stage_reads_the_named_keys_of_each_pack_in_walk_order() {
        let top = entry_files(
            &paths(&[
                "entity/pig.json",
                "items/a.json",
                "items/b.json",
                "subpacks/hd/items/b.json",
                "items/unnamed.json",
            ]),
            &subpacks(&["hd"]),
            Some(0),
        );
        let entries = vec![top, own(&["items/b.json", "items/z.json"])];
        let keys = || {
            paths(&[
                "items/z.json",
                "items/b.json",
                "items/a.json",
                "items/b.json",
                "entity/pig.json",
                "items/gone.json",
            ])
        };
        let mut walk = walk(WalkOrder::BottomUp, false, AppDataPlace::After);
        let texts = |items: &[WalkItem]| -> Vec<(WalkSource, String, String)> {
            items
                .iter()
                .map(|item| (item.source, item.path.to_string(), item.key.to_string()))
                .collect()
        };
        let item = |pack: usize, path: &str, key: &str| {
            (WalkSource::Pack(pack), path.to_owned(), key.to_owned())
        };
        assert_eq!(
            texts(&then_items(&walk, &entries, &[], keys())),
            [
                item(1, "items/b.json", "items/b.json"),
                item(1, "items/z.json", "items/z.json"),
                item(0, "items/a.json", "items/a.json"),
                item(0, "subpacks/hd/items/b.json", "items/b.json"),
            ]
        );
        walk.order = WalkOrder::TopDown;
        assert_eq!(
            texts(&then_items(&walk, &entries, &[], keys())),
            [
                item(0, "items/a.json", "items/a.json"),
                item(0, "subpacks/hd/items/b.json", "items/b.json"),
                item(1, "items/b.json", "items/b.json"),
                item(1, "items/z.json", "items/z.json"),
            ]
        );
        // The pack rule of the walk holds for the second stage too.
        walk.in_pack = Some(|path, _| path.as_str() != "items/b.json");
        let packs = [entry_with_shadowed(&[]), entry_with_shadowed(&[])];
        assert_eq!(
            texts(&then_items(&walk, &entries, &packs, keys())),
            [
                item(0, "items/a.json", "items/a.json"),
                item(1, "items/z.json", "items/z.json"),
            ]
        );
        assert_eq!(then_items(&walk, &entries, &packs, Vec::new()), []);
    }

    // The stages of a stack walk that run once per pack and per collection (listing a pack with
    // an active subpack, choosing a walk's files, the second stage) cost time linear in the
    // files: each is measured on four times the input.
    #[test]
    fn listing_and_walking_a_pack_is_linear_in_its_files() {
        use crate::test_cost::timed;

        let build = |count: usize| -> Vec<PackPath> {
            (0..count)
                .flat_map(|n| {
                    [
                        format!("entity/e{n:06}.json"),
                        format!("subpacks/hd/entity/e{n:06}.json"),
                        format!("textures/t{n:06}.png"),
                    ]
                })
                .map(|path| PackPath::new(&path).unwrap())
                .collect()
        };
        let walk = walk(WalkOrder::BottomUp, false, AppDataPlace::After);
        crate::test_cost::assert_each_linear_timed_at(
            ["the listing", "the walk", "the second stage"],
            20_000,
            |count| {
                let all = build(count);
                let hd = subpacks(&["hd"]);
                let ((files, shadowed), listing) = timed(|| entry_listing(&all, &hd, Some(0)));
                assert_eq!(shadowed.len() * 2, files.len());
                let entries = vec![files];
                let (items, walking) = timed(|| walk_items(&walk, &entries, &[], None));
                assert_eq!(items.len() * 2, entries[0].len());
                // The second stage names every texture: none is a file of the walk.
                let keys: Vec<PackPath> = entries[0]
                    .iter()
                    .filter(|file| file.key.as_str().starts_with("textures/"))
                    .map(|file| file.key.clone())
                    .collect();
                let (named, then) = timed(|| then_items(&walk, &entries, &[], keys));
                assert_eq!(named.len(), items.len());
                [listing, walking, then]
            },
        );
    }

    /// A resource pack manifest with UUID `n` and one subpack, `hd`.
    fn manifest(n: u8) -> String {
        format!(
            r#"{{
  "format_version": 2,
  "header": {{ "name": "rp {n}", "description": "", "uuid": "00000000-0000-4000-8000-0000000000{n:02}",
              "version": [1, 0, 0], "min_engine_version": [1, 21, 0] }},
  "modules": [{{ "type": "resources", "uuid": "00000000-0000-4000-9000-0000000000{n:02}", "version": [1, 0, 0] }}],
  "subpacks": [{{ "folder_name": "hd", "name": "HD", "memory_tier": 0 }}]
}}"#
        )
    }

    /// Path, key, stack index, base game.
    type Seen = (String, String, Option<usize>, bool);

    fn read_entity(file: &WalkFile, cx: &LoadContext) -> Seen {
        (
            file.path.to_string(),
            file.key.to_string(),
            file.pack.index(),
            cx.is_base_game_pack,
        )
    }

    fn keep(parsed: Vec<Seen>, stack: &StackCx<'_>) -> (Vec<Seen>, usize) {
        (parsed, stack.packs.len())
    }

    fn entities() -> CollectionLoader<(Vec<Seen>, usize), Seen> {
        CollectionLoader {
            walk: Walk {
                directories: &["entity"],
                depth: ListDepth::Recursive,
                accept: bedrock_addon_core::load::Accept::Fn(json),
                order: WalkOrder::BottomUp,
                directory_major: false,
                app_data: &[],
                app_data_place: AppDataPlace::After,
                in_pack: None,
                shadowed: None,
                read_error: None,
            },
            parse: read_entity,
            combine: keep,
            needs: &[],
            then: None,
        }
    }

    #[test]
    fn a_stack_walk_reads_each_packs_resident_files_and_its_active_subpack() {
        use bedrock_addon_core::diagnostic::{Diagnostics, codes as core_codes};
        use bedrock_addon_core::load::LoadOptions;
        use bedrock_addon_core::pack_identity::PackOrigin;

        use crate::pack::stack::{PackInstance, Resource};
        use crate::pack::tests::pack_from;

        let top = pack_from(
            &manifest(1),
            &[
                ("entity/pig.json", Some(b"{}")),
                ("subpacks/hd/entity/pig.json", Some(b"{}")),
                ("entity/cow.json", None),
            ],
            PackOrigin::World,
        );
        let bottom = pack_from(
            &manifest(2),
            &[("entity/sheep.json", Some(b"{}"))],
            PackOrigin::Package,
        );
        let stack = PackStack::<Resource>::new(
            vec![
                PackInstance {
                    pack: top,
                    origin: PackOrigin::World,
                    subpack: Some(0),
                    is_dependent: false,
                },
                PackInstance {
                    pack: bottom,
                    origin: PackOrigin::Package,
                    subpack: None,
                    is_dependent: false,
                },
            ],
            Vec::new(),
            LoadOptions::default(),
            Diagnostics::default(),
        );
        let ((files, packs), findings) = stack.drive(entities());
        assert_eq!(packs, 2);
        let seen: Vec<(&str, &str, Option<usize>, bool)> = files
            .iter()
            .map(|(path, key, index, base)| (path.as_str(), key.as_str(), *index, *base))
            .collect();
        assert_eq!(
            seen,
            [
                ("entity/sheep.json", "entity/sheep.json", Some(1), true),
                (
                    "subpacks/hd/entity/pig.json",
                    "entity/pig.json",
                    Some(0),
                    false
                ),
            ]
        );
        let not_loaded: Vec<_> = findings.iter().collect();
        assert_eq!(not_loaded.len(), 1);
        assert!(not_loaded[0].is(&core_codes::FILE_NOT_LOADED));
        assert_eq!(
            not_loaded[0].location.path.as_ref().map(PackPath::as_str),
            Some("entity/cow.json")
        );
    }
}
