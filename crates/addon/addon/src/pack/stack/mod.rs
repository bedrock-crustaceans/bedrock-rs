//! The typed pack stack: `PackStack<Behavior>` and `PackStack<Resource>`, their accessors, and
//! the `StackSet` of a world.

mod accessors;
mod collection;
pub(crate) mod compose;
#[cfg(all(feature = "lang", feature = "languages"))]
mod texts;
#[cfg(all(feature = "texture_file", feature = "texture_set"))]
mod textures;
mod vanilla;
mod walk;

pub use accessors::Collection;
pub(crate) use accessors::{ENTRIES, provider};
pub use collection::CollectionSet;
#[allow(
    unused_imports,
    reason = "the accessor lines of the enabled families use it; with no family feature on it is unused"
)]
pub(crate) use collection::{
    Erased, ErasedCollection, Loaded, erased_combine, erased_parse, erased_then, install,
};
#[cfg(all(feature = "texture_file", feature = "texture_set"))]
pub use textures::{LayerImage, TextureFile};
pub use vanilla::VANILLA_EXPERIMENTAL_PACKS;
#[cfg(test)]
pub(crate) use walk::mcb_text;
pub(crate) use walk::{
    WalkItem, WalkSource, context_for, mcb_source, pack_files, prepare, read_item, then_items,
    walk_file, walk_items,
};

use std::any::Any;
use std::collections::HashSet;
use std::fmt;
use std::marker::PhantomData;

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Code, Diagnostic, Diagnostics, JsonPath, Location, Phase, Severity,
};
use bedrock_addon_core::load::{ListedFile, LoadContext, LoadOptions, StackPack};
use bedrock_addon_core::molang;
use bedrock_addon_core::pack_identity::{
    PackIdVersion, PackOrigin, PackPath, PackRef, PackSide, PackType,
};

use super::Pack;
use super::report::StackReport;
use crate::manifest::StackPackManifest;

mod sealed {
    pub trait Sealed {}
}

/// The side of a pack stack. Implemented only by [`Behavior`] and [`Resource`].
pub trait Side: sealed::Sealed + Send + Sync + 'static {
    /// The runtime value, as stored in `Origin::side`.
    const SIDE: PackSide;
    /// The pack type every pack on this stack has.
    const PACK_TYPE: PackType;
    /// The Molang catalogue side its expressions compile against (`Server` / `Client`).
    const MOLANG_SIDE: molang::Side;
}

/// Marker for the behaviour-pack stack.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Behavior {}

impl sealed::Sealed for Behavior {}

impl Side for Behavior {
    const SIDE: PackSide = PackSide::Behavior;
    const PACK_TYPE: PackType = PackType::Behavior;
    const MOLANG_SIDE: molang::Side = molang::Side::Server;
}

/// Marker for the resource-pack stack.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Resource {}

impl sealed::Sealed for Resource {}

impl Side for Resource {
    const SIDE: PackSide = PackSide::Resource;
    const PACK_TYPE: PackType = PackType::Resource;
    const MOLANG_SIDE: molang::Side = molang::Side::Client;
}

/// One use of a pack on a stack (vanilla's `PackInstance`): the pack, where it came from, its
/// active subpack and whether another pack pulled it in.
#[derive(Clone, Debug)]
pub(crate) struct PackInstance {
    /// The pack.
    pub(crate) pack: Pack,
    /// Where it came from (`Package` is a base-game pack).
    pub(crate) origin: PackOrigin,
    /// The active subpack, by its index in the pack's `subpacks`.
    pub(crate) subpack: Option<usize>,
    /// Added as another pack's dependency.
    #[allow(
        dead_code,
        reason = "the game's `PackInstance` carries this flag; the stack report reads it"
    )]
    pub(crate) is_dependent: bool,
}

/// An ordered, resolved list of active packs, highest priority first, with the
/// collections loaded from it. `Send + Sync`; every accessor loads its family on first call
/// and caches it.
pub struct PackStack<S: Side> {
    /// The loaded packs, top first.
    instances: Vec<PackInstance>,
    /// Packs a missing dependency force-hid: on the stack, never loaded.
    hidden: Vec<PackInstance>,
    /// One entry per loaded pack, top first: the handle and the load context.
    packs: Vec<StackPack>,
    options: LoadOptions,
    /// The composition findings; a `StackSet` keeps both sides' on its behaviour stack.
    selection: Diagnostics,
    /// The pack icon lints of the stack's packs, lowest pack first.
    icons: Diagnostics,
    // a stack that is moved (out of `PackRepository::stacks`, into and out of a `StackLoader`)
    // would take that much of every frame it passes through.
    #[allow(
        dead_code,
        reason = "only the generated accessor lines read it; unused with no content feature on"
    )]
    caches: Box<accessors::Caches>,
    _side: PhantomData<fn() -> S>,
}

impl<S: Side> PackStack<S> {
    /// A stack of exactly these packs, top first, without dependency resolution or a vanilla
    /// layer. For tools that look at one pack: `PackStack::<Resource>::of([pack], opts)`.
    /// Fails if a pack has no module of `S`'s kind (`data` / `resources`); a pack with both
    /// modules is accepted on either stack.
    ///
    /// # Examples
    ///
    /// ```
    /// use bedrock_addon::load::LoadOptions;
    /// use bedrock_addon::manifest::Manifest;
    /// use bedrock_addon::pack::{Behavior, PackOrigin, PackStack, PackType, PackWriter};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let pack = PackWriter::new(Manifest::new("Ruby", PackType::Behavior)).into_pack(PackOrigin::World)?;
    /// let stack = PackStack::<Behavior>::of([pack], LoadOptions::default())?;
    /// assert_eq!(stack.packs().len(), 1);
    /// assert!(!stack.report().has_errors());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// [`WrongPackType`] when a pack does not hold a module of the stack's pack type.
    pub fn of(
        packs: impl IntoIterator<Item = Pack>,
        options: LoadOptions,
    ) -> Result<Self, WrongPackType> {
        let instances = packs
            .into_iter()
            .map(|pack| {
                if !pack.facts().has_module(S::PACK_TYPE) {
                    return Err(WrongPackType {
                        pack: pack.id().clone(),
                        expected: S::PACK_TYPE,
                        found: pack.pack_type(),
                    });
                }
                // No subpack: the stack is exactly the packs given.
                Ok(PackInstance {
                    origin: pack.report().pack_origin,
                    pack,
                    subpack: None,
                    is_dependent: false,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self::new(
            instances,
            Vec::new(),
            options,
            Diagnostics::default(),
        ))
    }

    /// A stack of these instances, top first.
    pub(crate) fn new(
        instances: Vec<PackInstance>,
        hidden: Vec<PackInstance>,
        options: LoadOptions,
        selection: Diagnostics,
    ) -> Self {
        let packs: Vec<StackPack> = instances
            .iter()
            .enumerate()
            .map(|(index, instance)| stack_pack(index, instance, &options))
            .collect();
        let mut selection = selection;
        if S::SIDE == PackSide::Resource {
            selection.extend(carried_in_vain(&packs));
        }
        let icons = icon_lints(&instances, &packs);
        Self {
            instances,
            hidden,
            packs,
            options,
            selection,
            icons,
            caches: Box::default(),
            _side: PhantomData,
        }
    }

    /// The packs, top first.
    pub fn packs(&self) -> impl ExactSizeIterator<Item = PackRef> + '_ {
        self.packs.iter().map(|entry| entry.pack.clone())
    }

    /// The options the stack was built with.
    pub fn options(&self) -> &LoadOptions {
        &self.options
    }

    /// Everything loaded so far, after the stack's own findings. First the stack's own: the
    /// pack-selection findings, on a resource stack the files its packs carry in vain
    /// (`IgnoredFile`, lowest pack first), then the pack icon lints of each pack (lowest pack
    /// first), as the packs were when the stack was built: a pack without a `pack_icon.png`
    /// is `PackIconMissing`; that an icon is no PNG or not square is said only of a pack built
    /// with the icon's bytes resident, since a load asks for no byte of an image (a caller
    /// with the bytes calls [`Pack::check_icon`]). Then each loaded collection's loader
    /// findings and diagnostics, in accessor-list order; a file that could not be read is
    /// listed once, by the first collection that walks it, though every collection that walks
    /// it keeps the `ReadError` among its own loader findings (not among its diagnostics).
    /// Collections not loaded yet are not loaded by it.
    pub fn report(&self) -> StackReport<'_> {
        let mut findings: Vec<&Diagnostic> = self.selection.iter().collect();
        findings.extend(self.icons.iter());
        self.collection_findings(&mut findings);
        once_per_unread_file(&mut findings);
        StackReport::new(findings)
    }

    /// Appends what every loaded collection of this side reported.
    pub(crate) fn collection_findings<'a>(&'a self, out: &mut Vec<&'a Diagnostic>) {
        for entry in Self::entries() {
            entry.report(self, out);
        }
    }

    /// The accessor lines of this side.
    pub(crate) fn entries() -> impl Iterator<Item = &'static dyn ErasedCollection> {
        accessors::ENTRIES
            .iter()
            .copied()
            .filter(|entry| entry.side() == S::SIDE)
    }

    /// Replaces the pack of the instance at `index` (the same pack, more of it resident).
    pub(crate) fn replace_pack(&mut self, index: usize, pack: Pack) {
        if let Some(instance) = self.instances.get_mut(index) {
            instance.pack = pack;
            if let Some(entry) = self.packs.get_mut(index) {
                *entry = listed(entry.clone(), instance);
            }
        }
    }

    /// The loaded packs, top first, as the walk reads them.
    pub(crate) fn instances(&self) -> &[PackInstance] {
        &self.instances
    }

    /// The packs a missing dependency hid.
    #[allow(dead_code, reason = "the stack report reads it")]
    pub(crate) fn hidden(&self) -> &[PackInstance] {
        &self.hidden
    }

    /// The stack entries a collection's `combine` sees (`StackCx::packs`).
    pub(crate) fn stack_packs(&self) -> &[StackPack] {
        &self.packs
    }

    /// The composition findings this stack carries.
    pub(crate) fn selection(&self) -> &Diagnostics {
        &self.selection
    }

    /// The stack the generated accessors read (the toy host of the macro test has its own).
    #[allow(
        dead_code,
        reason = "only the generated accessor lines call it; unused with no content feature on"
    )]
    pub(crate) fn as_stack(&self) -> &Self {
        self
    }
}

/// The packs, top first, the hidden ones and the collections loaded so far.
impl<S: Side> fmt::Debug for PackStack<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let collections: Vec<(Option<Collection>, String)> = Self::entries()
            .filter_map(|entry| Some((entry.collection(), entry.debug(self)?)))
            .collect();
        f.debug_struct("PackStack")
            .field("side", &S::SIDE)
            .field("packs", &self.packs)
            .field("hidden", &self.hidden)
            .field("collections", &collections)
            .finish_non_exhaustive()
    }
}

/// The entry of `instance` at stack `index`: its handle and its load context.
fn stack_pack(index: usize, instance: &PackInstance, options: &LoadOptions) -> StackPack {
    let facts = instance.pack.facts();
    let pack = PackRef::new(index, facts.id.clone(), facts.name.clone());
    let mut context = LoadContext::for_pack(
        options,
        facts.id.uuid,
        facts.id.pack_type,
        facts.min_engine_version.clone(),
        instance.origin.is_base_game(),
    );
    // The memory tier is mapped once per pack; a subpack takes its parent's.
    context.molang_version = molang::pack_molang_version(facts.min_engine_version.as_ref());
    #[cfg(feature = "script")]
    {
        context.runs_scripts = crate::families::script::starts_a_plugin(instance.pack.manifest());
    }
    let entry = StackPack::new(pack, instance.origin, context)
        .with_description(facts.description.as_deref())
        .with_capabilities(facts.capabilities.iter().map(AsRef::as_ref))
        .with_manifest(instance.pack.shared_manifest());
    listed(entry, instance)
}

/// `entry` with the file listing of `instance`: what the walks read, and the pack's
/// own files the active subpack shadows.
fn listed(entry: StackPack, instance: &PackInstance) -> StackPack {
    let facts = instance.pack.facts();
    let (files, shadowed) =
        walk::entry_listing(instance.pack.paths(), &facts.subpacks, instance.subpack);
    let shadowed = shadowed.into_iter().map(|path| ListedFile {
        key: path.clone(),
        path,
    });
    entry.with_files(files).with_shadowed_files(shadowed)
}

/// The files of the game's own data that a resource pack carries in vain: the client reads
/// each from its own data (or, the starter seeds, from its own resource packs), never from a
/// pack of the world.
// A copy in a pack has no effect and is reported as an `IgnoredFile` information.
const APP_ONLY_FILES: [&str; 5] = [
    "credits/credits.json",
    "CreativeStarterSeeds.txt",
    "StarterSeeds.txt",
    "manifest_publish.json",
    "texture_backwards_compatibility_mapping.json",
];

/// An `IgnoredFile` info for each file of [`APP_ONLY_FILES`] and each file below `persona/`
/// that a resource pack of the stack holds: lowest pack first, by the file's name alone (no
/// byte is read), at the file's own path (a copy in the active subpack is named where it
/// lies). A base-game pack's starter seeds are the game's own and are not reported.
fn carried_in_vain(packs: &[StackPack]) -> Diagnostics {
    let mut found = Diagnostics::default();
    let mut ignored = |path: &PackPath, pack: &StackPack| {
        found.push(Diagnostic::new(
            Code::from(&core_codes::IGNORED_FILE),
            Severity::Info,
            Phase::Validate,
            Location::new(
                Some(pack.pack.clone()),
                Some(path.clone()),
                JsonPath::new(),
                None,
            ),
        ));
    };
    for pack in packs.iter().rev() {
        let files = pack.files();
        for name in APP_ONLY_FILES {
            let own = pack.context.is_base_game_pack && name.ends_with("StarterSeeds.txt");
            // The listing is bytewise by key.
            let at =
                files.binary_search_by(|file| file.key.as_str().as_bytes().cmp(name.as_bytes()));
            if let Ok(at) = at
                && !own
            {
                ignored(&files[at].path, pack);
            }
        }
        for file in pack.files_below("persona") {
            ignored(&file.path, pack);
        }
    }
    found
}

/// Keeps the first `ReadError` of each file of each pack: a file two collections walk (an
/// `entity/` file is the client entities' and the attachables') and that cannot be read is in
/// each collection's own diagnostics, and once in a report.
pub(crate) fn once_per_unread_file(findings: &mut Vec<&Diagnostic>) {
    let mut seen: HashSet<(Option<&PackRef>, &str)> = HashSet::new();
    findings.retain(|finding| {
        // By legacy name: `READ_ERROR`, `READ_ERROR_WORDED` and the variant a family still
        // raises are each a read error.
        if finding.code.name() != "ReadError" {
            return true;
        }
        let Some(path) = finding.location.path.as_ref() else {
            return true;
        };
        seen.insert((finding.location.pack.as_ref(), path.as_str()))
    });
}

/// The pack icon lints ([`Pack::icon_findings`]) of each pack of the stack, lowest pack
/// first, each with its pack. A base-game pack without an icon is not reported: the game's
/// own slices ship none and no pack screen lists them.
// The game reads no pack icon at load, so these lints are the crate's own.
fn icon_lints(instances: &[PackInstance], packs: &[StackPack]) -> Diagnostics {
    let mut found = Diagnostics::default();
    for (instance, pack) in instances.iter().zip(packs).rev() {
        for mut finding in instance.pack.icon_findings() {
            if pack.context.is_base_game_pack
                && finding.code == Code::from(&core_codes::PACK_ICON_MISSING)
            {
                continue;
            }
            finding.location.pack = Some(pack.pack.clone());
            found.push(finding);
        }
    }
    found
}

/// The two stacks of one world.
pub struct StackSet {
    /// The behaviour-pack stack.
    pub behavior: PackStack<Behavior>,
    /// The resource-pack stack.
    pub resource: PackStack<Resource>,
}

/// Both stacks: their packs and the collections loaded so far.
impl fmt::Debug for StackSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StackSet")
            .field("behavior", &self.behavior)
            .field("resource", &self.resource)
            .finish()
    }
}

impl StackSet {
    /// Parses every collection of both stacks sequentially on the calling thread, from resident
    /// bytes, and returns everything they reported (parallel: `StackLoader`). Accessors called
    /// afterwards return the cached collections; collections already loaded are not loaded
    /// again. A file that is indexed but not resident is `FileNotLoaded`.
    ///
    /// The report lists both stacks' selection findings, their pack icon lints (those of a
    /// pack that is on both stacks once, as the behaviour stack's), then the behaviour
    /// stack's collections, then the resource stack's.
    pub fn load_all(&self) -> StackReport<'_> {
        for entry in accessors::ENTRIES {
            entry.load(self.stack_of(entry.side()));
        }
        self.report()
    }

    /// Puts `pack` (pack `index` of a loader) at every stack position it holds: `instances`
    /// says, per stack, which loader pack each position is.
    pub(crate) fn replace_pack_everywhere(
        &mut self,
        instances: &[Vec<usize>; 2],
        index: usize,
        pack: &Pack,
    ) {
        for (position, pack_index) in instances[0].iter().enumerate() {
            if *pack_index == index {
                self.behavior.replace_pack(position, pack.clone());
            }
        }
        for (position, pack_index) in instances[1].iter().enumerate() {
            if *pack_index == index {
                self.resource.replace_pack(position, pack.clone());
            }
        }
    }

    /// What both stacks have loaded so far, without loading anything.
    pub(crate) fn report(&self) -> StackReport<'_> {
        let mut findings: Vec<&Diagnostic> = self.behavior.selection().iter().collect();
        findings.extend(self.resource.selection().iter());
        // Each stack's pack icon lints. A pack with both modules is on both stacks with one
        // icon: its lints are listed once, as the behaviour stack's.
        let key = |finding: &'_ Diagnostic| {
            let pack = finding.location.pack.as_ref()?;
            Some((pack.id().clone(), finding.code.name()))
        };
        let listed: HashSet<_> = self.behavior.icons.iter().filter_map(key).collect();
        findings.extend(self.behavior.icons.iter());
        findings.extend(
            self.resource
                .icons
                .iter()
                .filter(|finding| key(finding).is_none_or(|key| !listed.contains(&key))),
        );
        self.behavior.collection_findings(&mut findings);
        self.resource.collection_findings(&mut findings);
        once_per_unread_file(&mut findings);
        StackReport::new(findings)
    }

    /// The stack of `side`, as the accessor lines take it.
    pub(crate) fn stack_of(&self, side: PackSide) -> &dyn Any {
        match side {
            PackSide::Behavior => &self.behavior,
            PackSide::Resource => &self.resource,
        }
    }
}

/// A pack given to [`PackStack::of`] has no module of the stack's kind. Read-only: the crate
/// produces it.
// A struct only the crate produces is `#[non_exhaustive]`, so a field can be added without
// breaking callers.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
#[error("pack {pack} is a {found:?} pack, not {expected:?}")]
pub struct WrongPackType {
    /// The pack.
    pub pack: PackIdVersion,
    /// The stack's pack type.
    pub expected: PackType,
    /// The pack's type.
    pub found: PackType,
}
