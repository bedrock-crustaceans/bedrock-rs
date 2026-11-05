//! `StackLoader`: reads, parses and combines the selected collections of a composed `StackSet`.
//! Each `Parse` job fills a slot keyed by collection and walk position, and a collection
//! combines once every slot is filled, in walk order. A collection with a second walk stage
//! (`CollectionLoader::then`) first asks, in a job, which further files its parsed
//! files name; their slots follow the walk's and are read and parsed the same way.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Arc;

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics};
use bedrock_addon_core::load::{
    DocumentSource, LoadContext, LoadOptions, Place, ReadRange, Registered, RootId, StackCx,
    StackPack,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bytes::Bytes;

use super::collection::ErasedCollection;
use super::io::{Charge, Expect, Io};
use super::job::{Cancel, Erased, JobDone, JobId, JobKind};
use super::pack::{internal_error, read_error};
use super::{Input, Loader, LoaderError, Output, Progress, Residency, Sans, StackOptions, Stage};
use crate::pack::open::storage_finding;
use crate::pack::stack::{
    ENTRIES, WalkItem, WalkSource, pack_files, provider, then_items, walk_file, walk_items,
};
use crate::pack::stack::{PackStack, Side};
use crate::pack::{FileSlot, Pack, StackSet};
use crate::storage::brarchive::{Brarchive, archive_dir};

/// Reads and parses the content of an already-composed [`StackSet`] (from
/// `PackRepository::stacks`, which stays pure: selection, dependencies, slices, subpacks).
/// Finishes with the same stacks, every requested collection filled.
///
/// A loader is driven by answering its outputs until [`Output::Done`]; the example of [`Loader`]
/// shows the loop.
pub struct StackLoader {
    io: Io,
    set: Option<StackSet>,
    /// The distinct packs of both stacks.
    packs: Vec<PackWork>,
    /// Per side (behaviour, resource), the pack of each stack index.
    instances: [Vec<usize>; 2],
    collections: Vec<CollectionWork>,
    /// Outstanding reads: the pack and path each answers.
    places: HashMap<Place, (usize, PackPath)>,
    jobs: HashMap<JobId, JobFor>,
    /// Brarchive reads and `Index` jobs still out; parsing starts when none is.
    brarchives_out: usize,
    parsing: bool,
    done: bool,
}

/// One distinct pack of the stacks and what loading it involves.
struct PackWork {
    pack: Pack,
    residency: Residency,
    /// Loose files read for the collections, by path.
    reads: BTreeMap<PackPath, Read>,
    /// The decoded brarchives, by path, merged in path order once all are in.
    brarchives: BTreeMap<PackPath, Brarchive>,
    /// Findings about the pack's brarchives, by archive path, so their order does not depend
    /// on completion order (they join the pack's report).
    findings: BTreeMap<PackPath, Diagnostic>,
    /// Bytes to keep resident (`Residency::All`).
    keep: BTreeMap<PackPath, Bytes>,
}

/// A loose file read for the collections.
enum Read {
    /// Not answered yet; the slots waiting for it.
    Waiting(Vec<(usize, usize)>),
    /// Answered; the slots that still have to parse it.
    Answered { bytes: Bytes, uses: usize },
    /// Answered with `Missing` or `Failed`.
    Failed(std::io::ErrorKind),
}

/// One selected collection on its way to its cache.
struct CollectionWork {
    entry: &'static dyn ErasedCollection,
    side: PackSide,
    /// One slot per walked file, in walk order.
    slots: Vec<Slot>,
    open: usize,
    /// Where the collection's second walk stage stands.
    then: Then,
    combining: bool,
    installed: bool,
}

/// The second walk stage of a collection (`CollectionLoader::then`).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Then {
    /// The collection has none, or its slots are there already.
    Settled,
    /// To ask once every slot of the walk is filled.
    Due,
    /// The job that names the files is out; the parsed files are lent to it.
    Asked,
}

/// One walked file of a collection.
struct Slot {
    pack: Option<usize>,
    stack_index: Option<usize>,
    /// The handle of the slot's pack, for its findings.
    pack_ref: Option<PackRef>,
    path: PackPath,
    key: PackPath,
    state: SlotState,
}

impl Slot {
    /// The unread slot of a walked item: `packs` are the loader's pack indices of the
    /// stack's entries, `stack_packs` the entries.
    fn pending(item: WalkItem, packs: &[usize], stack_packs: &[StackPack]) -> Slot {
        let stack_index = match item.source {
            WalkSource::Pack(index) => Some(index),
            WalkSource::AppData => None,
        };
        Slot {
            pack: stack_index.map(|index| packs[index]),
            stack_index,
            pack_ref: stack_index.map(|index| stack_packs[index].pack.clone()),
            path: item.path,
            key: item.key,
            state: SlotState::Pending,
        }
    }

    /// The walked item the slot stands for.
    fn item(&self) -> WalkItem {
        WalkItem {
            source: self
                .stack_index
                .map_or(WalkSource::AppData, WalkSource::Pack),
            path: self.path.clone(),
            key: self.key.clone(),
        }
    }
}

enum SlotState {
    Pending,
    Parsed(Erased),
    /// Parsed and lent to the job that names the second stage's files.
    Lent,
    /// Parsed and handed to the `Combine` job.
    Combined,
    /// The file could not be read or parsed; the finding is the collection's.
    Skipped(Box<Diagnostic>),
}

/// What a job is for.
#[derive(Clone)]
enum JobFor {
    Brarchive {
        pack: usize,
        path: PackPath,
    },
    Parse {
        collection: usize,
        slot: usize,
    },
    /// Names the files of a collection's second walk stage.
    Then {
        collection: usize,
    },
    /// `empty`: the replacement combine of no files after a panic.
    Combine {
        collection: usize,
        empty: bool,
    },
}

impl StackLoader {
    /// A loader over `stacks`, reading what `options` selects.
    #[allow(
        clippy::needless_pass_by_value,
        reason = "the options are passed by value and read once, here"
    )]
    pub fn new(stacks: StackSet, options: StackOptions) -> StackLoader {
        let selected = |entry: &&'static dyn ErasedCollection| {
            entry
                .collection()
                .is_none_or(|collection| options.collections.contains(collection))
        };
        // A selected collection whose `combine` needs the identifiers of another brings that
        // one along: it is loaded and installed too, selected or not.
        let needed: Vec<&'static dyn ErasedCollection> = ENTRIES
            .iter()
            .filter(|entry| selected(entry))
            .flat_map(|entry| {
                entry
                    .needs()
                    .iter()
                    .filter_map(|kind| provider(kind, entry.side(), ENTRIES))
            })
            .collect();
        let table: Vec<&'static dyn ErasedCollection> = ENTRIES
            .iter()
            .copied()
            .filter(|entry| selected(entry) || needed.iter().any(|n| same_entry(*n, *entry)))
            .collect();
        StackLoader::with_table(stacks, &options, table)
    }

    /// A loader over `stacks` for the collections of `table` (tests use toy tables).
    pub(crate) fn with_table(
        stacks: StackSet,
        options: &StackOptions,
        table: Vec<&'static dyn ErasedCollection>,
    ) -> StackLoader {
        let mut loader = StackLoader {
            io: Io::new(options.limits),
            set: None,
            packs: Vec::new(),
            instances: [Vec::new(), Vec::new()],
            collections: Vec::new(),
            places: HashMap::new(),
            jobs: HashMap::new(),
            brarchives_out: 0,
            parsing: false,
            done: false,
        };
        loader.io.progress.stage = Stage::Read;
        loader.instances[0] = loader.distinct(&stacks.behavior, options.behavior);
        loader.instances[1] = loader.distinct(&stacks.resource, options.resource);
        for entry in table {
            if entry.is_loaded(stacks.stack_of(entry.side())) {
                continue;
            }
            loader.collections.push(CollectionWork {
                entry,
                side: entry.side(),
                slots: Vec::new(),
                open: 0,
                then: if entry.has_then() {
                    Then::Due
                } else {
                    Then::Settled
                },
                combining: false,
                installed: false,
            });
        }
        loader.queue_reads(&stacks);
        loader.set = Some(stacks);
        loader.start_parsing_if_ready();
        loader
    }

    /// The pack of each stack index, deduplicated by identity of the `Pack`.
    fn distinct<S: Side>(&mut self, stack: &PackStack<S>, residency: Residency) -> Vec<usize> {
        stack
            .instances()
            .iter()
            .map(|instance| {
                if let Some(index) = self.packs.iter().position(|p| p.pack.same(&instance.pack)) {
                    return index;
                }
                self.packs.push(PackWork {
                    pack: instance.pack.clone(),
                    residency,
                    reads: BTreeMap::new(),
                    brarchives: BTreeMap::new(),
                    findings: BTreeMap::new(),
                    keep: BTreeMap::new(),
                });
                self.packs.len() - 1
            })
            .collect()
    }

    /// First, the brarchives of every pack, then the loose files a selected collection walks
    /// (every file under `Residency::All`). Packs from memory are never read.
    fn queue_reads(&mut self, set: &StackSet) {
        let mut wanted: Vec<BTreeSet<PackPath>> = vec![BTreeSet::new(); self.packs.len()];
        for work in &self.collections {
            let walk = work.entry.walk();
            let (entries, packs, stack_packs) = match work.side {
                PackSide::Behavior => (
                    set.behavior.listed_files(),
                    &self.instances[0],
                    set.behavior.stack_packs(),
                ),
                PackSide::Resource => (
                    set.resource.listed_files(),
                    &self.instances[1],
                    set.resource.stack_packs(),
                ),
            };
            for (index, files) in entries.iter().enumerate() {
                // A file the walk leaves out of this pack (`Walk::in_pack`) is not read; a
                // shadowed file the walk asks for (`Walk::shadowed`) is.
                for (_, path, _) in pack_files(&walk, files, stack_packs.get(index)) {
                    wanted[packs[index]].insert(path.clone());
                }
            }
        }
        let mut loose = Vec::new();
        for (index, work) in self.packs.iter_mut().enumerate() {
            let root = work.pack.location().clone();
            if root.root == RootId::MEMORY {
                continue;
            }
            for (path, slot) in work.pack.files().slots() {
                if !matches!(slot, FileSlot::Indexed) {
                    continue;
                }
                let place = root.join(path);
                let len = work.pack.files().size(path);
                if archive_dir(path).is_some() {
                    self.brarchives_out += 1;
                    self.places.insert(place.clone(), (index, path.clone()));
                    self.io.read(place, ReadRange::Whole, len, Charge::Release);
                } else if work.residency == Residency::All || wanted[index].contains(path) {
                    work.reads.insert(path.clone(), Read::Waiting(Vec::new()));
                    loose.push((index, path.clone(), place, work.residency, len));
                }
            }
        }
        for (index, path, place, residency, len) in loose {
            // A loose file under `Transient` stays charged until it is parsed.
            let hold = if residency == Residency::Transient {
                Charge::Hold
            } else {
                Charge::Release
            };
            self.places.insert(place.clone(), (index, path));
            self.io.read(place, ReadRange::Whole, len, hold);
        }
    }

    /// Once every brarchive is indexed: builds each pack's brarchive layer, walks every
    /// collection and issues its `Parse` jobs.
    fn start_parsing_if_ready(&mut self) {
        if self.parsing || self.brarchives_out > 0 {
            return;
        }
        self.parsing = true;
        self.io.progress.stage = Stage::Parse;
        let Some(mut set) = self.set.take() else {
            return;
        };
        for (index, work) in self.packs.iter_mut().enumerate() {
            if work.brarchives.is_empty() && work.findings.is_empty() {
                continue;
            }
            let mut layer = work.pack.files().store().brarchive.clone();
            for (path, archive) in std::mem::take(&mut work.brarchives) {
                if let Some(Err(error)) = archive_dir(&path).map(|dir| layer.add(&dir, &archive)) {
                    work.findings
                        .insert(path.clone(), storage_finding(&path, &error));
                }
            }
            let findings: Diagnostics = std::mem::take(&mut work.findings).into_values().collect();
            work.pack = work.pack.with_layer(layer, findings);
            set.replace_pack_everywhere(&self.instances, index, &work.pack);
        }
        for collection in 0..self.collections.len() {
            self.walk_collection(&set, collection);
        }
        self.set = Some(set);
        // A read no slot walks (a brarchive entry shadows it) is released at once.
        let unused: Vec<(usize, PackPath)> = self
            .packs
            .iter()
            .enumerate()
            .flat_map(|(index, work)| {
                work.reads
                    .iter()
                    .filter(|(_, read)| matches!(read, Read::Answered { uses: 0, .. }))
                    .map(move |(path, _)| (index, path.clone()))
            })
            .collect();
        for (pack, path) in unused {
            self.release(pack, &path);
        }
        for collection in 0..self.collections.len() {
            self.combine_if_ready(collection);
        }
    }

    /// The slots of one collection, in walk order, and the jobs that fill them.
    fn walk_collection(&mut self, set: &StackSet, collection: usize) {
        let work = &self.collections[collection];
        let walk = work.entry.walk();
        let (entries, packs, options, stack_packs) = match work.side {
            PackSide::Behavior => (
                set.behavior.listed_files(),
                &self.instances[0],
                set.behavior.options(),
                set.behavior.stack_packs(),
            ),
            PackSide::Resource => (
                set.resource.listed_files(),
                &self.instances[1],
                set.resource.options(),
                set.resource.stack_packs(),
            ),
        };
        let items = walk_items(&walk, &entries, stack_packs, options.app_data.as_ref());
        let slots: Vec<Slot> = items
            .into_iter()
            .map(|item| Slot::pending(item, packs, stack_packs))
            .collect();
        let count = slots.len();
        self.collections[collection].slots = slots;
        self.collections[collection].open = count;
        for slot in 0..count {
            self.feed_slot(set, collection, slot);
        }
    }

    /// The second stage of a collection whose walk is parsed: a job that names its files,
    /// given what the walk's files parsed to. The parsed files travel with the job and come
    /// back with its answer (`handle` runs no family code).
    fn ask_then(&mut self, collection: usize) {
        let work = &mut self.collections[collection];
        work.then = Then::Asked;
        let entry = work.entry;
        let mut parsed = Vec::new();
        for slot in &mut work.slots {
            if matches!(slot.state, SlotState::Parsed(_))
                && let SlotState::Parsed(value) =
                    std::mem::replace(&mut slot.state, SlotState::Lent)
            {
                parsed.push(value);
            }
        }
        let id = self.io.job(
            JobKind::Parse,
            0,
            Box::new(move |_: &Cancel| -> Erased {
                let keys = entry.then(&parsed);
                Box::new((parsed, keys))
            }),
        );
        self.jobs.insert(id, JobFor::Then { collection });
    }

    /// The second stage's slots, after the walk's, and the reads and jobs that fill them.
    fn walk_then(&mut self, set: &StackSet, collection: usize, keys: Vec<PackPath>) {
        let work = &self.collections[collection];
        let walk = work.entry.walk();
        let (entries, packs, stack_packs) = match work.side {
            PackSide::Behavior => (
                set.behavior.listed_files(),
                &self.instances[0],
                set.behavior.stack_packs(),
            ),
            PackSide::Resource => (
                set.resource.listed_files(),
                &self.instances[1],
                set.resource.stack_packs(),
            ),
        };
        let slots: Vec<Slot> = then_items(&walk, &entries, stack_packs, keys)
            .into_iter()
            .map(|item| Slot::pending(item, packs, stack_packs))
            .collect();
        let work = &mut self.collections[collection];
        let first = work.slots.len();
        let count = slots.len();
        work.slots.extend(slots);
        work.open += count;
        for slot in first..first + count {
            self.read_late(collection, slot);
            self.feed_slot(set, collection, slot);
        }
    }

    /// Makes the bytes of a slot added after the reads were queued available to `feed_slot`: a
    /// loose file nothing asked for yet is requested now, and one already read and released is
    /// taken from the kept bytes (`Residency::All`) or requested again.
    fn read_late(&mut self, collection: usize, slot: usize) {
        let entry = &self.collections[collection].slots[slot];
        let (Some(pack), path) = (entry.pack, entry.path.clone()) else {
            return;
        };
        let work = &mut self.packs[pack];
        let root = work.pack.location().clone();
        // A resident file (a brarchive entry, an archive member, memory) needs no read.
        if root.root == RootId::MEMORY || work.pack.files().is_resident(&path) {
            return;
        }
        match work.reads.get_mut(&path) {
            Some(Read::Answered { bytes, uses: 0 }) => {
                if let Some(kept) = work.keep.get(&path) {
                    *bytes = kept.clone();
                    return;
                }
            }
            Some(_) => return,
            None => {}
        }
        work.reads.insert(path.clone(), Read::Waiting(Vec::new()));
        let place = root.join(&path);
        let len = work.pack.files().size(&path);
        let hold = if work.residency == Residency::Transient {
            Charge::Hold
        } else {
            Charge::Release
        };
        self.places.insert(place.clone(), (pack, path));
        self.io.read(place, ReadRange::Whole, len, hold);
    }

    /// Issues the `Parse` job of a slot whose bytes are available, or waits for its read.
    fn feed_slot(&mut self, set: &StackSet, collection: usize, slot: usize) {
        let entry = &self.collections[collection].slots[slot];
        let (pack, path) = (entry.pack, entry.path.clone());
        let Some(pack) = pack else {
            let bytes = self
                .options_of(set, collection)
                .app_data
                .as_ref()
                .and_then(|data| data.get(&path))
                .cloned();
            match bytes {
                Some(bytes) => self.parse(set, collection, slot, ParseBytes::Loose(bytes, None)),
                None => self.skip(collection, slot, None),
            }
            return;
        };
        // A non-empty brarchive entry shadows the loose file.
        if self.packs[pack]
            .pack
            .files()
            .store()
            .brarchive
            .get(&path)
            .is_some()
        {
            let owner = self.packs[pack].pack.clone();
            self.parse(set, collection, slot, ParseBytes::Pack(owner));
            return;
        }
        match self.packs[pack].reads.get_mut(&path) {
            Some(Read::Waiting(waiting)) => waiting.push((collection, slot)),
            Some(Read::Answered { bytes, uses }) => {
                *uses += 1;
                let bytes = bytes.clone();
                let owner = self.packs[pack].pack.clone();
                self.parse(set, collection, slot, ParseBytes::Loose(bytes, Some(owner)));
            }
            Some(Read::Failed(kind)) => {
                let kind = *kind;
                let finding = self.unread(collection, &path, kind);
                self.skip(collection, slot, Some(finding));
            }
            None => {
                let owner = self.packs[pack].pack.clone();
                self.parse(set, collection, slot, ParseBytes::Pack(owner));
            }
        }
    }

    fn options_of<'a>(&self, set: &'a StackSet, collection: usize) -> &'a LoadOptions {
        match self.collections[collection].side {
            PackSide::Behavior => set.behavior.options(),
            PackSide::Resource => set.resource.options(),
        }
    }

    /// Issues the `Parse` job of a slot.
    fn parse(&mut self, set: &StackSet, collection: usize, slot: usize, source: ParseBytes) {
        let work = &self.collections[collection];
        let entry = work.entry;
        let side = work.side;
        let item = &work.slots[slot];
        let (pack_ref, context) = match (side, item.stack_index) {
            (PackSide::Behavior, Some(index)) => stack_pack(set.behavior.stack_packs(), index),
            (PackSide::Resource, Some(index)) => stack_pack(set.resource.stack_packs(), index),
            (PackSide::Behavior, None) => (PackRef::app_data(), set.behavior.app_data_context()),
            (PackSide::Resource, None) => (PackRef::app_data(), set.resource.app_data_context()),
        };
        let walked = item.item();
        let weight = match &source {
            ParseBytes::Loose(bytes, _) => bytes.len() as u64,
            ParseBytes::Pack(_) => 0,
        };
        let id = self.io.job(
            JobKind::Parse,
            weight,
            Box::new(move |cancel: &Cancel| -> Erased {
                let (bytes, document_source) = match source.bytes(&walked.path) {
                    Ok(prepared) => prepared,
                    Err(mut finding) => {
                        finding.location.pack = Some(pack_ref);
                        return Box::new(Err::<Erased, Box<Diagnostic>>(finding));
                    }
                };
                if cancel.is_cancelled() {
                    return Box::new(Err::<Erased, Box<Diagnostic>>(Box::new(internal_error(
                        None,
                    ))));
                }
                let file = walk_file(walked, pack_ref, side, bytes);
                let context = crate::pack::stack::context_for(&context, document_source);
                Box::new(Ok::<Erased, Box<Diagnostic>>(entry.parse(&file, &context)))
            }),
        );
        self.jobs.insert(id, JobFor::Parse { collection, slot });
    }

    /// The finding for a file of `collection` that could not be read, with the text the
    /// collection's walk has for it (`Walk::read_error`).
    fn unread(&self, collection: usize, path: &PackPath, kind: std::io::ErrorKind) -> Diagnostic {
        let mut finding = read_error(path, kind);
        if let Some(text) = self.collections[collection]
            .entry
            .walk()
            .read_error_text(path)
        {
            finding.code = bedrock_addon_core::diagnostic::DiagnosticCode::ReadError {
                kind,
                text: Some(text),
            };
        }
        finding
    }

    /// Fills a slot without parsing.
    fn skip(&mut self, collection: usize, slot: usize, finding: Option<Diagnostic>) {
        let mut finding = finding.unwrap_or_else(|| internal_error(None));
        let work = &mut self.collections[collection];
        if finding.location.pack.is_none() {
            finding.location.pack.clone_from(&work.slots[slot].pack_ref);
        }
        work.slots[slot].state = SlotState::Skipped(Box::new(finding));
        work.open -= 1;
    }

    /// The collections of this load whose identifiers the `combine` of `collection` needs:
    /// per kind, the one of the same side that registers it.
    fn providers(&self, collection: usize) -> Vec<usize> {
        let work = &self.collections[collection];
        work.entry
            .needs()
            .iter()
            .filter_map(|kind| provider(kind, work.side, ENTRIES))
            .filter_map(|entry| {
                self.collections
                    .iter()
                    .position(|other| same_entry(other.entry, entry))
            })
            .collect()
    }

    /// The identifiers the `combine` of `collection` sees: per kind it needs, those of the
    /// collection of its side that registers the kind, installed by this load or loaded
    /// before it. A kind nothing registers is left out.
    fn registered(&self, collection: usize, set: &StackSet) -> Vec<Arc<Registered>> {
        let work = &self.collections[collection];
        let stack = set.stack_of(work.side);
        work.entry
            .needs()
            .iter()
            .filter_map(|kind| provider(kind, work.side, ENTRIES))
            .filter_map(|entry| entry.registered(stack))
            .map(Arc::new)
            .collect()
    }

    /// Tries the `Combine` job of every collection: an installed collection may be what another
    /// waited for.
    fn combine_ready(&mut self) {
        for collection in 0..self.collections.len() {
            self.combine_if_ready(collection);
        }
    }

    /// Once every slot of the collection is filled, and every collection whose identifiers its
    /// `combine` needs is installed, issues its `Combine` job.
    fn combine_if_ready(&mut self, collection: usize) {
        let waits = self
            .providers(collection)
            .into_iter()
            .any(|provider| !self.collections[provider].installed);
        let work = &mut self.collections[collection];
        if work.open > 0 || work.combining || work.installed || !self.parsing {
            return;
        }
        // The walk is parsed: the second stage, if any, before anything combines.
        match work.then {
            Then::Due => return self.ask_then(collection),
            Then::Asked => return,
            Then::Settled => {}
        }
        if waits {
            return;
        }
        work.combining = true;
        self.io.progress.stage = Stage::Combine;
        let mut parsed = Vec::new();
        for slot in &mut work.slots {
            if matches!(slot.state, SlotState::Parsed(_))
                && let SlotState::Parsed(value) =
                    std::mem::replace(&mut slot.state, SlotState::Combined)
            {
                parsed.push(value);
            }
        }
        self.issue_combine(collection, parsed, false);
    }

    /// Issues the `Combine` job of a collection over `parsed`, in walk order.
    fn issue_combine(&mut self, collection: usize, parsed: Vec<Erased>, empty: bool) {
        let work = &self.collections[collection];
        let entry = work.entry;
        let Some(set) = self.set.as_ref() else {
            return;
        };
        let (options, packs) = cx_parts(set, work.side);
        let registered = self.registered(collection, set);
        let weight = parsed.len() as u64;
        // A `Combine` job runs to its end once started: a family's `combine` cannot be
        // interrupted, so the cancellation flag is checked before the job starts and not
        // between documents.
        let id = self.io.job(
            JobKind::Combine,
            weight,
            Box::new(move |_: &Cancel| -> Erased {
                let cx = StackCx::new(&options, &packs).with_registered(&registered);
                entry.combine(parsed, &cx)
            }),
        );
        self.jobs.insert(id, JobFor::Combine { collection, empty });
    }

    /// The combined collection goes into its cache with the slots' findings, in walk order.
    fn install(&mut self, collection: usize, value: Erased) {
        let work = &mut self.collections[collection];
        work.installed = true;
        let mut findings = Diagnostics::default();
        for slot in &work.slots {
            if let SlotState::Skipped(finding) = &slot.state {
                findings.push(Diagnostic::clone(finding));
            }
        }
        self.io.progress.count(&findings);
        if let Some(set) = self.set.as_ref() {
            work.entry.install(set.stack_of(work.side), value, findings);
        }
    }

    fn on_read(&mut self, location: &Place, answer: Result<Bytes, std::io::ErrorKind>) {
        let Some((pack, path)) = self.places.remove(location) else {
            return;
        };
        if archive_dir(&path).is_some() {
            match answer {
                Ok(bytes) => {
                    let files = self.packs[pack].pack.clone();
                    let job_path = path.clone();
                    let weight = bytes.len() as u64;
                    let id = self.io.job(
                        JobKind::Index,
                        weight,
                        Box::new(move |_: &Cancel| -> Erased {
                            let decoded = files
                                .files()
                                .decrypt(&job_path, bytes)
                                .and_then(Brarchive::decode)
                                .map_err(|error| Box::new(storage_finding(&job_path, &error)));
                            Box::new(decoded)
                        }),
                    );
                    self.jobs.insert(id, JobFor::Brarchive { pack, path });
                }
                Err(kind) => {
                    let finding = read_error(&path, kind);
                    self.packs[pack].findings.insert(path, finding);
                    self.brarchives_out -= 1;
                    self.start_parsing_if_ready();
                }
            }
            return;
        }
        let work = &mut self.packs[pack];
        let waiting = match work.reads.remove(&path) {
            Some(Read::Waiting(waiting)) => waiting,
            other => {
                if let Some(other) = other {
                    work.reads.insert(path, other);
                }
                return;
            }
        };
        match answer {
            Ok(bytes) => {
                if work.residency == Residency::All {
                    work.keep.insert(path.clone(), bytes.clone());
                }
                let uses = if self.parsing { waiting.len() } else { 0 };
                work.reads.insert(
                    path.clone(),
                    Read::Answered {
                        bytes: bytes.clone(),
                        uses,
                    },
                );
                if self.parsing {
                    if waiting.is_empty() {
                        self.release(pack, &path);
                    }
                    let owner = self.packs[pack].pack.clone();
                    let Some(set) = self.set.take() else { return };
                    for (collection, slot) in waiting {
                        self.parse(
                            &set,
                            collection,
                            slot,
                            ParseBytes::Loose(bytes.clone(), Some(owner.clone())),
                        );
                    }
                    self.set = Some(set);
                }
            }
            Err(kind) => {
                work.reads.insert(path.clone(), Read::Failed(kind));
                let place = work.pack.location().join(&path);
                self.io.release(&place);
                for (collection, slot) in waiting {
                    let finding = self.unread(collection, &path, kind);
                    self.skip(collection, slot, Some(finding));
                    self.combine_if_ready(collection);
                }
            }
        }
    }

    /// A loose read no slot needs any more: its charge and (under `Transient`) its bytes go.
    fn release(&mut self, pack: usize, path: &PackPath) {
        let work = &mut self.packs[pack];
        let place = work.pack.location().join(path);
        if let Some(Read::Answered { bytes, .. }) = work.reads.get_mut(path) {
            *bytes = Bytes::new();
        }
        self.io.release(&place);
    }

    fn on_job(&mut self, id: JobId, done: Option<JobDone>) {
        let Some(job) = self.jobs.remove(&id) else {
            return;
        };
        match job {
            JobFor::Brarchive { pack, path } => {
                match done.and_then(JobDone::take::<Result<Brarchive, Box<Diagnostic>>>) {
                    Some(Ok(archive)) => {
                        self.packs[pack].brarchives.insert(path, archive);
                    }
                    Some(Err(finding)) => {
                        self.packs[pack].findings.insert(path, *finding);
                    }
                    // A panicked brarchive `Index` job leaves it absent.
                    None => {
                        let finding = internal_error(Some(path.clone()));
                        self.packs[pack].findings.insert(path, finding);
                    }
                }
                self.brarchives_out -= 1;
                self.start_parsing_if_ready();
            }
            JobFor::Parse { collection, slot } => {
                let outcome = done.and_then(JobDone::take::<Result<Erased, Box<Diagnostic>>>);
                let path = self.collections[collection].slots[slot].path.clone();
                match outcome {
                    Some(Ok(value)) => {
                        let work = &mut self.collections[collection];
                        work.slots[slot].state = SlotState::Parsed(value);
                        work.open -= 1;
                    }
                    Some(Err(finding)) => self.skip(collection, slot, Some(*finding)),
                    // A panicked `Parse` job rejects its file.
                    None => self.skip(collection, slot, Some(internal_error(Some(path.clone())))),
                }
                self.used(collection, slot);
                self.combine_if_ready(collection);
            }
            JobFor::Then { collection } => {
                let answer = done.and_then(JobDone::take::<(Vec<Erased>, Vec<PackPath>)>);
                let work = &mut self.collections[collection];
                work.then = Then::Settled;
                let Some((parsed, keys)) = answer else {
                    // The job panicked and the parsed files went with it: as after a panicked
                    // `Combine`, the collection is installed empty.
                    work.combining = true;
                    work.slots.push(Slot {
                        pack: None,
                        stack_index: None,
                        pack_ref: None,
                        path: PackPath::ROOT,
                        key: PackPath::ROOT,
                        state: SlotState::Skipped(Box::new(internal_error(None))),
                    });
                    self.issue_combine(collection, Vec::new(), true);
                    return;
                };
                let mut parsed = parsed.into_iter();
                for slot in &mut work.slots {
                    if matches!(slot.state, SlotState::Lent)
                        && let Some(value) = parsed.next()
                    {
                        slot.state = SlotState::Parsed(value);
                    }
                }
                let Some(set) = self.set.take() else { return };
                self.walk_then(&set, collection, keys);
                self.set = Some(set);
                self.combine_if_ready(collection);
            }
            JobFor::Combine { collection, empty } => {
                if let Some(value) = done.and_then(|done| done.outcome) {
                    self.install(collection, value);
                    self.combine_ready();
                    return;
                }
                if empty {
                    // The combine of no files panicked too: nothing can be installed; the
                    // accessor loads lazily if called. A collection that needs this
                    // one's identifiers combines without them (the kind is out of view).
                    self.collections[collection].installed = true;
                    self.combine_ready();
                    return;
                }
                // A panicked `Combine` installs the collection empty, so the
                // cache is filled and later accessor calls do not retry. The empty collection
                // is combined in a job of its own (`handle` stays bookkeeping).
                let work = &mut self.collections[collection];
                work.slots.push(Slot {
                    pack: None,
                    stack_index: None,
                    pack_ref: None,
                    path: PackPath::ROOT,
                    key: PackPath::ROOT,
                    state: SlotState::Skipped(Box::new(internal_error(None))),
                });
                self.issue_combine(collection, Vec::new(), true);
            }
        }
    }

    /// A slot fed from a loose read is parsed: one use fewer.
    fn used(&mut self, collection: usize, slot: usize) {
        let item = &self.collections[collection].slots[slot];
        let Some(pack) = item.pack else { return };
        let path = item.path.clone();
        let release = match self.packs[pack].reads.get_mut(&path) {
            Some(Read::Answered { uses, .. }) if *uses > 0 => {
                *uses -= 1;
                *uses == 0
            }
            _ => false,
        };
        if release {
            self.release(pack, &path);
        }
    }

    /// Everything is installed: the packs keep what their residency says.
    fn finish(&mut self) -> Option<StackSet> {
        let mut set = self.set.take()?;
        for (index, work) in self.packs.iter_mut().enumerate() {
            if work.keep.is_empty() {
                continue;
            }
            work.pack = work.pack.with_loaded(std::mem::take(&mut work.keep));
            set.replace_pack_everywhere(&self.instances, index, &work.pack);
        }
        Some(set)
    }
}

/// Where a `Parse` job's bytes come from.
enum ParseBytes {
    /// Bytes the caller read (or app data); decrypted through the pack when it has one.
    Loose(Bytes, Option<Pack>),
    /// The pack's own resident file, archive entry or brarchive entry.
    Pack(Pack),
}

impl ParseBytes {
    /// The bytes a collection's `parse` gets: decrypted, MCB turned into JSON text.
    /// Also returns what the bytes were (`DocumentSource::Mcb` for a binary document turned
    /// into text, else `Json`), so a family can tell a binary source: it has no `null` rule
    /// and no JSON size check.
    fn bytes(self, path: &PackPath) -> Result<(Bytes, DocumentSource), Box<Diagnostic>> {
        match self {
            ParseBytes::Loose(bytes, None) => Ok((bytes, DocumentSource::Json)),
            ParseBytes::Loose(bytes, Some(pack)) => {
                let bytes = pack
                    .files()
                    .decrypt(path, bytes)
                    .map_err(|error| Box::new(storage_finding(path, &error)))?;
                crate::pack::stack::mcb_source(path, bytes)
            }
            ParseBytes::Pack(pack) => crate::pack::stack::prepare(&pack, path),
        }
    }
}

/// The handle and load context of the stack entry at `index`.
fn stack_pack(packs: &[StackPack], index: usize) -> (PackRef, LoadContext) {
    let entry = &packs[index];
    (entry.pack.clone(), entry.context.clone())
}

/// Whether two table entries are the same accessor line on the same side. By their variant
/// and side: the entries are zero-sized, so their addresses say nothing.
fn same_entry(a: &'static dyn ErasedCollection, b: &'static dyn ErasedCollection) -> bool {
    a.collection().is_some() && a.collection() == b.collection() && a.side() == b.side()
}

/// What a `Combine` job owns to lend a `StackCx`: the options and the stack entries.
fn cx_parts(set: &StackSet, side: PackSide) -> (Arc<LoadOptions>, Arc<[StackPack]>) {
    match side {
        PackSide::Behavior => (
            Arc::new(set.behavior.options().clone()),
            set.behavior.stack_packs().into(),
        ),
        PackSide::Resource => (
            Arc::new(set.resource.options().clone()),
            set.resource.stack_packs().into(),
        ),
    }
}

impl Sans for StackLoader {
    type Input = Input;
    type Output = Output<StackSet>;
    type Error = LoaderError;

    fn handle(&mut self, input: Input) -> Result<(), LoaderError> {
        if self.done {
            return Err(LoaderError::Finished);
        }
        match input {
            Input::File { location, bytes } => {
                self.io.check_answer(&location, Expect::Read)?;
                self.io.answer(&location, bytes.len() as u64);
                self.on_read(&location, Ok(bytes));
            }
            Input::Missing { location } => {
                self.io.check_answer(&location, Expect::Any)?;
                self.io.answer(&location, 0);
                self.on_read(&location, Err(std::io::ErrorKind::NotFound));
            }
            Input::Failed { location, kind } => {
                self.io.check_answer(&location, Expect::Any)?;
                self.io.answer(&location, 0);
                self.on_read(&location, Err(kind));
            }
            Input::Dir { location, .. } => {
                // The stack loader never lists.
                self.io.check_answer(&location, Expect::List)?;
                return Err(LoaderError::Unrequested { location });
            }
            Input::Key { identity, .. } => {
                self.io.check_key(&identity)?;
            }
            Input::JobDone(done) => {
                self.io.check_job(done.id())?;
                let id = done.id();
                self.io.job_done(id);
                self.on_job(id, Some(done));
            }
            Input::JobPanicked(id) => {
                self.io.check_job(id)?;
                self.io.job_done(id);
                self.on_job(id, None);
            }
        }
        Ok(())
    }

    fn poll(&mut self) -> Option<Self::Output> {
        if let Some(ask) = self.io.poll() {
            return Some(ask.into_output());
        }
        let installed = self.collections.iter().all(|work| work.installed);
        if self.done || !self.parsing || !installed || !self.io.is_idle() {
            return None;
        }
        self.done = true;
        self.io.progress.stage = Stage::Finished;
        self.finish().map(Output::Done)
    }
}

impl Loader<StackSet> for StackLoader {
    fn progress(&self) -> Progress {
        self.io.progress
    }
}

/// Prints the progress counters only.
impl std::fmt::Debug for StackLoader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StackLoader")
            .field("progress", &Loader::progress(self))
            .finish_non_exhaustive()
    }
}
