//! Reading a behavior-tree file the way vanilla's loader does: `AutomationBehaviorTreeGroup`
//! and one hand-written `load` per node type over a `Json::Value`. The walk is over an untyped
//! `Value` because vanilla's rules are not typed shapes: a member is looked up and, when it is
//! of the wrong type, ignored.

use facet_value::VObject;

use super::{BehaviorTree, Node, Param};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Severity, VanillaText,
};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::id::BehaviorTreeId;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::json::{PrepareError, Prepared};

/// The only member of the document root vanilla reads.
pub(super) const PAYLOAD: &str = "minecraft:behavior_tree";
/// The tree's key.
pub(super) const NAME: &str = "minecraft:name";
/// The root node.
pub(super) const ROOT: &str = "minecraft:root";

/// Vanilla's text for a file that is not JSON, with the reader's message.
pub(super) const PARSE_ERROR: &str = "parse error:\n{}";

/// Vanilla's text for an element of a block position that is not integral.
pub(super) const EXPECTED_INTEGRAL: &str = "Expected Integral Number.";
/// Vanilla's text for an element of a position that is not a number.
pub(super) const EXPECTED_NUMBER: &str = "Expected Number.";

/// What the loader read of one file.
pub(super) struct Parsed {
    /// The name the file gives its tree, when it has one.
    pub(super) name: Option<BehaviorTreeId>,
    /// The tree, when the file loads.
    pub(super) tree: Option<BehaviorTree>,
    /// The findings.
    pub(super) diagnostics: Diagnostics,
}

/// Reads one behavior-tree file.
pub(super) fn parse_document(bytes: &[u8]) -> Parsed {
    let mut reader = Reader::default();
    let (name, tree) = reader.document(bytes);
    Parsed {
        name,
        tree,
        diagnostics: reader.diagnostics,
    }
}

/// The type and body of a node: the one member of an object with exactly one member.
fn one_member(value: &Value) -> Option<(&str, &Value)> {
    let object = value.as_object().filter(|object| object.len() == 1)?;
    let (kind, body) = object.iter().next()?;
    Some((kind.as_str(), body))
}

/// The members of `object` other than `read`, in document order.
fn rest(object: &VObject, read: &[&str]) -> UnknownMembers {
    let mut extra = UnknownMembers::default();
    for (key, value) in object {
        if !read.contains(&key.as_str()) {
            extra.insert(key.as_str().to_owned(), value.clone());
        }
    }
    extra
}

/// One file's walk.
#[derive(Default)]
struct Reader {
    diagnostics: Diagnostics,
}

impl Reader {
    /// An `InvalidType` information at `at`: a value of another JSON type than its member's is
    /// read as vanilla reads it (a bool as 1 / 0 or as its text, a number as the empty text,
    /// a body that is no object as an empty one), and the read is said (ours; vanilla
    /// is silent, so an information).
    fn wrong_type(&mut self, at: &JsonPath) {
        self.report(
            at,
            DiagnosticCode::InvalidType { text: None },
            Severity::Info,
        );
    }

    /// [`as_string`], with the information of [`Self::wrong_type`] at `at` for a value that is
    /// neither absent, `null` nor a string.
    fn text<'v>(&mut self, value: Option<&'v Value>, default: &'v str, at: &JsonPath) -> &'v str {
        if value.is_some_and(|value| !value.is_null() && !value.is_string()) {
            self.wrong_type(at);
        }
        as_string(value, default)
    }

    fn report(&mut self, at: &JsonPath, code: DiagnosticCode, severity: Severity) {
        self.diagnostics
            .push(Diagnostic::error(code).severity(severity).at(at.clone()));
    }

    /// A kept error at `at`: the game logs `code`'s line at level 3 and loads the tree.
    fn kept_error(&mut self, at: &JsonPath, code: DiagnosticCode) {
        self.diagnostics
            .push_kept_error(Diagnostic::error(code).at(at.clone()));
    }

    fn document(&mut self, bytes: &[u8]) -> (Option<BehaviorTreeId>, Option<BehaviorTree>) {
        // Ours: vanilla's reader checks neither the encoding nor the nesting (a tree nests two
        // levels per node: the limit bounds the walk below).
        let ours = |message: &str| DiagnosticCode::MalformedInput {
            message: message.to_owned(),
            text: None,
        };
        let prepared = match Prepared::non_empty(bytes) {
            Ok(prepared) => prepared,
            Err(error) => {
                let (code, severity) = match error {
                    // Vanilla skips an empty text without a word: an information.
                    PrepareError::Empty => {
                        (DiagnosticCode::EmptyDocument { text: None }, Severity::Info)
                    }
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
                self.report(
                    &JsonPath::new(),
                    DiagnosticCode::MalformedInput {
                        message: message.clone(),
                        text: Some(VanillaText::Formatted {
                            template: PARSE_ERROR,
                            args: vec![message],
                        }),
                    },
                    Severity::Error,
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
                DiagnosticCode::MissingRequiredField { text: None },
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
            self.report(&root_at, DiagnosticCode::InvalidRoot, Severity::Error);
            return (Some(name), None);
        };
        let Some(root) = self.node(kind, body, &root_at, true) else {
            return (Some(name), None);
        };
        // Vanilla looks up these three keys and no other: the rest (a `format_version`, for one) is kept, unread.
        let tree = BehaviorTree {
            name: name.clone(),
            root,
            raw: raw.to_owned(),
            extra: payload.map_or_else(UnknownMembers::default, |payload| {
                rest(payload, &[NAME, ROOT])
            }),
            root_extra: document
                .as_object()
                .map_or_else(UnknownMembers::default, |root| rest(root, &[PAYLOAD])),
        };
        (Some(name), Some(tree))
    }

    /// The node of type `kind` with `body`, the one member of the object at `at`. An unknown
    /// type gives no node: an error that refuses the tree at its `root`, a kept error below it.
    // `BehaviorFactory::loadNodeDefinition` and the 21 `load` overrides;
    // the keys, default keys and defaults are those of the overrides.
    #[allow(clippy::too_many_lines, reason = "one arm per node type")]
    fn node(&mut self, kind: &str, body: &Value, at: &JsonPath, root: bool) -> Option<Node> {
        // A body that is not an object reads as an empty one: vanilla's `operator[]` turns it
        // into one before the lookup.
        let empty = VObject::new();
        let at = &at.child(kind);
        if !body.is_object() && !body.is_null() {
            // Read as an empty body, and said.
            self.wrong_type(at);
        }
        let m = &mut Members::new(body.as_object().unwrap_or(&empty));
        let node = match kind {
            "minecraft:sequence" => Node::Sequence {
                children: self.children(m, at),
                extra: self.rest(m, at),
            },
            "minecraft:selector" => Node::Selector {
                children: self.children(m, at),
                extra: self.rest(m, at),
            },
            "minecraft:repeat_until_success" => Node::RepeatUntilSuccess {
                max_attempts: self.param(m, at, &MAX_ATTEMPTS, 0),
                child: self.child(m, at),
                extra: self.rest(m, at),
            },
            "minecraft:repeat_until_failure" => Node::RepeatUntilFailure {
                child: self.child(m, at),
                extra: self.rest(m, at),
            },
            "minecraft:inverter" => Node::Inverter {
                child: self.child(m, at),
                extra: self.rest(m, at),
            },
            "minecraft:subtree" => Node::Subtree {
                subtree_id: BehaviorTreeId::parse(self.text(
                    m.take("subtree_id"),
                    "",
                    &at.child("subtree_id"),
                )),
                data_items: self.data_items(m, at),
                extra: self.rest(m, at),
            },
            "minecraft:activate_tool" => Node::ActivateTool {
                block_pos: self.param(m, at, &BLOCK_POS, [0; 3]),
                extra: self.rest(m, at),
            },
            "minecraft:attack" => Node::Attack {
                ticks: self.param(m, at, &NUM_OF_TICKS, 0),
                extra: self.rest(m, at),
            },
            "minecraft:break_block" => Node::BreakBlock {
                block_pos: self.param(m, at, &BLOCK_POS, [0; 3]),
                ticks: self.param(m, at, &NUM_OF_TICKS, 0),
                extra: self.rest(m, at),
            },
            "minecraft:consume_item" => Node::ConsumeItem {
                extra: self.rest(m, at),
            },
            "minecraft:find_block" => Node::FindBlock {
                block_name: self.param(m, at, &BLOCK_NAME, String::new()),
                search_radius: self.param(m, at, &SEARCH_RADIUS, FAILED_SEARCH_RADIUS),
                extra: self.rest(m, at),
            },
            "minecraft:find_entity" => Node::FindEntity {
                entity_name: self.param(m, at, &ENTITY_NAME, String::new()),
                search_radius: self.param(m, at, &SEARCH_RADIUS, FAILED_SEARCH_RADIUS),
                extra: self.rest(m, at),
            },
            "minecraft:fly" => {
                let (should_be_flying, throw_if_no_change) = self.fly(m, at);
                Node::Fly {
                    should_be_flying,
                    throw_if_no_change,
                    extra: self.rest(m, at),
                }
            }
            "minecraft:get_interaction_position_for_block" => {
                Node::GetInteractionPositionForBlock {
                    target: self.param(m, at, &TARGET_BLOCK_POS, [0; 3]),
                    anchor: self.param(m, at, &ANCHOR_BLOCK_POS, [0; 3]),
                    facing: self.param(m, at, &FACING_STRING, String::new()),
                    max_search_distance: self.param(m, at, &MAX_SEARCH_DISTANCE, 0),
                    extra: self.rest(m, at),
                }
            }
            "minecraft:interact" => Node::Interact {
                // `make_unique<InteractActionDefinition>` stores 1.
                ticks: self.param(m, at, &NUM_OF_TICKS, Node::INTERACT_DEFAULT_TICKS),
                extra: self.rest(m, at),
            },
            "minecraft:look_at" => Node::LookAt {
                block_pos: self.param(m, at, &BLOCK_POS, [0; 3]),
                extra: self.rest(m, at),
            },
            "minecraft:look_at_entity" => Node::LookAtEntity {
                entity_name: self.param(m, at, &ENTITY_NAME, String::new()),
                search_radius: self.param(m, at, &SEARCH_RADIUS, FAILED_SEARCH_RADIUS),
                extra: self.rest(m, at),
            },
            "minecraft:move" => Node::Move {
                direction: self.param(m, at, &DIRECTION, String::new()),
                second_direction: self.param(m, at, &SECOND_DIRECTION, String::new()),
                ticks: self.param(m, at, &MAX_NUMBER_OF_TICKS, 0),
                extra: self.rest(m, at),
            },
            "minecraft:move_to" => Node::MoveTo {
                pos: self.param(m, at, &BLOCK_POS, [0.0; 3]),
                distance_epsilon: self.param(m, at, &DISTANCE_EPSILON, 0.0),
                extra: self.rest(m, at),
            },
            "minecraft:place_block" => Node::PlaceBlock {
                extra: self.rest(m, at),
            },
            "minecraft:shoot_bow" => Node::ShootBow {
                shots: self.param(m, at, &NUM_OF_SHOTS, 0),
                extra: self.rest(m, at),
            },
            "minecraft:use_entity" => Node::UseEntity {
                extra: self.rest(m, at),
            },
            "minecraft:wait" => Node::Wait {
                seconds: self.wait_duration(m, at),
                extra: self.rest(m, at),
            },
            "minecraft:wait_ticks" => Node::WaitTicks {
                ticks: self.param(m, at, &NUM_OF_TICKS, 0),
                extra: self.rest(m, at),
            },
            _ => {
                // `Unknown behavior %s\n` (Error, `LogArea::Entity`); the
                // node is dropped. `BehaviorFactory::loadNodeDefinition` logs it at
                // level 3, area 13: at the root no tree is made; a child is left out and the
                // tree loads, a kept error.
                let code = DiagnosticCode::UnknownNode {
                    name: kind.to_owned(),
                };
                if root {
                    self.report(at, code, Severity::Error);
                } else {
                    self.kept_error(at, code);
                }
                return None;
            }
        };
        Some(node)
    }

    /// A literal and its key member. The literal is read when it is present and of its type;
    /// otherwise it is `failed` and the key is the key member's text or the default key.
    // The load rule of every leaf:
    // `parseValue<T>(body[literal], member, log = false)`, and only when it fails
    // `body[id_key].asString(default_key)`.
    fn param<T: Literal>(
        &mut self,
        members: &mut Members<'_>,
        at: &JsonPath,
        pair: &Pair,
        failed: T,
    ) -> Param<T> {
        let literal_at = at.child(pair.literal);
        let written = members.take(pair.literal).filter(|value| !value.is_null());
        let id = members.take(pair.id);
        if let Some(value) = written {
            if let Some(literal) = T::read(self, value, &literal_at) {
                return Param {
                    literal,
                    key: pair.preset.to_owned(),
                };
            }
            // Vanilla ignores a literal of the wrong type without a word.
            self.report(&literal_at, DiagnosticCode::LiteralIgnored, Severity::Info);
        }
        Param {
            literal: failed,
            key: self
                .text(id, pair.default_key, &at.child(pair.id))
                .to_owned(),
        }
    }

    /// `wait_duration`: an integer is the literal, and the same member read as text is the
    /// key (default `duration`).
    // `WaitDefinition::load` reads the one key twice.
    fn wait_duration(&mut self, members: &mut Members<'_>, at: &JsonPath) -> Param<i32> {
        const KEY: &str = WAIT_DURATION;
        let value = members.take(KEY);
        let written = value.filter(|value| !value.is_null());
        if let Some(literal) = written.and_then(integral) {
            if written.is_some_and(Value::is_bool) {
                self.wrong_type(&at.child(KEY));
            }
            return Param {
                literal,
                key: String::new(),
            };
        }
        // A string is the documented key form; any other type is an ignored literal.
        if written.is_some_and(|value| !value.is_string()) {
            self.report(
                &at.child(KEY),
                DiagnosticCode::LiteralIgnored,
                Severity::Info,
            );
        }
        Param {
            literal: 0,
            key: as_string(value, WAIT_DEFAULT_KEY).to_owned(),
        }
    }

    /// The two flags of `minecraft:fly`. Vanilla reads both literals into the one member
    /// `mShouldBeFlying` and the node uses it for both flags, so both literals here are that
    /// value: the throw literal if it was read, else the flying literal, else `false`.
    // `FlyDefinition::load`; `FlyNode::initializeFromDefinition` falls back to +0x30 for both
    // flags.
    fn fly(&mut self, members: &mut Members<'_>, at: &JsonPath) -> (Param<bool>, Param<bool>) {
        let flying = self.read_literal::<bool>(members, at, &SHOULD_BE_FLYING);
        let throw = self.read_literal::<bool>(members, at, &SHOULD_THROW);
        if flying.0.is_some() && throw.0.is_some() {
            // The flying literal is overwritten by the throw literal.
            self.report(
                &at.child(SHOULD_BE_FLYING.literal),
                DiagnosticCode::LiteralIgnored,
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
        members: &mut Members<'_>,
        at: &JsonPath,
        pair: &Pair,
    ) -> (Option<T>, String) {
        let literal_at = at.child(pair.literal);
        let written = members.take(pair.literal).filter(|value| !value.is_null());
        let id = members.take(pair.id);
        let literal = written.and_then(|value| T::read(self, value, &literal_at));
        if written.is_some() && literal.is_none() {
            self.report(&literal_at, DiagnosticCode::LiteralIgnored, Severity::Info);
        }
        let key = self
            .text(id, pair.default_key, &at.child(pair.id))
            .to_owned();
        (literal, key)
    }

    /// `children` of a composite: the nodes of an array, in order; `None` when the member is
    /// absent or not an array. An element that is not a one-member object, or whose type is
    /// unknown, is left out.
    // `CompositeDefinition::_compositeLoadChildrenBehaviors`: all silent
    // but the unknown type, so the shape findings are informations.
    fn children(&mut self, members: &mut Members<'_>, at: &JsonPath) -> Option<Vec<Node>> {
        let at = at.child("children");
        let Some(array) = members.take("children").and_then(Value::as_array) else {
            self.report(
                &at,
                DiagnosticCode::InvalidType { text: None },
                Severity::Info,
            );
            return None;
        };
        let mut nodes = Vec::new();
        for (index, element) in array.iter().enumerate() {
            let element_at = at.index(index);
            let Some((kind, body)) = one_member(element) else {
                self.report(&element_at, DiagnosticCode::InvalidNode, Severity::Info);
                continue;
            };
            nodes.extend(self.node(kind, body, &element_at, false));
        }
        Some(nodes)
    }

    /// `child` of a decorator: one node; `None` when the member is absent, not a one-member
    /// object, or of an unknown type.
    // `DecoratorDefinition::_decoratorLoadChildBehavior`: silent but the
    // unknown type, as `children`.
    fn child(&mut self, members: &mut Members<'_>, at: &JsonPath) -> Option<Box<Node>> {
        let at = at.child("child");
        let Some((kind, body)) = members.take("child").and_then(one_member) else {
            self.report(&at, DiagnosticCode::InvalidNode, Severity::Info);
            return None;
        };
        self.node(kind, body, &at, false).map(Box::new)
    }

    /// `data_items` of a subtree: a string `s` is `(s, s)`, an object with both `from` and
    /// `to` is that pair; the member is one item or an array of items. Any other item is left
    /// out.
    // `SubtreeDefinition::load` and its `$_0`.
    fn data_items(&mut self, members: &mut Members<'_>, at: &JsonPath) -> Vec<(String, String)> {
        let at = at.child("data_items");
        let Some(value) = members.take("data_items").filter(|value| !value.is_null()) else {
            return Vec::new();
        };
        let mut items = Vec::new();
        let mut item = |reader: &mut Self, value: &Value, at: &JsonPath| {
            if let Some(text) = value.as_string() {
                items.push((text.as_str().to_owned(), text.as_str().to_owned()));
                return;
            }
            let pair = value
                .as_object()
                .and_then(|object| Some((object.get("from")?, object.get("to")?)));
            match pair {
                Some((from, to)) => items.push((
                    reader.text(Some(from), "", &at.child("from")).to_owned(),
                    reader.text(Some(to), "", &at.child("to")).to_owned(),
                )),
                // Left out in silence by vanilla: an information.
                None => reader.report(
                    at,
                    DiagnosticCode::InvalidType { text: None },
                    Severity::Info,
                ),
            }
        };
        match value.as_array() {
            Some(array) => {
                for (index, element) in array.iter().enumerate() {
                    item(self, element, &at.index(index));
                }
            }
            None => item(self, value, &at),
        }
        items
    }

    /// The members of a node body nobody read, kept in an `UnknownMembers`, each with an
    /// `UnrecognizedField` information (vanilla reads no other key and says nothing).
    fn rest(&mut self, members: &Members<'_>, at: &JsonPath) -> UnknownMembers {
        let mut extra = UnknownMembers::default();
        for (key, value) in members.object {
            let key = key.as_str();
            if members.read.contains(&key) {
                continue;
            }
            self.report(
                &at.child(key),
                DiagnosticCode::UnrecognizedField { text: None },
                Severity::Info,
            );
            extra.insert(key.to_owned(), value.clone());
        }
        extra
    }
}

/// The members of a node body as the walker reads them.
struct Members<'v> {
    object: &'v VObject,
    read: Vec<&'static str>,
}

impl<'v> Members<'v> {
    fn new(object: &'v VObject) -> Self {
        Self {
            object,
            read: Vec::new(),
        }
    }

    /// Member `key`, now read (whether or not it is there).
    fn take(&mut self, key: &'static str) -> Option<&'v Value> {
        self.read.push(key);
        self.object.get(key)
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
const FAILED_SEARCH_RADIUS: i32 = Node::FAILED_SEARCH_RADIUS;

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
trait Literal: Sized {
    fn read(reader: &mut Reader, value: &Value, at: &JsonPath) -> Option<Self>;
}

impl Literal for i32 {
    fn read(reader: &mut Reader, value: &Value, at: &JsonPath) -> Option<Self> {
        if value.is_bool() {
            reader.wrong_type(at);
        }
        integral(value)
    }
}

impl Literal for f32 {
    fn read(reader: &mut Reader, value: &Value, at: &JsonPath) -> Option<Self> {
        if value.is_bool() {
            reader.wrong_type(at);
        }
        Some(finite(reader, numeric(value)?, at))
    }
}

/// `read` as far as a float holds it: a number beyond a float is the largest finite float of
/// its sign, with an `InvalidValue` information at `at` (the literal held an
/// infinity, and the tree is loaded and written back).
fn finite(reader: &mut Reader, read: f32, at: &JsonPath) -> f32 {
    if read.is_infinite() {
        reader.report(
            at,
            DiagnosticCode::InvalidValue { text: None },
            Severity::Info,
        );
        return f32::MAX.copysign(read);
    }
    read
}

impl Literal for bool {
    fn read(_: &mut Reader, value: &Value, _: &JsonPath) -> Option<Self> {
        value.as_bool()
    }
}

impl Literal for String {
    fn read(_: &mut Reader, value: &Value, _: &JsonPath) -> Option<Self> {
        Some(value.as_string()?.as_str().to_owned())
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
    reader: &mut Reader,
    value: &Value,
    at: &JsonPath,
    element: fn(&Value) -> Option<T>,
    text: &'static str,
) -> Option<[T; 3]> {
    let array = value.as_array().filter(|array| array.len() == 3)?;
    let mut out = [T::default(); 3];
    for (index, (slot, written)) in out.iter_mut().zip(array.iter()).enumerate() {
        if written.is_null() {
            // Stays at the default, as vanilla leaves it; said.
            reader.wrong_type(&at.index(index));
            continue;
        }
        if written.is_bool() {
            reader.wrong_type(&at.index(index));
        }
        match element(written) {
            Some(read) => *slot = read,
            None => reader.kept_error(
                &at.index(index),
                DiagnosticCode::InvalidType {
                    text: Some(VanillaText::Fixed(text)),
                },
            ),
        }
    }
    Some(out)
}

impl Literal for [i32; 3] {
    fn read(reader: &mut Reader, value: &Value, at: &JsonPath) -> Option<Self> {
        triple(reader, value, at, integral, EXPECTED_INTEGRAL)
    }
}

impl Literal for [f32; 3] {
    fn read(reader: &mut Reader, value: &Value, at: &JsonPath) -> Option<Self> {
        let mut read = triple(reader, value, at, numeric, EXPECTED_NUMBER)?;
        for (index, slot) in read.iter_mut().enumerate() {
            *slot = finite(reader, *slot, &at.index(index));
        }
        Some(read)
    }
}
