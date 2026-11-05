//! Writing a root or a composition: the members as they are held, nothing invented, and
//! nothing written that the client's strict parse refuses (the same rules `read.rs` reads by,
//! here over the value).

use facet_value::{VArray, VObject};

use super::{Binding, Catalogue, ChildSet, Control, FORMAT_VERSION, Layout, UiComposition, UiRoot};
use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::json::{EncodeError, MAX_JSON_DEPTH, Unrepresentable, Value};

const ROOT_KEY: &str = "minecraft:ui-root";
const COMPOSITION_KEY: &str = "minecraft:ui-composition";

/// The nesting of the envelope, the payload, `layout` and `markup` above the first control.
const ABOVE_MARKUP: usize = 3;

/// The refusal of the member at `members` below `key`.
fn refusal(key: &str, members: &[&str]) -> EncodeError {
    let mut at = JsonPath::of_keys([key]);
    for member in members {
        at.push(member);
    }
    EncodeError::NotRepresentable {
        at,
        what: Unrepresentable::Unsupported,
    }
}

/// A non-empty namespace, `:`, a non-empty name.
fn namespaced(identifier: &str) -> bool {
    identifier
        .split_once(':')
        .is_some_and(|(namespace, name)| !namespace.is_empty() && !name.is_empty())
}

/// How deep `value` nests arrays and objects (a scalar is 0), counted no further than
/// `limit + 1`; no recursion.
fn nesting(value: &Value, limit: usize) -> usize {
    let mut deepest = 0;
    let mut pending = vec![(value, 0_usize)];
    while let Some((value, above)) = pending.pop() {
        let depth = if let Some(object) = value.as_object() {
            pending.extend(object.iter().map(|(_, member)| (member, above + 1)));
            above + 1
        } else if let Some(array) = value.as_array() {
            pending.extend(array.iter().map(|element| (element, above + 1)));
            above + 1
        } else {
            above
        };
        deepest = deepest.max(depth);
        if deepest > limit {
            break;
        }
    }
    deepest
}

/// One array of controls being checked.
struct Frame<'a> {
    controls: &'a [Control],
    next: usize,
    set: ChildSet,
}

/// The path below the payload key of `member` of the control the walk is at.
fn member_path(frames: &[Frame<'_>], member: &str) -> JsonPath {
    let mut at = JsonPath::of_keys(["layout", "markup"]);
    for (index, frame) in frames.iter().enumerate() {
        if index > 0 {
            at.push("children");
        }
        at.push_index(frame.next.saturating_sub(1));
    }
    at.push(member);
    at
}

/// The member of `control`, standing at `depth` where `set` is expected, that `catalogue`
/// refuses or that nests past the reader's depth limit; `Ok` with the set of its children
/// when it has them.
fn refused_member(
    control: &Control,
    set: ChildSet,
    depth: usize,
    catalogue: Catalogue,
) -> Result<Option<ChildSet>, &'static str> {
    let rule = catalogue
        .rule(control.component)
        .filter(|_| catalogue.allows(set, control.component) && depth <= MAX_JSON_DEPTH)
        .ok_or("component")?;
    let fits = control
        .attribs
        .as_ref()
        .is_none_or(|value| depth + nesting(value, MAX_JSON_DEPTH) <= MAX_JSON_DEPTH);
    let bound = match rule.attribs {
        Binding::Unbound => control.attribs.is_none(),
        Binding::Optional => true,
        Binding::Required => control.attribs.is_some(),
    };
    if !bound || !fits {
        return Err("attribs");
    }
    // A type with a child set writes `children`, no other type does; the array is one level
    // below the control's object.
    let written = control.children.is_some();
    if rule.children.is_some() != written || (written && depth >= MAX_JSON_DEPTH) {
        return Err("children");
    }
    Ok(rule.children)
}

/// The first member of the tree below `markup` that `catalogue` refuses or that nests past the
/// reader's depth limit, as its path below the payload key; `None` when the tree is one the
/// reader takes. Walks the tree on a list, in document order.
fn first_refused<'a>(
    markup: &'a [Control],
    set: ChildSet,
    catalogue: Catalogue,
) -> Option<JsonPath> {
    let mut frames: Vec<Frame<'a>> = vec![Frame {
        controls: markup,
        next: 0,
        set,
    }];
    loop {
        let level = frames.len();
        let frame = frames.last_mut()?;
        let controls = frame.controls;
        let Some(control) = controls.get(frame.next) else {
            frames.pop();
            continue;
        };
        frame.next += 1;
        // The control's object is at this depth.
        let depth = ABOVE_MARKUP + 2 * level;
        match refused_member(control, frame.set, depth, catalogue) {
            Err(member) => return Some(member_path(&frames, member)),
            Ok(None) => {}
            Ok(Some(below)) => frames.push(Frame {
                controls: control.children(),
                next: 0,
                set: below,
            }),
        }
    }
}

/// The catalogue that takes the whole tree, the newest first; the refusal of the newest when
/// none does. The writer has no engine to ask: a document some supported engine loads is
/// written, a document none loads is not.
fn check_tree(key: &str, layout: &Layout, set: ChildSet) -> Result<(), EncodeError> {
    if layout.markup.is_empty() {
        // `markup` must hold at least one control.
        return Err(refusal(key, &["layout", "markup"]));
    }
    let mut newest = None;
    for catalogue in Catalogue::ALL.into_iter().rev() {
        match first_refused(&layout.markup, set, catalogue) {
            None => return Ok(()),
            Some(at) => newest = newest.or(Some(at)),
        }
    }
    let mut at = JsonPath::of_keys([key]);
    if let Some(below) = newest {
        at.append(&below);
    }
    Err(EncodeError::NotRepresentable {
        at,
        what: Unrepresentable::Unsupported,
    })
}

/// One control as JSON: `component`, then `attribs` and `children` when it holds them. The
/// tree passed `check_tree`, which bounds this recursion by the reader's depth limit.
fn control(control: &Control) -> Value {
    let mut object = VObject::new();
    object.insert("component", Value::from(control.component.name()));
    if let Some(attribs) = &control.attribs {
        object.insert("attribs", attribs.clone());
    }
    if let Some(children) = &control.children {
        object.insert("children", controls(children));
    }
    Value::from(object)
}

/// An array of controls as JSON.
fn controls(list: &[Control]) -> Value {
    let mut array = VArray::new();
    for element in list {
        array.push(control(element));
    }
    Value::from(array)
}

/// `{"<key>": <value>}`.
fn single(key: &str, value: Value) -> Value {
    let mut object = VObject::new();
    object.insert(key, value);
    Value::from(object)
}

/// The payload from its three members.
fn payload(identifier: &str, attribs: Value, layout: &Layout) -> Value {
    let mut object = VObject::new();
    object.insert("description", single("identifier", Value::from(identifier)));
    object.insert("attribs", attribs);
    object.insert("layout", single("markup", controls(&layout.markup)));
    Value::from(object)
}

/// The envelope around `payload`.
fn document(key: &str, payload: Value) -> Value {
    let mut root = VObject::new();
    root.insert("format_version", Value::from(FORMAT_VERSION.to_string()));
    root.insert(key, payload);
    Value::from(root)
}

/// The payload of a root.
pub(crate) fn root_payload(root: &UiRoot) -> Result<Value, EncodeError> {
    let identifier = root.description.identifier.as_str();
    if !namespaced(identifier) {
        return Err(refusal(ROOT_KEY, &["description", "identifier"]));
    }
    if root.attribs.root_point.is_empty() {
        // `root_point` must not be empty.
        return Err(refusal(ROOT_KEY, &["attribs", "root_point"]));
    }
    check_tree(ROOT_KEY, &root.layout, ChildSet::RootControl)?;
    let attribs = single("root_point", Value::from(root.attribs.root_point.as_str()));
    Ok(payload(identifier, attribs, &root.layout))
}

/// The whole document of a root.
pub(crate) fn root_document(root: &UiRoot) -> Result<Value, EncodeError> {
    Ok(document(ROOT_KEY, root_payload(root)?))
}

/// The payload of a composition.
pub(crate) fn composition_payload(composition: &UiComposition) -> Result<Value, EncodeError> {
    let identifier = composition.description.identifier.as_str();
    if !namespaced(identifier) {
        return Err(refusal(COMPOSITION_KEY, &["description", "identifier"]));
    }
    let mut attribs = VObject::new();
    if let Some(extension_point) = &composition.attribs.extension_point {
        if extension_point.is_empty() {
            return Err(refusal(COMPOSITION_KEY, &["attribs", "extension_point"]));
        }
        attribs.insert("extension_point", Value::from(extension_point.as_str()));
    }
    check_tree(COMPOSITION_KEY, &composition.layout, ChildSet::PanelContent)?;
    Ok(payload(
        identifier,
        Value::from(attribs),
        &composition.layout,
    ))
}

/// The whole document of a composition.
pub(crate) fn composition_document(composition: &UiComposition) -> Result<Value, EncodeError> {
    Ok(document(COMPOSITION_KEY, composition_payload(composition)?))
}
