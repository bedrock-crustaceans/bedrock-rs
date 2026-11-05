//! Redstone: `redstone_conductivity`, `redstone_consumer`, `redstone_producer`.

use facet::Facet;

use super::types::Facing;
use bedrock_addon_core::schema::{self, Schema};

/// `minecraft:redstone_conductivity`: whether the block conducts redstone power.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct RedstoneConductivity {
    /// JSON `redstone_conductor`; `false` when absent.
    #[facet(schema::fallback = false)]
    pub redstone_conductor: Option<bool>,
    /// JSON `allows_wire_to_step_down`; `true` when absent.
    #[facet(schema::fallback = true)]
    pub allows_wire_to_step_down: Option<bool>,
}

/// `minecraft:redstone_consumer`: the block takes redstone power (custom components see it).
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct RedstoneConsumer {
    /// The lowest power it reacts to (JSON `min_power`, `0..=15`); 0 when absent.
    #[facet(schema::fallback = 0, schema::max = 15)]
    pub min_power: Option<u8>,
    /// Passes the power on (JSON `propagates_power`); `false` when absent.
    #[facet(schema::fallback = false)]
    pub propagates_power: Option<bool>,
}

/// `minecraft:redstone_producer`: the block gives off redstone power.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct RedstoneProducer {
    /// The power level (JSON `power`, required, `0..=15`).
    #[facet(schema::max = 15)]
    pub power: u8,
    /// The face it strongly powers (JSON `strongly_powered_face`).
    pub strongly_powered_face: Option<Facing>,
    /// The faces that give power (JSON `connected_faces`); all six when absent.
    pub connected_faces: Option<Vec<Facing>>,
    /// Faces turn with the block's rotation (JSON `transform_relative`); `true` when absent.
    #[facet(schema::fallback = true)]
    pub transform_relative: Option<bool>,
}
