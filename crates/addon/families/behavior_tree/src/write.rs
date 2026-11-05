//! Writing a tree back as JSON: the name and the root node, each parameter in the form it
//! reads back from, the document's unknown members after the known ones of their level (the
//! Value level), one named like a member already written left out.
//!
//! A parameter is written as one of vanilla's two members at a time. Its key decides: under
//! the key a read literal leaves (empty; `block_name` for `find_block`'s block name) it is the
//! literal member, under any other key the key member, and nothing where that reads the same
//! as the absent member (the pair's default key; a default literal where the key a read
//! literal leaves is also the default key). A value built in memory with an empty key and a
//! literal that is not the default means the literal.
//!
//! So a member vanilla ignored is left out (a literal of the wrong type beside a string key,
//! a key beside a literal that was read, an explicit default key), and no default is written
//! for a member the author left out. One state has only one text: a key member that is no
//! string reads as the key `""`, which is the state of a read default literal, and is written
//! as the literal member with the value vanilla holds, also where the file had the key
//! member alone.

use facet_value::{VArray, VObject};

use super::read::{
    ANCHOR_BLOCK_POS, BLOCK_NAME, BLOCK_POS, DIRECTION, DISTANCE_EPSILON, ENTITY_NAME,
    FACING_STRING, MAX_ATTEMPTS, MAX_NUMBER_OF_TICKS, MAX_SEARCH_DISTANCE, NAME, NUM_OF_SHOTS,
    NUM_OF_TICKS, PAYLOAD, Pair, ROOT, SEARCH_RADIUS, SECOND_DIRECTION, SHOULD_BE_FLYING,
    SHOULD_THROW, TARGET_BLOCK_POS, WAIT_DEFAULT_KEY, WAIT_DURATION,
};
use super::{BehaviorTree, Node, Param};
use bedrock_addon_core::diagnostic::{Diagnostic, JsonPath, JsonPathSegment};
use bedrock_addon_core::document::Unknown;
use bedrock_addon_core::json::{
    EncodeError, MAX_JSON_DEPTH, Unrepresentable, Value, WriteStyle, merge_unknown, write_json,
};

impl BehaviorTree {
    /// Writes the tree as JSON text to `out`; on failure nothing is appended.
    pub(super) fn write_into(
        &self,
        out: &mut Vec<u8>,
        style: WriteStyle,
    ) -> Result<(), EncodeError> {
        write_json(&self.to_value(&Unknown::new())?.0, out, style)
    }

    /// [`write_into`](Self::write_into) with the document's unknown members, after the known
    /// members of their object; what could not be written is told.
    pub(super) fn write_into_with(
        &self,
        unknown: &Unknown,
        out: &mut Vec<u8>,
        style: WriteStyle,
    ) -> Result<Vec<Diagnostic>, EncodeError> {
        let (document, told) = self.to_value(unknown)?;
        write_json(&document, out, style)?;
        Ok(told)
    }

    /// The document as a JSON value, with `unknown`'s members. A tree whose text would nest
    /// past the reader's limit is not written: the reader would reject it.
    fn to_value(&self, unknown: &Unknown) -> Result<(Value, Vec<Diagnostic>), EncodeError> {
        let payload_at = JsonPath::of_keys([PAYLOAD]);
        let root_at = payload_at.child(ROOT);
        let too_deep = || EncodeError::NotRepresentable {
            at: root_at.clone(),
            what: Unrepresentable::Unsupported,
        };
        // Before anything recurses: the nodes, walked with an explicit stack. Past this point
        // no walk is deeper than the limit.
        if self.nesting() > MAX_JSON_DEPTH {
            return Err(too_deep());
        }
        let mut payload = VObject::new();
        payload.insert(NAME, self.name.as_str());
        payload.insert(ROOT, node_value(&self.root, &root_at)?);
        let mut root = VObject::new();
        root.insert(PAYLOAD, payload);
        let mut document = root.into_value();
        let unknown = not_written(&document, unknown);
        let told = merge_unknown(&mut document, &unknown);
        // And exactly, with what the members themselves add (an `[x, y, z]`, `data_items`, an
        // unknown member).
        if value_nesting(&document) > MAX_JSON_DEPTH {
            return Err(too_deep());
        }
        Ok((document, told))
    }

    /// The deepest level of arrays and objects among the node bodies of the document as it
    /// would be written: the root object is level 1, the payload 2, the root node's object 3
    /// and its body 4; a decorator's child sits two levels below its node, a composite's
    /// children three.
    fn nesting(&self) -> usize {
        let mut deepest = 2;
        let mut pending = vec![(&self.root, 3_usize)];
        while let Some((node, object)) = pending.pop() {
            let body = object + 1;
            deepest = deepest.max(body);
            if deepest > MAX_JSON_DEPTH {
                // Deep enough to refuse: the rest of the tree need not be walked.
                break;
            }
            let below = match node {
                Node::Sequence { .. } | Node::Selector { .. } => body + 2,
                _ => body + 1,
            };
            pending.extend(node.nodes().iter().map(|node| (node, below)));
        }
        deepest
    }
}

/// The members of `unknown` to write into `document`: each but one named like a member its
/// object already has.
fn not_written(document: &Value, unknown: &Unknown) -> Unknown {
    let mut kept = Unknown::new();
    for (at, key, value) in unknown.iter() {
        let written = object_at(document, at).is_some_and(|object| object.contains_key(key));
        if !written {
            kept.put(at, key, value.clone());
        }
    }
    kept
}

/// The object at `at` in `document`, when there is one.
fn object_at<'v>(document: &'v Value, at: &JsonPath) -> Option<&'v VObject> {
    let mut value = document;
    for segment in at.segments() {
        value = match segment {
            JsonPathSegment::Key(key) => value.as_object()?.get(key)?,
            JsonPathSegment::Index(index) => value.as_array()?.get(index)?,
        };
    }
    value.as_object()
}

/// The deepest level of arrays and objects in `value` (0 for a scalar), without recursion.
fn value_nesting(value: &Value) -> usize {
    let mut deepest = 0;
    let mut pending = vec![(value, 0_usize)];
    while let Some((value, above)) = pending.pop() {
        if let Some(array) = value.as_array() {
            deepest = deepest.max(above + 1);
            pending.extend(array.iter().map(|element| (element, above + 1)));
        } else if let Some(object) = value.as_object() {
            deepest = deepest.max(above + 1);
            pending.extend(object.iter().map(|(_, member)| (member, above + 1)));
        }
    }
    deepest
}

/// A literal as a JSON value; `at` is its path, for a number JSON cannot spell.
trait Written: PartialEq + Default {
    fn value(&self, at: &JsonPath) -> Result<Value, EncodeError>;
}

impl Written for i32 {
    fn value(&self, _: &JsonPath) -> Result<Value, EncodeError> {
        Ok(Value::from(i64::from(*self)))
    }
}

impl Written for bool {
    fn value(&self, _: &JsonPath) -> Result<Value, EncodeError> {
        Ok(Value::from(*self))
    }
}

impl Written for String {
    fn value(&self, _: &JsonPath) -> Result<Value, EncodeError> {
        Ok(Value::from(self.as_str()))
    }
}

impl Written for f32 {
    /// The shortest decimal that reads back to the same `f32`, as a real.
    fn value(&self, at: &JsonPath) -> Result<Value, EncodeError> {
        let what = if self.is_nan() {
            Unrepresentable::NaN
        } else if *self == f32::INFINITY {
            Unrepresentable::Infinity
        } else if *self == f32::NEG_INFINITY {
            Unrepresentable::NegativeInfinity
        } else {
            // `{:?}` of a finite `f32` is a decimal an `f64` parses.
            return Ok(Value::from(
                format!("{self:?}").parse::<f64>().unwrap_or_default(),
            ));
        };
        Err(EncodeError::NotRepresentable {
            at: at.clone(),
            what,
        })
    }
}

impl<T: Written + Copy> Written for [T; 3] {
    fn value(&self, at: &JsonPath) -> Result<Value, EncodeError> {
        let elements = self
            .iter()
            .enumerate()
            .map(|(index, element)| element.value(&at.index(index)))
            .collect::<Result<Vec<Value>, EncodeError>>()?;
        Ok(VArray::from(elements).into())
    }
}

/// Writes `param` as the member of `pair` it reads back from.
fn put<T: Written>(
    body: &mut VObject,
    at: &JsonPath,
    pair: &Pair,
    param: &Param<T>,
) -> Result<(), EncodeError> {
    // The key a read literal has is the pair's preset: empty, or `block_name` for
    // `find_block`'s block name. Under that key the literal is what the file had; a default
    // literal under a preset that is also the default key reads the same as the absent member.
    let read = param.key == pair.preset;
    let absent = read && pair.preset == pair.default_key && param.literal == T::default();
    // A key that is neither is a key member's text, the empty string included (a key member
    // that is no string reads as one). A value built with a literal and an empty key under a
    // preset pair means the literal.
    let built_literal = param.key.is_empty() && param.literal != T::default();
    if absent {
        return Ok(());
    }
    if read || built_literal {
        body.insert(pair.literal, param.literal.value(&at.child(pair.literal))?);
    } else if param.key != pair.default_key {
        body.insert(pair.id, param.key.as_str());
    }
    Ok(())
}

/// A node as its one-member object.
#[allow(clippy::too_many_lines, reason = "one arm per node type")]
fn node_value(node: &Node, at: &JsonPath) -> Result<Value, EncodeError> {
    let kind = node.type_name();
    let at = &at.child(kind);
    let mut body = VObject::new();
    let b = &mut body;
    match node {
        Node::Sequence { children } | Node::Selector { children } => {
            if let Some(children) = children {
                let children_at = at.child("children");
                let nodes = children
                    .iter()
                    .enumerate()
                    .map(|(index, node)| node_value(node, &children_at.index(index)))
                    .collect::<Result<Vec<Value>, EncodeError>>()?;
                b.insert("children", VArray::from(nodes));
            }
        }
        Node::RepeatUntilSuccess {
            child: node,
            max_attempts,
        } => {
            put(b, at, &MAX_ATTEMPTS, max_attempts)?;
            put_child(b, at, node.as_deref())?;
        }
        Node::RepeatUntilFailure { child: node } | Node::Inverter { child: node } => {
            put_child(b, at, node.as_deref())?;
        }
        Node::Subtree {
            subtree_id,
            data_items,
        } => {
            if !subtree_id.as_str().is_empty() {
                b.insert("subtree_id", subtree_id.as_str());
            }
            if !data_items.is_empty() {
                let items: Vec<Value> = data_items
                    .iter()
                    .map(|(from, to)| {
                        if from == to {
                            return Value::from(from.as_str());
                        }
                        let mut item = VObject::new();
                        item.insert("from", from.as_str());
                        item.insert("to", to.as_str());
                        item.into_value()
                    })
                    .collect();
                b.insert("data_items", VArray::from(items));
            }
        }
        Node::ActivateTool { block_pos } | Node::LookAt { block_pos } => {
            put(b, at, &BLOCK_POS, block_pos)?;
        }
        Node::Attack { ticks } | Node::Interact { ticks } | Node::WaitTicks { ticks } => {
            put(b, at, &NUM_OF_TICKS, ticks)?;
        }
        Node::BreakBlock { block_pos, ticks } => {
            put(b, at, &BLOCK_POS, block_pos)?;
            put(b, at, &NUM_OF_TICKS, ticks)?;
        }
        Node::ConsumeItem {} | Node::PlaceBlock {} | Node::UseEntity {} => {}
        Node::FindBlock {
            block_name,
            search_radius,
        } => {
            put(b, at, &BLOCK_NAME, block_name)?;
            put(b, at, &SEARCH_RADIUS, search_radius)?;
        }
        Node::FindEntity {
            entity_name,
            search_radius,
        }
        | Node::LookAtEntity {
            entity_name,
            search_radius,
        } => {
            put(b, at, &ENTITY_NAME, entity_name)?;
            put(b, at, &SEARCH_RADIUS, search_radius)?;
        }
        Node::Fly {
            should_be_flying,
            throw_if_no_change,
        } => {
            put(b, at, &SHOULD_BE_FLYING, should_be_flying)?;
            put(b, at, &SHOULD_THROW, throw_if_no_change)?;
        }
        Node::GetInteractionPositionForBlock {
            target,
            anchor,
            facing,
            max_search_distance,
        } => {
            put(b, at, &TARGET_BLOCK_POS, target)?;
            put(b, at, &ANCHOR_BLOCK_POS, anchor)?;
            put(b, at, &FACING_STRING, facing)?;
            put(b, at, &MAX_SEARCH_DISTANCE, max_search_distance)?;
        }
        Node::Move {
            direction,
            second_direction,
            ticks,
        } => {
            put(b, at, &DIRECTION, direction)?;
            put(b, at, &SECOND_DIRECTION, second_direction)?;
            put(b, at, &MAX_NUMBER_OF_TICKS, ticks)?;
        }
        Node::MoveTo {
            pos,
            distance_epsilon,
        } => {
            put(b, at, &BLOCK_POS, pos)?;
            put(b, at, &DISTANCE_EPSILON, distance_epsilon)?;
        }
        Node::ShootBow { shots } => {
            put(b, at, &NUM_OF_SHOTS, shots)?;
        }
        Node::Wait { seconds } => {
            // The one member is the literal as an integer or the key as a string.
            if seconds.key.is_empty() {
                b.insert(WAIT_DURATION, i64::from(seconds.literal));
            } else if seconds.key != WAIT_DEFAULT_KEY {
                b.insert(WAIT_DURATION, seconds.key.as_str());
            }
        }
    }
    let mut object = VObject::new();
    object.insert(kind, body);
    Ok(object.into_value())
}

/// `child` of a decorator, when it has one.
fn put_child(body: &mut VObject, at: &JsonPath, node: Option<&Node>) -> Result<(), EncodeError> {
    if let Some(node) = node {
        body.insert("child", node_value(node, &at.child("child"))?);
    }
    Ok(())
}
