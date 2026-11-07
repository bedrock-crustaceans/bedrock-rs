//! Curves (`particle_effect/curves`): named functions of a Molang input, each published to the
//! effect's Molang as the variable it is named after.

#![allow(
    clippy::single_match_else,
    reason = "the two arms are the two outcomes of a read: the value, or the line about it"
)]

use facet_value::VObject;
use indexmap::IndexMap;

use super::codes;
use super::read::{Era, Fields, Members, Node, Opt, Reader, Writer, object, put, req, walk};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{CodeDef, DiagnosticCode, Severity};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::json::{EncodeError, Value};
use bedrock_addon_core::molang::{Molang, MolangVisitor};

/// One curve of an effect, keyed in `curves` by the Molang variable it sets
/// (`"variable.fade"`).
///
/// JSON: an object whose `type` is `linear`, `bezier`, `catmull_rom` or `bezier_chain`.
#[derive(Clone, Debug, PartialEq)]
pub enum Curve {
    /// A `linear`, `bezier` or `catmull_rom` curve: a list of nodes over a range.
    Simple(SimpleCurve),
    /// A `bezier_chain` curve: nodes at positions, each with values and slopes.
    BezierChain(BezierChainCurve),
}

/// The kind of a [`SimpleCurve`] (JSON `type`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SimpleCurveType {
    /// Straight lines between the nodes (JSON `"linear"`): 1 to 100 nodes.
    Linear,
    /// One cubic Bézier segment (JSON `"bezier"`): exactly 4 nodes.
    Bezier,
    /// A Catmull-Rom spline (JSON `"catmull_rom"`): 4 to 100 nodes.
    CatmullRom,
}

impl SimpleCurveType {
    /// The JSON spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Linear => "linear",
            Self::Bezier => "bezier",
            Self::CatmullRom => "catmull_rom",
        }
    }

    /// The fewest and the most nodes the game takes for the kind.
    pub fn node_limits(self) -> (usize, usize) {
        match self {
            Self::Linear => (1, 100),
            Self::Bezier => (4, 4),
            Self::CatmullRom => (4, 100),
        }
    }

    fn from_name(text: &str) -> Option<Self> {
        [Self::Linear, Self::Bezier, Self::CatmullRom]
            .into_iter()
            .find(|kind| kind.as_str() == text)
    }
}

impl Node for SimpleCurveType {
    fn read(value: &Value, _: &mut Reader<'_>, _: Opt) -> Option<Self> {
        // `Curve::read` has checked the member.
        value
            .as_string()
            .and_then(|text| Self::from_name(text.as_str()))
    }

    fn fallback(_: Opt) -> Self {
        Self::Linear
    }

    fn write(&self, _: &mut Writer) -> Result<Value, EncodeError> {
        Ok(Value::from(self.as_str()))
    }
}

object! {
    /// A `linear`, `bezier` or `catmull_rom` curve.
    pub struct SimpleCurve finish = simple_curve {
        /// The kind (JSON `type`, required).
        req kind: SimpleCurveType = "type", Opt::NONE;
        /// The value the curve is looked up at (JSON `input`, required).
        req input: Molang = "input", Opt::required(0.0);
        /// The input range the nodes are spread over, from 0 (JSON `horizontal_range`;
        /// default 1).
        opt horizontal_range: Molang = "horizontal_range", Opt::with_default(1.0),
            default Molang::Const(1.0);
        /// The control nodes, as written (JSON `nodes`, required).
        req nodes: Vec<Molang> = "nodes", Opt::NONE.wrong(&codes::CURVE_NODES);
    }
}

impl SimpleCurve {
    /// Whether the number of nodes is one the game takes for the kind.
    pub fn has_valid_node_count(&self) -> bool {
        let (least, most) = self.kind.node_limits();
        (least..=most).contains(&self.nodes.len())
    }

    /// The nodes the game uses: the written ones, and after a count it does not take
    /// ("Invalid number of control nodes in curve!  Defaulting curve to 0.0") as many nodes
    /// of 0 as the kind needs at least, appended to them.
    pub fn effective_nodes(&self) -> Vec<Molang> {
        let mut nodes = self.nodes.clone();
        if !self.has_valid_node_count() {
            let (least, _) = self.kind.node_limits();
            nodes.extend(std::iter::repeat_n(Molang::Const(0.0), least));
        }
        nodes
    }
}

/// The node count of the kind, in both eras.
fn simple_curve(curve: &mut SimpleCurve, m: Members<'_>, r: &mut Reader<'_>) {
    let listed = m.get("nodes").is_some_and(Value::is_array);
    if listed && !curve.has_valid_node_count() {
        r.key("nodes", |r| r.warn(&codes::CURVE_NODE_COUNT));
    }
}

object! {
    /// One node of a `bezier_chain` curve: the value and the slope on each side of it.
    ///
    /// JSON: `value` or both of `left_value` and `right_value`; `slope` or both of
    /// `left_slope` and `right_slope`.
    pub struct BezierChainNode finish = chain_node {
        /// The value on both sides (JSON `value`).
        opt value: f32 = "value", Opt::NONE;
        /// The value left of the node (JSON `left_value`).
        opt left_value: f32 = "left_value", Opt::NONE;
        /// The value right of the node (JSON `right_value`).
        opt right_value: f32 = "right_value", Opt::NONE;
        /// The slope on both sides (JSON `slope`).
        opt slope: f32 = "slope", Opt::NONE;
        /// The slope left of the node (JSON `left_slope`).
        opt left_slope: f32 = "left_slope", Opt::NONE;
        /// The slope right of the node (JSON `right_slope`).
        opt right_slope: f32 = "right_slope", Opt::NONE;
    }
}

impl BezierChainNode {
    /// The values left and right of the node: the one-sided member, else `value`, else 0.
    pub fn values(&self) -> (f32, f32) {
        let shared = self.value.unwrap_or(0.0);
        (
            self.left_value.unwrap_or(shared),
            self.right_value.unwrap_or(shared),
        )
    }

    /// The slopes left and right of the node: the one-sided member, else `slope`, else 0.
    pub fn slopes(&self) -> (f32, f32) {
        let shared = self.slope.unwrap_or(0.0);
        (
            self.left_slope.unwrap_or(shared),
            self.right_slope.unwrap_or(shared),
        )
    }
}

/// Each side needs its value and its slope, shared or one-sided.
fn chain_node(node: &mut BezierChainNode, _: Members<'_>, r: &mut Reader<'_>) {
    let both = |shared: Option<f32>, left: Option<f32>, right: Option<f32>| {
        shared.is_some() || (left.is_some() && right.is_some())
    };
    if !both(node.value, node.left_value, node.right_value) {
        r.warn(&codes::CHAIN_VALUE);
    }
    if !both(node.slope, node.left_slope, node.right_slope) {
        r.warn(&codes::CHAIN_SLOPE);
    }
}

/// A `bezier_chain` curve.
///
/// JSON: `{"type": "bezier_chain", "input": …, "nodes": {"<position>": {…}}}`.
#[derive(Clone, Debug, PartialEq)]
pub struct BezierChainCurve {
    /// The value the curve is looked up at (JSON `input`, required).
    pub input: Molang,
    /// The nodes by their position, a decimal string; the game sorts them by it (JSON
    /// `nodes`, required).
    pub nodes: IndexMap<String, BezierChainNode>,
    /// Unknown members.
    pub extra: UnknownMembers,
}

const CHAIN_INPUT: Opt = Opt::required(0.0);
const CHAIN_NODES: Opt = Opt::NONE.wrong(&codes::CURVE_NODE_DATA);

impl Fields for BezierChainCurve {
    fn known(key: &str) -> bool {
        matches!(key, "type" | "input" | "nodes")
    }

    fn read_members(m: Members<'_>, r: &mut Reader<'_>) -> Self {
        Self {
            input: req(m, "input", CHAIN_INPUT, r),
            nodes: req(m, "nodes", CHAIN_NODES, r),
            extra: m.extra(Self::known, r),
        }
    }

    fn write_members(&self, out: &mut VObject, w: &mut Writer) -> Result<(), EncodeError> {
        out.insert("type", Value::from("bezier_chain"));
        put(out, "input", &self.input, w)?;
        put(out, "nodes", &self.nodes, w)?;
        w.extra(out, &self.extra, Self::known)
    }

    fn walk_members(&mut self, era: Era, v: &mut MolangVisitor<'_>) {
        walk("input", &mut self.input, era, CHAIN_INPUT, v);
    }
}

/// A curve that is not made: its other members go with it, which the line at the one member
/// does not say. The crate notes it at the curve, at the information level, because the game
/// leaves the curve out too. With `legacy_only`, a 1.20.80 document is refused for this
/// instead.
fn left_out(r: &mut Reader<'_>, legacy_only: bool) -> Option<Curve> {
    if r.legacy() || !legacy_only {
        r.push(
            DiagnosticCode::from(&core_codes::DROPPED_FIELD),
            Severity::Info,
        );
    }
    None
}

impl Node for Curve {
    fn read(value: &Value, r: &mut Reader<'_>, _: Opt) -> Option<Self> {
        let Some(members) = Members::of(value) else {
            r.wrong_type(value, Some(&codes::CURVE_NO_DATA));
            return None;
        };
        let Some(written) = members.get("type") else {
            r.missing(members, "type", Some(&codes::CURVE_TYPE));
            return left_out(r, true);
        };
        let Some(kind) = written.as_string().map(facet_value::VString::as_str) else {
            r.key("type", |r| r.wrong_type(written, Some(&codes::CURVE_TYPE)));
            return left_out(r, true);
        };
        // A curve whose nodes the legacy parser cannot read is not made.
        let legacy_shape = |nodes_ok: bool, text, r: &mut Reader<'_>| {
            if r.legacy() && !nodes_ok {
                r.key("nodes", |r| r.warn(text));
                return false;
            }
            true
        };
        let nodes = members.get("nodes");
        if kind == "bezier_chain" {
            let listed = nodes.is_some_and(Value::is_object);
            if !legacy_shape(listed, &codes::CURVE_NODE_DATA, r) {
                return left_out(r, true);
            }
            return Some(Self::BezierChain(BezierChainCurve::read_members(
                members, r,
            )));
        }
        if SimpleCurveType::from_name(kind).is_none() {
            // A 1.20.80 document logs the same line and leaves the curve out.
            r.key("type", |r| r.warn(&codes::CURVE_TYPE));
            return left_out(r, false);
        }
        if !legacy_shape(nodes.is_some_and(Value::is_array), &codes::CURVE_NODES, r) {
            return left_out(r, true);
        }
        Some(Self::Simple(SimpleCurve::read_members(members, r)))
    }

    fn fallback(o: Opt) -> Self {
        Self::Simple(SimpleCurve::fallback(o))
    }

    fn write(&self, w: &mut Writer) -> Result<Value, EncodeError> {
        match self {
            Self::Simple(curve) => curve.write(w),
            Self::BezierChain(curve) => curve.write(w),
        }
    }

    fn walk(&mut self, era: Era, o: Opt, v: &mut MolangVisitor<'_>) {
        match self {
            Self::Simple(curve) => curve.walk(era, o, v),
            Self::BezierChain(curve) => curve.walk(era, o, v),
        }
    }
}

/// Reads `curves` or `events`: a map whose entries are read one by one; an entry without a
/// name is left out with `unnamed`.
pub(crate) fn named_map<T: Node>(
    m: Members<'_>,
    key: &str,
    not_object: &'static CodeDef<0>,
    unnamed: &'static CodeDef<0>,
    r: &mut Reader<'_>,
) -> Option<IndexMap<String, T>> {
    let value = m.get(key)?;
    r.key(key, |r| {
        let Some(entries) = Members::of(value) else {
            r.wrong_type(value, Some(not_object));
            return None;
        };
        let mut read = IndexMap::new();
        for (name, entry) in entries.iter() {
            if name.is_empty() && r.legacy() {
                r.key(name, |r| r.warn(unnamed));
                continue;
            }
            if let Some(entry) = r.key(name, |r| T::read(entry, r, Opt::NONE)) {
                read.insert(name.to_owned(), entry);
            }
        }
        Some(read)
    })
}

/// A member that must be an object in a 1.20.80 document, for the callers that read its
/// entries themselves (the 1.20.80 reader alone).
pub(crate) fn object_member<'v>(
    m: Members<'v>,
    key: &str,
    r: &mut Reader<'_>,
) -> Option<Members<'v>> {
    let Some(value) = m.get(key) else {
        r.missing(m, key, None);
        return None;
    };
    let members = Members::of(value);
    if members.is_none() {
        r.key(key, |r| r.wrong_type(value, None));
    }
    members
}

#[cfg(test)]
mod tests {
    use facet_value::VObject;

    use super::*;
    use crate::event::EventNode;
    use crate::linear::{assert_reads_linear, object_of, reading};

    /// `{"<key>": {"<prefix>0": <entry>, …}}` with `count` entries: the payload `named_map`
    /// reads `key` of.
    fn payload(key: &str, count: usize, prefix: &str, entry: &str) -> Value {
        let mut payload = VObject::new();
        payload.insert(key, object_of(count, prefix, entry));
        payload.into()
    }

    /// `named_map` alone on a payload parsed beforehand, in both eras.
    fn assert_named_map_is_linear<T: Node>(key: &'static str, prefix: &str, entry: &str) {
        for era in [Era::Legacy, Era::Cereal] {
            assert_reads_linear(
                &format!("named_map of {key} ({era:?})"),
                |count| payload(key, count, prefix, entry),
                |payload, count| {
                    let members = Members::of(payload).expect("an object");
                    let (read, found) = reading(era, |r| {
                        named_map::<T>(
                            members,
                            key,
                            &codes::NO_CURVES,
                            &codes::CURVE_WITHOUT_NAME,
                            r,
                        )
                    });
                    assert_eq!(read.as_ref().map(IndexMap::len), Some(count));
                    assert!(found.is_empty());
                    (read, found)
                },
            );
        }
    }

    #[test]
    fn named_map_reads_100_000_curves_in_linear_time() {
        assert_named_map_is_linear::<Curve>(
            "curves",
            "variable.c",
            r#"{"type": "linear", "input": 0, "nodes": [0, 1]}"#,
        );
    }

    #[test]
    fn named_map_reads_100_000_events_in_linear_time() {
        assert_named_map_is_linear::<EventNode>("events", "e", r#"{"log": "x"}"#);
    }
}
