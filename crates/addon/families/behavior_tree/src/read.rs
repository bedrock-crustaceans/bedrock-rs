//! Reading a behavior-tree file the way vanilla's loader does: `AutomationBehaviorTreeGroup`
//! and one hand-written `load` per node type over a `Json::Value`. The walk is over an untyped
//! `Value` because vanilla's rules are not typed shapes: a member is looked up and, when it is
//! of the wrong type, ignored.

use facet_value::VObject;

use super::codes;
use super::{BehaviorTree, Node, Param};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    CodeDef, Diagnostic, DiagnosticCode, DiagnosticDetail, Diagnostics, JsonPath, Severity,
};
use bedrock_addon_core::document::Unknown;
use bedrock_addon_core::id::BehaviorTreeId;
use bedrock_addon_core::json::kit::{self, Disposition, Read};
use bedrock_addon_core::json::{PrepareError, Prepared, Value};
use bedrock_addon_core::versioning::{MemberFailure, Reading};

/// The only member of the document root vanilla reads.
pub(super) const PAYLOAD: &str = "minecraft:behavior_tree";
/// The tree's key.
pub(super) const NAME: &str = "minecraft:name";
/// The root node.
pub(super) const ROOT: &str = "minecraft:root";

/// What the loader read of one file.
pub(super) struct Parsed {
    /// The name the file gives its tree, when it has one.
    pub(super) name: Option<BehaviorTreeId>,
    /// The tree, when the file loads.
    pub(super) tree: Option<BehaviorTree>,
    /// The findings.
    pub(super) diagnostics: Diagnostics,
}

/// How the loader reads: vanilla's `Json::Value` walk takes a member of another JSON type as it
/// reads it and says nothing, so a wrong type is said as an information and the member takes
/// what vanilla gives it; an unknown member of a node body is kept and said as an
/// information too ([`Reader::new`]).
const READING: Reading = Reading::OWN.with_on_member_failure(MemberFailure::DefaultAndTell);

/// Reads one behavior-tree file.
pub(super) fn parse_document(bytes: &[u8]) -> Parsed {
    let mut diagnostics = Diagnostics::default();
    let mut reader = Reader::new(&mut diagnostics);
    let (name, tree) = reader.document(bytes);
    let unknown = reader.unknown;
    // The members the walk kept go on with the findings, into the document's table.
    diagnostics.unknown_mut().absorb(&JsonPath::new(), unknown);
    Parsed {
        name,
        tree,
        diagnostics,
    }
}

/// The type and body of a node: the one member of an object with exactly one member.
fn one_member(value: &Value) -> Option<(&str, &Value)> {
    let object = value.as_object().filter(|object| object.len() == 1)?;
    let (kind, body) = object.iter().next()?;
    Some((kind.as_str(), body))
}

/// Keeps the members of `object` (at `at`) other than `read` in `unknown`, in document order,
/// without a word.
fn keep_rest(unknown: &mut Unknown, object: &VObject, at: &JsonPath, read: &[&str]) {
    for (key, value) in object {
        if !read.contains(&key.as_str()) {
            unknown.put(at, key.as_str(), value.clone());
        }
    }
}

/// One file's walk: the kit's read, telling the findings, and the unknown members kept, by
/// the path of their object in the file as it is written back.
pub(super) struct Reader<'d> {
    read: Read<'d>,
    unknown: Unknown,
}

impl<'d> Reader<'d> {
    /// A walk telling into `findings`, unknown members as informations (vanilla reads no
    /// other key and says nothing).
    fn new(findings: &'d mut Diagnostics) -> Self {
        Self {
            read: Read::new(READING, findings).unknown_severity(Severity::Info),
            unknown: Unknown::new(),
        }
    }

    /// [`as_string`], with the kit's wrong-type information at `node` for a value that is
    /// neither `null` nor a string: a value of another JSON type than its member's is read
    /// as vanilla reads it (a bool as its text, a number as the empty text).
    pub(super) fn text<'v>(&mut self, node: Option<&kit::Node<'v>>, default: &'v str) -> &'v str {
        if let Some(node) = node.filter(|node| !node.value().is_null() && !node.value().is_string())
        {
            self.read.wrong_type(node);
        }
        as_string(node.map(kit::Node::value), default)
    }

    fn report(&mut self, at: &JsonPath, code: DiagnosticCode, severity: Severity) {
        self.read
            .push(Diagnostic::error(code).severity(severity).at(at.clone()));
    }

    /// A kept error at `at`: the game logs `code`'s line at level 3 and loads the tree.
    fn kept_error(&mut self, at: &JsonPath, code: DiagnosticCode) {
        self.read
            .push(Diagnostic::error(code).at(at.clone()).into_kept());
    }

    fn document(&mut self, bytes: &[u8]) -> (Option<BehaviorTreeId>, Option<BehaviorTree>) {
        // Ours: vanilla's reader checks neither the encoding nor the nesting (a tree nests two
        // levels per node: the limit bounds the walk below).
        let ours = |message: &'static str| DiagnosticCode::from(core_codes::SYNTAX.with([message]));
        let prepared = match Prepared::non_empty(bytes) {
            Ok(prepared) => prepared,
            Err(error) => {
                let (code, severity) = match error {
                    // Vanilla skips an empty text without a word: an information.
                    PrepareError::Empty => (
                        DiagnosticCode::from(&core_codes::EMPTY_DOCUMENT),
                        Severity::Info,
                    ),
                    PrepareError::NotUtf8(_) => (ours("the file is not UTF-8"), Severity::Error),
                    PrepareError::TooDeep(_) => {
                        (ours("the JSON nests too deeply"), Severity::Error)
                    }
                };
                self.report(&JsonPath::new(), code, severity);
                return (None, None);
            }
        };
        let text = prepared.as_str();
        // The tree keeps the text as written: the byte order mark is three bytes of valid
        // UTF-8, so the bytes are a string too.
        let raw = std::str::from_utf8(bytes).unwrap_or(text);
        // Of a member written twice the last stands.
        let cleaned = bedrock_addon_core::json::last_wins(text);
        let text: &str = &cleaned;
        // `Json::Reader::parse(…, collectComments = false)`: comments are accepted.
        let document = match bedrock_addon_core::json::from_str_jsonc::<Value>(text) {
            Ok(document) => document,
            Err(error) => {
                let message = error.to_string();
                // `parse error:\n%s` with the reader's messages (Error, `LogArea::Ai`); the
                // file is skipped.
                self.read.push(
                    Diagnostic::of(codes::PARSE_ERROR.with([message.clone()]))
                        .detail(DiagnosticDetail::Syntax(message))
                        .at(JsonPath::new()),
                );
                return (None, None);
            }
        };
        let payload = document
            .as_object()
            .and_then(|root| root.get(PAYLOAD))
            .and_then(Value::as_object);
        let name = payload
            .and_then(|payload| payload.get(NAME))
            .and_then(Value::as_string)
            .map(facet_value::VString::as_str)
            .filter(|name| !name.is_empty());
        let payload_at = JsonPath::of_keys([PAYLOAD]);
        let Some(name) = name else {
            // Null, not a string or empty, also when the root or the payload is not an
            // object: vanilla skips the file without a word.
            self.report(
                &payload_at.child(NAME),
                DiagnosticCode::from(&core_codes::MISSING_REQUIRED_FIELD),
                Severity::Error,
            );
            return (None, None);
        };
        let name = BehaviorTreeId::parse(name);
        let root_at = payload_at.child(ROOT);
        let Some((kind, body)) = payload
            .and_then(|payload| payload.get(ROOT))
            .and_then(one_member)
        else {
            self.report(
                &root_at,
                DiagnosticCode::from(&core_codes::INVALID_ROOT),
                Severity::Error,
            );
            return (Some(name), None);
        };
        let Some(root) = self.node(kind, body, &root_at, true) else {
            return (Some(name), None);
        };
        // Vanilla looks up these three keys and no other: the rest (a `format_version`, for one) is kept, unread.
        if let Some(payload) = payload {
            keep_rest(&mut self.unknown, payload, &payload_at, &[NAME, ROOT]);
        }
        if let Some(root) = document.as_object() {
            keep_rest(&mut self.unknown, root, &JsonPath::new(), &[PAYLOAD]);
        }
        let tree = BehaviorTree {
            name: name.clone(),
            root,
            raw: raw.to_owned(),
        };
        (Some(name), Some(tree))
    }

    /// The node of type `kind` with `body`, the one member of the object at `at`. An unknown
    /// type gives no node: an error that refuses the tree at its `root`, a kept error below it.
    // `BehaviorFactory::loadNodeDefinition`; each type's read is its row of `kinds::KINDS`.
    fn node(&mut self, kind: &str, body: &Value, at: &JsonPath, root: bool) -> Option<Node> {
        // A body that is not an object reads as an empty one: vanilla's `operator[]` turns it
        // into one before the lookup.
        let empty = Value::from(VObject::new());
        let at = &at.child(kind);
        if !body.is_object() && !body.is_null() {
            // Read as an empty body, and said.
            self.read.wrong_type(&kit::Node::new(body, at.clone()));
        }
        let object = if body.is_object() { body } else { &empty };
        let m = &mut Body {
            members: self.read.object(&kit::Node::new(object, at.clone()))?,
            read: Vec::new(),
        };
        let Some(row) = super::kinds::find(kind) else {
            // `Unknown behavior %s\n` (Error, `LogArea::Entity`); the
            // node is dropped. `BehaviorFactory::loadNodeDefinition` logs it at
            // level 3, area 13: at the root no tree is made; a child is left out and the
            // tree loads, a kept error.
            let code = DiagnosticCode::from(core_codes::UNKNOWN_NODE.with([kind.to_owned()]));
            if root {
                self.report(at, code, Severity::Error);
            } else {
                self.kept_error(at, code);
            }
            return None;
        };
        let node = (row.read)(self, m, at);
        self.rest(m);
        Some(node)
    }

    /// A literal and its key member. The literal is read when it is present and of its type;
    /// otherwise it is `failed` and the key is the key member's text or the default key.
    // The load rule of every leaf:
    // `parseValue<T>(body[literal], member, log = false)`, and only when it fails
    // `body[id_key].asString(default_key)`.
    pub(super) fn param<T: Literal>(
        &mut self,
        body: &mut Body<'_>,
        pair: &Pair,
        failed: T,
    ) -> Param<T> {
        let written = body
            .take(pair.literal)
            .filter(|node| !node.value().is_null());
        let id = body.take(pair.id);
        if let Some(node) = &written {
            if let Some(literal) = T::read(self, node) {
                return Param {
                    literal,
                    key: pair.preset.to_owned(),
                };
            }
            // Vanilla ignores a literal of the wrong type without a word.
            self.report(
                node.path(),
                DiagnosticCode::from(&core_codes::LITERAL_IGNORED),
                Severity::Info,
            );
        }
        Param {
            literal: failed,
            key: self.text(id.as_ref(), pair.default_key).to_owned(),
        }
    }

    /// `wait_duration`: an integer is the literal, and the same member read as text is the
    /// key (default `duration`).
    // `WaitDefinition::load` reads the one key twice.
    pub(super) fn wait_duration(&mut self, body: &mut Body<'_>) -> Param<i32> {
        let value = body.take(WAIT_DURATION);
        let written = value.as_ref().filter(|node| !node.value().is_null());
        if let Some(node) = written {
            if let Some(literal) = integral(node.value()) {
                if node.value().is_bool() {
                    self.read.wrong_type(node);
                }
                return Param {
                    literal,
                    key: String::new(),
                };
            }
            // A string is the documented key form; any other type is an ignored literal.
            if !node.value().is_string() {
                self.report(
                    node.path(),
                    DiagnosticCode::from(&core_codes::LITERAL_IGNORED),
                    Severity::Info,
                );
            }
        }
        Param {
            literal: 0,
            key: as_string(value.as_ref().map(kit::Node::value), WAIT_DEFAULT_KEY).to_owned(),
        }
    }

    /// The two flags of `minecraft:fly`. Vanilla reads both literals into the one member
    /// `mShouldBeFlying` and the node uses it for both flags, so both literals here are that
    /// value: the throw literal if it was read, else the flying literal, else `false`.
    // `FlyDefinition::load`; `FlyNode::initializeFromDefinition` falls back to +0x30 for both
    // flags.
    pub(super) fn fly(&mut self, body: &mut Body<'_>, at: &JsonPath) -> (Param<bool>, Param<bool>) {
        let flying = self.read_literal::<bool>(body, &SHOULD_BE_FLYING);
        let throw = self.read_literal::<bool>(body, &SHOULD_THROW);
        if flying.0.is_some() && throw.0.is_some() {
            // The flying literal is overwritten by the throw literal.
            self.report(
                &at.child(SHOULD_BE_FLYING.literal),
                DiagnosticCode::from(&core_codes::LITERAL_IGNORED),
                Severity::Info,
            );
        }
        let literal = throw.0.or(flying.0).unwrap_or(false);
        let param = |(read, key): (Option<bool>, String)| Param {
            literal,
            key: if read.is_some() { String::new() } else { key },
        };
        (param(flying), param(throw))
    }

    /// One literal of `pair` as [`param`](Self::param) reads it: the literal when it was read,
    /// and the key the pair has when it was not.
    fn read_literal<T: Literal>(
        &mut self,
        body: &mut Body<'_>,
        pair: &Pair,
    ) -> (Option<T>, String) {
        let written = body
            .take(pair.literal)
            .filter(|node| !node.value().is_null());
        let id = body.take(pair.id);
        let literal = written.as_ref().and_then(|node| T::read(self, node));
        if let Some(node) = written.as_ref().filter(|_| literal.is_none()) {
            self.report(
                node.path(),
                DiagnosticCode::from(&core_codes::LITERAL_IGNORED),
                Severity::Info,
            );
        }
        let key = self.text(id.as_ref(), pair.default_key).to_owned();
        (literal, key)
    }

    /// `children` of a composite: the nodes of an array, in order; `None` when the member is
    /// absent or not an array. An element that is not a one-member object, or whose type is
    /// unknown, is left out.
    // `CompositeDefinition::_compositeLoadChildrenBehaviors`: all silent
    // but the unknown type, so the shape findings are informations.
    pub(super) fn children(&mut self, body: &mut Body<'_>, at: &JsonPath) -> Option<Vec<Node>> {
        let elements = match body.take("children") {
            Some(node) if node.value().is_array() => self.read.array(&node)?,
            // Absent too: an information at the member.
            _ => {
                self.report(
                    &at.child("children"),
                    DiagnosticCode::from(&core_codes::INVALID_TYPE),
                    Severity::Info,
                );
                return None;
            }
        };
        let mut nodes = Vec::new();
        let mut kept = Vec::new();
        for (index, element) in elements.iter().enumerate() {
            let Some((kind, body)) = one_member(element.value()) else {
                self.report(
                    element.path(),
                    DiagnosticCode::from(&core_codes::INVALID_NODE),
                    Severity::Info,
                );
                continue;
            };
            if let Some(node) = self.node(kind, body, element.path(), false) {
                nodes.push(node);
                kept.push(index);
            }
        }
        if kept.len() < elements.len() {
            // The members kept below a child go with it to its place in the written array.
            self.unknown.left_out(elements.node().path(), &kept);
        }
        Some(nodes)
    }

    /// `child` of a decorator: one node; `None` when the member is absent, not a one-member
    /// object, or of an unknown type.
    // `DecoratorDefinition::_decoratorLoadChildBehavior`: silent but the
    // unknown type, as `children`.
    pub(super) fn child(&mut self, body: &mut Body<'_>, at: &JsonPath) -> Option<Box<Node>> {
        let at = at.child("child");
        let child = body.take("child");
        let Some((kind, body)) = child.as_ref().and_then(|node| one_member(node.value())) else {
            self.report(
                &at,
                DiagnosticCode::from(&core_codes::INVALID_NODE),
                Severity::Info,
            );
            return None;
        };
        self.node(kind, body, &at, false).map(Box::new)
    }

    /// `data_items` of a subtree: a string `s` is `(s, s)`, an object with both `from` and
    /// `to` is that pair; the member is one item or an array of items. Any other item is left
    /// out.
    // `SubtreeDefinition::load` and its `$_0`.
    pub(super) fn data_items(&mut self, body: &mut Body<'_>) -> Vec<(String, String)> {
        let Some(node) = body
            .take("data_items")
            .filter(|node| !node.value().is_null())
        else {
            return Vec::new();
        };
        let mut items = Vec::new();
        if !node.value().is_array() {
            self.data_item(&node, &mut items);
        } else if let Some(elements) = self.read.array(&node) {
            for element in elements.iter() {
                self.data_item(&element, &mut items);
            }
        }
        items
    }

    /// One item of `data_items`, onto `items`; an item that is neither a string nor an object
    /// with both `from` and `to` is left out in silence by vanilla: an information.
    fn data_item(&mut self, node: &kit::Node<'_>, items: &mut Vec<(String, String)>) {
        let value = node.value();
        if let Some(text) = value.as_string() {
            items.push((text.as_str().to_owned(), text.as_str().to_owned()));
            return;
        }
        let pair = value
            .as_object()
            .and_then(|object| Some((object.get("from")?, object.get("to")?)));
        match pair {
            Some((from, to)) => {
                let from = kit::Node::new(from, node.path().child("from"));
                let to = kit::Node::new(to, node.path().child("to"));
                items.push((
                    self.text(Some(&from), "").to_owned(),
                    self.text(Some(&to), "").to_owned(),
                ));
            }
            None => self.read.wrong_type(node),
        }
    }

    /// Keeps the members of a node body nobody read, each with an `UnrecognizedField`
    /// information (vanilla reads no other key and says nothing).
    fn rest(&mut self, body: &Body<'_>) {
        let at = body.members.node().path();
        for (key, value) in self
            .read
            .unknown(&body.members, &body.read, Disposition::Keep)
        {
            self.unknown.put(at, key, value.clone());
        }
    }
}

/// The members of a node body as the walker reads them: every key looked up is read, whether
/// or not the body has it.
pub(super) struct Body<'v> {
    members: kit::Members<'v>,
    read: Vec<&'static str>,
}

impl<'v> Body<'v> {
    /// Member `key`, now read (whether or not it is there).
    pub(super) fn take(&mut self, key: &'static str) -> Option<kit::Node<'v>> {
        self.read.push(key);
        self.members.get(key)
    }
}

/// A literal member, its key member, the default key, and the key the parameter has when the
/// literal was read.
pub(super) struct Pair {
    pub(super) literal: &'static str,
    pub(super) id: &'static str,
    pub(super) default_key: &'static str,
    pub(super) preset: &'static str,
}

const fn pair(literal: &'static str, id: &'static str, default_key: &'static str) -> Pair {
    Pair {
        literal,
        id,
        default_key,
        preset: "",
    }
}

pub(super) const MAX_ATTEMPTS: Pair = pair(
    "desired_max_number_of_attempts",
    "max_number_of_attempts_id",
    "max_number_of_attempts",
);
pub(super) const BLOCK_POS: Pair = pair("block_pos", "block_pos_id", "target_block_pos");
pub(super) const NUM_OF_TICKS: Pair =
    pair("num_of_ticks", "num_of_ticks_id", "target_num_of_ticks");
/// `find_block` sets its key to `block_name` before it reads the literal.
pub(super) const BLOCK_NAME: Pair = Pair {
    literal: "block_name",
    id: "block_name_id",
    default_key: Node::BLOCK_NAME_KEY,
    preset: Node::BLOCK_NAME_KEY,
};
pub(super) const SEARCH_RADIUS: Pair =
    pair("search_radius", "search_radius_id", "target_search_radius");
/// The key member of an entity name is `entity_type`.
pub(super) const ENTITY_NAME: Pair = pair("entity_name", "entity_type", "entity_name");
pub(super) const SHOULD_BE_FLYING: Pair = pair(
    "should_be_flying_bool",
    "should_be_flying_id",
    "should_be_flying",
);
pub(super) const SHOULD_THROW: Pair = pair(
    "should_throw_event_if_no_state_change_necessary_bool",
    "should_throw_event_if_no_state_change_necessary_id",
    "should_throw_event_if_no_state_change_necessary",
);
pub(super) const TARGET_BLOCK_POS: Pair = pair(
    "target_block_pos",
    "target_block_pos_id",
    "target_block_pos",
);
pub(super) const ANCHOR_BLOCK_POS: Pair = pair(
    "anchor_block_pos",
    "anchor_block_pos_id",
    "anchor_block_pos",
);
pub(super) const FACING_STRING: Pair = pair("facing_string", "facing_string_id", "facing_string");
pub(super) const MAX_SEARCH_DISTANCE: Pair = pair(
    "max_search_distance",
    "max_search_distance_id",
    "max_search_distance",
);
pub(super) const DIRECTION: Pair = pair("desired_direction", "direction_id", "direction");
pub(super) const SECOND_DIRECTION: Pair = pair(
    "desired_second_direction",
    "second_direction_id",
    "second_direction",
);
pub(super) const MAX_NUMBER_OF_TICKS: Pair = pair(
    "max_number_of_ticks",
    "number_of_ticks_id",
    "number_of_ticks",
);
pub(super) const DISTANCE_EPSILON: Pair = pair(
    "distance_epsilon",
    "distance_epsilon_id",
    "target_distance_epsilon",
);
pub(super) const NUM_OF_SHOTS: Pair =
    pair("num_of_shots", "num_of_shots_id", "target_num_of_shots");

/// The one member of `minecraft:wait`: the literal as an integer, the key as a string.
pub(super) const WAIT_DURATION: &str = "wait_duration";
/// The key of `minecraft:wait` when the member is absent.
pub(super) const WAIT_DEFAULT_KEY: &str = "duration";

/// The `search_radius` literal when its member is absent or of the wrong type.
pub(super) const FAILED_SEARCH_RADIUS: i32 = Node::FAILED_SEARCH_RADIUS;

/// `Json::Value::asString(default)`: the default for an absent or null value, a string
/// itself, a bool as `true` / `false`, and the empty string for anything else.
fn as_string<'v>(value: Option<&'v Value>, default: &'v str) -> &'v str {
    match value {
        None => default,
        Some(value) if value.is_null() => default,
        Some(value) => match (value.as_string(), value.as_bool()) {
            (Some(text), _) => text.as_str(),
            (None, Some(true)) => "true",
            (None, Some(false)) => "false",
            (None, None) => "",
        },
    }
}

/// An integer literal: Mojang's `isIntegral` is true for an integer and for a bool (1 / 0),
/// false for a real, so `1.0` is no integer.
// `isIntegral` tests the type mask 0x26 (int, uint, bool). An integer
// outside `i32` is not read.
fn integral(value: &Value) -> Option<i32> {
    if let Some(flag) = value.as_bool() {
        return Some(i32::from(flag));
    }
    let number = value.as_number().filter(|number| number.is_integer())?;
    number.to_i32()
}

/// A float literal: Mojang's `isNumeric` is true for any number and for a bool.
// `isNumeric` uses the mask 0x2e.
#[allow(
    clippy::cast_possible_truncation,
    reason = "a JSON number narrows to the f32 vanilla stores"
)]
fn numeric(value: &Value) -> Option<f32> {
    if let Some(flag) = value.as_bool() {
        return Some(f32::from(u8::from(flag)));
    }
    Some(value.as_number()?.to_f64_lossy() as f32)
}

/// How a literal of type `Self` is read from a JSON value; `None` when the value is of the
/// wrong type.
pub(super) trait Literal: Sized {
    fn read(reader: &mut Reader<'_>, node: &kit::Node<'_>) -> Option<Self>;
}

impl Literal for i32 {
    fn read(reader: &mut Reader<'_>, node: &kit::Node<'_>) -> Option<Self> {
        if node.value().is_bool() {
            reader.read.wrong_type(node);
        }
        integral(node.value())
    }
}

impl Literal for f32 {
    fn read(reader: &mut Reader<'_>, node: &kit::Node<'_>) -> Option<Self> {
        if node.value().is_bool() {
            reader.read.wrong_type(node);
        }
        Some(finite(reader, numeric(node.value())?, node.path()))
    }
}

/// `read` as far as a float holds it: a number beyond a float is the largest finite float of
/// its sign, with an `InvalidValue` information at `at` (the literal held an
/// infinity, and the tree is loaded and written back).
fn finite(reader: &mut Reader<'_>, read: f32, at: &JsonPath) -> f32 {
    if read.is_infinite() {
        reader.report(
            at,
            DiagnosticCode::from(&core_codes::INVALID_VALUE),
            Severity::Info,
        );
        return f32::MAX.copysign(read);
    }
    read
}

impl Literal for bool {
    fn read(_: &mut Reader<'_>, node: &kit::Node<'_>) -> Option<Self> {
        node.value().as_bool()
    }
}

impl Literal for String {
    fn read(_: &mut Reader<'_>, node: &kit::Node<'_>) -> Option<Self> {
        Some(node.value().as_string()?.as_str().to_owned())
    }
}

/// `[x, y, z]`: an array of exactly three elements, each read by `element`. An element of
/// the wrong type stays at the default with vanilla's `text`, a kept error; a `null` element
/// stays at the default (vanilla: without a word; ours: an `InvalidType` information). Any
/// array of three counts as read.
// `parseValue<BlockPos>` / `parseValue<Vec3>`: the element
// messages are logged (Error, `LogArea::Json`), the array-shape ones are not. 1.26.60:
// `parseValue<int>` logs "Expected Integral Number." and `parseValue<float>` "Expected
// Number." at level 3, area 17; `parseValue<BlockPos>` and
// `parseValue<Vec3>` return true after their three elements whatever they gave, so the leaf and the tree are kept: a kept error.
fn triple<T: Copy + Default>(
    reader: &mut Reader<'_>,
    node: &kit::Node<'_>,
    element: fn(&Value) -> Option<T>,
    line: &'static CodeDef<0>,
) -> Option<[T; 3]> {
    node.value().as_array().filter(|array| array.len() == 3)?;
    let elements = reader.read.array(node)?;
    let mut out = [T::default(); 3];
    for (slot, written) in out.iter_mut().zip(elements.iter()) {
        if written.value().is_null() {
            // Stays at the default, as vanilla leaves it; said.
            reader.read.wrong_type(&written);
            continue;
        }
        if written.value().is_bool() {
            reader.read.wrong_type(&written);
        }
        match element(written.value()) {
            Some(read) => *slot = read,
            None => reader.kept_error(written.path(), DiagnosticCode::from(line)),
        }
    }
    Some(out)
}

impl Literal for [i32; 3] {
    fn read(reader: &mut Reader<'_>, node: &kit::Node<'_>) -> Option<Self> {
        triple(reader, node, integral, &codes::EXPECTED_INTEGRAL)
    }
}

impl Literal for [f32; 3] {
    fn read(reader: &mut Reader<'_>, node: &kit::Node<'_>) -> Option<Self> {
        let mut read = triple(reader, node, numeric, &codes::EXPECTED_NUMBER)?;
        for (index, slot) in read.iter_mut().enumerate() {
            *slot = finite(reader, *slot, &node.path().index(index));
        }
        Some(read)
    }
}
