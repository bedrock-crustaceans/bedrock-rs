//! `metadata/engine_modules/engine-after-events-ordering.json`: the order in which the engine
//! dispatches after-events, per `@minecraft/server` version. chorus's event pump
//! consumes it; this crate only models it.

use facet::Facet;

use crate::meta::MetaError;

/// The after-events ordering file.
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct AfterEventsOrdering {
    /// `module_type`: `after_events_ordering`.
    pub module_type: String,
    /// `name`: `engine-after_events_ordering`.
    pub name: String,
    /// `after_events_order_by_version`: one order per module version, including versions no
    /// metadata file exists for (`3.0.0-alpha`, `2.11.0-internal`).
    pub after_events_order_by_version: Vec<OrderForVersion>,
}

/// The dispatch order of one module version.
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct OrderForVersion {
    /// `name`: `<public module> <version>` (`@minecraft/server 2.10.0`).
    pub name: String,
    /// `version`: the version, `-alpha` and `-internal` ones included.
    pub version: String,
    /// `event_order`: the event classes in dispatch order.
    pub event_order: Vec<EventName>,
}

/// One entry of `event_order`.
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct EventName {
    /// `name`: the event class (`PlayerJoinAfterEvent`).
    pub name: String,
}

impl AfterEventsOrdering {
    /// Reads the file strictly: a key no field takes is an error.
    pub fn parse(bytes: &[u8]) -> Result<Self, MetaError> {
        crate::meta::parse::strict(bytes).map(|(file, _)| file)
    }
}

impl OrderForVersion {
    /// The module part of [`name`](Self::name), before the space (`@minecraft/server`); the
    /// whole name when it has no space.
    pub fn module(&self) -> &str {
        self.name
            .split_once(' ')
            .map_or(&*self.name, |(module, _)| module)
    }
}
