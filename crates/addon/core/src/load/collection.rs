//! The per-collection loader signature the stack-accessor macro calls (its shape is
//! fixed): what a collection reads, how one file parses, and how the parsed files combine.
//!
//! An accessor line `blocks: Registry<Block> = block::LOADER, feature = "block",
//! side = Behavior;` names a constant `LOADER: CollectionLoader<Registry<Block>, P>`. The
//! loaders read the files its [`Walk`] matches, run `parse` once per file (a
//! `Parse` job) and `combine` once per collection, over every parsed file in walk order (the
//! `Combine` job), which also sees the whole stack ([`StackCx`]); the lazy accessor does the
//! same on the calling thread. Hidden: the family modules of `bedrock_addon` provide these,
//! nobody else.
//!
//! # What one collection sees of the others
//!
//! A `combine` that has to know whether the stack registers an identifier of another family (a
//! crafting catalogue placing a block) asks [`StackCx::registered`]; it never names the other
//! family's module. The collection declares the kinds it asks about in
//! [`CollectionLoader::needs`] (`Kind::NAME`s). The loaders combine and install the collection
//! that registers each of those kinds first and hand its identifiers over as a [`Registered`].
//! A collection that registers a kind for the others declares no needs of its own, so the
//! needs of a stack never form a cycle; `bedrock_addon` asserts that at compile time for each
//! registering collection.

use std::any::Any;
use std::fmt;
use std::sync::Arc;

use bytes::Bytes;
use indexmap::IndexMap;

use super::{ListDepth, LoadContext, LoadOptions};
use crate::diagnostic::{Diagnostic, VanillaText};
use crate::document::{Content, Definition, Files, Merge, Merged, Registry};
use crate::id::{Id, Kind};
use crate::pack_identity::{PackOrigin, PackPath, PackRef, PackSide};
use crate::version::{ENGINE_VERSION, SemVersion};

/// One collection's loader: its walk, its per-file parse and its combine step.
///
/// `C` is the collection (`Registry<Block>`, `Files<LootTable>`, `Merged<Languages>`); `P` is
/// what one file parses to (for a registry, the file's `Contribution`s; for files, the copy and
/// its key; for a merged value, the layer or its rejection).
pub struct CollectionLoader<C, P> {
    /// Which files of each pack (and of the app data) the collection reads, and in what order.
    pub walk: Walk,
    /// Parses one file. Runs in a `Parse` job: anywhere, in any order.
    pub parse: fn(&WalkFile, &LoadContext) -> P,
    /// Combines every parsed file, in walk order, into the collection. Runs once, in the
    /// `Combine` job, after every `parse` of the collection, with the whole stack in view
    /// (packs without a matching file included).
    pub combine: fn(Vec<P>, &StackCx<'_>) -> C,
    /// The identifier kinds (`Kind::NAME`) whose registrations `combine` asks the stack for
    /// through [`StackCx::registered`]; empty for a collection that asks for none. The
    /// collection that registers a kind is combined before this one. A collection that
    /// registers a kind for others must leave this empty.
    pub needs: &'static [&'static str],
    /// A second walk stage, for a collection whose list of files is itself a file of the
    /// packs (the JSON UI files that `ui/_ui_defs.json` names); `None` for a collection one
    /// walk reads whole.
    ///
    /// Called once, after every `parse` of the walk's files, with what they parsed to, in
    /// walk order. It returns the keys of the further files to read: paths as the collection
    /// keys them (no subpack folder), in any order, repeats allowed. The loader then reads,
    /// from each pack of the stack that has it, the file at each key (the active subpack's
    /// in place of the pack's own), and `parse`s it: packs in the walk's
    /// [`order`](Walk::order), each pack's keys bytewise. `combine` gets the walk's files
    /// followed by these. A file the walk itself read is not read again; a file
    /// [`Walk::in_pack`] leaves out of its pack is not read; a key no pack has is nothing
    /// (no read, no finding: the family reports a listed file it did not get). The stage
    /// never reads app data, and no file but those named is asked for.
    pub then: Option<NamedFiles<P>>,
}

/// The function of a collection's second walk stage ([`CollectionLoader::then`]): from what
/// the walk's files parsed to, in walk order, the keys of the further files to read.
pub type NamedFiles<P> = fn(&[&P]) -> Vec<PackPath>;

impl<C, P> Clone for CollectionLoader<C, P> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<C, P> Copy for CollectionLoader<C, P> {}

/// The walk only.
impl<C, P> fmt::Debug for CollectionLoader<C, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CollectionLoader")
            .field("walk", &self.walk)
            .finish_non_exhaustive()
    }
}

/// Which files a collection reads, and the order its `combine` sees them in.
///
/// The order is fixed by the walk, never by when a read or a job finished:
/// packs in [`order`](Self::order), within a pack its matching files sorted bytewise; with
/// [`directory_major`](Self::directory_major), each directory over every pack before the next
/// directory (client entities: every `entity/` before any `attachables/`). App data, when the
/// family reads it, comes [`before`](AppDataPlace::Before) or [`after`](AppDataPlace::After)
/// the packs.
#[derive(Clone, Copy, Debug)]
pub struct Walk {
    /// Directories below the pack root (`blocks`, `textures/ui`); `""` is the pack root itself.
    pub directories: &'static [&'static str],
    /// Direct children of a directory, or every file below it.
    pub depth: ListDepth,
    /// Which files count, given their path in the pack (an extension rule, a fixed name).
    pub accept: Accept,
    /// The pack order.
    pub order: WalkOrder,
    /// Each directory over every pack before the next directory, instead of each pack's
    /// directories before the next pack.
    pub directory_major: bool,
    /// The app-data directories read (members of `AppData::DIRECTORIES`); empty for none.
    pub app_data: &'static [&'static str],
    /// Where the app-data files come in the walk.
    pub app_data_place: AppDataPlace,
    /// Which of a pack's matching files the walk reads, given the file's path in the pack and
    /// the pack's load context; `None` reads them all. A file this refuses is no file of the
    /// walk: its bytes are never asked for and `parse` never sees it (structure templates of a
    /// base-game pack). App-data files are not asked about.
    pub in_pack: Option<fn(&PackPath, &LoadContext) -> bool>,
    /// Which of a pack's own files that its active subpack shadows
    /// ([`StackPack::shadowed_files`]) the walk also reads, given the file's own path; `None`
    /// reads none, so the walk has one file per key. A shadowed file the rule takes must also
    /// be a file the walk would read unshadowed ([`matches_in`](Self::matches_in)). It is a
    /// [`WalkFile`] whose `path` and `key` are both the file's own path, placed just before
    /// the subpack's file of that key: `combine` tells the pack's copy (`path == key`, first)
    /// from the subpack's (a texture list the subpack's list is appended to).
    pub shadowed: Option<fn(&PackPath) -> bool>,
    /// Vanilla's content-log text for a file of the walk whose bytes could not be read, with
    /// `{}` where the file's path in its pack goes; `None` for a family whose loader has no
    /// text of its own. The stack driver puts it on the file's
    /// [`ReadError`](crate::diagnostic::DiagnosticCode::ReadError)
    /// ([`read_error_text`](Self::read_error_text)).
    pub read_error: Option<&'static str>,
}

/// Which files a [`Walk`] takes, given their path in the pack. Data rather than a function
/// pointer, so that a `const fn` preset can name an extension or a path.
#[derive(Clone, Copy, Debug)]
pub enum Accept {
    /// Files whose last segment ends in `.<ext>`, the letter case compared exactly
    /// ([`PackPath::has_extension`]): `"json"`, `"lang"`.
    Ext(&'static str),
    /// The files at these paths, each compared whole and exactly: a family's fixed files
    /// (`ui/_ui_defs.json` and `ui/_global_variables.json`).
    Exact(&'static [&'static str]),
    /// The files a rule of the family's takes.
    Fn(fn(&PackPath) -> bool),
}

impl Accept {
    /// Whether the file at `path` counts.
    pub fn takes(&self, path: &PackPath) -> bool {
        match self {
            Self::Ext(ext) => path.has_extension(ext),
            Self::Exact(exact) => exact.contains(&path.as_str()),
            Self::Fn(accept) => accept(path),
        }
    }
}

/// The presets and their modifiers: a walk is a preset and the fields that differ from it.
/// Every preset reads packs only, lowest pack first, each pack's directories before the next
/// pack, with no in-pack rule, no shadowed file and no read-error text.
impl Walk {
    /// A preset's fields around its directories, depth and rule.
    const fn preset(
        directories: &'static [&'static str],
        depth: ListDepth,
        accept: Accept,
    ) -> Walk {
        Walk {
            directories,
            depth,
            accept,
            order: WalkOrder::BottomUp,
            directory_major: false,
            app_data: &[],
            app_data_place: AppDataPlace::After,
            in_pack: None,
            shadowed: None,
            read_error: None,
        }
    }

    /// The `.json` files directly in `directories`, one level deep: not the files of their
    /// subdirectories ([`json_tree_in`](Self::json_tree_in) reads those too).
    pub const fn json_children_in(directories: &'static [&'static str]) -> Walk {
        Self::preset(directories, ListDepth::Children, Accept::Ext("json"))
    }

    /// The `.json` files anywhere below `directories`, the whole tree: the files directly in
    /// them and in every subdirectory ([`json_children_in`](Self::json_children_in) reads the
    /// first only).
    pub const fn json_tree_in(directories: &'static [&'static str]) -> Walk {
        Self::preset(directories, ListDepth::Recursive, Accept::Ext("json"))
    }

    /// The files at `paths` in each pack (paths from the pack root), whichever of them the
    /// pack has.
    pub const fn exact_files(paths: &'static [&'static str]) -> Walk {
        Self::preset(&[""], ListDepth::Recursive, Accept::Exact(paths))
    }

    /// No file at all: a listing-only collection that reads no bytes (its `combine` asks
    /// the packs' listings).
    pub const fn none() -> Walk {
        Self::preset(&[], ListDepth::Children, Accept::Fn(|_| false))
    }

    /// The same walk with another rule.
    #[must_use]
    pub const fn accept(mut self, accept: Accept) -> Walk {
        self.accept = accept;
        self
    }

    /// The same walk, highest pack first.
    #[must_use]
    pub const fn top_down(mut self) -> Walk {
        self.order = WalkOrder::TopDown;
        self
    }

    /// The same walk, each directory over every pack before the next directory.
    #[must_use]
    pub const fn directory_major(mut self) -> Walk {
        self.directory_major = true;
        self
    }

    /// The same walk, also reading the app-data `directories`, placed at `place`.
    #[must_use]
    pub const fn app_data(
        mut self,
        directories: &'static [&'static str],
        place: AppDataPlace,
    ) -> Walk {
        self.app_data = directories;
        self.app_data_place = place;
        self
    }

    /// The same walk, reading of each pack only the files `in_pack` takes.
    #[must_use]
    pub const fn in_pack(mut self, in_pack: fn(&PackPath, &LoadContext) -> bool) -> Walk {
        self.in_pack = Some(in_pack);
        self
    }

    /// The same walk, also reading the shadowed files `shadowed` takes.
    #[must_use]
    pub const fn shadowed(mut self, shadowed: fn(&PackPath) -> bool) -> Walk {
        self.shadowed = Some(shadowed);
        self
    }

    /// The same walk, with vanilla's text for a file it could not read.
    #[must_use]
    pub const fn read_error(mut self, template: &'static str) -> Walk {
        self.read_error = Some(template);
        self
    }
}

impl Walk {
    /// The index in [`directories`](Self::directories) of the directory `path` (a path in the
    /// pack, or in the app data) belongs to, if the walk reads it.
    pub fn matches(&self, path: &PackPath) -> Option<usize> {
        if !self.accept.takes(path) {
            return None;
        }
        self.directories
            .iter()
            .position(|directory| below(path.as_str(), directory, self.depth))
    }

    /// The text of [`read_error`](Self::read_error) for the file at `path`, if the walk's
    /// family has one.
    pub fn read_error_text(&self, path: &PackPath) -> Option<VanillaText> {
        self.read_error.map(|template| VanillaText::Formatted {
            template,
            args: vec![path.as_str().to_owned()],
        })
    }

    /// [`matches`](Self::matches) for a file of the pack loaded with `cx`: `None` also when
    /// [`in_pack`](Self::in_pack) leaves the file out of that pack's walk.
    pub fn matches_in(&self, path: &PackPath, cx: &LoadContext) -> Option<usize> {
        let directory = self.matches(path)?;
        self.in_pack
            .is_none_or(|in_pack| in_pack(path, cx))
            .then_some(directory)
    }

    /// The index in [`directories`](Self::directories) of the directory of `path`, a file of
    /// the pack loaded with `cx` that the pack's active subpack shadows, if the walk reads it
    /// all the same: [`shadowed`](Self::shadowed) takes it and
    /// [`matches_in`](Self::matches_in) would read it.
    pub fn matches_shadowed(&self, path: &PackPath, cx: &LoadContext) -> Option<usize> {
        let shadowed = self.shadowed?;
        let directory = self.matches_in(path, cx)?;
        shadowed(path).then_some(directory)
    }

    /// Whether the walk reads the app-data file at `path`.
    pub fn matches_app_data(&self, path: &PackPath) -> bool {
        self.accept.takes(path)
            && self
                .app_data
                .iter()
                .any(|directory| below(path.as_str(), directory, self.depth))
    }
}

/// `path` lies in `directory` at `depth`.
fn below(path: &str, directory: &str, depth: ListDepth) -> bool {
    let rest = if directory.is_empty() {
        path
    } else {
        match path
            .strip_prefix(directory)
            .and_then(|rest| rest.strip_prefix('/'))
        {
            Some(rest) => rest,
            None => return false,
        }
    };
    !rest.is_empty() && (depth == ListDepth::Recursive || !rest.contains('/'))
}

/// The pack order of a walk. The stack is stored top-first (index 0 is the highest priority).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WalkOrder {
    /// Lowest priority first, as most of vanilla's loaders visit a stack.
    BottomUp,
    /// Highest priority first.
    TopDown,
}

/// Where a walk reads its app-data files.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppDataPlace {
    /// Before every pack (attachables).
    Before,
    /// After every pack (features).
    After,
}

/// One file a walk reached, handed to a collection's `parse`.
#[derive(Clone)]
pub struct WalkFile {
    /// The pack, or `PackRef::app_data()` for an app-data file.
    pub pack: PackRef,
    /// The stack side.
    pub side: PackSide,
    /// The file's path in its pack (subpack folder included), for `Origin::path`.
    pub path: PackPath,
    /// The path the collection keys by: [`path`](Self::path) without a subpack folder.
    pub key: PackPath,
    /// The file's bytes as stored (inflated and decrypted by the loader).
    pub bytes: Bytes,
}

/// The location only, never the bytes.
impl fmt::Debug for WalkFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WalkFile")
            .field("pack", &self.pack.name())
            .field("side", &self.side)
            .field("path", &self.path)
            .field("key", &self.key)
            .field("len", &self.bytes.len())
            .finish()
    }
}

/// The identifiers of one kind that the packs of a stack registered, each with the pack whose
/// definition is the effective one: what a loaded collection shows the `combine` of another
/// ([`StackCx::registered`]). Identifiers compare as [`Id`]s of the kind do.
pub struct Registered {
    kind: &'static str,
    len: usize,
    /// An `IndexMap<Id<K>, PackRef>` of the kind `K` named by `kind`.
    ids: Box<dyn Any + Send + Sync>,
}

impl Registered {
    /// The identifiers of kind `K` with the pack that registered each; of two equal
    /// identifiers the first is kept.
    pub fn new<K: Kind>(ids: impl IntoIterator<Item = (Id<K>, PackRef)>) -> Self {
        let mut map: IndexMap<Id<K>, PackRef> = IndexMap::new();
        for (id, pack) in ids {
            map.entry(id).or_insert(pack);
        }
        Self {
            kind: K::NAME,
            len: map.len(),
            ids: Box::new(map),
        }
    }

    /// The kind's `Kind::NAME`.
    pub fn kind(&self) -> &'static str {
        self.kind
    }

    /// How many identifiers.
    pub fn len(&self) -> usize {
        self.len
    }

    /// No identifier.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The pack that registered `id`; `None` when no pack did, or when `K` is another kind.
    pub fn by<K: Kind>(&self, id: &Id<K>) -> Option<&PackRef> {
        self.entry(id).map(|(_, pack)| pack)
    }

    /// The identifier equal to `id` as its definition spells it, and the pack that registered
    /// it.
    fn entry<K: Kind>(&self, id: &Id<K>) -> Option<(&str, &PackRef)> {
        let ids = self.ids.downcast_ref::<IndexMap<Id<K>, PackRef>>()?;
        ids.get_key_value(id)
            .map(|(written, pack)| (written.as_str(), pack))
    }
}

/// The kind and the number of identifiers.
impl fmt::Debug for Registered {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Registered")
            .field("kind", &self.kind)
            .field("len", &self.len)
            .finish_non_exhaustive()
    }
}

/// What a stack says about one identifier ([`StackCx::registered`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Registration<'a> {
    /// The stack shows this `combine` no collection of the identifier's kind: the collection
    /// did not declare the kind in [`CollectionLoader::needs`], or no collection compiled in
    /// registers it (the family's feature is off).
    NotInView,
    /// The kind is in view and no pack of the stack registered the identifier.
    Absent,
    /// A pack of the stack registered the identifier.
    By {
        /// The pack whose definition of the identifier is the effective one.
        pack: &'a PackRef,
        /// The identifier as that definition spells it: equal to the one asked about as
        /// identifiers of the kind compare, not always byte for byte (a default namespace, a
        /// doubled `/` in a function name). A rule that matches on bytes compares this.
        written: &'a str,
    },
}

impl Registration<'_> {
    /// Whether a pack of the stack registered the identifier.
    pub fn is_registered(&self) -> bool {
        matches!(self, Self::By { .. })
    }
}

/// What a collection's `combine` sees of the stack beyond its parsed files: the world options,
/// every pack of the stack, top first, whether or not it had a file the walk matched, and the
/// identifiers the collections it [needs](CollectionLoader::needs) registered.
#[derive(Clone, Copy)]
#[non_exhaustive]
pub struct StackCx<'a> {
    /// The stack's options.
    pub options: &'a LoadOptions,
    /// Every pack of the stack, top first (index 0 is the highest priority).
    pub packs: &'a [StackPack],
    /// The registered identifiers in view, one entry per kind.
    registered: &'a [Arc<Registered>],
}

impl<'a> StackCx<'a> {
    /// The context of a stack loaded with `options` (built by `bedrock_addon`'s
    /// loaders), with no other collection in view.
    pub fn new(options: &'a LoadOptions, packs: &'a [StackPack]) -> Self {
        Self {
            options,
            packs,
            registered: &[],
        }
    }

    /// This context with the registered identifiers of the collections the combining
    /// collection needs in view, one [`Registered`] per kind (built by
    /// `bedrock_addon`'s loaders, from the collections already combined).
    #[must_use]
    pub fn with_registered(mut self, registered: &'a [Arc<Registered>]) -> Self {
        self.registered = registered;
        self
    }

    /// What answers for the game's own content: the host's provider when the stack's
    /// options gave one, else the generated tables, as [`LoadContext::built_ins`] answers.
    pub fn built_ins(&self) -> &'a dyn super::BuiltIns {
        static TABLES: crate::vanilla::GeneratedBuiltIns = crate::vanilla::GeneratedBuiltIns;
        match &self.options.built_ins {
            Some(host) => host.as_ref(),
            None => &TABLES,
        }
    }

    /// Whether a pack of the stack registered `id`, and which: the answer of the stack's
    /// loaded collection of kind `K` (its effective definitions, stable or beta).
    /// [`Registration::NotInView`] unless the combining collection declared `K::NAME` in
    /// [`CollectionLoader::needs`] and a collection that registers the kind is compiled in.
    pub fn registered<K: Kind>(&self, id: &Id<K>) -> Registration<'a> {
        match self.registered.iter().find(|r| r.kind() == K::NAME) {
            None => Registration::NotInView,
            Some(registered) => match registered.entry(id) {
                Some((written, pack)) => Registration::By { pack, written },
                None => Registration::Absent,
            },
        }
    }

    /// The highest `min_engine_version` over the stack's packs (the game computes the same
    /// maximum, and client entities and attachables compare `description.min_engine_version`
    /// against it); the engine version when no pack has one.
    // The maximum over the stack, the current game version for an empty stack. Vanilla filters
    // the packs by a per-pack flag that is not established; every pack with a version counts
    // here.
    pub fn full_stack_min_engine_version(&self) -> SemVersion {
        self.packs
            .iter()
            .filter_map(|pack| pack.context.min_engine_version.as_ref())
            .map(|version| version.version.clone())
            .max()
            .unwrap_or_else(|| {
                SemVersion::new(
                    ENGINE_VERSION.major,
                    ENGINE_VERSION.minor,
                    ENGINE_VERSION.patch,
                )
            })
    }
}

/// The packs only (`LoadOptions` has no `Debug`).
impl fmt::Debug for StackCx<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StackCx")
            .field("packs", &self.packs)
            .finish_non_exhaustive()
    }
}

/// One pack of a stack as a collection's `combine` sees it: its handle, where it came from and
/// its load context.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct StackPack {
    /// The pack's handle (stack index, identity, name).
    pub pack: PackRef,
    /// Where the pack came from (`Package` is a base-game pack).
    pub origin: PackOrigin,
    /// The pack's load context (`min_engine_version`, `is_base_game_pack`, …).
    pub context: LoadContext,
    /// The manifest header's `description` as written; `None` when the header has none. A
    /// header may name the pack keyword `pack.description` here, which the pack's own `.lang`
    /// files translate.
    pub description: Option<Box<str>>,
    /// The names of the manifest's `capabilities`, as written; empty when the manifest has
    /// none. Content that only takes effect with a capability (`pbr`, `raytraced`) reads it.
    pub capabilities: Vec<Box<str>>,
    /// The pack's file listing ([`files`](Self::files)).
    files: Listing,
    /// The pack's own files the active subpack shadows
    /// ([`shadowed_files`](Self::shadowed_files)).
    shadowed: Listing,
    /// The document the pack was opened with ([`opened`](Self::opened)).
    opened: Option<Opened>,
}

/// The document a stack pack was opened with, of a type core does not name; shared, and
/// shown as a fixed word only.
#[derive(Clone)]
struct Opened(Arc<dyn Any + Send + Sync>);

impl fmt::Debug for Opened {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("opened document")
    }
}

/// One file of a stack pack as the stack reads it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListedFile {
    /// The file's path in its pack (subpack folder included).
    pub path: PackPath,
    /// The path a collection keys the file by: [`path`](Self::path) without a subpack folder.
    pub key: PackPath,
}

/// A stack pack's file listing; shared, and shown as its length only.
#[derive(Clone, Default)]
struct Listing(Arc<[ListedFile]>);

impl fmt::Debug for Listing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} files", self.0.len())
    }
}

impl StackPack {
    /// A stack entry from its parts (built by `bedrock_addon`'s loaders).
    pub fn new(pack: PackRef, origin: PackOrigin, context: LoadContext) -> Self {
        Self {
            pack,
            origin,
            context,
            description: None,
            capabilities: Vec::new(),
            files: Listing::default(),
            shadowed: Listing::default(),
            opened: None,
        }
    }

    /// This entry with the pack's file listing (built by `bedrock_addon`'s loaders):
    /// every file the stack reads from the pack, bytewise by key.
    #[must_use]
    pub fn with_files(mut self, files: impl IntoIterator<Item = ListedFile>) -> Self {
        self.files = Listing(files.into_iter().collect());
        self
    }

    /// Every file the stack reads from the pack, by name only. Listing a file asks for none
    /// of its bytes, so a `combine` that needs to know which files exist (which pack has an
    /// image, a sound) reads this instead of putting the files in its [`Walk`].
    ///
    /// - **What**: the pack's own files outside its declared subpack folders, with the active
    ///   subpack's files in place of the pack's at the same key; a file a brarchive supplies
    ///   is listed once the archive is read. Every pack of the stack has its listing, a
    ///   base-game pack too.
    /// - **Keys**: [`ListedFile::key`] is the path a collection keys the file by (a subpack's
    ///   file without its `subpacks/<folder>/` prefix); [`ListedFile::path`] is what a reader
    ///   asks the pack for.
    /// - **Order**: bytewise by key, each key once: the order a [`Walk`] visits a pack in.
    ///
    /// Empty when the loader gave no listing.
    pub fn files(&self) -> &[ListedFile] {
        &self.files.0
    }

    /// The files of [`files`](Self::files) keyed below `directory` (`"render_controllers"`,
    /// `"textures/ui"`), at any depth, bytewise by key; all of them for `""`. How a `combine`
    /// learns of the files of its directory that its [`Walk`] does not read (a note, a
    /// picture, a file of another extension), to report them unread. Found by bisection: the
    /// cost is the files returned, not the pack's.
    pub fn files_below(&self, directory: &str) -> &[ListedFile] {
        let files = self.files();
        if directory.is_empty() {
            return files;
        }
        // Bytewise order puts every key with the prefix `<directory>/` in one run.
        let prefix = format!("{directory}/");
        let from = files.partition_point(|file| file.key.as_str() < prefix.as_str());
        let len = files[from..].partition_point(|file| file.key.as_str().starts_with(&prefix));
        &files[from..from + len]
    }

    /// This entry with the pack's own files that the active subpack shadows (built by
    /// `bedrock_addon`'s loaders).
    #[must_use]
    pub fn with_shadowed_files(mut self, files: impl IntoIterator<Item = ListedFile>) -> Self {
        self.shadowed = Listing(files.into_iter().collect());
        self
    }

    /// The pack's own files that a file of the active subpack stands in for, bytewise by
    /// path (`key` and `path` are the same: the file's own). They are no file of
    /// [`files`](Self::files) and no walk reads them; a rule that combines the pack's copy
    /// with the subpack's (a texture list the subpack's list is appended to) learns here that
    /// the pack has one. Empty without an active subpack.
    pub fn shadowed_files(&self) -> &[ListedFile] {
        &self.shadowed.0
    }

    /// This entry with the manifest header's `description` (built by `bedrock_addon`'s
    /// loaders).
    #[must_use]
    pub fn with_description(mut self, description: Option<&str>) -> Self {
        self.description = description.map(Box::from);
        self
    }

    /// This entry with the document the pack was opened with (built by `bedrock_addon`'s
    /// loaders: the pack's manifest). Shared, not copied; one document per entry, a second
    /// call replaces the first.
    #[must_use]
    pub fn with_opened<T: Any + Send + Sync>(mut self, opened: Arc<T>) -> Self {
        self.opened = Some(Opened(opened));
        self
    }

    /// The document the pack was opened with, if it is a `T`: the loaders put the pack's
    /// manifest here, as it was read when the pack was opened, so a `combine` that needs more
    /// of it than the entry's own fields reads it here instead of parsing the file again.
    /// `None` for an entry built by hand, and for any other `T`. The manifest crate's
    /// extension trait names the one type the loaders store.
    pub fn opened<T: Any>(&self) -> Option<&T> {
        self.opened.as_ref()?.0.downcast_ref()
    }

    /// This entry with the names of the manifest's `capabilities` (built by
    /// `bedrock_addon`'s loaders).
    #[must_use]
    pub fn with_capabilities<'a>(
        mut self,
        capabilities: impl IntoIterator<Item = &'a str>,
    ) -> Self {
        self.capabilities = capabilities.into_iter().map(Box::from).collect();
        self
    }
}

/// The files below `directory` of each pack of the stack that the collection's walk does not
/// read (`read` is its `Walk::accept`), lowest pack first and each pack's bytewise by key:
/// what a `combine` reports as ignored without their bytes ever being asked for.
pub fn unread_files<'a>(
    stack: &StackCx<'a>,
    directory: &'a str,
    read: fn(&PackPath) -> bool,
) -> impl Iterator<Item = (&'a StackPack, &'a ListedFile)> + use<'a> {
    stack.packs.iter().rev().flat_map(move |pack| {
        pack.files_below(directory)
            .iter()
            .filter(move |file| !read(&file.key))
            .map(move |file| (pack, file))
    })
}

/// A collection type whose diagnostics a `StackReport` lists.
pub trait Reported {
    /// Every finding of the collection, in its own order.
    fn reported(&self) -> Box<dyn Iterator<Item = &Diagnostic> + '_>;
    /// The identifiers the collection registered, when it is one that registers a kind
    /// (a `Registry`); the stack shows them to the collections that need the kind.
    fn registered(&self) -> Option<Registered> {
        None
    }
}

impl<T: Definition> Reported for Registry<T> {
    fn reported(&self) -> Box<dyn Iterator<Item = &Diagnostic> + '_> {
        Box::new(self.diagnostics())
    }
    fn registered(&self) -> Option<Registered> {
        Some(Registry::registered(self))
    }
}

impl<T: Content> Reported for Files<T> {
    fn reported(&self) -> Box<dyn Iterator<Item = &Diagnostic> + '_> {
        Box::new(self.diagnostics())
    }
}

impl<T: Merge, L> Reported for Merged<T, L> {
    fn reported(&self) -> Box<dyn Iterator<Item = &Diagnostic> + '_> {
        Box::new(self.diagnostics())
    }
}

/// The diagnostics of a slice line: each element's, in order.
impl<T: Reported> Reported for Vec<T> {
    fn reported(&self) -> Box<dyn Iterator<Item = &Diagnostic> + '_> {
        Box::new(self.iter().flat_map(Reported::reported))
    }
}
