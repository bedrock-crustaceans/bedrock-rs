//! Loading the bundles of a stack: the walk of each pack's `manifest.json` and
//! `scripts/**/*.js`, one bundle per pack.
//!
//! This is where the three parts of scripting meet. Each pack's manifest is the one the pack
//! was opened with, and the script module's stage 1 (`ScriptModule`) reads it; one discovery serves
//! the stack, the highest pack first (stages 2 and 3: the entry, the gates on each
//! dependency, the versions the catalog serves); and the plugin it makes of a pack is what
//! the pack's files are scanned against. A rule the module and the bundle both state (one
//! script module per pack, the language, the entry, the main file) is raised by the module,
//! once; of the missing main file the bundle adds the file that differs only in case.

use std::collections::HashMap;

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity, VanillaText,
};
use bedrock_addon_core::load::{
    Accept, CollectionLoader, LoadContext, StackCx, StackPack, Walk, WalkFile,
};
use bedrock_addon_core::pack_identity::{PackPath, PackSide, PackType};

use super::scan::{Member, Module, Parts, build, is_js, member, missing_main};
use super::{EntryRef, ScanOptions, ScriptBundle, ScriptPath, ScriptRoot};
use crate::module::Stage1;
use crate::{
    ModuleCatalog, ScriptContext, ScriptDiscovery, ScriptFiles, ScriptModule, ScriptRuntime,
    ScriptTarget,
};
use bedrock_addon_core::load::Reported;
use bedrock_addon_core::manifest::{
    CapabilityRegistry, CapabilityValidation, StackPackManifest, TrustLevel,
};

/// The pack's manifest.
const MANIFEST: &str = "manifest.json";

/// The files the bundles may read: each pack's manifest (which module runs the scripts, and
/// from which entry) and its script files. The other files below `scripts/` are listed by
/// name only (`StackPack::files_below`).
fn is_bundle_file(path: &PackPath) -> bool {
    path.as_str() == MANIFEST || ScriptPath::of_pack_path(path).is_some()
}

/// Which of a pack's bundle files the walk reads: its manifest always, its script files
/// only when its script module starts a plugin. The script files of every other pack are
/// listed by name (`StackPack::files_below`) and no byte of them is asked for: nothing of
/// such a pack ever runs.
fn is_read_in(path: &PackPath, cx: &LoadContext) -> bool {
    cx.runs_scripts || path.as_str() == MANIFEST
}

/// The manifest and `scripts/**/*.js` of every pack, the highest pack first.
// The game gives each pack of the stack its own script source rooted at `scripts/`, and
// lists the `.js` files below it recursively.
const WALK: Walk = Walk::json_tree_in(&[""])
    .accept(Accept::Fn(is_bundle_file))
    .top_down()
    .in_pack(is_read_in);

/// One file of the walk, as its `Parse` job leaves it.
#[derive(Debug)]
pub struct Walked(File);

/// What a file of the walk is.
#[derive(Debug)]
enum File {
    /// A pack's manifest, for stage 1 to read in `combine`, where the pack's opened manifest
    /// is in view.
    Manifest {
        /// The pack's place in the stack.
        pack: Option<usize>,
        /// The stack's side.
        side: PackSide,
        /// The file's path in its pack.
        path: PackPath,
        /// The context the file was walked with.
        cx: Box<LoadContext>,
        /// The file's bytes, read again only for a path that is not the root `manifest.json`
        /// or an entry built by hand, which has no opened manifest.
        bytes: bytes::Bytes,
    },
    /// A script file.
    Member {
        /// The pack's place in the stack.
        pack: Option<usize>,
        /// The file, read and lexed.
        member: Box<Member>,
    },
}

/// A registry every vanilla capability passes, for the manifest of a stack entry built by hand
/// (which has no opened manifest): a pack's manifest is accepted when the pack is opened, with
/// the host's own registry.
fn accepting() -> CapabilityRegistry {
    let mut registry = CapabilityRegistry::empty();
    for name in [
        "chemistry",
        "editorExtension",
        "script_eval",
        "raytraced",
        "pbr",
    ] {
        registry.insert(name, |_| CapabilityValidation::Succeeded {
            trust: TrustLevel::NotTrusted,
        });
    }
    registry
}

/// One file of the walk.
fn parse_walk_file(file: &WalkFile, cx: &LoadContext) -> Walked {
    let pack = file.pack.index();
    if let Some(name) = ScriptPath::of_pack_path(&file.key) {
        return Walked(File::Member {
            pack,
            member: Box::new(member(name, &file.path, &file.bytes)),
        });
    }
    Walked(File::Manifest {
        pack,
        side: file.side,
        path: file.path.clone(),
        cx: Box::new(cx.clone()),
        bytes: file.bytes.clone(),
    })
}

/// Stage 1 on the manifest of `entry` at `path`, walked with `cx`. The pack's own findings
/// about its manifest were told when it was opened: of the script side only what stage 1 adds
/// is kept. The opened manifest is the pack's root `manifest.json`; a file of another path (a
/// `manifest.json` of the active subpack, standing in by its key) and the manifest of an entry
/// built by hand (which has no opened manifest) are read from the walked bytes.
fn stage1(entry: &StackPack, path: &PackPath, cx: &LoadContext, bytes: &[u8]) -> Stage1 {
    match entry.manifest() {
        Some(manifest) if path.as_str() == ROOT_MANIFEST => {
            ScriptModule::stage1(manifest, path, cx)
        }
        _ => ScriptModule::read(path, bytes, cx, &accepting()),
    }
}

/// The path of the manifest a pack is opened with.
const ROOT_MANIFEST: &str = "manifest.json";

/// What the walk read of one pack.
#[derive(Default)]
struct OfPack {
    side: Option<PackSide>,
    module: Option<ScriptModule>,
    findings: Diagnostics,
    refused: bool,
    members: Vec<Member>,
}

/// The script files and the other files below `scripts/` of the pack `entry`, both bytewise:
/// every file the pack lists there, with what the walk read of it.
fn members_of(entry: &StackPack, read: Vec<Member>) -> (Vec<Member>, Vec<PackPath>) {
    let mut read: HashMap<ScriptPath, Member> = read
        .into_iter()
        .map(|member| (member.name.clone(), member))
        .collect();
    let mut members = Vec::new();
    let mut others = Vec::new();
    // The listing is bytewise by key, which is the order of the names.
    for file in entry.files_below(ScriptRoot::DIRECTORY) {
        match ScriptPath::of_pack_path(&file.key) {
            Some(name) => members.push(
                read.remove(&name)
                    .unwrap_or_else(|| Member::listed(name, &file.path)),
            ),
            None => others.push(file.path.clone()),
        }
    }
    if !read.is_empty() {
        // A file the walk read of a pack that lists none (a stack entry built by hand).
        members.extend(read.into_values());
        members.sort_by(|a, b| a.name.cmp(&b.name));
    }
    (members, others)
}

/// What the walk read, by the pack's place in the stack of `entries`, with stage 1 read of
/// each manifest.
fn by_pack(parsed: Vec<Walked>, entries: &[StackPack]) -> Vec<OfPack> {
    let mut packs: Vec<OfPack> = Vec::new();
    packs.resize_with(entries.len(), OfPack::default);
    for Walked(file) in parsed {
        match file {
            File::Manifest {
                pack,
                side,
                path,
                cx,
                bytes,
            } => {
                if let Some((of_pack, entry)) =
                    pack.and_then(|index| Some((packs.get_mut(index)?, entries.get(index)?)))
                {
                    let read = stage1(entry, &path, &cx, &bytes);
                    of_pack.side = Some(side);
                    of_pack.module = read.module;
                    of_pack.findings = read.added;
                    of_pack.refused = read.refused;
                }
            }
            File::Member { pack, member } => {
                if let Some(of_pack) = pack.and_then(|index| packs.get_mut(index)) {
                    of_pack.members.push(*member);
                }
            }
        }
    }
    packs
}

/// One bundle per pack that has a script module or a file below `scripts/`, the highest pack
/// first.
fn combine(parsed: Vec<Walked>, stack: &StackCx<'_>) -> Vec<ScriptBundle> {
    let packs = by_pack(parsed, stack.packs);
    // One discovery for the stack, the highest pack first: a `script` module is a behaviour
    // pack's and a `client_script` module a resource pack's, so a stack is one target and
    // its modules are discovered together.
    let catalog = ModuleCatalog::shipped();
    let mut discovery = ScriptDiscovery::new(&catalog);
    let mut bundles = Vec::new();
    for (entry, of_pack) in stack.packs.iter().zip(packs) {
        // The side of the stack the manifest was walked on; a pack whose manifest the walk
        // did not hand over is taken by its type.
        let side = of_pack.side.unwrap_or(match entry.context.pack_type {
            PackType::Resource => PackSide::Resource,
            _ => PackSide::Behavior,
        });
        let target = match side {
            PackSide::Behavior => ScriptTarget::Server,
            PackSide::Resource => ScriptTarget::Client,
        };
        let module = of_pack.module.filter(|module| module.target == target);
        let below = entry.files_below(ScriptRoot::DIRECTORY);
        if module.is_none() && !of_pack.refused && below.is_empty() && of_pack.members.is_empty() {
            continue;
        }
        let (members, others) = members_of(entry, of_pack.members);
        let in_pack = |mut finding: Diagnostic| {
            finding.location.pack = Some(entry.pack.clone());
            finding
        };
        let mut findings: Vec<Diagnostic> = of_pack.findings.into_iter().map(in_pack).collect();
        let found = module.as_ref().map(|module| {
            let files = ScriptFiles::from_members(members.iter().map(|m| m.name.as_str()));
            discovery.discover(module, &files, &ScriptContext::for_pack(&entry.context))
        });
        if let Some(found) = &found {
            for finding in found.diagnostics() {
                let mut finding = in_pack(finding.clone());
                if let DiagnosticCode::UnresolvedReference {
                    text: Some(VanillaText::PluginMissingMainFile { main, .. }),
                } = &finding.code
                {
                    // The file that differs from the main file only in case is the hint.
                    finding.detail = Some(Box::new(missing_main(
                        members.iter().map(|member| &member.name),
                        &ScriptPath::new(main.as_str()),
                    )));
                }
                findings.push(finding);
            }
        }
        let state = match (&module, found.as_ref().and_then(|found| found.plugin())) {
            // Stage 1 refuses the manifest: no module is read of it, and its finding says so.
            (None, _) if of_pack.refused => Module::Dropped { entry: None },
            // A pack of the game: its `scripts/` are the engine's to read (the wrapper files of
            // the library packs), whatever its modules.
            (None, _) if entry.context.is_base_game_pack => Module::OfTheGame,
            (None, _) => Module::None,
            (Some(_), Some(plugin)) => Module::Plugin(plugin),
            (Some(module), None) => Module::Dropped {
                entry: module
                    .entry
                    .as_deref()
                    .filter(|_| ScriptRuntime::from_language(module.language.as_deref()).is_some())
                    .map(EntryRef::of),
            },
        };
        if matches!(state, Module::Plugin(_)) && !entry.context.runs_scripts {
            // The walk reads a pack's script files only on a context that says the pack
            // runs scripts; this one does not say so (a stack entry built by hand), so the
            // files of this plugin are listed and unread, and nothing below is about them.
            if let Some(unread) = unread_scripts(&members, &state) {
                findings.push(in_pack(unread));
            }
        }
        // The `.js` files outside `scripts/`: of a pack with a script module, started or not.
        let outside = if module.is_some() {
            entry
                .files()
                .iter()
                .filter(|file| {
                    is_js(&file.key) && !file.key.as_str().starts_with(ScriptRoot::PREFIX)
                })
                .map(|file| file.path.clone())
                .collect()
        } else {
            Vec::new()
        };
        let mut bundle = build(Parts {
            pack: entry.pack.clone(),
            side,
            members,
            others,
            outside,
            module: state,
            findings,
            options: ScanOptions::default(),
        });
        // The module is the pack's whether or not its script starts.
        bundle.module = module.as_ref().map(crate::ScriptModuleRef::from);
        bundles.push(bundle);
    }
    bundles
}

/// The one finding for a plugin whose script files the walk left unread: an information at
/// the main file (at the first unread file when the main file is not among them), since the
/// graph of the bundle is then drawn over names alone. `None` when every file was read.
fn unread_scripts(members: &[Member], module: &Module<'_>) -> Option<Diagnostic> {
    let unread = |member: &&Member| member.file.content.is_none();
    let first = members.iter().find(unread)?;
    let main = match module {
        Module::Plugin(plugin) => members
            .iter()
            .filter(unread)
            .find(|member| member.name == plugin.main),
        _ => None,
    };
    let at = main.unwrap_or(first);
    Some(Diagnostic::new(
        DiagnosticCode::FileNotLoaded,
        Severity::Info,
        Phase::Read,
        Location::new(None, Some(at.file.path.clone()), JsonPath::new(), None),
    ))
}

impl Reported for ScriptBundle {
    fn reported(&self) -> Box<dyn Iterator<Item = &Diagnostic> + '_> {
        Box::new(self.findings())
    }
}

/// The loader of `stack.scripts()`: the script files of every pack of the stack, one bundle
/// per pack. The walk is the same on both sides; the side decides which module type runs the
/// scripts (`script` in a behaviour pack, `client_script` in a resource pack).
pub const LOADER: CollectionLoader<Vec<ScriptBundle>, Walked> = CollectionLoader {
    walk: WALK,
    parse: parse_walk_file,
    combine,
    needs: &[],
    then: None,
};
