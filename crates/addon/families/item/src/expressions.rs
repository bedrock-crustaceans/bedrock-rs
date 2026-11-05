//! The Molang expressions of an item's legacy members, which the document keeps as JSON: the
//! `condition` of each event response (`events.*`), the `condition` of each legacy trigger, and
//! the `launch_power` / `angle_offset` of each `shoot` response. They are read once when the
//! document is parsed and walked by the Molang passes with the item's other fields; the JSON
//! itself is written back as it was.

use bedrock_addon_core::molang::{
    Molang, MolangFieldSpec, MolangFields, MolangList, MolangVersionSource, MolangVisitor,
    RawVersion, ReadField, ReaderKind, Step, read, read_list,
};

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Severity, VanillaText,
};
use bedrock_addon_core::json::Value;

/// The spec of a legacy item expression: it compiles at the item's format version. The game
/// logs the Molang parser's refusal at the content log's error level, and every consumer of
/// the expression drops the result and goes on, so the item loads: a kept error. The event
/// loader tests no result for an event `condition`, and the legacy trigger and the `shoot`
/// response drop the expression the same way.
const fn spec(reader: ReaderKind) -> MolangFieldSpec {
    MolangFieldSpec::KEPT
        .reader(reader)
        .version(MolangVersionSource::FormatVersion)
}

/// An event response's `condition` (`ReaderKind::ScalarOrArray`, inferred).
const EVENT_CONDITION: MolangFieldSpec = spec(ReaderKind::ScalarOrArray);
/// A legacy trigger's `condition` (`ReaderKind::StrictVersioned`).
const TRIGGER_CONDITION: MolangFieldSpec = spec(ReaderKind::StrictVersioned);
/// A `shoot` response's `launch_power` / `angle_offset` (`ReaderKind::SchemaValidated`).
const SHOOT: MolangFieldSpec = spec(ReaderKind::SchemaValidated);

/// One step of a member path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Segment {
    /// An object member.
    Key(String),
    /// An array element.
    Index(usize),
}

/// What a legacy expression is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    /// `events.*…condition`.
    EventCondition,
    /// A legacy trigger's `condition`.
    TriggerCondition,
    /// A `shoot` response's member.
    Shoot,
}

/// One expression and where it is, below the item object.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Expression {
    pub(crate) path: Vec<Segment>,
    pub(crate) kind: Kind,
    value: Slot,
}

/// The typed value of an expression.
#[derive(Clone, Debug, PartialEq)]
enum Slot {
    One(Molang),
    List(MolangList),
    /// A `shoot` member of a type the typed node cannot parse: no expression, but its leaf
    /// event is inert ([`drop_failed_shoot_events`]).
    Refused,
}

/// The legacy expressions of one item, in document order.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Expressions(pub(crate) Vec<Expression>);

impl Expressions {
    /// The expressions of `events`, of the triggers in `components` (the kept component keys,
    /// as written) and of the read side-car, with their version; a value the field's reader
    /// refuses is told as an `InvalidType` finding (graded as the game grades that type of
    /// value) and not kept.
    pub(crate) fn read(
        events: Option<&Value>,
        components: &[(&str, &Value)],
        side_car: Option<&Value>,
        context_version: RawVersion,
        at: &JsonPath,
        d: &mut Diagnostics,
    ) -> Self {
        let mut out = Collector {
            version: context_version,
            at,
            d,
            found: Vec::new(),
        };
        if let Some(events) = events {
            out.events(events, &mut vec![Segment::Key("events".to_owned())]);
        }
        for (key, value) in components {
            let mut path = vec![
                Segment::Key("components".to_owned()),
                Segment::Key((*key).to_owned()),
            ];
            out.component_triggers(key, value, &mut path);
        }
        if let Some(object) = side_car.and_then(Value::as_object) {
            for (key, member) in object {
                let mut path = vec![
                    Segment::Key("__deprecated_components".to_owned()),
                    Segment::Key(key.as_str().to_owned()),
                ];
                if key.as_str() == "legacy_on_dig_trigger" {
                    out.component_triggers("minecraft:digger", member, &mut path);
                } else if let Some(inner) = member.as_object() {
                    for (inner_key, trigger) in inner {
                        path.push(Segment::Key(inner_key.as_str().to_owned()));
                        out.trigger(trigger, &mut path);
                        path.pop();
                    }
                }
            }
        }
        Expressions(out.found)
    }
}

/// The reading state.
struct Collector<'a> {
    version: RawVersion,
    at: &'a JsonPath,
    d: &'a mut Diagnostics,
    found: Vec<Expression>,
}

/// Whether the object at `path` (below the item, starting at `events`) is an event: a member of
/// `events`, or an element of the `sequence` / `randomize` of one.
fn is_event_node(path: &[Segment]) -> bool {
    match path {
        [Segment::Key(_), Segment::Key(_)] => true,
        [.., Segment::Key(list), Segment::Index(_)] => list == "sequence" || list == "randomize",
        _ => false,
    }
}

/// The trigger members of each legacy component.
fn trigger_keys(component: &str) -> &'static [&'static str] {
    match component {
        "minecraft:food" => &["on_consume"],
        "minecraft:on_use" => &["on_use"],
        "minecraft:on_use_on" => &["on_use_on"],
        "minecraft:weapon" => &["on_hurt_entity", "on_not_hurt_entity", "on_hit_block"],
        "minecraft:chargeable" => &["on_complete"],
        "minecraft:digger" => &["on_dig", "__deprecated_on_dig"],
        _ => &[],
    }
}

impl Collector<'_> {
    /// Walks an `events` value: every `condition`, and every `shoot` response.
    fn events(&mut self, value: &Value, path: &mut Vec<Segment>) {
        if let Some(object) = value.as_object() {
            // An event with both `randomize` and `sequence` makes the game log a line at the
            // content log's warning level and ignore the sequence; without `randomize` the
            // sequence is loaded. The same loader serves block events. The sequence is not
            // walked here, as the game parses no condition in it.
            let ignores_sequence = is_event_node(path)
                && object.contains_key("randomize")
                && object.contains_key("sequence");
            if ignores_sequence {
                let mut at = self.at.clone();
                append(&mut at, path);
                at.push("sequence");
                self.d.push(
                    Diagnostic::warn(DiagnosticCode::IgnoredMember {
                        text: Some(VanillaText::Fixed(
                            bedrock_addon_core::diagnostic::EVENT_BOTH_RANDOMIZE_AND_SEQUENCE,
                        )),
                    })
                    .at(at),
                );
            }
            for (key, member) in object {
                if ignores_sequence && key.as_str() == "sequence" {
                    continue;
                }
                path.push(Segment::Key(key.as_str().to_owned()));
                match key.as_str() {
                    "condition" => self.keep(member, path, Kind::EventCondition),
                    "shoot" => {
                        if let Some(shoot) = member.as_object() {
                            for field in ["launch_power", "angle_offset"] {
                                if let Some(value) = shoot.get(field) {
                                    path.push(Segment::Key(field.to_owned()));
                                    self.keep(value, path, Kind::Shoot);
                                    path.pop();
                                }
                            }
                        }
                        self.events(member, path);
                    }
                    _ => self.events(member, path),
                }
                path.pop();
            }
        } else if let Some(array) = value.as_array() {
            for (i, element) in array.iter().enumerate() {
                path.push(Segment::Index(i));
                self.events(element, path);
                path.pop();
            }
        }
    }

    /// The triggers of the component `key` (and of its `destroy_speeds` for a digger).
    fn component_triggers(&mut self, key: &str, value: &Value, path: &mut Vec<Segment>) {
        let Some(object) = value.as_object() else {
            return;
        };
        for member in trigger_keys(key) {
            if let Some(trigger) = object.get(member) {
                path.push(Segment::Key((*member).to_owned()));
                self.trigger(trigger, path);
                path.pop();
            }
        }
        if key == "minecraft:digger"
            && let Some(speeds) = object.get("destroy_speeds").and_then(Value::as_array)
        {
            path.push(Segment::Key("destroy_speeds".to_owned()));
            for (i, speed) in speeds.iter().enumerate() {
                path.push(Segment::Index(i));
                if let Some(speed) = speed.as_object() {
                    for member in ["on_dig", "__deprecated_on_dig"] {
                        if let Some(trigger) = speed.get(member) {
                            path.push(Segment::Key(member.to_owned()));
                            self.trigger(trigger, path);
                            path.pop();
                        }
                    }
                }
                path.pop();
            }
            path.pop();
        }
    }

    /// A trigger's `condition`.
    fn trigger(&mut self, trigger: &Value, path: &mut Vec<Segment>) {
        if let Some(condition) = trigger.as_object().and_then(|t| t.get("condition")) {
            path.push(Segment::Key("condition".to_owned()));
            self.keep(condition, path, Kind::TriggerCondition);
            path.pop();
        }
    }

    /// Reads `value` with `kind`'s reader and keeps it, or reports the refusal.
    fn keep(&mut self, value: &Value, path: &[Segment], kind: Kind) {
        // The game's string reader of a trigger `condition` takes no number or bool: the
        // strict Molang reader would take a constant, but the game refuses the item.
        if kind == Kind::TriggerCondition
            && (value.as_number().is_some() || value.as_bool().is_some())
        {
            self.refuse(value, path, kind);
            return;
        }
        let slot = match kind {
            Kind::EventCondition => read_list(value, self.version).map(|l| l.map(Slot::List)),
            Kind::TriggerCondition | Kind::Shoot => {
                let reader = if kind == Kind::Shoot {
                    ReaderKind::SchemaValidated
                } else {
                    ReaderKind::StrictVersioned
                };
                read(value, reader, self.version).map(|field| match field {
                    ReadField::Value(value) => Some(Slot::One(value)),
                    _ => None,
                })
            }
        };
        match slot {
            Ok(Some(value)) => self.found.push(Expression {
                path: path.to_vec(),
                kind,
                value,
            }),
            Ok(None) => {}
            Err(_) => self.refuse(value, path, kind),
        }
    }
}

impl Collector<'_> {
    /// Tells a value the Molang reader refuses as the wrong type. The grade is the game's line
    /// for that type, not the reader's:
    /// - an event `condition`: the game logs "Type not convertible to Molang expression" at
    ///   the error level for an object and drops the result: a kept error. A `null` element is
    ///   read in silence, so the crate's own refusal is an information.
    /// - a legacy trigger's `condition`: it is a string or the object form; any other type is
    ///   an "invalid string" error of the item's parse, so the item is refused: an error that
    ///   rejects.
    /// - a `shoot` member: no child option can parse it, the game logs "child '%s' not valid
    ///   here." at the error level and the validation fails: a kept error, and the leaf event
    ///   is inert.
    fn refuse(&mut self, value: &Value, path: &[Segment], kind: Kind) {
        let mut at = self.at.clone();
        append(&mut at, path);
        let tell = |severity| {
            Diagnostic::error(DiagnosticCode::InvalidType { text: None })
                .severity(severity)
                .at(at.clone())
        };
        let told = tell(Severity::Error);
        match kind {
            Kind::EventCondition if !holds_object(value) => self.d.push(tell(Severity::Info)),
            Kind::EventCondition => self.d.push_kept_error(told),
            Kind::TriggerCondition => self.d.push(told),
            Kind::Shoot => {
                self.d.push_kept_error(told);
                self.found.push(Expression {
                    path: path.to_vec(),
                    kind,
                    value: Slot::Refused,
                });
            }
        }
    }
}

/// Whether `value` is, or holds in an array, an object.
fn holds_object(value: &Value) -> bool {
    value.is_object() || value.as_array().is_some_and(|a| a.iter().any(holds_object))
}

/// `path` appended to `at`.
pub(crate) fn append(at: &mut JsonPath, path: &[Segment]) {
    for segment in path {
        match segment {
            Segment::Key(key) => at.push(key),
            Segment::Index(i) => at.push_index(*i),
        }
    }
}

impl MolangFields for Expressions {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        for expression in &mut self.0 {
            let Some((last, parents)) = expression.path.split_last() else {
                continue;
            };
            for segment in parents {
                visitor.enter(step(segment));
            }
            let spec = match expression.kind {
                Kind::EventCondition => &EVENT_CONDITION,
                Kind::TriggerCondition => &TRIGGER_CONDITION,
                Kind::Shoot => &SHOOT,
            };
            match &mut expression.value {
                Slot::One(value) => visitor.field(step(last), spec, value),
                Slot::List(elements) => visitor.field(step(last), spec, elements),
                Slot::Refused => {}
            }
            for _ in parents {
                visitor.leave();
            }
        }
    }
}

/// A segment as a visitor step.
fn step(segment: &Segment) -> Step<'_> {
    match segment {
        Segment::Key(key) => Step::from(key.as_str()),
        Segment::Index(i) => Step::from(*i),
    }
}

/// Makes inert, in `events`, the leaf event holding each `shoot` whose expression path is in
/// `failed`: the typed node fails validation on a refused string, the event's response
/// collection fails with it, and the game marks the event as failed and evaluates nothing of
/// it. So all of that event's responses and its `trigger` are gone, its siblings and parent
/// stay. The game keeps the event registered, so it is emptied here and not removed (a
/// removal would leave the `trigger` references to it dangling). A holder with `sequence` /
/// `randomize` is a container the game never validates, so there only the `shoot` response
/// goes. The event's other expressions go with it. `at` is the item object's path; the
/// result is how many events were made inert.
pub(crate) fn drop_failed_shoot_events(
    expressions: &mut Expressions,
    events: &mut Option<Value>,
    failed: &[JsonPath],
    at: &JsonPath,
) -> usize {
    // the path of each event holding a failed shoot (…/shoot/<field> minus its last two steps)
    let mut holders: Vec<Vec<Segment>> = Vec::new();
    for expression in &expressions.0 {
        if expression.kind != Kind::Shoot || expression.path.len() < 3 {
            continue;
        }
        let mut full = at.clone();
        append(&mut full, &expression.path);
        if expression.value != Slot::Refused && !failed.contains(&full) {
            continue;
        }
        let holder = expression.path[..expression.path.len() - 2].to_vec();
        if !holders.contains(&holder) {
            holders.push(holder);
        }
    }
    let mut inert = 0;
    for holder in &holders {
        // the path starts at `events`
        let Some(object) = events
            .as_mut()
            .and_then(|events| navigate(events, &holder[1..]))
            .and_then(Value::as_object_mut)
        else {
            continue;
        };
        if object.contains_key("sequence") || object.contains_key("randomize") {
            object.remove("shoot");
            expressions
                .0
                .retain(|e| !(e.kind == Kind::Shoot && e.path.starts_with(holder)));
        } else {
            object.clear();
            expressions.0.retain(|e| !e.path.starts_with(holder));
            inert += 1;
        }
    }
    inert
}

/// The value at `path` below `value`.
fn navigate<'v>(value: &'v mut Value, path: &[Segment]) -> Option<&'v mut Value> {
    path.iter().try_fold(value, |value, segment| match segment {
        Segment::Key(key) => value.as_object_mut()?.get_mut(key),
        Segment::Index(i) => value.as_array_mut()?.get_mut(*i),
    })
}
