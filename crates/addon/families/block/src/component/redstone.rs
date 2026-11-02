//! Redstone: `redstone_conductivity`, `redstone_consumer`, `redstone_producer`.

use bedrock_addon_core::json::{StrictBool, StrictInt};
use facet::Facet;

use super::types::Facing;
use bedrock_addon_core::document::Extra;

/// `minecraft:redstone_conductivity`: whether the block conducts redstone power.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct RedstoneConductivity {
    /// JSON `redstone_conductor`; `false` when absent.
    #[facet(default)]
    pub redstone_conductor: Option<StrictBool>,
    /// JSON `allows_wire_to_step_down`; `true` when absent.
    #[facet(default)]
    pub allows_wire_to_step_down: Option<StrictBool>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl RedstoneConductivity {
    /// Whether the block conducts power: `redstone_conductor`, `false` when absent.
    #[must_use]
    pub fn redstone_conductor(&self) -> bool {
        self.redstone_conductor.is_some_and(|b| b.0)
    }

    /// Whether wire may step down the block: `allows_wire_to_step_down`, `true` when absent.
    #[must_use]
    pub fn allows_wire_to_step_down(&self) -> bool {
        self.allows_wire_to_step_down.is_none_or(|b| b.0)
    }
}

/// `minecraft:redstone_consumer`: the block takes redstone power (custom components see it).
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct RedstoneConsumer {
    /// The lowest power it reacts to (JSON `min_power`, `0..=15`); 0 when absent.
    #[facet(default)]
    pub min_power: Option<StrictInt<u8>>,
    /// Passes the power on (JSON `propagates_power`); `false` when absent.
    #[facet(default)]
    pub propagates_power: Option<StrictBool>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl RedstoneConsumer {
    /// The lowest power it reacts to: `min_power`, 0 when absent.
    #[must_use]
    pub fn min_power(&self) -> u8 {
        self.min_power.map_or(0, |power| power.0)
    }

    /// Whether it passes the power on: `propagates_power`, `false` when absent.
    #[must_use]
    pub fn propagates_power(&self) -> bool {
        self.propagates_power.is_some_and(|b| b.0)
    }
}

/// `minecraft:redstone_producer`: the block gives off redstone power.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct RedstoneProducer {
    /// The power level (JSON `power`, required, `0..=15`).
    pub power: StrictInt<u8>,
    /// The face it strongly powers (JSON `strongly_powered_face`).
    #[facet(default)]
    pub strongly_powered_face: Option<Facing>,
    /// The faces that give power (JSON `connected_faces`); all six when absent.
    #[facet(default)]
    pub connected_faces: Option<Vec<Facing>>,
    /// Faces turn with the block's rotation (JSON `transform_relative`); `true` when absent.
    #[facet(default)]
    pub transform_relative: Option<StrictBool>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl RedstoneProducer {
    /// Whether the faces turn with the block's rotation: `transform_relative`, `true` when
    /// absent.
    #[must_use]
    pub fn transform_relative(&self) -> bool {
        self.transform_relative.is_none_or(|b| b.0)
    }
}
