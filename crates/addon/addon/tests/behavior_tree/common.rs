//! Fixture loading and the two golden files of a fixture: `expected.json` (the registry after
//! loading: name → parsed tree and raw text, the nine built-in trees included) and
//! `expected.toml` (the findings: kind, severity, pack, file, path and, for a text of
//! vanilla's, the message).
//!
//! A fixture is `<name>/pack/behavior_trees/…` (one pack) or
//! `<name>/stack/<index>_<pack>/behavior_trees/…` (index 0 is the highest priority); a fixture
//! without either is the empty stack. The `expected.json` files were produced from the
//! family's tables by a second implementation written before the reader; the tests compare
//! the loaded registry against them.

#![allow(dead_code, reason = "each test module uses part of the helpers")]

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{Diagnostic, Diagnostics, JsonPath, Phase, Severity};
use bedrock_addon::document::{Content, Document, Registry, Rejected, Unknown};
use bedrock_addon::families::behavior_tree::{BehaviorTree, Node, Param};
use bedrock_addon::id::BehaviorTreeId;
use bedrock_addon::json::Value;
use bedrock_addon::load::{LoadContext, LoadOptions};
use bedrock_addon::pack::{Behavior, OpenOptions, Pack, PackOrigin, PackStack};
use bedrock_addon::storage::PackPath;
use bytes::Bytes;
use facet_value::{VArray, VObject};
use uuid::Uuid;

/// The fixture directory of the family.
pub fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/behavior_tree/behavior_tree")
}

/// The text of fixture file `path` (relative to the family's fixture directory).
pub fn fixture_text(path: &str) -> String {
    let bytes = std::fs::read(fixtures().join(path)).unwrap_or_else(|e| panic!("{path}: {e}"));
    String::from_utf8(bytes).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// Every file below `root`, as pack-relative paths, sorted.
pub fn files_below(root: &Path) -> Vec<(String, Vec<u8>)> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(root, &path, out);
            } else {
                let relative = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                out.push((relative, std::fs::read(&path).unwrap()));
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out.sort();
    out
}

/// A behaviour pack `n`, opened from memory, holding `files` (pack-relative paths).
pub fn pack(n: u128, files: Vec<(String, Vec<u8>)>) -> Pack {
    let uuid = Uuid::from_u128(n);
    let module = Uuid::from_u128(n ^ (0xabcd_u128 << 64));
    let manifest = format!(
        r#"{{"format_version": 2, "header": {{"name": "bp {n}", "description": "", "uuid": "{uuid}", "version": [1, 0, 0], "min_engine_version": [1, 21, 0]}}, "modules": [{{"type": "data", "uuid": "{module}", "version": [1, 0, 0]}}]}}"#
    );
    let mut all: Vec<(PackPath, Bytes)> = vec![(
        PackPath::from_static("manifest.json"),
        Bytes::from(manifest),
    )];
    all.extend(
        files
            .into_iter()
            .map(|(path, bytes)| (PackPath::new(&path).unwrap(), Bytes::from(bytes))),
    );
    let options = OpenOptions {
        origin: PackOrigin::Unknown,
        ..OpenOptions::default()
    };
    Pack::from_files(all, &options).expect("the pack opens")
}

/// A pack holding the files `(path below behavior_trees/, text)`.
pub fn tree_pack(n: u128, files: &[(&str, &str)]) -> Pack {
    pack(
        n,
        files
            .iter()
            .map(|(path, text)| (format!("behavior_trees/{path}"), text.as_bytes().to_vec()))
            .collect(),
    )
}

/// The packs of fixture `name`, top first.
pub fn fixture_packs(name: &str) -> Vec<Pack> {
    let fixture = fixtures().join(name);
    let single = fixture.join("pack");
    if single.is_dir() {
        return vec![pack(1, files_below(&single))];
    }
    let stack = fixture.join("stack");
    if !stack.is_dir() {
        return Vec::new();
    }
    let mut roots: Vec<PathBuf> = std::fs::read_dir(&stack)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    roots.sort();
    roots
        .iter()
        .enumerate()
        .map(|(index, root)| {
            let folder = root.file_name().unwrap().to_string_lossy().into_owned();
            assert!(
                folder.starts_with(&format!("{index}_")),
                "{folder}: stack folders are <index>_<pack>, index 0 the highest priority"
            );
            pack(index as u128 + 1, files_below(root))
        })
        .collect()
}

/// A behaviour stack of `packs`, top first.
pub fn stack_of(packs: Vec<Pack>) -> PackStack<Behavior> {
    PackStack::<Behavior>::of(packs, LoadOptions::default()).unwrap()
}

/// The behaviour stack of fixture `name`.
pub fn fixture_stack(name: &str) -> PackStack<Behavior> {
    stack_of(fixture_packs(name))
}

/// A tree name.
pub fn id(text: &str) -> BehaviorTreeId {
    BehaviorTreeId::parse(text)
}

/// Parses `text` as `behavior_trees/test.json`, a file outside any stack.
pub fn parse(text: &str) -> Result<Document<BehaviorTree>, Rejected> {
    let path = PackPath::from_static("behavior_trees/test.json");
    BehaviorTree::parse(&path, text.as_bytes(), &LoadContext::detached())
}

/// `text`, which must load.
pub fn loaded(text: &str) -> Document<BehaviorTree> {
    parse(text).unwrap_or_else(|rejected| panic!("rejected:\n{:#?}", rejected.diagnostics))
}

/// `text`, which must be rejected.
pub fn rejected(text: &str) -> Rejected {
    match parse(text) {
        Ok(document) => panic!(
            "loaded:\n{:#?}\n{:#?}",
            document.value(),
            document.diagnostics()
        ),
        Err(rejected) => rejected,
    }
}

/// A file naming the tree `name` with the root node `root` (JSON text).
pub fn tree_text(name: &str, root: &str) -> String {
    format!(
        r#"{{"minecraft:behavior_tree": {{"minecraft:name": "{name}", "minecraft:root": {root}}}}}"#
    )
}

/// The root node of the tree `{"<node type>": <body>}`, which must load.
pub fn root_of(node: &str) -> (Node, Diagnostics) {
    let document = loaded(&tree_text("t", node));
    (document.root.clone(), document.diagnostics().clone())
}

// ---- findings --------------------------------------------------------------------------------

/// The JSON path of the root node's body members: `…/minecraft:root/<type>`.
pub const ROOT: &str = "minecraft:behavior_tree/minecraft:root";

/// The findings with code `kind` at JSON path `path`.
pub fn at<'d>(diagnostics: &'d Diagnostics, kind: &str, path: &str) -> Vec<&'d Diagnostic> {
    diagnostics
        .iter()
        .filter(|d| d.code.name() == kind && d.location.json_path.to_string() == path)
        .collect()
}

/// Exactly one finding with code `kind` at `path`, at `severity`; returns it.
pub fn one<'d>(
    diagnostics: &'d Diagnostics,
    kind: &str,
    severity: Severity,
    path: &str,
) -> &'d Diagnostic {
    let found = at(diagnostics, kind, path);
    assert_eq!(
        found.len(),
        1,
        "expected one {kind} at `{path}`, found {}:\n{diagnostics:#?}",
        found.len()
    );
    assert_eq!(found[0].severity, severity, "{:#?}", found[0]);
    found[0]
}

/// No finding at all.
pub fn silent(diagnostics: &Diagnostics) {
    assert!(
        diagnostics.is_empty(),
        "unexpected findings:\n{diagnostics:#?}"
    );
}

/// The codes of `diagnostics`, in order.
pub fn kinds(diagnostics: &Diagnostics) -> Vec<&'static str> {
    diagnostics.iter().map(|d| d.code.name()).collect()
}

// ---- expected.json ---------------------------------------------------------------------------

fn param<T>(param: &Param<T>, literal: impl Fn(&T) -> Value) -> Value {
    let mut out = VObject::new();
    out.insert("literal", literal(&param.literal));
    out.insert("key", param.key.as_str());
    out.into_value()
}

#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "passed as the `Fn(&T)` of `param`"
)]
fn int(value: &i32) -> Value {
    Value::from(i64::from(*value))
}

#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "passed as the `Fn(&T)` of `param`"
)]
fn float(value: &f32) -> Value {
    Value::from(f64::from(*value))
}

#[allow(clippy::ptr_arg, reason = "the literal of a `Param<String>`")]
fn text(value: &String) -> Value {
    Value::from(value.as_str())
}

#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "passed as the `Fn(&T)` of `param`"
)]
fn boolean(value: &bool) -> Value {
    Value::from(*value)
}

fn block_pos(value: &[i32; 3]) -> Value {
    VArray::from(value.iter().map(int).collect::<Vec<_>>()).into()
}

fn vec3(value: &[f32; 3]) -> Value {
    VArray::from(value.iter().map(float).collect::<Vec<_>>()).into()
}

/// The unknown members the document keeps for the object at `at`, when it keeps any.
fn extra_value(unknown: &Unknown, at: &JsonPath) -> Option<Value> {
    let mut out = VObject::new();
    for (key, value) in unknown.members_at(at) {
        out.insert(key, value.clone());
    }
    (!out.is_empty()).then(|| out.into_value())
}

fn optional_node(node: Option<&Node>, unknown: &Unknown, at: &JsonPath) -> Value {
    node.map_or(Value::NULL, |node| dump_node_in(node, unknown, at))
}

/// [`dump_node_in`] of a node outside a document: it has no unknown members.
pub fn dump_node(node: &Node) -> Value {
    dump_node_in(node, &Unknown::new(), &JsonPath::new())
}

/// A node as `expected.json` writes it: `type`, then the variant's fields by their Rust names,
/// a parameter as `{"literal": …, "key": …}`, and `extra` when the node has unknown members
/// (those `unknown` keeps for its body; `at` is the node's object).
#[allow(clippy::too_many_lines, reason = "one arm per node type")]
pub fn dump_node_in(node: &Node, unknown: &Unknown, at: &JsonPath) -> Value {
    let body = at.child(node.type_name());
    let mut out = VObject::new();
    let mut put = |key: &str, value: Value| {
        out.insert(key, value);
    };
    match node {
        Node::Sequence { children } | Node::Selector { children } => {
            put(
                "type",
                Value::from(if matches!(node, Node::Sequence { .. }) {
                    "minecraft:sequence"
                } else {
                    "minecraft:selector"
                }),
            );
            put(
                "children",
                children.as_ref().map_or(Value::NULL, |children| {
                    VArray::from(
                        children
                            .iter()
                            .enumerate()
                            .map(|(index, node)| {
                                dump_node_in(node, unknown, &body.child("children").index(index))
                            })
                            .collect::<Vec<_>>(),
                    )
                    .into()
                }),
            );
        }
        Node::RepeatUntilSuccess {
            child,
            max_attempts,
        } => {
            put("type", "minecraft:repeat_until_success".into());
            put("max_attempts", param(max_attempts, int));
            put(
                "child",
                optional_node(child.as_deref(), unknown, &body.child("child")),
            );
        }
        Node::RepeatUntilFailure { child } => {
            put("type", "minecraft:repeat_until_failure".into());
            put(
                "child",
                optional_node(child.as_deref(), unknown, &body.child("child")),
            );
        }
        Node::Inverter { child } => {
            put("type", "minecraft:inverter".into());
            put(
                "child",
                optional_node(child.as_deref(), unknown, &body.child("child")),
            );
        }
        Node::Subtree {
            subtree_id,
            data_items,
        } => {
            put("type", "minecraft:subtree".into());
            put("subtree_id", subtree_id.as_str().into());
            let items: Vec<Value> = data_items
                .iter()
                .map(|(from, to)| {
                    VArray::from(vec![Value::from(from.as_str()), Value::from(to.as_str())]).into()
                })
                .collect();
            put("data_items", VArray::from(items).into());
        }
        Node::ActivateTool { block_pos: pos } => {
            put("type", "minecraft:activate_tool".into());
            put("block_pos", param(pos, block_pos));
        }
        Node::Attack { ticks } => {
            put("type", "minecraft:attack".into());
            put("ticks", param(ticks, int));
        }
        Node::BreakBlock {
            block_pos: pos,
            ticks,
        } => {
            put("type", "minecraft:break_block".into());
            put("block_pos", param(pos, block_pos));
            put("ticks", param(ticks, int));
        }
        Node::ConsumeItem {} => {
            put("type", "minecraft:consume_item".into());
        }
        Node::FindBlock {
            block_name,
            search_radius,
        } => {
            put("type", "minecraft:find_block".into());
            put("block_name", param(block_name, text));
            put("search_radius", param(search_radius, int));
        }
        Node::FindEntity {
            entity_name,
            search_radius,
        } => {
            put("type", "minecraft:find_entity".into());
            put("entity_name", param(entity_name, text));
            put("search_radius", param(search_radius, int));
        }
        Node::Fly {
            should_be_flying,
            throw_if_no_change,
        } => {
            put("type", "minecraft:fly".into());
            put("should_be_flying", param(should_be_flying, boolean));
            put("throw_if_no_change", param(throw_if_no_change, boolean));
        }
        Node::GetInteractionPositionForBlock {
            target,
            anchor,
            facing,
            max_search_distance,
        } => {
            put(
                "type",
                "minecraft:get_interaction_position_for_block".into(),
            );
            put("target", param(target, block_pos));
            put("anchor", param(anchor, block_pos));
            put("facing", param(facing, text));
            put("max_search_distance", param(max_search_distance, int));
        }
        Node::Interact { ticks } => {
            put("type", "minecraft:interact".into());
            put("ticks", param(ticks, int));
        }
        Node::LookAt { block_pos: pos } => {
            put("type", "minecraft:look_at".into());
            put("block_pos", param(pos, block_pos));
        }
        Node::LookAtEntity {
            entity_name,
            search_radius,
        } => {
            put("type", "minecraft:look_at_entity".into());
            put("entity_name", param(entity_name, text));
            put("search_radius", param(search_radius, int));
        }
        Node::Move {
            direction,
            second_direction,
            ticks,
        } => {
            put("type", "minecraft:move".into());
            put("direction", param(direction, text));
            put("second_direction", param(second_direction, text));
            put("ticks", param(ticks, int));
        }
        Node::MoveTo {
            pos,
            distance_epsilon,
        } => {
            put("type", "minecraft:move_to".into());
            put("pos", param(pos, vec3));
            put("distance_epsilon", param(distance_epsilon, float));
        }
        Node::PlaceBlock {} => {
            put("type", "minecraft:place_block".into());
        }
        Node::ShootBow { shots } => {
            put("type", "minecraft:shoot_bow".into());
            put("shots", param(shots, int));
        }
        Node::UseEntity {} => {
            put("type", "minecraft:use_entity".into());
        }
        Node::Wait { seconds } => {
            put("type", "minecraft:wait".into());
            put("seconds", param(seconds, int));
        }
        Node::WaitTicks { ticks } => {
            put("type", "minecraft:wait_ticks".into());
            put("ticks", param(ticks, int));
        }
        other => panic!("a node type the dump does not know: {other:?}"),
    }
    if let Some(extra) = extra_value(unknown, &body) {
        put("extra", extra);
    }
    out.into_value()
}

/// A tree as `expected.json` writes it: `root`, `raw`, and `extra` / `root_extra` when it has
/// unknown members.
pub fn dump_tree(document: &Document<BehaviorTree>) -> Value {
    let unknown = document.unknown();
    let payload = JsonPath::of_keys(["minecraft:behavior_tree"]);
    let mut out = VObject::new();
    out.insert(
        "root",
        dump_node_in(&document.root, unknown, &payload.child("minecraft:root")),
    );
    out.insert("raw", document.raw.as_str());
    if let Some(extra) = extra_value(unknown, &payload) {
        out.insert("extra", extra);
    }
    if let Some(root_extra) = extra_value(unknown, &JsonPath::new()) {
        out.insert("root_extra", root_extra);
    }
    out.into_value()
}

/// `value` as text with object members sorted and every number as a float, so two values
/// compare by content.
pub fn canon(value: &Value) -> String {
    fn write(value: &Value, indent: usize, out: &mut String) {
        let pad = "  ".repeat(indent);
        if value.is_null() {
            out.push_str("null");
        } else if let Some(flag) = value.as_bool() {
            out.push_str(&flag.to_string());
        } else if let Some(number) = value.as_number() {
            write!(out, "{:?}", number.to_f64_lossy()).unwrap();
        } else if let Some(text) = value.as_string() {
            write!(out, "{:?}", text.as_str()).unwrap();
        } else if let Some(array) = value.as_array() {
            out.push('[');
            for element in array {
                write!(out, "\n{pad}  ").unwrap();
                write(element, indent + 1, out);
                out.push(',');
            }
            write!(out, "\n{pad}]").unwrap();
        } else if let Some(object) = value.as_object() {
            let mut members: Vec<(&str, &Value)> = object
                .iter()
                .map(|(key, value)| (key.as_str(), value))
                .collect();
            members.sort_by_key(|(key, _)| *key);
            out.push('{');
            for (key, member) in members {
                write!(out, "\n{pad}  {key:?}: ").unwrap();
                write(member, indent + 1, out);
                out.push(',');
            }
            write!(out, "\n{pad}}}").unwrap();
        } else {
            panic!("not a JSON value: {value:?}");
        }
    }
    let mut out = String::new();
    write(value, 0, &mut out);
    out
}

/// The registry as `expected.json` writes it: every effective tree by name.
pub fn dump_registry(registry: &Registry<BehaviorTree>) -> Value {
    let mut out = VObject::new();
    for (name, document) in registry {
        assert_eq!(
            name, &document.name,
            "a tree is registered under its minecraft:name"
        );
        out.insert(name.as_str(), dump_tree(document));
    }
    out.into_value()
}

/// The registry equals fixture `name`'s `expected.json`.
pub fn assert_registry(name: &str, registry: &Registry<BehaviorTree>) {
    let expected: Value = facet_json::from_str(&fixture_text(&format!("{name}/expected.json")))
        .unwrap_or_else(|e| panic!("{name}/expected.json: {e}"));
    let entries = expected.as_object().expect("an object by tree name").len();
    assert_eq!(
        registry.len(),
        entries,
        "{name}: the registry has {} trees, expected.json {entries}: {:?}",
        registry.len(),
        registry
            .iter()
            .map(|(id, _)| id.as_str())
            .collect::<Vec<_>>()
    );
    let (found, wanted) = (canon(&dump_registry(registry)), canon(&expected));
    assert!(
        found == wanted,
        "{name}: the registry differs from expected.json\n--- loaded ---\n{found}\n--- expected ---\n{wanted}"
    );
}

// ---- expected.toml ---------------------------------------------------------------------------

/// One `[[diagnostic]]` of an `expected.toml`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Finding {
    /// The file, pack-relative.
    pub file: String,
    /// The JSON path, segments joined with `/`.
    pub path: String,
    /// The code's name.
    pub kind: String,
    /// `info`, `warning` or `error`.
    pub severity: String,
    /// The stack index of the pack (0 when the table has no `pack`).
    pub pack: usize,
    /// The message, compared only where the table gives one (a text of vanilla's).
    pub message: Option<String>,
    /// `link` for a finding of the link pass; a loading finding has no `phase`.
    pub link: bool,
}

impl Finding {
    /// What two findings are ordered and compared by; the message only when `with_message`.
    fn key(&self, with_message: bool) -> (&str, &str, &str, &str, usize, Option<&str>) {
        (
            &self.file,
            &self.path,
            &self.kind,
            &self.severity,
            self.pack,
            self.message.as_deref().filter(|_| with_message),
        )
    }
}

/// `found` equals `wanted`, in any order; a message is compared where `wanted` gives one.
pub fn same_findings(found: &mut [Finding], wanted: &mut [Finding]) -> bool {
    found.sort_by(|a, b| a.key(false).cmp(&b.key(false)));
    wanted.sort_by(|a, b| a.key(false).cmp(&b.key(false)));
    found.len() == wanted.len()
        && found.iter().zip(wanted.iter()).all(|(found, wanted)| {
            let with_message = wanted.message.is_some();
            found.key(with_message) == wanted.key(with_message)
        })
}

/// The `[[diagnostic]]` tables of an `expected.toml`: `key = "string"` and `pack = <index>`
/// lines, `#` comments.
pub fn read_findings(toml: &str) -> Vec<Finding> {
    let mut out: Vec<Finding> = Vec::new();
    for line in toml.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[[diagnostic]]" {
            out.push(Finding::default());
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .unwrap_or_else(|| panic!("not `key = value`: {line}"));
        let (key, value) = (key.trim(), value.trim());
        let finding = out
            .last_mut()
            .unwrap_or_else(|| panic!("`{line}` before the first [[diagnostic]]"));
        // A TOML basic string has JSON's escapes.
        let string = || -> String {
            facet_json::from_str(value).unwrap_or_else(|e| panic!("`{line}`: {e}"))
        };
        match key {
            "kind" => finding.kind = string(),
            "severity" => finding.severity = string(),
            "file" => finding.file = string(),
            "path" => finding.path = string(),
            "message" => finding.message = Some(string()),
            "phase" => {
                assert_eq!(string(), "link", "`{line}`: the only phase written is link");
                finding.link = true;
            }
            "pack" => finding.pack = value.parse().unwrap_or_else(|e| panic!("`{line}`: {e}")),
            other => panic!("unknown key `{other}` in `{line}`"),
        }
    }
    out
}

/// A finding of the registry in `expected.toml`'s terms.
pub fn finding_of(diagnostic: &Diagnostic) -> Finding {
    Finding {
        file: diagnostic
            .location
            .path
            .as_ref()
            .map(|path| path.as_str().to_owned())
            .unwrap_or_default(),
        path: diagnostic.location.json_path.to_string(),
        kind: diagnostic.code.name().to_owned(),
        severity: diagnostic.severity.to_string(),
        pack: diagnostic
            .location
            .pack
            .as_ref()
            .and_then(bedrock_addon::pack::PackRef::index)
            .unwrap_or(usize::MAX),
        message: Some(diagnostic.code.to_string()),
        link: diagnostic.phase == Phase::Link,
    }
}

/// The findings of fixture `name`'s `expected.toml`: the link pass's, or the loading ones.
pub fn expected_findings(name: &str, link: bool) -> Vec<Finding> {
    read_findings(&fixture_text(&format!("{name}/expected.toml")))
        .into_iter()
        .filter(|finding| finding.link == link)
        .collect()
}

/// The registry's findings equal the loading findings of fixture `name`'s `expected.toml`, in
/// any order.
pub fn assert_findings(name: &str, registry: &Registry<BehaviorTree>) {
    let mut wanted = expected_findings(name, false);
    let mut found: Vec<Finding> = registry.diagnostics().map(finding_of).collect();
    assert!(
        same_findings(&mut found, &mut wanted),
        "{name}: the findings differ from expected.toml\n--- loaded ---\n{found:#?}\n--- expected ---\n{wanted:#?}"
    );
}

/// Loads fixture `name` and requires its registry and findings to equal `expected.json` and
/// `expected.toml`; returns the stack for the row's own assertions.
pub fn check(name: &str) -> PackStack<Behavior> {
    let stack = fixture_stack(name);
    let registry = stack.behavior_trees();
    assert_registry(name, registry);
    assert_findings(name, registry);
    stack
}
