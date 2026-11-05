//! Behavior trees: `BehaviorTree`, `Node`.
//!
//! Automation behavior trees: `behavior_trees/**/*.json` of a behaviour pack,
//! `minecraft:behavior_tree`. Not a version chain: no file has a `format_version` and no loader
//! reads one, so the family is one unversioned struct read by one hand-written walker.
//! Vanilla loads these files in development builds only; retail 1.26.45 has no loader.

#![cfg_attr(docsrs, feature(doc_cfg))]

#[doc(hidden)]
pub use link::link as link_subtrees;

mod builtin;
mod kinds;
mod link;
mod read;
mod write;

#[cfg(test)]
mod tests;

use crate::read::parse_document;
use bedrock_addon_core::id::BehaviorTreeId;
use std::collections::{HashMap, HashSet};

use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, Phase};
use bedrock_addon_core::document::unversioned;
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, EngineVersionDecision, JsonContent,
    NoBeta, Origin, Placed, Registry, RegistryBuilder, Rejected, Unknown, default_path,
};
use bedrock_addon_core::id::kind;
use bedrock_addon_core::json::{EncodeError, WriteStyle};
use bedrock_addon_core::load::{CollectionLoader, LoadContext, StackCx, Walk, WalkFile};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::ENGINE_VERSION;

/// A block position literal: `[x, y, z]` of integers.
pub type BlockPos = [i32; 3];

/// A position literal: `[x, y, z]` of numbers.
pub type Vec3 = [f32; 3];

/// A behavior-tree file (`minecraft:behavior_tree`): a named tree of action nodes that
/// vanilla's automation test framework runs on the local player.
///
/// The members of the document root other than `minecraft:behavior_tree` (a `format_version`,
/// for one) and those of `minecraft:behavior_tree` other than `minecraft:name` and
/// `minecraft:root` are kept in the document's unknown table ([`Document::unknown`]); vanilla
/// reads none of them.
///
/// JSON: `{"minecraft:behavior_tree": {"minecraft:name": "chop_wood", "minecraft:root":
/// {"minecraft:sequence": {"children": [...]}}}}`. The tree is keyed by `minecraft:name`; the
/// file name plays no part. The family has no `format_version`: vanilla reads only
/// `minecraft:behavior_tree` from the document root and only `minecraft:name` and
/// `minecraft:root` from it.
///
/// Vanilla loads these files in development builds only (the retail game has no loader), and
/// only the development commands `/bhave` and `/automate` run a tree.
#[derive(Clone, Debug, PartialEq)]
pub struct BehaviorTree {
    /// `minecraft:name`: the key of the tree, a non-empty string. A file without one is not
    /// loaded.
    pub name: BehaviorTreeId,
    /// `minecraft:root`: an object with exactly one member, the root node.
    pub root: Node,
    /// The whole file text as read (byte order mark, comments and line ends included): what a
    /// development server sends to a joining client as the payload of `AddBehaviorTreePacket`,
    /// one packet per tree. Empty for a tree built in memory. Writing a tree writes
    /// [`name`](Self::name) and [`root`](Self::root), never this text.
    pub raw: String,
}

/// A node parameter as vanilla keeps it: a literal and the name of a blackboard entry.
///
/// A leaf reads each parameter from a pair of members, a literal (`num_of_ticks`) and a key
/// (`num_of_ticks_id`). When the literal is present and of the right type, it is the value and
/// the key stays empty. Otherwise the literal keeps its default and the key is the `…_id`
/// member's text, or the parameter's default key when that member is absent. When the tree
/// runs, the blackboard entry named by `key` wins if it exists with the parameter's type;
/// the literal is used otherwise.
///
/// A literal of the wrong JSON type is ignored, not an error. An integer literal takes an
/// integer or a bool (`true` is 1), never a real such as `1.0`; a float literal takes any
/// number or a bool.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Param<T> {
    /// The literal: the value as written, or the parameter's default when the literal member
    /// is absent or of the wrong type.
    pub literal: T,
    /// The blackboard key: empty when the literal was read; otherwise the `…_id` member's
    /// text, or the parameter's default key.
    pub key: String,
}

/// One node of a behavior tree: a JSON object with exactly one member, whose name is the
/// node type (`minecraft:sequence`) and whose value is the node's body.
///
/// There are 24 node types. A body that is not an object reads as an empty one. The body
/// members the node type does not read are kept in the document's unknown table
/// ([`Document::unknown`]) at the body's path, `…/<node type>`.
///
/// A tree read from a file nests at most as deep as the reader's limit of 128 JSON levels
/// (62 decorators around a leaf). A tree built in memory may nest deeper: it is dropped and
/// walked ([`nodes`](Self::nodes)) without a stack frame per level, and the writer refuses it,
/// but `Clone`, `PartialEq` and `Debug` recurse once per level, as derived impls do.
///
/// Because a node has its own `Drop`, a field cannot be moved out of a `Node` by pattern:
/// match on a reference, or take the field with `std::mem::take`.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Node {
    /// `minecraft:sequence`: a composite over its children, in order.
    Sequence {
        /// `children`: an array of nodes. `None` when the member is absent or not an array;
        /// the node then has no children. An element that is not a one-member object, or
        /// whose type is unknown, is left out.
        children: Option<Vec<Node>>,
    },
    /// `minecraft:selector`: a composite over its children, in order.
    Selector {
        /// `children`: an array of nodes. `None` when the member is absent or not an array;
        /// the node then has no children. An element that is not a one-member object, or
        /// whose type is unknown, is left out.
        children: Option<Vec<Node>>,
    },
    /// `minecraft:repeat_until_success`: a decorator that repeats its child.
    RepeatUntilSuccess {
        /// `child`: one node. `None` when the member is absent, not a one-member object, or
        /// of an unknown type.
        child: Option<Box<Node>>,
        /// `desired_max_number_of_attempts` (default 0), or the key
        /// `max_number_of_attempts_id` (default key `max_number_of_attempts`).
        max_attempts: Param<i32>,
    },
    /// `minecraft:repeat_until_failure`: a decorator that repeats its child.
    RepeatUntilFailure {
        /// `child`: one node. `None` when the member is absent, not a one-member object, or
        /// of an unknown type.
        child: Option<Box<Node>>,
    },
    /// `minecraft:inverter`: a decorator that inverts its child's result.
    Inverter {
        /// `child`: one node. `None` when the member is absent, not a one-member object, or
        /// of an unknown type.
        child: Option<Box<Node>>,
    },
    /// `minecraft:subtree`: runs another tree.
    Subtree {
        /// `subtree_id`: the `minecraft:name` of the other tree, resolved when the node runs;
        /// empty when the member is absent. Loading checks neither that the tree exists nor
        /// that subtrees form no cycle.
        subtree_id: BehaviorTreeId,
        /// `data_items`: the blackboard entries copied into the subtree, as `(from, to)`. A
        /// string `s` is `(s, s)`; an object with both `from` and `to` is that pair; the
        /// member is one such item or an array of them. Any other item is left out.
        data_items: Vec<(String, String)>,
    },
    /// `minecraft:activate_tool`.
    ActivateTool {
        /// `block_pos` (default `[0, 0, 0]`), or the key `block_pos_id` (default key
        /// `target_block_pos`).
        block_pos: Param<BlockPos>,
    },
    /// `minecraft:attack`.
    Attack {
        /// `num_of_ticks` (default 0), or the key `num_of_ticks_id` (default key
        /// `target_num_of_ticks`).
        ticks: Param<i32>,
    },
    /// `minecraft:break_block`.
    BreakBlock {
        /// `block_pos` (default `[0, 0, 0]`), or the key `block_pos_id` (default key
        /// `target_block_pos`).
        block_pos: Param<BlockPos>,
        /// `num_of_ticks` (default 0), or the key `num_of_ticks_id` (default key
        /// `target_num_of_ticks`).
        ticks: Param<i32>,
    },
    /// `minecraft:consume_item`: no parameters.
    ConsumeItem {},
    /// `minecraft:find_block`.
    FindBlock {
        /// `block_name` (default empty), or the key `block_name_id`. The key is `block_name`
        /// from the start, so it is `block_name` also when the literal was read.
        block_name: Param<String>,
        /// `search_radius`, or the key `search_radius_id` (default key
        /// `target_search_radius`). The literal is 32 when the member is absent or of the
        /// wrong type.
        search_radius: Param<i32>,
    },
    /// `minecraft:find_entity`.
    FindEntity {
        /// `entity_name` (default empty), or the key `entity_type` (default key
        /// `entity_name`).
        entity_name: Param<String>,
        /// `search_radius`, or the key `search_radius_id` (default key
        /// `target_search_radius`). The literal is 32 when the member is absent or of the
        /// wrong type.
        search_radius: Param<i32>,
    },
    /// `minecraft:fly`.
    ///
    /// Vanilla stores both literals in one member: `should_throw_event_if_no_state_change_
    /// necessary_bool`, when it is a bool, replaces the value of `should_be_flying_bool`, and
    /// the node uses that one value for both flags. Both `literal`s here are that value: the
    /// throw literal if it was read, else the flying literal, else `false`.
    Fly {
        /// `should_be_flying_bool`, or the key `should_be_flying_id` (default key
        /// `should_be_flying`).
        should_be_flying: Param<bool>,
        /// `should_throw_event_if_no_state_change_necessary_bool`, or the key
        /// `should_throw_event_if_no_state_change_necessary_id` (default key
        /// `should_throw_event_if_no_state_change_necessary`).
        throw_if_no_change: Param<bool>,
    },
    /// `minecraft:get_interaction_position_for_block`.
    GetInteractionPositionForBlock {
        /// `target_block_pos` (default `[0, 0, 0]`), or the key `target_block_pos_id` (default
        /// key `target_block_pos`).
        target: Param<BlockPos>,
        /// `anchor_block_pos` (default `[0, 0, 0]`), or the key `anchor_block_pos_id` (default
        /// key `anchor_block_pos`).
        anchor: Param<BlockPos>,
        /// `facing_string` (default empty), or the key `facing_string_id` (default key
        /// `facing_string`).
        facing: Param<String>,
        /// `max_search_distance` (default 0), or the key `max_search_distance_id` (default
        /// key `max_search_distance`).
        max_search_distance: Param<i32>,
    },
    /// `minecraft:interact`.
    Interact {
        /// `num_of_ticks` (default 1), or the key `num_of_ticks_id` (default key
        /// `target_num_of_ticks`).
        ticks: Param<i32>,
    },
    /// `minecraft:look_at`.
    LookAt {
        /// `block_pos` (default `[0, 0, 0]`), or the key `block_pos_id` (default key
        /// `target_block_pos`).
        block_pos: Param<BlockPos>,
    },
    /// `minecraft:look_at_entity`.
    LookAtEntity {
        /// `entity_name` (default empty), or the key `entity_type` (default key
        /// `entity_name`).
        entity_name: Param<String>,
        /// `search_radius`, or the key `search_radius_id` (default key
        /// `target_search_radius`). The literal is 32 when the member is absent or of the
        /// wrong type. Loaded, but the node never uses it.
        search_radius: Param<i32>,
    },
    /// `minecraft:move`.
    Move {
        /// `desired_direction` (default empty; `forward` in vanilla's files), or the key
        /// `direction_id` (default key `direction`).
        direction: Param<String>,
        /// `desired_second_direction` (default empty), or the key `second_direction_id`
        /// (default key `second_direction`).
        second_direction: Param<String>,
        /// `max_number_of_ticks` (default 0), or the key `number_of_ticks_id` (default key
        /// `number_of_ticks`).
        ticks: Param<i32>,
    },
    /// `minecraft:move_to`.
    MoveTo {
        /// `block_pos`, a position of numbers (default `[0, 0, 0]`), or the key
        /// `block_pos_id` (default key `target_block_pos`). The blackboard entry may be a
        /// position of numbers or a block position.
        pos: Param<Vec3>,
        /// `distance_epsilon` (default 0), or the key `distance_epsilon_id` (default key
        /// `target_distance_epsilon`).
        distance_epsilon: Param<f32>,
    },
    /// `minecraft:place_block`: no parameters.
    PlaceBlock {},
    /// `minecraft:shoot_bow`.
    ShootBow {
        /// `num_of_shots` (default 0), or the key `num_of_shots_id` (default key
        /// `target_num_of_shots`).
        shots: Param<i32>,
    },
    /// `minecraft:use_entity`: no parameters.
    UseEntity {},
    /// `minecraft:wait`: waits whole seconds.
    Wait {
        /// `wait_duration` as an integer, in seconds (default 0), or the same member as a
        /// string, which is then the key (default key `duration`).
        seconds: Param<i32>,
    },
    /// `minecraft:wait_ticks`.
    WaitTicks {
        /// `num_of_ticks` (default 0), or the key `num_of_ticks_id` (default key
        /// `target_num_of_ticks`).
        ticks: Param<i32>,
    },
}

impl Node {
    /// The nodes directly below this one, in order: a composite's children, a decorator's
    /// child; none for any other node.
    pub fn nodes(&self) -> &[Node] {
        match self {
            Self::Sequence { children, .. } | Self::Selector { children, .. } => {
                children.as_deref().unwrap_or_default()
            }
            Self::RepeatUntilSuccess { child, .. }
            | Self::RepeatUntilFailure { child, .. }
            | Self::Inverter { child, .. } => child.as_deref().map_or(&[], std::slice::from_ref),
            _ => &[],
        }
    }
}

impl Node {
    /// The nodes directly below this one, moved out of it: the node is left without them.
    fn take_nodes(&mut self) -> Vec<Node> {
        match self {
            Self::Sequence { children, .. } | Self::Selector { children, .. } => {
                children.take().unwrap_or_default()
            }
            Self::RepeatUntilSuccess { child, .. }
            | Self::RepeatUntilFailure { child, .. }
            | Self::Inverter { child, .. } => {
                child.take().map(|node| vec![*node]).unwrap_or_default()
            }
            _ => Vec::new(),
        }
    }
}

/// Drops the nodes below without a stack frame per level: a tree built in memory may nest
/// without bound, and the drop the compiler writes recurses through every `child`.
impl Drop for Node {
    fn drop(&mut self) {
        // Every node is taken apart on this one list; a node dropped at the end of a turn of
        // the loop has no node below it left, so its own drop goes no deeper.
        let mut pending = self.take_nodes();
        while let Some(mut node) = pending.pop() {
            pending.append(&mut node.take_nodes());
        }
    }
}

impl BehaviorTree {
    /// A tree named `name` with the root node `root` and no raw text.
    pub fn new(name: BehaviorTreeId, root: Node) -> Self {
        Self {
            name,
            root,
            raw: String::new(),
        }
    }
}

impl<T> Param<T> {
    /// A parameter given as a literal: the key is empty.
    pub fn literal(value: T) -> Self {
        Self {
            literal: value,
            key: String::new(),
        }
    }
}

impl<T: Default> Param<T> {
    /// A parameter read from the blackboard entry `key`; the literal is the type's default.
    ///
    /// It is written as the key member, without the literal. Two keys are not: the pair's
    /// default key is what an absent member reads as and writes nothing, and the key a read
    /// literal leaves (the empty string; [`Node::BLOCK_NAME_KEY`] for `find_block`'s block
    /// name, which is its default key too) stands for the literal, so `Param::key("")` is
    /// written as the literal member with the default literal: the only text that reads
    /// back to it.
    pub fn key(key: impl Into<String>) -> Self {
        Self {
            literal: T::default(),
            key: key.into(),
        }
    }
}

impl<T> Param<T> {
    /// A parameter read from the blackboard entry `key`, with the literal vanilla keeps beside
    /// it. For the two parameters whose literal is not the type's default when the literal
    /// member is absent: [`Node::FAILED_SEARCH_RADIUS`] and [`Node::INTERACT_DEFAULT_TICKS`].
    /// The literal is not written; a file that gives the key reads back to it.
    pub fn key_with(key: impl Into<String>, literal: T) -> Self {
        Self {
            literal,
            key: key.into(),
        }
    }
}

impl Node {
    /// The `search_radius` literal of `minecraft:find_block`, `minecraft:find_entity` and
    /// `minecraft:look_at_entity` when the literal member is absent or of the wrong type.
    pub const FAILED_SEARCH_RADIUS: i32 = 32;
    /// The `num_of_ticks` literal of `minecraft:interact` when the literal member is absent
    /// or of the wrong type.
    pub const INTERACT_DEFAULT_TICKS: i32 = 1;
    /// The key `minecraft:find_block`'s `block_name` has when its literal was read: build a
    /// block name given as a literal with this key, and it reads back equal.
    pub const BLOCK_NAME_KEY: &'static str = "block_name";
}

#[cfg(test)]
mod param_derive_tests {
    use super::Param;
    use std::collections::HashSet;

    #[test]
    fn a_param_of_an_integer_is_eq_and_hash() {
        let a = Param {
            literal: 1_i32,
            key: String::new(),
        };
        let mut set = HashSet::new();
        set.insert(a.clone());
        assert!(set.contains(&a));
        assert_eq!(a, a.clone());
    }
}

/// The directory behavior-tree files live in.
const DIRECTORY: &str = "behavior_trees";

impl bedrock_addon_core::__private::Sealed for BehaviorTree {}

impl Content for BehaviorTree {
    const NAME: &'static str = "behavior tree";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];
    /// The writer writes every member it knows from the value, so a member of the same key in
    /// the table is never written over it.
    const UNKNOWN_PRECEDENCE: bedrock_addon_core::json::Precedence =
        bedrock_addon_core::json::Precedence::KnownWins;

    /// Parses one behavior-tree file; the rules between files (one tree per name, the
    /// built-in trees) apply to a stack, not to a single file.
    fn parse(path: &PackPath, bytes: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        parse_file(PackRef::app_data(), PackSide::Behavior, path, bytes).1
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        self.write_into(out, style)
    }

    /// The writer's own merge of the unknown members (`Self::UNKNOWN_PRECEDENCE`: one named like a
    /// member the writer writes is left out), kept for its depth check: the members' nesting
    /// counts against the reader's limit.
    fn write_json_with(
        &self,
        unknown: &Unknown,
        out: &mut Vec<u8>,
        style: WriteStyle,
    ) -> Result<Vec<Diagnostic>, EncodeError> {
        self.write_into_with(unknown, out, style)
    }
}

impl JsonContent for BehaviorTree {}

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
#[doc(hidden)]
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
    let origin = Origin::file(pack, side, path.clone());
    // A number no double holds is read as the largest finite one, with a warning.
    let finite = bedrock_addon_core::document::FiniteText::of(bytes);
    let parsed = parse_document(finite.bytes());
    // Every finding is about this file of this pack.
    let mut diagnostics = parsed.diagnostics.located(&origin);
    finite.settle(&mut diagnostics, Some(&origin));
    let result = match parsed.tree {
        // The reader gives a tree only when it raised no error.
        // The family has no `format_version`: nothing was written, and the tree is read as
        // the engine reads it.
        Some(tree) => Ok(Document::new(
            tree,
            origin,
            unversioned::version_at(ENGINE_VERSION),
            diagnostics,
        )),
        None => Err(Rejected::new(origin, diagnostics)),
    };
    (parsed.name, result)
}

/// `behavior_trees/**`, packs from the lowest priority to the highest, as vanilla walks the
/// stack.
// `AutomationBehaviorTreeGroup::_loadTrees`: `getFullStack()` from the last
// entry to the first; `forEachIn("behavior_trees", …)` reads sub-folders and keeps every path.
const WALK: Walk = Walk::json_tree_in(&[DIRECTORY]);

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

/// A combine-time information about the file at `path` of `pack`.
fn notice(code: DiagnosticCode, pack: &PackRef, path: &PackPath) -> Diagnostic {
    let file = Origin::file(pack.clone(), PackSide::Behavior, path.clone());
    Diagnostic::info(code).phase(Phase::Validate).in_file(&file)
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
                notice(code, &earlier.pack, &earlier.path)
            } else {
                let code = DiagnosticCode::ShadowedDefinition { text: None };
                notice(code, &earlier.pack, &earlier.path)
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
            notice(code, &claim.pack, &claim.path)
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
    for (pack, ignored) in
        bedrock_addon_core::load::unread_files(stack, DIRECTORY, PackPath::is_json)
    {
        builder.push_diagnostic(notice(
            DiagnosticCode::IgnoredFile { text: None },
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
/// ones, by name. The family's own: a tree file is unversioned (no `JsonDocument`), the walk
/// keeps a nameless file apart (`FileOutcome`) and the combine adds the built-in trees and the
/// last-file-claims-the-name rule, which `load::registry` has no shape for.
#[doc(hidden)]
pub const LOADER: CollectionLoader<Registry<BehaviorTree>, FileOutcome> = CollectionLoader {
    walk: WALK,
    parse: parse_walk_file,
    combine,
    needs: &[],
    then: None,
};
