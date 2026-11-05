//! Loading behavior trees: `Content` / `Definition`, the walk of `behavior_trees/` and the
//! combine into one registry keyed by `minecraft:name`.

use super::parse::parse_document;
use super::{BehaviorTree, builtin};
use std::collections::{HashMap, HashSet};

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity,
};
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, EngineVersionDecision, NoBeta,
    Origin, Placed, Registry, RegistryBuilder, Rejected, VersionInfo, default_path,
};
use bedrock_addon_core::id::{BehaviorTreeId, kind};
use bedrock_addon_core::json::{EncodeError, WriteStyle};
use bedrock_addon_core::load::{
    Accept, AppDataPlace, CollectionLoader, ListDepth, LoadContext, StackCx, Walk, WalkFile,
    WalkOrder,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::ENGINE_VERSION;

/// The directory behavior-tree files live in.
const DIRECTORY: &str = "behavior_trees";

impl bedrock_addon_core::__private::Sealed for BehaviorTree {}

impl Content for BehaviorTree {
    const NAME: &'static str = "behavior tree";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    /// Parses one behavior-tree file; the rules between files (one tree per name, the
    /// built-in trees) apply to a stack, not to a single file.
    fn parse(path: &PackPath, bytes: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        parse_file(PackRef::app_data(), PackSide::Behavior, path, bytes).1
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        self.write_into(out, style)
    }

    fn to_json(&self) -> Result<String, EncodeError> {
        let mut out = Vec::new();
        self.write_json(&mut out, WriteStyle::default())?;
        Ok(String::from_utf8(out).unwrap_or_default())
    }
}

impl Definition for BehaviorTree {
    type Kind = kind::BehaviorTree;
    type File = BehaviorTree;
    type Beta = NoBeta<kind::BehaviorTree>;
}

impl Placed for BehaviorTree {
    /// `behavior_trees/<name>.json`.
    fn default_path(&self) -> PackPath {
        default_path(DIRECTORY, &self.name)
    }
}

/// One file's outcome: the name it gives its tree, and the document or its rejection.
#[derive(Debug)]
pub struct FileOutcome {
    /// The file's pack and path.
    pack: PackRef,
    path: PackPath,
    /// What the file is.
    read: Read,
}

/// What the loader made of one walked file.
#[derive(Debug)]
enum Read {
    /// A file that names no tree (not JSON, no usable `minecraft:name`).
    Nameless(Rejected),
    /// A file that names a tree, and the tree or why it does not load.
    Named(
        BehaviorTreeId,
        Box<Result<Document<BehaviorTree>, Rejected>>,
    ),
}

/// Loads one behavior-tree file of `pack`: the name it gives its tree, and the document or
/// its rejection.
fn parse_file(
    pack: PackRef,
    side: PackSide,
    path: &PackPath,
    bytes: &[u8],
) -> (
    Option<BehaviorTreeId>,
    Result<Document<BehaviorTree>, Rejected>,
) {
    let origin = Origin::new(pack, side, path.clone(), None, Vec::new());
    // A number no double holds is read as the largest finite one, with a warning.
    let finite = bedrock_addon_core::document::FiniteText::of(bytes);
    let parsed = parse_document(finite.bytes());
    // Every finding is about this file of this pack.
    let mut diagnostics: Diagnostics = parsed
        .diagnostics
        .into_iter()
        .map(|mut finding| {
            finding.location.pack = Some(origin.pack.clone());
            finding.location.path = Some(origin.path.clone());
            finding
        })
        .collect();
    finite.settle(&mut diagnostics, Some(&origin));
    let result = match parsed.tree {
        // The reader gives a tree only when it raised no error.
        Some(tree) => Ok(Document::new(tree, origin, unversioned(), diagnostics)),
        None => Err(Rejected::new(origin, diagnostics)),
    };
    (parsed.name, result)
}

/// The family has no `format_version`: nothing was written, and the tree is read as the
/// engine reads it.
fn unversioned() -> VersionInfo {
    VersionInfo::new(None, false, ENGINE_VERSION, ENGINE_VERSION, false)
}

/// A behavior-tree file: the extension is `json`.
fn is_json(path: &PackPath) -> bool {
    path.as_str()
        .rsplit_once('.')
        .is_some_and(|(_, extension)| extension == "json")
}

/// `behavior_trees/**`, packs from the lowest priority to the highest, as vanilla walks the
/// stack.
// `AutomationBehaviorTreeGroup::_loadTrees`: `getFullStack()` from the last
// entry to the first; `forEachIn("behavior_trees", …)` reads sub-folders and keeps every path.
const WALK: Walk = Walk {
    directories: &[DIRECTORY],
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

fn parse_walk_file(file: &WalkFile, _: &LoadContext) -> FileOutcome {
    let read = match parse_file(file.pack.clone(), file.side, &file.path, &file.bytes) {
        (Some(name), result) => Read::Named(name, Box::new(result)),
        (None, Ok(document)) => {
            // A document always has its name; a nameless one is rejected as it stands.
            let origin = document.origin().clone();
            Read::Nameless(Rejected::new(origin, document.diagnostics().clone()))
        }
        (None, Err(rejected)) => Read::Nameless(rejected),
    };
    FileOutcome {
        pack: file.pack.clone(),
        path: file.path.clone(),
        read,
    }
}

/// The trees built into the game, as documents of no pack (the app-data layer) at
/// `behavior_trees/<name>.json`.
fn built_ins() -> impl Iterator<Item = (BehaviorTreeId, Document<BehaviorTree>)> {
    builtin::TREES.iter().filter_map(|(name, text)| {
        let path = default_path(DIRECTORY, &BehaviorTreeId::parse(*name));
        let (name, result) = parse_file(
            PackRef::app_data(),
            PackSide::Behavior,
            &path,
            text.as_bytes(),
        );
        Some((name?, result.ok()?))
    })
}

/// A combine-time finding about the file at `path` of `pack`.
fn notice(code: DiagnosticCode, severity: Severity, pack: &PackRef, path: &PackPath) -> Diagnostic {
    Diagnostic::new(
        code,
        severity,
        Phase::Validate,
        Location::new(
            Some(pack.clone()),
            Some(path.clone()),
            JsonPath::new(),
            None,
        ),
    )
}

/// The file that holds a name while the walk goes on: its pack and path, and whether it
/// loaded.
struct Claim {
    pack: PackRef,
    path: PackPath,
    loaded: bool,
}

/// One tree per name. Vanilla first maps every name to the last file of the walk that gives
/// it (so the highest-priority pack's, and inside a pack the last path's), then loads each
/// name's file, and loads the built-in trees last: a file that names a tree takes the name
/// whether or not it loads, and a built-in tree replaces whatever has its name.
// `_loadTrees`: `try_emplace`, then an unconditional overwrite
// of the path and pack; `addBehaviorTree` assigns by name;
// `initializeTrees` loads the nine `AutomationTrees` texts after the packs.
fn combine(parsed: Vec<FileOutcome>, stack: &StackCx<'_>) -> Registry<BehaviorTree> {
    // Who holds each name at the end of the walk, and what each file's arrival replaced.
    let mut claims: HashMap<BehaviorTreeId, Claim> = HashMap::new();
    let mut replaced: Vec<Option<Diagnostic>> = Vec::with_capacity(parsed.len());
    for outcome in &parsed {
        let Read::Named(name, result) = &outcome.read else {
            replaced.push(None);
            continue;
        };
        let claim = Claim {
            pack: outcome.pack.clone(),
            path: outcome.path.clone(),
            loaded: result.is_ok(),
        };
        let earlier = claims.insert(name.clone(), claim);
        replaced.push(earlier.filter(|earlier| earlier.loaded).map(|earlier| {
            if earlier.pack == outcome.pack {
                // Ours: paths are walked sorted, so the last path of the pack wins; vanilla
                // says nothing (`AutomationBehaviorTreeGroup::addBehaviorTree`
                // emplaces without a log): an information.
                let code = DiagnosticCode::DuplicateIdentifier { text: None };
                notice(code, Severity::Info, &earlier.pack, &earlier.path)
            } else {
                let code = DiagnosticCode::ShadowedDefinition { text: None };
                notice(code, Severity::Info, &earlier.pack, &earlier.path)
            }
        }));
    }
    let shadowed_by_built_in: Vec<Diagnostic> = builtin::TREES
        .iter()
        .filter_map(|(name, _)| claims.get(&BehaviorTreeId::parse(*name)))
        .filter(|claim| claim.loaded)
        .map(|claim| {
            let code = DiagnosticCode::ShadowedByBuiltin;
            // Ours: `addBehaviorTree` emplaces the built-in tree without a
            // line, like a duplicate within a pack.
            notice(code, Severity::Info, &claim.pack, &claim.path)
        })
        .collect();
    // The files that hold a name at the end of the walk. One that failed never reaches the
    // decision, and a built-in tree, pushed last, replaces a pack tree that holds its name.
    let winners: HashSet<(PackRef, PackPath)> = claims
        .into_values()
        .map(|claim| (claim.pack, claim.path))
        .collect();
    // The registry's rule is the outcome above: a tree is effective when its file holds the
    // name at the end, and listed among its name's layers otherwise.
    let decide = move |_: Option<&Document<BehaviorTree>>,
                       incoming: &Document<BehaviorTree>,
                       _: &mut Diagnostics| {
        let origin = incoming.origin();
        if origin.pack.is_app_data()
            || winners.contains(&(origin.pack.clone(), origin.path.clone()))
        {
            EngineVersionDecision::Replace
        } else {
            EngineVersionDecision::Ignore
        }
    };
    let mut builder = RegistryBuilder::new(
        CombineRule::EngineVersionMerge {
            decide: Box::new(decide),
            // Never called: `decide` never merges.
            merge: |_, _| {},
        },
        CombineRule::top_wins(),
        stack.options,
    );
    // Ours: a file of the folder that is no `.json` file is not a tree file (vanilla has no
    // extension filter: it parses every file of the folder). Reported from the packs' file
    // lists; the walk reads the `.json` files only.
    for (pack, ignored) in bedrock_addon_core::load::unread_files(stack, DIRECTORY, is_json) {
        builder.push_diagnostic(notice(
            DiagnosticCode::IgnoredFile { text: None },
            Severity::Info,
            &pack.pack,
            &ignored.path,
        ));
    }
    for (outcome, replaced) in parsed.into_iter().zip(replaced) {
        match outcome.read {
            Read::Nameless(rejected) => builder.push(Contribution::Rejected(None, rejected)),
            Read::Named(name, result) => builder.push(match *result {
                Ok(document) => Contribution::Stable(name, document),
                Err(rejected) => Contribution::Rejected(Some(name), rejected),
            }),
        }
        if let Some(replaced) = replaced {
            builder.push_diagnostic(replaced);
        }
    }
    for (name, document) in built_ins() {
        builder.push(Contribution::Stable(name, document));
    }
    for finding in shadowed_by_built_in {
        builder.push_diagnostic(finding);
    }
    builder.finish()
}

/// The loader of `bp.behavior_trees()`: the trees of every behaviour pack and the built-in
/// ones, by name.
pub const fn load_registry() -> CollectionLoader<Registry<BehaviorTree>, FileOutcome> {
    CollectionLoader {
        walk: WALK,
        parse: parse_walk_file,
        combine,
        needs: &[],
        then: None,
    }
}
