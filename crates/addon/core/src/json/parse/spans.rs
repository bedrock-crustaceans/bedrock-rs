//! The tree of spans [`parse`](super::parse) builds beside the value: where each value, member
//! key and element of the text lies.

use std::borrow::Cow;

use super::fault::to_u32;
use crate::diagnostic::{JsonPath, JsonPathSegment, Span};
use crate::json::JsonSlice;

/// What a node of the tree is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpanKind {
    /// A string, a number, `true`, `false` or `null`.
    Scalar,
    /// An array.
    Array,
    /// An object.
    Object,
}

/// What a number token is that a reader may have to tell of (see [`NumberNote`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum NumberKind {
    /// A token no double holds (`1e999`, an integer of more than 309 digits): the value is
    /// an infinity, of the token's sign.
    BeyondDouble,
    /// An integer token past 64 bits that a double holds: the value is the double it spells,
    /// as vanilla's readers read it.
    WideInteger,
    /// `-0`, `-0.0`, `-0e0`: the value is `0` for `-0`, which has no negative zero, and `-0.0`
    /// for the others; the text is at the token's span.
    NegativeZero,
}

/// A number token of the text that a reader may have to tell of, found in the one pass.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NumberNote {
    /// Where the number is, from the root.
    pub path: JsonPath,
    /// The token, in the file (the slice's offset added).
    pub span: Span,
    /// What it is.
    pub kind: NumberKind,
}

/// One node: a value of the text.
#[derive(Clone, Copy, Debug)]
pub(super) struct Node {
    pub(super) offset: u32,
    pub(super) len: u32,
    pub(super) kind: SpanKind,
    pub(super) number: Option<NumberKind>,
    /// The first of its children in `Spans::edges`.
    pub(super) first: u32,
    /// How many children.
    pub(super) count: u32,
}

/// One member or element: the key's token (empty for an element) and the value's node.
#[derive(Clone, Copy, Debug)]
pub(super) struct Edge {
    pub(super) key_offset: u32,
    pub(super) key_len: u32,
    pub(super) node: u32,
}

/// The spans of a parsed text, a tree that mirrors its [`Value`](crate::json::Value).
///
/// Every span is a byte range of the text as given, with the offset of the slice it was read
/// from added ([`parse_slice`](super::parse_slice)): a value's span runs from its first byte to
/// its last (a string with its quotes, an object with its braces), white space and comments
/// outside it.
///
/// Which copy of a key written twice the tree holds is the [`Winner`](crate::json::Winner)
/// the text was parsed with: `First` the first, `Last` the last (and the member stands where
/// that copy does), `Keep` every copy in the file's order. [`SpanNode::member`] is the last
/// copy of a key whatever the rule, as [`span_at`](crate::json::span_at) takes it.
#[derive(Clone, Debug)]
pub struct Spans<'t> {
    text: &'t str,
    base: u32,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl<'t> Spans<'t> {
    pub(super) fn new(text: &'t str, base: usize, nodes: Vec<Node>, edges: Vec<Edge>) -> Self {
        Self {
            text,
            base: to_u32(base),
            nodes,
            edges,
        }
    }

    /// The root value.
    pub fn root(&self) -> SpanNode<'_, 't> {
        SpanNode {
            spans: self,
            index: 0,
        }
    }

    /// The whole text: what [`span_at`](crate::json::span_at) gives for the empty path.
    pub fn whole(&self) -> Span {
        Span {
            offset: self.base,
            len: to_u32(self.text.len()),
        }
    }

    /// The whole text, at the file offset it was read at.
    pub fn text(&self) -> JsonSlice<'t> {
        JsonSlice {
            text: self.text,
            offset: self.base as usize,
        }
    }

    /// The span of the value `path` names, as [`span_at`](crate::json::span_at) gives it on
    /// the text: the whole text for the empty path, `None` when a step of the path is not
    /// in the text (a member that is not there, an index past the end, a step into a value
    /// that is no container of that kind).
    pub fn at(&self, path: &JsonPath) -> Option<Span> {
        let mut at: Option<SpanNode<'_, 't>> = None;
        for segment in path.segments() {
            at = Some(at.unwrap_or_else(|| self.root()).child(segment)?);
        }
        Some(at.map_or_else(|| self.whole(), |node| node.span()))
    }

    /// The span of the value `path` names, or of the deepest value on the way when a step
    /// cannot be taken, or the whole text: [`span_reached`](crate::json::span_reached).
    pub fn reached(&self, path: &JsonPath) -> Span {
        let mut at: Option<SpanNode<'_, 't>> = None;
        for segment in path.segments() {
            match at.unwrap_or_else(|| self.root()).child(segment) {
                Some(next) => at = Some(next),
                None => break,
            }
        }
        at.map_or_else(|| self.whole(), |node| node.span())
    }

    /// The number tokens of the tree that [`NumberNote`] tells of, in the order of the text.
    pub(super) fn notes(&self) -> Vec<NumberNote> {
        let mut notes = Vec::new();
        let mut path = JsonPath::new();
        // The nodes still to visit, each with the segment that leads to it; `None` closes the
        // segment of the node visited before.
        let mut open: Vec<Option<(u32, Option<Step>)>> = vec![Some((0, None))];
        while let Some(step) = open.pop() {
            let Some((index, segment)) = step else {
                path.pop();
                continue;
            };
            let node = self.nodes[index as usize];
            let node_ref = SpanNode { spans: self, index };
            match segment {
                Some(Step::Key(edge)) => path.push(&node_ref.spans.key_of(edge)),
                Some(Step::Index(i)) => path.push_index(i),
                None => {}
            }
            if segment.is_some() {
                open.push(None);
            }
            if let Some(kind) = node.number {
                notes.push(NumberNote {
                    path: path.clone(),
                    span: node_ref.span(),
                    kind,
                });
            }
            let first = node.first as usize;
            let children = &self.edges[first..first + node.count as usize];
            // Pushed last to first, so the first child is visited first.
            for (i, edge) in children.iter().enumerate().rev() {
                let step = match node.kind {
                    SpanKind::Object => Step::Key(*edge),
                    SpanKind::Array => Step::Index(i),
                    SpanKind::Scalar => continue,
                };
                open.push(Some((edge.node, Some(step))));
            }
        }
        notes
    }

    /// The key of `edge`, as the text spells it, unescaped.
    fn key_of(&self, edge: Edge) -> Cow<'t, str> {
        let from = (edge.key_offset as usize + 1).min(self.text.len());
        let to = (edge.key_offset as usize + edge.key_len as usize)
            .saturating_sub(1)
            .max(from)
            .min(self.text.len());
        let raw = self.text.get(from..to).unwrap_or_default();
        if raw.contains('\\') {
            Cow::Owned(super::token::unescape(raw))
        } else {
            Cow::Borrowed(raw)
        }
    }

    fn span_of(&self, node: Node) -> Span {
        Span {
            offset: self.base.saturating_add(node.offset),
            len: node.len,
        }
    }
}

/// How the walk of [`Spans::notes`] came to a node.
#[derive(Clone, Copy)]
enum Step {
    Key(Edge),
    Index(usize),
}

/// A node of a [`Spans`] tree: one value of the text, and below it its members or elements.
#[derive(Clone, Copy, Debug)]
pub struct SpanNode<'s, 't> {
    spans: &'s Spans<'t>,
    index: u32,
}

/// One member of an object, as [`SpanNode::members`] lists it.
#[derive(Clone, Debug)]
pub struct SpanMember<'s, 't> {
    /// The key as the text spells it, unescaped.
    pub key: Cow<'t, str>,
    /// The key's token, quotes included.
    pub key_span: Span,
    /// The value.
    pub value: SpanNode<'s, 't>,
}

impl<'s, 't> SpanNode<'s, 't> {
    fn node(&self) -> Node {
        self.spans.nodes[self.index as usize]
    }

    fn children(&self) -> &'s [Edge] {
        let node = self.node();
        let first = node.first as usize;
        &self.spans.edges[first..first + node.count as usize]
    }

    fn at(&self, edge: Edge) -> SpanNode<'s, 't> {
        SpanNode {
            spans: self.spans,
            index: edge.node,
        }
    }

    /// The value's span.
    pub fn span(&self) -> Span {
        self.spans.span_of(self.node())
    }

    /// The value's token as the text spells it, with its file offset.
    pub fn text(&self) -> JsonSlice<'t> {
        let node = self.node();
        let from = node.offset as usize;
        JsonSlice {
            text: self
                .spans
                .text
                .get(from..from + node.len as usize)
                .unwrap_or_default(),
            offset: self.span().offset as usize,
        }
    }

    /// Whether the value is a scalar, an array or an object.
    pub fn kind(&self) -> SpanKind {
        self.node().kind
    }

    /// What a number token is that a reader may have to tell of; `None` for a value that is
    /// no such number.
    pub fn number(&self) -> Option<NumberKind> {
        self.node().number
    }

    /// How many members or elements the tree holds for the value (with a repeated key kept
    /// by `Winner::Keep`, each copy); 0 for a scalar.
    pub fn len(&self) -> usize {
        self.node().count as usize
    }

    /// Whether the value has no member or element.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The last member of an object that has `key`; `None` when the value is no object or has
    /// none.
    pub fn member(&self, key: &str) -> Option<SpanNode<'s, 't>> {
        if self.kind() != SpanKind::Object {
            return None;
        }
        self.children()
            .iter()
            .rev()
            .find(|edge| self.spans.key_of(**edge) == key)
            .map(|edge| self.at(*edge))
    }

    /// The span of the key token of the last member that has `key`.
    pub fn key_span(&self, key: &str) -> Option<Span> {
        if self.kind() != SpanKind::Object {
            return None;
        }
        self.children()
            .iter()
            .rev()
            .find(|edge| self.spans.key_of(**edge) == key)
            .map(|edge| self.spans.key_span_of(*edge))
    }

    /// The element of an array at `index`; `None` when the value is no array or is shorter.
    pub fn element(&self, index: usize) -> Option<SpanNode<'s, 't>> {
        if self.kind() != SpanKind::Array {
            return None;
        }
        self.children().get(index).map(|edge| self.at(*edge))
    }

    /// The member at `index` in the file's order; `None` when the value is no object or has
    /// fewer.
    pub fn member_at(&self, index: usize) -> Option<SpanMember<'s, 't>> {
        if self.kind() != SpanKind::Object {
            return None;
        }
        let edge = *self.children().get(index)?;
        Some(SpanMember {
            key: self.spans.key_of(edge),
            key_span: self.spans.key_span_of(edge),
            value: self.at(edge),
        })
    }

    /// The members of an object in the file's order (none for any other value).
    pub fn members(&self) -> impl Iterator<Item = SpanMember<'s, 't>> + use<'s, 't> {
        let object = self.kind() == SpanKind::Object;
        let node = *self;
        let edges: &'s [Edge] = if object { self.children() } else { &[] };
        edges.iter().map(move |edge| SpanMember {
            key: node.spans.key_of(*edge),
            key_span: node.spans.key_span_of(*edge),
            value: node.at(*edge),
        })
    }

    /// The elements of an array in order (none for any other value).
    pub fn elements(&self) -> impl Iterator<Item = SpanNode<'s, 't>> + use<'s, 't> {
        let array = self.kind() == SpanKind::Array;
        let node = *self;
        let edges: &'s [Edge] = if array { self.children() } else { &[] };
        edges.iter().map(move |edge| node.at(*edge))
    }

    /// The node one step of a path leads to.
    pub fn child(&self, segment: JsonPathSegment<'_>) -> Option<SpanNode<'s, 't>> {
        match segment {
            JsonPathSegment::Key(key) => self.member(key),
            JsonPathSegment::Index(index) => self.element(index),
        }
    }
}

impl Spans<'_> {
    fn key_span_of(&self, edge: Edge) -> Span {
        Span {
            offset: self.base.saturating_add(edge.key_offset),
            len: edge.key_len,
        }
    }
}
