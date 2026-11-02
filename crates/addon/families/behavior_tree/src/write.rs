//! Writing a tree back as JSON: the name and the root node, each parameter in the form it
//! reads back from, unknown members after the known ones of their level (the Value level).
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

use super::parse::{
    ANCHOR_BLOCK_POS, BLOCK_NAME, BLOCK_POS, DIRECTION, DISTANCE_EPSILON, ENTITY_NAME,
    FACING_STRING, MAX_ATTEMPTS, MAX_NUMBER_OF_TICKS, MAX_SEARCH_DISTANCE, NAME, NUM_OF_SHOTS,
    NUM_OF_TICKS, PAYLOAD, Pair, ROOT, SEARCH_RADIUS, SECOND_DIRECTION, SHOULD_BE_FLYING,
    SHOULD_THROW, TARGET_BLOCK_POS, WAIT_DEFAULT_KEY, WAIT_DURATION, child, element_of,
};
use super::{BehaviorTree, Node, Param};
use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::{
    EncodeError, MAX_JSON_DEPTH, Unrepresentable, Value, WriteStyle, write_json,
};

impl BehaviorTree {
    /// Writes the tree as JSON text to `out`; on failure nothing is appended.
    pub(super) fn write_into(
        &self,
        out: &mut Vec<u8>,
        style: WriteStyle,
    ) -> Result<(), EncodeError> {
        write_json(&self.to_value()?, out, style)
    }

    /// The document as a JSON value. A tree whose text would nest past the reader's limit is
    /// not written: the reader would reject it.
    fn to_value(&self) -> Result<Value, EncodeError> {
        let payload_at = child(&JsonPath::new(), PAYLOAD);
        let root_at = child(&payload_at, ROOT);
        let too_deep = || EncodeError::NotRepresentable {
            at: root_at.clone(),
            what: Unrepresentable::Unsupported,
        };
        // Before anything recurses: the nodes and the unknown members, walked with explicit
        // stacks. Past this point no walk is deeper than the limit.
        if self.nesting() > MAX_JSON_DEPTH {
            return Err(too_deep());
        }
        let mut payload = VObject::new();
        payload.insert(NAME, self.name.as_str());
        payload.insert(ROOT, node_value(&self.root, &root_at)?);
        extend(&mut payload, &self.extra);
        let mut root = VObject::new();
        root.insert(PAYLOAD, payload);
        extend(&mut root, &self.root_extra);
        let document = root.into_value();
        // And exactly, with what the members themselves add (an `[x, y, z]`, `data_items`).
        if value_nesting(&document) > MAX_JSON_DEPTH {
            return Err(too_deep());
        }
        Ok(document)
    }

    /// The deepest level of arrays and objects among the node bodies and the unknown members
    /// of the document as it would be written: the root object is level 1, the payload 2, the
    /// root node's object 3 and its body 4; a decorator's child sits two levels below its
    /// node, a composite's children three.
    fn nesting(&self) -> usize {
        let extras = |extra: &Extra, level: usize| {
            extra
                .iter()
                .map(|(_, value)| level + value_nesting(value))
                .max()
                .unwrap_or(level)
        };
        let mut deepest = extras(&self.root_extra, 1).max(extras(&self.extra, 2));
        let mut pending = vec![(&self.root, 3_usize)];
        while let Some((node, object)) = pending.pop() {
            let body = object + 1;
            deepest = deepest.max(extras(node.extra(), body));
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

/// The unknown members of a level, after its known ones; one named like a member already
/// written is left out.
fn extend(object: &mut VObject, extra: &Extra) {
    for (key, value) in extra.iter() {
        if !object.contains_key(key) {
            object.insert(key, value.clone());
        }
    }
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
            .map(|(index, element)| element.value(&element_of(at, index)))
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
        body.insert(pair.literal, param.literal.value(&child(at, pair.literal))?);
    } else if param.key != pair.default_key {
        body.insert(pair.id, param.key.as_str());
    }
    Ok(())
}

/// A node as its one-member object.
#[allow(clippy::too_many_lines, reason = "one arm per node type")]
fn node_value(node: &Node, at: &JsonPath) -> Result<Value, EncodeError> {
    let kind = node.type_name();
    let at = &child(at, kind);
    let mut body = VObject::new();
    let b = &mut body;
    let extra = match node {
        Node::Sequence { children, extra } | Node::Selector { children, extra } => {
            if let Some(children) = children {
                let children_at = child(at, "children");
                let nodes = children
                    .iter()
                    .enumerate()
                    .map(|(index, node)| node_value(node, &element_of(&children_at, index)))
                    .collect::<Result<Vec<Value>, EncodeError>>()?;
                b.insert("children", VArray::from(nodes));
            }
            extra
        }
        Node::RepeatUntilSuccess {
            child: node,
            max_attempts,
            extra,
        } => {
            put(b, at, &MAX_ATTEMPTS, max_attempts)?;
            put_child(b, at, node.as_deref())?;
            extra
        }
        Node::RepeatUntilFailure { child: node, extra } | Node::Inverter { child: node, extra } => {
            put_child(b, at, node.as_deref())?;
            extra
        }
        Node::Subtree {
            subtree_id,
            data_items,
            extra,
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
            extra
        }
        Node::ActivateTool { block_pos, extra } | Node::LookAt { block_pos, extra } => {
            put(b, at, &BLOCK_POS, block_pos)?;
            extra
        }
        Node::Attack { ticks, extra }
        | Node::Interact { ticks, extra }
        | Node::WaitTicks { ticks, extra } => {
            put(b, at, &NUM_OF_TICKS, ticks)?;
            extra
        }
        Node::BreakBlock {
            block_pos,
            ticks,
            extra,
        } => {
            put(b, at, &BLOCK_POS, block_pos)?;
            put(b, at, &NUM_OF_TICKS, ticks)?;
            extra
        }
        Node::ConsumeItem { extra } | Node::PlaceBlock { extra } | Node::UseEntity { extra } => {
            extra
        }
        Node::FindBlock {
            block_name,
            search_radius,
            extra,
        } => {
            put(b, at, &BLOCK_NAME, block_name)?;
            put(b, at, &SEARCH_RADIUS, search_radius)?;
            extra
        }
        Node::FindEntity {
            entity_name,
            search_radius,
            extra,
        }
        | Node::LookAtEntity {
            entity_name,
            search_radius,
            extra,
        } => {
            put(b, at, &ENTITY_NAME, entity_name)?;
            put(b, at, &SEARCH_RADIUS, search_radius)?;
            extra
        }
        Node::Fly {
            should_be_flying,
            throw_if_no_change,
            extra,
        } => {
            put(b, at, &SHOULD_BE_FLYING, should_be_flying)?;
            put(b, at, &SHOULD_THROW, throw_if_no_change)?;
            extra
        }
        Node::GetInteractionPositionForBlock {
            target,
            anchor,
            facing,
            max_search_distance,
            extra,
        } => {
            put(b, at, &TARGET_BLOCK_POS, target)?;
            put(b, at, &ANCHOR_BLOCK_POS, anchor)?;
            put(b, at, &FACING_STRING, facing)?;
            put(b, at, &MAX_SEARCH_DISTANCE, max_search_distance)?;
            extra
        }
        Node::Move {
            direction,
            second_direction,
            ticks,
            extra,
        } => {
            put(b, at, &DIRECTION, direction)?;
            put(b, at, &SECOND_DIRECTION, second_direction)?;
            put(b, at, &MAX_NUMBER_OF_TICKS, ticks)?;
            extra
        }
        Node::MoveTo {
            pos,
            distance_epsilon,
            extra,
        } => {
            put(b, at, &BLOCK_POS, pos)?;
            put(b, at, &DISTANCE_EPSILON, distance_epsilon)?;
            extra
        }
        Node::ShootBow { shots, extra } => {
            put(b, at, &NUM_OF_SHOTS, shots)?;
            extra
        }
        Node::Wait { seconds, extra } => {
            // The one member is the literal as an integer or the key as a string.
            if seconds.key.is_empty() {
                b.insert(WAIT_DURATION, i64::from(seconds.literal));
            } else if seconds.key != WAIT_DEFAULT_KEY {
                b.insert(WAIT_DURATION, seconds.key.as_str());
            }
            extra
        }
    };
    extend(&mut body, extra);
    let mut object = VObject::new();
    object.insert(kind, body);
    Ok(object.into_value())
}

/// `child` of a decorator, when it has one.
fn put_child(body: &mut VObject, at: &JsonPath, node: Option<&Node>) -> Result<(), EncodeError> {
    if let Some(node) = node {
        body.insert("child", node_value(node, &child(at, "child"))?);
    }
    Ok(())
}
