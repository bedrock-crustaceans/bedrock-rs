//! The deprecated side-car: legacy components an upgraded document keeps for write-back and for
//! hosts that still run them (`deprecated`).

use bedrock_addon_core::json::{StrictBool, StrictFloat, StrictInt, StrictString};
use facet::Facet;
use indexmap::IndexMap;

use super::TRIGGER_CONDITION;
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::molang::{Molang, MolangFields, MolangVisitor};
use bedrock_addon_core::shared::FilterSubject;

/// The legacy components of one section (the base or a permutation) that the 1.26.20
/// conversion moved aside (JSON `deprecated`, the crate's own member). Only documents below 1.26.20 of
/// allow-listed packs (vanilla's legacy creator features) carry them.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct DeprecatedComponents {
    /// JSON `on_fall_on`.
    #[facet(default)]
    pub on_fall_on: Option<DeprecatedBaseTrigger>,
    /// JSON `on_interact`.
    #[facet(default)]
    pub on_interact: Option<DeprecatedBaseTrigger>,
    /// JSON `on_placed`.
    #[facet(default)]
    pub on_placed: Option<DeprecatedBaseTrigger>,
    /// JSON `on_player_destroyed`.
    #[facet(default)]
    pub on_player_destroyed: Option<DeprecatedBaseTrigger>,
    /// JSON `on_player_placing`.
    #[facet(default)]
    pub on_player_placing: Option<DeprecatedBaseTrigger>,
    /// JSON `on_step_off`.
    #[facet(default)]
    pub on_step_off: Option<DeprecatedBaseTrigger>,
    /// JSON `on_step_on`.
    #[facet(default)]
    pub on_step_on: Option<DeprecatedBaseTrigger>,
    /// JSON `queued_ticking`.
    #[facet(default)]
    pub queued_ticking: Option<DeprecatedQueuedTicking>,
    /// JSON `random_ticking`.
    #[facet(default)]
    pub random_ticking: Option<DeprecatedRandomTicking>,
    /// JSON `breathability`.
    #[facet(default)]
    pub breathability: Option<Breathing>,
    /// JSON `unit_cube`: the section was a full unit cube.
    #[facet(default)]
    pub unit_cube: Option<StrictBool>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl DeprecatedComponents {
    /// Nothing was moved aside.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self == &DeprecatedComponents::default()
    }

    /// The seven triggers by JSON name, mutably.
    pub(crate) fn triggers_mut(
        &mut self,
    ) -> [(&'static str, &mut Option<DeprecatedBaseTrigger>); 7] {
        [
            ("on_fall_on", &mut self.on_fall_on),
            ("on_interact", &mut self.on_interact),
            ("on_placed", &mut self.on_placed),
            ("on_player_destroyed", &mut self.on_player_destroyed),
            ("on_player_placing", &mut self.on_player_placing),
            ("on_step_off", &mut self.on_step_off),
            ("on_step_on", &mut self.on_step_on),
        ]
    }
}

/// A legacy trigger component (the game's deprecated trigger): an event to run, its
/// condition and target.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct DeprecatedBaseTrigger {
    /// The event of `deprecated_event_handlers` to run (JSON `event`).
    #[facet(default)]
    pub event: Option<StrictString>,
    /// When it runs (JSON `condition`).
    #[facet(default)]
    pub condition: Option<Molang>,
    /// Who the event targets (JSON `target`: `self`, `other`, …).
    #[facet(default)]
    pub target: Option<FilterSubject>,
    /// `on_fall_on` only: the fall distance it needs (JSON `min_fall_distance`).
    #[facet(default)]
    pub min_fall_distance: Option<StrictFloat<f32>>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl MolangFields for DeprecatedBaseTrigger {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visitor.field("condition", &TRIGGER_CONDITION, &mut self.condition);
    }
}

/// The legacy `minecraft:queued_ticking` component.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct DeprecatedQueuedTicking {
    /// JSON `min_ticks`.
    #[facet(default = StrictInt(0_u32))]
    pub min_ticks: StrictInt<u32>,
    /// JSON `max_ticks`.
    #[facet(default = StrictInt(0_u32))]
    pub max_ticks: StrictInt<u32>,
    /// JSON `looping`.
    #[facet(default = StrictBool(false))]
    pub looping: StrictBool,
    /// JSON `on_tick`.
    #[facet(default)]
    pub on_tick: DeprecatedBaseTrigger,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// The legacy `minecraft:random_ticking` component.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct DeprecatedRandomTicking {
    /// JSON `on_tick`.
    #[facet(default)]
    pub on_tick: DeprecatedBaseTrigger,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// The legacy `minecraft:breathability` value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum Breathing {
    /// `solid`.
    Solid,
    /// `air`.
    Air,
}

impl MolangFields for DeprecatedComponents {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        for (name, trigger) in self.triggers_mut() {
            visitor.nested(name, trigger);
        }
        if let Some(ticking) = &mut self.queued_ticking {
            visitor.enter("queued_ticking".into());
            visitor.nested("on_tick", &mut ticking.on_tick);
            visitor.leave();
        }
        if let Some(ticking) = &mut self.random_ticking {
            visitor.enter("random_ticking".into());
            visitor.nested("on_tick", &mut ticking.on_tick);
            visitor.leave();
        }
    }
}

/// A legacy block event (the game's deprecated definition event), kept for write-back and for
/// hosts that still run them (JSON `deprecated_event_handlers.<name>`, the crate's own member).
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct DeprecatedDefinitionEvent {
    /// The weight of a `randomize` entry (JSON `probability`).
    #[facet(default)]
    pub probability: Option<StrictFloat<f32>>,
    /// When the event runs (JSON `condition`).
    #[facet(default)]
    pub condition: Option<Molang>,
    /// The event the node fires (JSON `trigger`).
    #[facet(default)]
    pub trigger: Option<DefinitionTrigger>,
    /// `sequence` children, run in order (JSON `sequence`).
    #[facet(default, skip_serializing_if = Vec::is_empty)]
    pub sequence: Vec<DeprecatedDefinitionEvent>,
    /// `randomize` children, one picked by weight (JSON `randomize`).
    #[facet(default, skip_serializing_if = Vec::is_empty)]
    pub randomize: Vec<DeprecatedDefinitionEvent>,
    /// The responses (`set_block_state`, `run_command`, …), one object each, as written (JSON
    /// `responses`).
    #[facet(default, skip_serializing_if = Vec::is_empty)]
    pub responses: Vec<IndexMap<String, Value>>,
    /// Unknown members. `default`: the reader does not fill a skipped member of a node
    /// that is an element of a node of its own type, and refuses every nested node for it.
    #[facet(skip, default, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// The event a legacy event node fires (the game's definition trigger): its name, target and
/// condition.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct DefinitionTrigger {
    /// The event of `deprecated_event_handlers` (JSON `event`).
    #[facet(default)]
    pub event: Option<StrictString>,
    /// Who the event targets (JSON `target`).
    #[facet(default)]
    pub target: Option<FilterSubject>,
    /// When it fires (JSON `condition`).
    #[facet(default)]
    pub condition: Option<Molang>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl MolangFields for DefinitionTrigger {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visitor.field("condition", &TRIGGER_CONDITION, &mut self.condition);
    }
}

impl MolangFields for DeprecatedDefinitionEvent {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visitor.field("condition", &TRIGGER_CONDITION, &mut self.condition);
        visitor.nested("trigger", &mut self.trigger);
        // an event with both lists loads the `randomize` alone: the game logs it and does not evaluate the `sequence`
        if self.randomize.is_empty() || self.sequence.is_empty() {
            visitor.nested("sequence", &mut self.sequence);
        }
        visitor.nested("randomize", &mut self.randomize);
    }
}
