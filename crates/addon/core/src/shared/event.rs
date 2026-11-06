//! `EventTrigger`: an event fired on a subject when a filter passes.

use facet_value::VObject;

use super::filter::{Filter, FilterSubject};
use super::read::{self, ROOT, ShapeError, wrong_type};
use super::unrecognized;
use crate::diagnostic::{Diagnostics, JsonPath, Severity};
use crate::document::UnknownMembers;
use crate::json::Value;

/// An event trigger: the game's entity trigger (typed entity components) and the item
/// side-car `Trigger` (the same object without `filters`).
///
/// JSON: `"ns:event"` (the event on `self`, no filter) or `{"event"?: string, "target"?: subject,
/// "filters"?: filter}`; an object without `event` fires nothing and is a condition only (vanilla's
/// `on_interact` of an interaction that needs no event). A legacy `condition` (the game's definition
/// trigger) and any other member are kept in `extra`; the typed trigger has no condition, and the using family reads it from
/// there for its diagnostics.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[facet(opaque, proxy = Value)]
#[doc(alias = "trigger")]
pub struct EventTrigger {
    /// The event to fire (JSON `event`), looked up in the definition of the subject `target`
    /// names: an event name, not an identifier. `None`: no event, the trigger is a condition.
    pub event: Option<String>,
    /// Whose definition has the event (JSON `target`); absent means `self`.
    pub target: Option<FilterSubject>,
    /// The condition (JSON `filters`); absent means always.
    pub filters: Option<Filter>,
    /// A legacy `condition` and other members.
    pub extra: UnknownMembers,
    /// Read from (and written as) the string form.
    pub bare: bool,
}

impl EventTrigger {
    /// The event on `self` with no filter, written as a string.
    pub fn new(event: impl Into<String>) -> Self {
        Self {
            event: Some(event.into()),
            target: None,
            filters: None,
            extra: UnknownMembers::default(),
            bare: true,
        }
    }

    /// The findings of the trigger's `filters` (an unknown test name), located under
    /// `at/filters`, and of its unknown members, each at `unknown` (the grade of an unknown
    /// member by the reading the caller reads under).
    pub fn check(&self, at: &JsonPath, unknown: Severity) -> Diagnostics {
        let mut out = Diagnostics::default();
        if let Some(filters) = &self.filters {
            let mut path = at.clone();
            path.push("filters");
            out.extend(filters.check(&path, unknown));
        }
        unrecognized(&self.extra, at, unknown, &mut out);
        out
    }
}

impl TryFrom<Value> for EventTrigger {
    type Error = ShapeError;

    fn try_from(value: Value) -> Result<Self, ShapeError> {
        if let Some(event) = value.as_string() {
            return Ok(Self::new(event.as_str()));
        }
        let object = value
            .as_object()
            .ok_or_else(|| wrong_type(ROOT, "a string or an object", &value))?;
        let member = |key| object.get(key).filter(|v| !v.is_null());
        let mut extra = UnknownMembers::default();
        for (key, value) in object {
            if !matches!(key.as_str(), "event" | "target" | "filters") {
                extra.insert(key.as_str().to_owned(), value.clone());
            }
        }
        Ok(Self {
            event: member("event")
                .map(|v| read::string("event", v))
                .transpose()?,
            target: member("target")
                .map(|v| FilterSubject::read(v).map_err(|f| f.key("target")))
                .transpose()?,
            filters: member("filters")
                .map(|v| Filter::read(v).map_err(|f| f.key("filters")))
                .transpose()?,
            extra,
            bare: false,
        })
    }
}

impl TryFrom<&EventTrigger> for Value {
    type Error = ShapeError;

    fn try_from(trigger: &EventTrigger) -> Result<Self, ShapeError> {
        if let Some(event) = &trigger.event
            && trigger.bare
            && trigger.target.is_none()
            && trigger.filters.is_none()
            && trigger.extra.is_empty()
        {
            return Ok(Value::from(event.as_str()));
        }
        let mut object = VObject::new();
        if let Some(event) = &trigger.event {
            object.insert("event", Value::from(event.as_str()));
        }
        if let Some(target) = trigger.target {
            object.insert("target", Value::from(target.name()));
        }
        if let Some(filters) = &trigger.filters {
            object.insert("filters", filters.write().map_err(|f| f.key("filters"))?);
        }
        for (key, value) in trigger.extra.iter() {
            object.insert(key, value.clone());
        }
        Ok(Value::from(object))
    }
}
