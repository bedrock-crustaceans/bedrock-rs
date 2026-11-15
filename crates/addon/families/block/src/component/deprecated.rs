//! The deprecated side-car: legacy components an upgraded document keeps for write-back and for
//! hosts that still run them (`deprecated`).

use facet::Facet;
use indexmap::IndexMap;

use super::TRIGGER_CONDITION;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::molang::{Molang, MolangVisitor};
use bedrock_addon_core::schema::{self, Schema};
use bedrock_addon_core::shared::FilterSubject;

/// The legacy components of one section (the base or a permutation) that the 1.26.20
/// conversion moved aside (JSON `deprecated`, the crate's own member). Only documents below 1.26.20 of
/// allow-listed packs (vanilla's legacy creator features) carry them.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct DeprecatedComponents {
    /// JSON `on_fall_on`.
    pub on_fall_on: Option<DeprecatedBaseTrigger>,
    /// JSON `on_interact`.
    pub on_interact: Option<DeprecatedBaseTrigger>,
    /// JSON `on_placed`.
    pub on_placed: Option<DeprecatedBaseTrigger>,
    /// JSON `on_player_destroyed`.
    pub on_player_destroyed: Option<DeprecatedBaseTrigger>,
    /// JSON `on_player_placing`.
    pub on_player_placing: Option<DeprecatedBaseTrigger>,
    /// JSON `on_step_off`.
    pub on_step_off: Option<DeprecatedBaseTrigger>,
    /// JSON `on_step_on`.
    pub on_step_on: Option<DeprecatedBaseTrigger>,
    /// JSON `queued_ticking`.
    pub queued_ticking: Option<DeprecatedQueuedTicking>,
    /// JSON `random_ticking`.
    pub random_ticking: Option<DeprecatedRandomTicking>,
    /// JSON `breathability`.
    pub breathability: Option<Breathing>,
    /// JSON `unit_cube`: the section was a full unit cube.
    pub unit_cube: Option<bool>,
}

impl DeprecatedComponents {
    /// Nothing was moved aside.
    pub fn is_empty(&self) -> bool {
        self == &DeprecatedComponents::default()
    }
}

/// A legacy trigger component (the game's deprecated trigger): an event to run, its
/// condition and target.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct DeprecatedBaseTrigger {
    /// The event of `deprecated_event_handlers` to run (JSON `event`).
    pub event: Option<String>,
    /// When it runs (JSON `condition`).
    #[facet(schema::molang = TRIGGER_CONDITION)]
    pub condition: Option<Molang>,
    /// Who the event targets (JSON `target`: `self`, `other`, …).
    pub target: Option<FilterSubject>,
    /// `on_fall_on` only: the fall distance it needs (JSON `min_fall_distance`).
    pub min_fall_distance: Option<f32>,
}

/// The legacy `minecraft:queued_ticking` component.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct DeprecatedQueuedTicking {
    /// JSON `min_ticks`.
    #[facet(default)]
    pub min_ticks: u32,
    /// JSON `max_ticks`.
    #[facet(default)]
    pub max_ticks: u32,
    /// JSON `looping`.
    #[facet(default)]
    pub looping: bool,
    /// JSON `on_tick`.
    #[facet(default)]
    pub on_tick: DeprecatedBaseTrigger,
}

/// The legacy `minecraft:random_ticking` component.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct DeprecatedRandomTicking {
    /// JSON `on_tick`.
    #[facet(default)]
    pub on_tick: DeprecatedBaseTrigger,
}

/// The legacy `minecraft:breathability` value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet, Schema)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum Breathing {
    /// `solid`.
    Solid,
    /// `air`.
    Air,
}

/// A legacy block event (the game's deprecated definition event), kept for write-back and for
/// hosts that still run them (JSON `deprecated_event_handlers.<name>`, the crate's own member).
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::molang_with = walk_event)]
pub struct DeprecatedDefinitionEvent {
    /// The weight of a `randomize` entry (JSON `probability`).
    pub probability: Option<f32>,
    /// When the event runs (JSON `condition`).
    #[facet(schema::molang = TRIGGER_CONDITION)]
    pub condition: Option<Molang>,
    /// The event the node fires (JSON `trigger`).
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
}

/// The event a legacy event node fires (the game's definition trigger): its name, target and
/// condition.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct DefinitionTrigger {
    /// The event of `deprecated_event_handlers` (JSON `event`).
    pub event: Option<String>,
    /// Who the event targets (JSON `target`).
    pub target: Option<FilterSubject>,
    /// When it fires (JSON `condition`).
    #[facet(schema::molang = TRIGGER_CONDITION)]
    pub condition: Option<Molang>,
}

/// The Molang of an event: its condition, its trigger and its children. An event with both lists
/// loads the `randomize` alone: the game logs it and does not evaluate the `sequence`.
fn walk_event(event: &mut DeprecatedDefinitionEvent, visitor: &mut MolangVisitor<'_>) {
    visitor.field("condition", &TRIGGER_CONDITION, &mut event.condition);
    visitor.nested("trigger", &mut event.trigger);
    if event.randomize.is_empty() || event.sequence.is_empty() {
        visitor.nested("sequence", &mut event.sequence);
    }
    visitor.nested("randomize", &mut event.randomize);
}
