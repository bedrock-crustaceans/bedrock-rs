//! Curves (`particle_effect/curves`): named functions of a Molang input, each published to the
//! effect's Molang as the variable it is named after.

#![allow(
    clippy::single_match_else,
    reason = "the two arms are the two outcomes of a read: the value, or the line about it"
)]

use indexmap::IndexMap;

use super::codes;
use super::derived::{KEPT, L0, L1, LEGACY, STRICT, entries, legacy, line, note, object, wrong};
use super::read::defaults;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::molang::{Molang, MolangFields, MolangVisitor};
use bedrock_addon_core::schema::{self, FromJson, Schema};

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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, facet::Facet, Schema)]
#[non_exhaustive]
#[repr(u8)]
pub enum SimpleCurveType {
    /// Straight lines between the nodes (JSON `"linear"`): 1 to 100 nodes.
    #[facet(rename = "linear")]
    Linear,
    /// One cubic Bézier segment (JSON `"bezier"`): exactly 4 nodes.
    #[facet(rename = "bezier")]
    Bezier,
    /// A Catmull-Rom spline (JSON `"catmull_rom"`): 4 to 100 nodes.
    #[facet(rename = "catmull_rom")]
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
}

/// A `linear`, `bezier` or `catmull_rom` curve.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
#[facet(
    schema::unknown = "report",
    schema::molang = KEPT,
    schema::after_read = simple_curve
)]
pub struct SimpleCurve {
    /// The kind (JSON `type`, required; [`Curve`]'s read has checked it names one).
    #[facet(
        rename = "type",
        default = SimpleCurveType::Linear,
        schema::era(STRICT, missing = core_codes::MISSING_REQUIRED_FIELD)
    )]
    pub kind: SimpleCurveType,
    /// The value the curve is looked up at (JSON `input`, required).
    #[facet(
        default = Molang::Const(0.0),
        schema::era(LEGACY, missing = codes::REQUIRED_MISSING, molang = L0),
        schema::era(STRICT, missing = core_codes::MISSING_REQUIRED_FIELD)
    )]
    pub input: Molang,
    /// The input range the nodes are spread over, from 0 (JSON `horizontal_range`;
    /// default 1).
    #[facet(schema::era(LEGACY, molang = L1))]
    pub horizontal_range: Option<Molang>,
    /// The control nodes, as written (JSON `nodes`, required).
    #[facet(
        default,
        schema::era(LEGACY, wrong_type = codes::CURVE_NODES, molang = L0),
        schema::era(STRICT, missing = core_codes::MISSING_REQUIRED_FIELD)
    )]
    pub nodes: Vec<Molang>,
    /// Unknown members.
    #[facet(skip_deserializing)]
    pub extra: UnknownMembers,
}

defaults!(SimpleCurve {
    horizontal_range: Molang = "horizontal_range", Molang::Const(1.0);
});

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
fn simple_curve(curve: &mut SimpleCurve, node: &Node<'_>, read: &mut Read<'_>) {
    let listed = node
        .value()
        .as_object()
        .and_then(|members| members.get("nodes"))
        .is_some_and(Value::is_array);
    if listed && !curve.has_valid_node_count() {
        line(read, &node.path().child("nodes"), &codes::CURVE_NODE_COUNT);
    }
}

/// One node of a `bezier_chain` curve: the value and the slope on each side of it.
///
/// JSON: `value` or both of `left_value` and `right_value`; `slope` or both of
/// `left_slope` and `right_slope`.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
#[facet(schema::unknown = "report", schema::after_read = chain_node)]
pub struct BezierChainNode {
    /// The value on both sides (JSON `value`).
    pub value: Option<f32>,
    /// The value left of the node (JSON `left_value`).
    pub left_value: Option<f32>,
    /// The value right of the node (JSON `right_value`).
    pub right_value: Option<f32>,
    /// The slope on both sides (JSON `slope`).
    pub slope: Option<f32>,
    /// The slope left of the node (JSON `left_slope`).
    pub left_slope: Option<f32>,
    /// The slope right of the node (JSON `right_slope`).
    pub right_slope: Option<f32>,
    /// Unknown members.
    #[facet(skip_deserializing)]
    pub extra: UnknownMembers,
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
fn chain_node(node: &mut BezierChainNode, at: &Node<'_>, read: &mut Read<'_>) {
    let both = |shared: Option<f32>, left: Option<f32>, right: Option<f32>| {
        shared.is_some() || (left.is_some() && right.is_some())
    };
    if !both(node.value, node.left_value, node.right_value) {
        line(read, at.path(), &codes::CHAIN_VALUE);
    }
    if !both(node.slope, node.left_slope, node.right_slope) {
        line(read, at.path(), &codes::CHAIN_SLOPE);
    }
}

/// A `bezier_chain` curve.
///
/// JSON: `{"type": "bezier_chain", "input": …, "nodes": {"<position>": {…}}}`.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
#[facet(
    schema::unknown = "report",
    schema::quiet = "type",
    schema::molang = KEPT
)]
pub struct BezierChainCurve {
    /// The value the curve is looked up at (JSON `input`, required).
    #[facet(
        default = Molang::Const(0.0),
        schema::era(LEGACY, missing = codes::REQUIRED_MISSING, molang = L0),
        schema::era(STRICT, missing = core_codes::MISSING_REQUIRED_FIELD)
    )]
    pub input: Molang,
    /// The nodes by their position, a decimal string; the game sorts them by it (JSON
    /// `nodes`, required).
    #[facet(
        default,
        schema::read_with = chain_nodes,
        schema::era(STRICT, missing = core_codes::MISSING_REQUIRED_FIELD)
    )]
    pub nodes: IndexMap<String, BezierChainNode>,
    /// Unknown members.
    #[facet(skip_deserializing)]
    pub extra: UnknownMembers,
}

/// The nodes of a chain, the legacy parser's line the same for the map and for an entry that
/// is no object.
fn chain_nodes(node: &Node<'_>, read: &mut Read<'_>) -> Option<IndexMap<String, BezierChainNode>> {
    let code = Some(&codes::CURVE_NODE_DATA);
    entries(node, read, code, |_, entry, read| object(entry, read, code))
}

/// A curve that is not made: its other members go with it, which the line at the one member
/// does not say. The crate notes it at the curve, at the information level, because the game
/// leaves the curve out too. With `legacy_only`, a 1.20.80 document is refused for this
/// instead.
fn left_out(read: &mut Read<'_>, node: &Node<'_>, legacy_only: bool) -> Option<Curve> {
    if legacy(read) || !legacy_only {
        note(read, node.path(), &core_codes::DROPPED_FIELD);
    }
    None
}

/// The curve its `type` names: the legacy parser makes none of a curve whose nodes it cannot
/// read, and neither reader one of a type it does not know.
impl FromJson for Curve {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let Some(members) = node.value().as_object() else {
            wrong(read, node, Some(&codes::CURVE_NO_DATA));
            return None;
        };
        let at = |key: &str| node.path().child(key);
        let Some(written) = members.get("type") else {
            if legacy(read) {
                line(read, &at("type"), &codes::CURVE_TYPE);
            } else if let Some(members) = read.object(node) {
                read.required(&members, "type");
            }
            return left_out(read, node, true);
        };
        let Some(kind) = written.as_string().map(facet_value::VString::as_str) else {
            wrong(
                read,
                &Node::new(written, at("type")),
                Some(&codes::CURVE_TYPE),
            );
            return left_out(read, node, true);
        };
        let shape = |listed: bool, code, read: &mut Read<'_>| {
            if legacy(read) && !listed {
                line(read, &at("nodes"), code);
                return false;
            }
            true
        };
        let nodes = members.get("nodes");
        if kind == "bezier_chain" {
            if !shape(
                nodes.is_some_and(Value::is_object),
                &codes::CURVE_NODE_DATA,
                read,
            ) {
                return left_out(read, node, true);
            }
            return BezierChainCurve::from_json(node, read).map(Self::BezierChain);
        }
        if SimpleCurveType::from_name(kind).is_none() {
            // A 1.20.80 document logs the same line and leaves the curve out.
            line(read, &at("type"), &codes::CURVE_TYPE);
            return left_out(read, node, false);
        }
        if !shape(
            nodes.is_some_and(Value::is_array),
            &codes::CURVE_NODES,
            read,
        ) {
            return left_out(read, node, true);
        }
        SimpleCurve::from_json(node, read).map(Self::Simple)
    }
}

impl MolangFields for Curve {
    fn visit_molang(&mut self, v: &mut MolangVisitor<'_>) {
        match self {
            Self::Simple(curve) => curve.visit_molang(v),
            Self::BezierChain(curve) => curve.visit_molang(v),
        }
    }
}

bedrock_addon_core::impl_schema_leaf!(Curve);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linear::{ERAS, assert_reads_linear, object_of, reading, root};

    /// The reader of `curves` or `events` alone on a map parsed beforehand, in both eras.
    fn assert_named_map_is_linear<T>(
        key: &'static str,
        prefix: &str,
        entry: &str,
        named: fn(&Node<'_>, &mut Read<'_>) -> Option<IndexMap<String, T>>,
    ) {
        for (era, legacy) in ERAS {
            assert_reads_linear(
                &format!("the map of {key} ({era})"),
                |count| object_of(count, prefix, entry),
                |map, count| {
                    let (read, found) = reading(legacy, |r| named(&root(map), r));
                    assert_eq!(read.as_ref().map(IndexMap::len), Some(count));
                    assert!(found.is_empty());
                    (read, found)
                },
            );
        }
    }

    #[test]
    fn named_map_reads_100_000_curves_in_linear_time() {
        assert_named_map_is_linear(
            "curves",
            "variable.c",
            r#"{"type": "linear", "input": 0, "nodes": [0, 1]}"#,
            crate::read::curves,
        );
    }

    #[test]
    fn named_map_reads_100_000_events_in_linear_time() {
        assert_named_map_is_linear("events", "e", r#"{"log": "x"}"#, crate::read::events);
    }
}
