//! Reading a behavior-tree file the way vanilla's loader does: `AutomationBehaviorTreeGroup`
//! and one `load` per node type over a `Json::Value`.
//!
//! The file front is a function of its own ([`parse_document`]: the text, the name that ends a
//! file without one, the root that must be a node). The nodes derive their read: [`Node`] is an
//! externally tagged enum, each body a struct variant read in declaration order, its unknown
//! members kept and told after it. A parameter is a field fed by two members, its literal and
//! its key (`schema::members`, `Param: FromMembers<2>`); the leaves here are what vanilla's
//! `parseValue` takes as a literal and what `asString` makes of a key. Vanilla's rules are not
//! typed shapes: a member of the wrong type is ignored, so a wrong type is said as an
//! information and the member takes what vanilla gives it.

#![allow(
    clippy::unnecessary_wraps,
    clippy::option_option,
    clippy::needless_pass_by_value,
    reason = "the reads of the fields have the signatures `schema::read_with` and \
              `schema::members` give them: an `Option` they never leave empty, an `Option` \
              field's `Option`, a default some of them only look at"
)]

use facet_value::VObject;

use super::codes;
use super::{BehaviorTree, Node, Param};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Code, CodeDef, Diagnostic, DiagnosticDetail, Diagnostics, JsonPath, Severity,
};
use bedrock_addon_core::id::BehaviorTreeId;
use bedrock_addon_core::json::kit::{self, MemberOrder, Profile, Read, UnknownAt};
use bedrock_addon_core::json::{PrepareError, Prepared, Value};
use bedrock_addon_core::schema::{ElementFailure, FromJson, FromMembers, read_document, read_list};
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
/// what vanilla gives it; a `null` member is an absent one; an unknown member of a node body
/// is kept and said as an information too.
const READING: Reading = Reading::OWN
    .with_on_member_failure(MemberFailure::DefaultAndTell)
    .with_null(bedrock_addon_core::versioning::NullHandling::AbsentSilently)
    .with_keeps_unknown(true);

/// Each `load` looks its members up one by one; the members of a body nobody looks up are
/// kept and told after it.
const PROFILE: Profile = Profile::EVERY_FAILURE
    .with_members(MemberOrder::Declaration)
    .with_unknown(UnknownAt::Last);

/// Reads one behavior-tree file.
pub(super) fn parse_document(bytes: &[u8]) -> Parsed {
    let mut diagnostics = Diagnostics::default();
    let mut read = Read::new(READING, &mut diagnostics)
        .profile(PROFILE)
        .unknown_severity(Severity::Info);
    let (name, tree) = document(&mut read, bytes);
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

/// Keeps the members of `object` (at `at`) other than `read` in the document's table, in
/// document order, without a word.
fn keep_rest(read: &mut Read<'_>, object: &VObject, at: &JsonPath, known: &[&str]) {
    for (key, value) in object {
        if !known.contains(&key.as_str()) {
            read.findings_mut()
                .unknown_mut()
                .put(at, key.as_str(), value.clone());
        }
    }
}

fn report(read: &mut Read<'_>, at: &JsonPath, code: Code, severity: Severity) {
    read.push(Diagnostic::error(code).severity(severity).at(at.clone()));
}

/// A kept error at `at`: the game logs `code`'s line at level 3 and loads the tree.
fn kept_error(read: &mut Read<'_>, at: &JsonPath, code: Code) {
    read.push(Diagnostic::error(code).at(at.clone()).into_kept());
}

/// [`as_string`], with the kit's wrong-type information at `node` for a value that is neither
/// `null` nor a string: a value of another JSON type than its member's is read as vanilla reads
/// it (a bool as its text, a number as the empty text).
fn text(read: &mut Read<'_>, node: Option<&kit::Node<'_>>, default: &str) -> String {
    if let Some(node) = node.filter(|node| !node.value().is_null() && !node.value().is_string()) {
        read.wrong_type(node);
    }
    as_string(node.map(kit::Node::value), default).to_owned()
}

fn document(read: &mut Read<'_>, bytes: &[u8]) -> (Option<BehaviorTreeId>, Option<BehaviorTree>) {
    // Ours: vanilla's reader checks neither the encoding nor the nesting (a tree nests two
    // levels per node: the limit bounds the walk below).
    let ours = |message: &'static str| core_codes::SYNTAX.with([message]);
    let prepared = match Prepared::non_empty(bytes) {
        Ok(prepared) => prepared,
        Err(error) => {
            let (code, severity) = match error {
                // Vanilla skips an empty text without a word: an information.
                PrepareError::Empty => (Code::from(&core_codes::EMPTY_DOCUMENT), Severity::Info),
                PrepareError::NotUtf8(_) => (ours("the file is not UTF-8"), Severity::Error),
                PrepareError::TooDeep(_) => (ours("the JSON nests too deeply"), Severity::Error),
            };
            report(read, &JsonPath::new(), code, severity);
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
    let document = match bedrock_addon_core::json::from_str_jsonc(text) {
        Ok(document) => document,
        Err(error) => {
            let message = error.kind.to_string();
            // `parse error:\n%s` with the reader's messages (Error, `LogArea::Ai`); the
            // file is skipped.
            read.push(
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
        report(
            read,
            &payload_at.child(NAME),
            Code::from(&core_codes::MISSING_REQUIRED_FIELD),
            Severity::Error,
        );
        return (None, None);
    };
    let name = BehaviorTreeId::parse(name);
    let root_at = payload_at.child(ROOT);
    let root = payload.and_then(|payload| payload.get(ROOT));
    if root.and_then(one_member).is_none() {
        report(
            read,
            &root_at,
            Code::from(&core_codes::INVALID_ROOT),
            Severity::Error,
        );
        return (Some(name), None);
    }
    let Some(Root(root)) =
        root.and_then(|root| read_document::<Root>(&kit::Node::new(root, root_at.clone()), read))
    else {
        return (Some(name), None);
    };
    // Vanilla looks up these three keys and no other: the rest (a `format_version`, for one) is
    // kept, unread.
    if let Some(payload) = payload {
        keep_rest(read, payload, &payload_at, &[NAME, ROOT]);
    }
    if let Some(root) = document.as_object() {
        keep_rest(read, root, &JsonPath::new(), &[PAYLOAD]);
    }
    let tree = BehaviorTree {
        name: name.clone(),
        root,
        raw: raw.to_owned(),
    };
    (Some(name), Some(tree))
}

/// The root node: a node whose unknown type refuses the tree.
struct Root(Node);

impl FromJson for Root {
    fn from_json(node: &kit::Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let (kind, body) = one_member(node.value())?;
        self::node(node, kind, body, read, true).map(Root)
    }
}

/// The node of type `kind` with `body`, the one member of the object `object`. An unknown type
/// gives no node: an error that refuses the tree at its `root`, a kept error below it.
// `BehaviorFactory::loadNodeDefinition`; each type's read is its variant's.
fn node(
    object: &kit::Node<'_>,
    kind: &str,
    body: &Value,
    read: &mut Read<'_>,
    root: bool,
) -> Option<Node> {
    let at = object.path().child(kind);
    if !body.is_object() && !body.is_null() {
        // Read as an empty body, and said: vanilla's `operator[]` turns it into an object
        // before the lookup.
        read.wrong_type(&kit::Node::new(body, at.clone()));
    }
    if super::kinds::find(kind).is_none() {
        // `Unknown behavior %s\n` (Error, `LogArea::Entity`); the node is dropped.
        // `BehaviorFactory::loadNodeDefinition` logs it at level 3, area 13: at the root no
        // tree is made; a child is left out and the tree loads, a kept error.
        let code = core_codes::UNKNOWN_NODE.with([kind.to_owned()]);
        if root {
            report(read, &at, code, Severity::Error);
        } else {
            kept_error(read, &at, code);
        }
        return None;
    }
    if body.is_object() {
        return Node::from_json(object, read);
    }
    let mut empty = VObject::new();
    empty.insert(kind, Value::from(VObject::new()));
    let empty = Value::from(empty);
    Node::from_json(&kit::Node::new(&empty, object.path().clone()), read)
}

/// `children` of a composite: the nodes of an array, in order; `None` when the member is
/// absent or not an array. An element that is not a one-member object, or whose type is
/// unknown, is left out.
// `CompositeDefinition::_compositeLoadChildrenBehaviors`: all silent but the unknown type, so
// the shape findings are informations.
pub(super) fn children(
    body: &kit::Node<'_>,
    [children]: &[Option<kit::Node<'_>>; 1],
    _: Option<Vec<Node>>,
    read: &mut Read<'_>,
) -> Option<Option<Vec<Node>>> {
    let Some(children) = children.as_ref().filter(|node| node.value().is_array()) else {
        // Absent too: an information at the member.
        let at = body.path().child("children");
        report(
            read,
            &at,
            Code::from(&core_codes::INVALID_TYPE),
            Severity::Info,
        );
        return Some(None);
    };
    let element = |element: &kit::Node<'_>, read: &mut Read<'_>| {
        let Some((kind, body)) = one_member(element.value()) else {
            report(
                read,
                element.path(),
                Code::from(&core_codes::INVALID_NODE),
                Severity::Info,
            );
            return None;
        };
        node(element, kind, body, read, false)
    };
    Some(read_list(children, read, ElementFailure::Drop, element))
}

/// `child` of a decorator: one node; `None` when the member is absent, not a one-member
/// object, or of an unknown type.
// `DecoratorDefinition::_decoratorLoadChildBehavior`: silent but the unknown type, as
// `children`.
pub(super) fn child(
    body: &kit::Node<'_>,
    [child]: &[Option<kit::Node<'_>>; 1],
    _: Option<Box<Node>>,
    read: &mut Read<'_>,
) -> Option<Option<Box<Node>>> {
    let shaped = child
        .as_ref()
        .and_then(|child| Some((child, one_member(child.value())?)));
    let Some((child, (kind, body))) = shaped else {
        let at = body.path().child("child");
        report(
            read,
            &at,
            Code::from(&core_codes::INVALID_NODE),
            Severity::Info,
        );
        return Some(None);
    };
    Some(node(child, kind, body, read, false).map(Box::new))
}

/// `subtree_id`: its text as vanilla's `asString` makes it.
pub(super) fn subtree_id(node: &kit::Node<'_>, read: &mut Read<'_>) -> Option<BehaviorTreeId> {
    Some(BehaviorTreeId::parse(text(read, Some(node), "")))
}

/// `data_items` of a subtree: a string `s` is `(s, s)`, an object with both `from` and `to` is
/// that pair; the member is one item or an array of items. Any other item is left out.
// `SubtreeDefinition::load` and its `$_0`.
pub(super) fn data_items(
    node: &kit::Node<'_>,
    read: &mut Read<'_>,
) -> Option<Vec<(String, String)>> {
    let mut items = Vec::new();
    if !node.value().is_array() {
        data_item(node, &mut items, read);
    } else if let Some(elements) = read.array(node) {
        for element in elements.iter() {
            data_item(&element, &mut items, read);
        }
    }
    Some(items)
}

/// One item of `data_items`, onto `items`; an item that is neither a string nor an object with
/// both `from` and `to` is left out in silence by vanilla: an information.
fn data_item(node: &kit::Node<'_>, items: &mut Vec<(String, String)>, read: &mut Read<'_>) {
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
            items.push((text(read, Some(&from), ""), text(read, Some(&to), "")));
        }
        None => read.wrong_type(node),
    }
}

/// A literal and its key member: the literal is read when it is present and of its type;
/// otherwise it is the default's and the key is the key member's text or the default key.
// The load rule of every leaf: `parseValue<T>(body[literal], member, log = false)`, and only
// when it fails `body[id_key].asString(default_key)`.
impl<T: Literal> FromMembers<2> for Param<T> {
    fn from_members(
        _: &kit::Node<'_>,
        members: &[Option<kit::Node<'_>>; 2],
        default: Self,
        read: &mut Read<'_>,
    ) -> Option<Self> {
        Some(param(members, default, read, ""))
    }
}

/// [`Param`]'s read, `preset` the key the parameter has when its literal is read.
fn param<T: Literal>(
    [literal, id]: &[Option<kit::Node<'_>>; 2],
    default: Param<T>,
    read: &mut Read<'_>,
    preset: &str,
) -> Param<T> {
    if let Some(node) = literal.as_ref().filter(|node| !node.value().is_null()) {
        if let Some(literal) = T::read(node, read) {
            return Param {
                literal,
                key: preset.to_owned(),
            };
        }
        // Vanilla ignores a literal of the wrong type without a word.
        report(
            read,
            node.path(),
            Code::from(&core_codes::LITERAL_IGNORED),
            Severity::Info,
        );
    }
    Param {
        key: text(read, id.as_ref(), &default.key),
        literal: default.literal,
    }
}

/// `block_name` of `minecraft:find_block`, which sets its key to `block_name` before it reads
/// the literal.
pub(super) fn block_name(
    _: &kit::Node<'_>,
    members: &[Option<kit::Node<'_>>; 2],
    default: Param<String>,
    read: &mut Read<'_>,
) -> Option<Param<String>> {
    Some(param(members, default, read, Node::BLOCK_NAME_KEY))
}

/// `wait_duration`: an integer is the literal, and the same member read as text is the key
/// (default `duration`).
// `WaitDefinition::load` reads the one key twice.
pub(super) fn wait_duration(node: &kit::Node<'_>, read: &mut Read<'_>) -> Option<Param<i32>> {
    if let Some(literal) = integral(node.value()) {
        if node.value().is_bool() {
            read.wrong_type(node);
        }
        return Some(Param {
            literal,
            key: String::new(),
        });
    }
    // A string is the documented key form; any other type is an ignored literal.
    if !node.value().is_string() {
        report(
            read,
            node.path(),
            Code::from(&core_codes::LITERAL_IGNORED),
            Severity::Info,
        );
    }
    Some(Param {
        literal: 0,
        key: as_string(Some(node.value()), WAIT_DEFAULT_KEY).to_owned(),
    })
}

/// The flying flag of `minecraft:fly`, fed by its own literal and key and the throw literal.
/// Vanilla reads both literals into the one member `mShouldBeFlying` and the node uses it for
/// both flags, so both literals here are that value: the throw literal if it was read, else the
/// flying literal, else `false`.
// `FlyDefinition::load`; `FlyNode::initializeFromDefinition` falls back to +0x30 for both
// flags.
pub(super) fn flying(
    _: &kit::Node<'_>,
    [literal, id, throw]: &[Option<kit::Node<'_>>; 3],
    default: Param<bool>,
    read: &mut Read<'_>,
) -> Option<Param<bool>> {
    let (own, key) = flag(literal.as_ref(), id.as_ref(), &default.key, read);
    Some(Param {
        literal: written_bool(throw.as_ref())
            .or(own)
            .unwrap_or(default.literal),
        key,
    })
}

/// The throw flag of `minecraft:fly`, fed by its own literal and key and the flying literal,
/// which it overwrites: said at the flying literal when both are read.
pub(super) fn throw(
    _: &kit::Node<'_>,
    [literal, id, flying]: &[Option<kit::Node<'_>>; 3],
    default: Param<bool>,
    read: &mut Read<'_>,
) -> Option<Param<bool>> {
    let (own, key) = flag(literal.as_ref(), id.as_ref(), &default.key, read);
    let flying_literal = written_bool(flying.as_ref());
    if let (Some(_), Some(_), Some(flying)) = (own, flying_literal, flying) {
        report(
            read,
            flying.path(),
            Code::from(&core_codes::LITERAL_IGNORED),
            Severity::Info,
        );
    }
    Some(Param {
        literal: own.or(flying_literal).unwrap_or(default.literal),
        key,
    })
}

/// A bool literal as `minecraft:fly` reads it, and its key: the key member's text (always read)
/// when the literal was not read, else empty.
fn flag(
    literal: Option<&kit::Node<'_>>,
    id: Option<&kit::Node<'_>>,
    default_key: &str,
    read: &mut Read<'_>,
) -> (Option<bool>, String) {
    let written = literal.filter(|node| !node.value().is_null());
    let value = written.and_then(|node| node.value().as_bool());
    if let Some(node) = written.filter(|_| value.is_none()) {
        report(
            read,
            node.path(),
            Code::from(&core_codes::LITERAL_IGNORED),
            Severity::Info,
        );
    }
    let key = text(read, id, default_key);
    (value, if value.is_some() { String::new() } else { key })
}

/// The bool a literal member holds, when it is one.
fn written_bool(node: Option<&kit::Node<'_>>) -> Option<bool> {
    node?.value().as_bool()
}

/// The key of `minecraft:wait` when the member is absent.
pub(super) const WAIT_DEFAULT_KEY: &str = "duration";

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
/// wrong type (the caller says it).
pub trait Literal: Sized {
    fn read(node: &kit::Node<'_>, read: &mut Read<'_>) -> Option<Self>;
}

impl Literal for i32 {
    fn read(node: &kit::Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        if node.value().is_bool() {
            read.wrong_type(node);
        }
        integral(node.value())
    }
}

impl Literal for f32 {
    fn read(node: &kit::Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        if node.value().is_bool() {
            read.wrong_type(node);
        }
        Some(finite(read, numeric(node.value())?, node.path()))
    }
}

/// `read` as far as a float holds it: a number beyond a float is the largest finite float of
/// its sign, with an `InvalidValue` information at `at` (the literal held an infinity, and the
/// tree is loaded and written back).
fn finite(read: &mut Read<'_>, value: f32, at: &JsonPath) -> f32 {
    if value.is_infinite() {
        report(
            read,
            at,
            Code::from(&core_codes::INVALID_VALUE),
            Severity::Info,
        );
        return f32::MAX.copysign(value);
    }
    value
}

impl Literal for bool {
    fn read(node: &kit::Node<'_>, _: &mut Read<'_>) -> Option<Self> {
        node.value().as_bool()
    }
}

impl Literal for String {
    fn read(node: &kit::Node<'_>, _: &mut Read<'_>) -> Option<Self> {
        Some(node.value().as_string()?.as_str().to_owned())
    }
}

/// `[x, y, z]`: an array of exactly three elements, each read by `element`. An element of
/// the wrong type stays at the default with vanilla's `text`, a kept error; a `null` element
/// stays at the default (vanilla: without a word; ours: an `InvalidType` information). Any
/// array of three counts as read.
// `parseValue<BlockPos>` / `parseValue<Vec3>`: the element messages are logged (Error,
// `LogArea::Json`), the array-shape ones are not. 1.26.60: `parseValue<int>` logs "Expected
// Integral Number." and `parseValue<float>` "Expected Number." at level 3, area 17;
// `parseValue<BlockPos>` and `parseValue<Vec3>` return true after their three elements whatever
// they gave, so the leaf and the tree are kept: a kept error.
fn triple<T: Copy + Default>(
    read: &mut Read<'_>,
    node: &kit::Node<'_>,
    element: fn(&Value) -> Option<T>,
    line: &'static CodeDef<0>,
) -> Option<[T; 3]> {
    node.value().as_array().filter(|array| array.len() == 3)?;
    let elements = read.array(node)?;
    let mut out = [T::default(); 3];
    for (slot, written) in out.iter_mut().zip(elements.iter()) {
        if written.value().is_null() {
            // Stays at the default, as vanilla leaves it; said.
            read.wrong_type(&written);
            continue;
        }
        if written.value().is_bool() {
            read.wrong_type(&written);
        }
        match element(written.value()) {
            Some(value) => *slot = value,
            None => kept_error(read, written.path(), Code::from(line)),
        }
    }
    Some(out)
}

impl Literal for [i32; 3] {
    fn read(node: &kit::Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        triple(read, node, integral, &codes::EXPECTED_INTEGRAL)
    }
}

impl Literal for [f32; 3] {
    fn read(node: &kit::Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let mut value = triple(read, node, numeric, &codes::EXPECTED_NUMBER)?;
        for (index, slot) in value.iter_mut().enumerate() {
            *slot = finite(read, *slot, &node.path().index(index));
        }
        Some(value)
    }
}
