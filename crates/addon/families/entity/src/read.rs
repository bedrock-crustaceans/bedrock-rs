//! Reading the newest struct ([`Entity`], and [`Beta`](super::Beta) with it) from its payload.
//!
//! The payload is read as a shell whose open parts stay raw slices: each component is parsed
//! from its own slice (a failing one is dropped, the rest load), each property, the brain and
//! the scripts are built member by member so a bad one is reported at its path.

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity, Span, Told,
    VanillaText, code_for_deserialize_error_kind,
};
use bedrock_addon_core::document::{ComponentMap, ComponentSet, Extra};
use bedrock_addon_core::id::EntityId;
use bedrock_addon_core::json::{
    JsonSlice, Misreading, SliceError, Value, keep_members_and_range_members, parse_at,
};
use bedrock_addon_core::load::{Experiment, LoadContext};
use bedrock_addon_core::molang::MolangSource;
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput};
use bedrock_addon_core::versioning::{ConvertMode, Family, ParseCx};
use facet::Facet;
use facet_json::RawJson;
use indexmap::IndexMap;

use super::component::EntityComponent;
use super::goal;
use super::history::first_typed;
use super::load::STRICT_SLICES;
use super::schema;
use super::{
    AnimateEntry, Brain, ComponentGroup, DeprecatedData, Description, Entity, EntityVersion, Event,
    Property, PropertyDefault, PropertyType, Scripts, SpawnCategory, UpgradeEntry, Written,
};

/// The format version from which `description.aliases` is not read.
pub(crate) const ALIASES_DEPRECATED: FormatVersion = FormatVersion::new(1, 21, 10);
/// The format version that introduced `entity_version` and `upgrades` (behind Upcoming Creator
/// Features until 1.26.30).
pub(crate) const ENTITY_VERSION_SINCE: FormatVersion = FormatVersion::new(1, 26, 20);
/// The format version from which `entity_version` and `upgrades` need no toggle.
pub(crate) const ENTITY_VERSION_STABLE: FormatVersion = FormatVersion::new(1, 26, 30);
/// The format version that introduced the root `brain` member.
pub(crate) const BRAIN_SINCE: FormatVersion = FormatVersion::new(1, 21, 130);

/// What the newest struct is parsed from.
#[derive(Facet)]
struct Shell<'a> {
    #[facet(default)]
    description: Option<RawJson<'a>>,
    #[facet(default)]
    components: Option<IndexMap<String, RawJson<'a>>>,
    #[facet(default)]
    component_groups: Option<IndexMap<String, RawJson<'a>>>,
    #[facet(default)]
    events: Option<IndexMap<String, Value>>,
    #[facet(default)]
    brain: Option<RawJson<'a>>,
    #[facet(default)]
    upgrades: Option<IndexMap<String, Value>>,
    #[facet(default, rename = "__deprecated")]
    deprecated: Option<DeprecatedShell>,
    // `SKIP_UPGRADES_KEY`: read by the chain as a flag; the newest struct does not keep it.
    #[facet(default)]
    do_not_upgrade: Option<Value>,
    // The pre-1.8 in-payload version (`Family::payload_version`), consumed by the selection.
    #[facet(default)]
    format_version: Option<Value>,
    #[facet(skip, rename = "\u{0}extra")]
    extra: Extra,
}

/// `__deprecated`.
#[derive(Facet)]
struct DeprecatedShell {
    #[facet(default)]
    aliases: Option<IndexMap<String, Value>>,
    #[facet(skip, rename = "\u{0}extra")]
    extra: Extra,
}

/// What `description` is parsed from.
#[derive(Facet)]
struct DescriptionShell<'a> {
    identifier: EntityId,
    #[facet(default)]
    runtime_identifier: Option<EntityId>,
    #[facet(default)]
    is_spawnable: Option<bool>,
    #[facet(default)]
    is_summonable: Option<bool>,
    #[facet(default)]
    properties: Option<IndexMap<String, RawJson<'a>>>,
    #[facet(default)]
    aliases: Option<IndexMap<String, Value>>,
    #[facet(default)]
    animations: Option<IndexMap<String, String>>,
    #[facet(default)]
    scripts: Option<RawJson<'a>>,
    #[facet(default)]
    spawn_category: Option<Value>,
    #[facet(default)]
    property_memory_mapping: Option<Value>,
    #[facet(default)]
    entity_version: Option<Value>,
    // Vanilla's own files carry it and vanilla reads nothing from it: kept without a warning.
    #[facet(default)]
    is_experimental: Option<Value>,
    #[facet(skip, rename = "\u{0}extra")]
    extra: Extra,
}

/// What a property is parsed from.
#[derive(Facet)]
struct PropertyShell<'a> {
    #[facet(rename = "type")]
    kind: String,
    #[facet(default)]
    default: Option<RawJson<'a>>,
    #[facet(default)]
    range: Option<Vec<Value>>,
    #[facet(default)]
    values: Option<Vec<String>>,
    #[facet(default)]
    client_sync: Option<bool>,
    #[facet(skip, rename = "\u{0}extra")]
    extra: Extra,
}

/// What `scripts` is parsed from.
#[derive(Facet)]
struct ScriptsShell<'a> {
    #[facet(default)]
    animate: Option<Vec<RawJson<'a>>>,
    #[facet(skip, rename = "\u{0}extra")]
    extra: Extra,
}

/// The state of one read: the context and where findings go.
pub(crate) struct Reader<'c, 'd> {
    /// The load context.
    pub load: &'c LoadContext,
    /// The document's original version, which the version rules compare against.
    pub original_version: FormatVersion,
    /// Where findings go.
    pub d: &'d mut Diagnostics,
    /// Whether spans are meaningful: false when the text was written by an upgrade step.
    pub spans: bool,
}

impl Reader<'_, '_> {
    /// A finding at `at`.
    pub(crate) fn report(
        &mut self,
        code: DiagnosticCode,
        severity: Severity,
        at: JsonPath,
        span: Option<Span>,
    ) {
        let span = span.filter(|_| self.spans);
        self.d.push(Diagnostic::new(
            code,
            severity,
            Phase::Parse,
            Location::new(None, None, at, span),
        ));
    }

    /// A kept error: the game logs `code`'s line at the error level and loads the entity.
    fn report_kept(&mut self, code: DiagnosticCode, at: JsonPath, span: Option<Span>) {
        let span = span.filter(|_| self.spans);
        self.d.push_kept_error(Diagnostic::new(
            code,
            Severity::Error,
            Phase::Parse,
            Location::new(None, None, at, span),
        ));
    }

    /// A failed typed parse of a slice, at `base` followed by the failing member.
    fn slice_error(
        &mut self,
        error: &SliceError,
        base: &JsonPath,
        slice: JsonSlice<'_>,
        severity: Severity,
    ) {
        let blank = slice.text.trim().is_empty();
        let code =
            code_for_deserialize_error_kind(&error.error.kind, !error.path.is_empty(), blank);
        self.report(code, severity, append(base, &error.path), Some(error.span));
    }

    /// [`Self::typed`] with a failure that is a kept error: the game logs it at the error level
    /// and loads the entity.
    pub(crate) fn typed_kept<'a, T: Facet<'a>>(
        &mut self,
        slice: JsonSlice<'a>,
        base: &JsonPath,
    ) -> Option<T> {
        let before = self.d.len();
        let value = self.typed::<T>(slice, base, Severity::Error);
        for finding in self.d.iter_mut().skip(before) {
            if finding.severity == Severity::Error && !finding.kept {
                *finding = finding.clone().into_kept();
            }
        }
        value
    }

    /// Parses `T` from `slice`, keeping its unknown members in its `extra` fields (each one an
    /// `UnrecognizedField` warning); a failure is reported at `base` with `severity`.
    pub(crate) fn typed<'a, T: Facet<'a>>(
        &mut self,
        slice: JsonSlice<'a>,
        base: &JsonPath,
        severity: Severity,
    ) -> Option<T> {
        let mut value = match parse_at::<T>(slice) {
            Ok(value) => value,
            Err(error) => {
                self.slice_error(&error, base, slice, severity);
                return None;
            }
        };
        if let Ok(json) = bedrock_addon_core::json::from_str_jsonc::<Value>(slice.text) {
            let (unknown, misread) = keep_unknown(&mut value, &json);
            for (path, _) in unknown {
                // The game logs this at the warning level for a document below 1.21.89 and lifts it
                // to an error from that version on.
                self.report(
                    DiagnosticCode::UnrecognizedField { text: None },
                    Severity::Warning,
                    append(base, &path),
                    None,
                );
            }
            for path in misread {
                // Facet's coercion, which jsoncpp's `as*` conversions make in silence: the game
                // writes no content-log line, so the crate reports an information.
                self.report(
                    DiagnosticCode::InvalidType { text: None },
                    Severity::Info,
                    append(base, &path),
                    None,
                );
            }
        }
        Some(value)
    }
}

/// The one place a typed part of an entity (a component, a goal, the description) is
/// compared with the JSON it was read from: the unknown members, kept in the `extra`
/// of the struct they belong to, among them the members a range given as an object does not
/// read (`{"a": 1}` where a number or `{min, max}` stands; reported, not kept); and the
/// places where the typed read took a value of another JSON type in silence (a `null` read
/// as a struct of defaults, a scalar facet coerced), each an `InvalidType` warning of the
/// caller.
pub(crate) fn keep_unknown<'f, T: Facet<'f>>(
    value: &mut T,
    json: &Value,
) -> (Vec<(JsonPath, Value)>, Vec<JsonPath>) {
    let (unknown, misread) = keep_members_and_range_members(value, json);
    let misread = misread
        .into_iter()
        .filter(|misread| matches!(misread.kind, Misreading::WrongType))
        .map(|misread| misread.path)
        .collect();
    (unknown, misread)
}

/// `base` followed by `relative`.
pub(crate) fn append(base: &JsonPath, relative: &JsonPath) -> JsonPath {
    let mut path = base.clone();
    path.append(relative);
    path
}

/// `base` followed by `key`: the one function behind the component readers' `join` and this
/// reader's `child`.
pub(crate) use crate::component::parse::join as child;

/// The root span of a slice.
pub(crate) fn span_of(slice: JsonSlice<'_>) -> Span {
    let to_u32 = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
    Span {
        offset: to_u32(slice.offset),
        len: to_u32(slice.text.len()),
    }
}

/// The slice a raw member captured, with its file offset.
fn nested<'a>(within: JsonSlice<'a>, raw: &RawJson<'a>) -> JsonSlice<'a> {
    // A slice the borrowed reader captured always lies in `within`; anything else reads as
    // `null`, which every caller rejects.
    within.nested(raw).unwrap_or(JsonSlice {
        text: "null",
        offset: within.offset,
    })
}

/// A raw member as a `Value` (`null` when it is no JSON, which the shell's parse rules out).
fn json_of(slice: JsonSlice<'_>) -> Value {
    bedrock_addon_core::json::from_str_jsonc::<Value>(slice.text).unwrap_or(Value::NULL)
}

/// Reads the newest struct from its payload.
pub(crate) fn entity(
    payload: JsonSlice<'_>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<Entity> {
    let mut reader = Reader {
        load: cx.load,
        original_version: cx.original_version,
        d,
        spans: true,
    };
    let entity = read_entity(&mut reader, payload, &cx.at)?;
    if let Some(found) =
        member_version_value(payload).and_then(|version| payload_version_unread(Some(&version), cx))
    {
        reader.d.push(found);
    }
    Some(entity)
}

/// The payload's own `format_version` member, when it has one.
fn member_version_value(payload: JsonSlice<'_>) -> Option<Value> {
    let members =
        bedrock_addon_core::json::from_str_borrowed_jsonc::<IndexMap<String, RawJson<'_>>>(
            payload.text,
        )
        .ok()?;
    bedrock_addon_core::json::from_str_jsonc::<Value>(members.get("format_version")?.as_str()).ok()
}

/// What is to tell of the payload's `format_version` (`Family::payload_version`): nothing
/// when it is the version the document was read at (the envelope has none, or the same);
/// otherwise the envelope's won, the member was not read and is not written (an information:
/// the game reads the envelope's alone).
pub(crate) fn payload_version_unread(
    version: Option<&Value>,
    cx: &ParseCx<'_>,
) -> Option<Diagnostic> {
    let version = version.filter(|version| !version.is_null())?;
    if FormatVersionInput::from_json(version) == Some(cx.written) {
        return None;
    }
    Some(Diagnostic::new(
        DiagnosticCode::UnrecognizedField { text: None },
        Severity::Info,
        Phase::Parse,
        Location::new(None, None, child(&cx.at, "format_version"), None),
    ))
}

/// [`entity`] with an explicit reader: the newest struct from `payload`, findings below `at`.
pub(crate) fn read_entity(
    r: &mut Reader<'_, '_>,
    payload: JsonSlice<'_>,
    at: &JsonPath,
) -> Option<Entity> {
    let start = r.d.len();
    let entity = read_entity_members(r, payload, at);
    if r.original_version >= STRICT_SLICES {
        // `entity_version` is checked by hand (its "Error loading 'entity_version' …" texts),
        // not read by the slice's types: a member inside it that its checks do not read is no
        // unknown member of the slice (the entity-version check tells it); the member itself, below 1.26.20, is.
        let hand_checked = child(&child(at, "description"), "entity_version");
        unknown_members_are_errors(r.d, start, &hand_checked);
    }
    entity
}

/// A document of a slice the game reads with the strict error codes (1.21.89 on, and beta) is
/// refused for an unknown member or component of what the slice types. The
/// framework turns the payload's own findings into errors when the newest struct parses
/// the file; a document the steps carry here is read after it, so the read does it for what
/// it told from `start` on. A component converted from below its typed version is no part of
/// the slice (its findings are the conversion's, `Phase::Upgrade`) and keeps its warnings.
fn unknown_members_are_errors(d: &mut Diagnostics, start: usize, hand_checked: &JsonPath) {
    for finding in d.iter_mut().skip(start) {
        if finding.severity == Severity::Warning
            && finding.phase == Phase::Parse
            && !(finding.location.json_path.starts_with(hand_checked)
                && finding.location.json_path.len() > hand_checked.len())
            && matches!(
                finding.code,
                DiagnosticCode::UnrecognizedField { .. }
                    | DiagnosticCode::UnrecognizedComponent { .. }
            )
        {
            finding.severity = Severity::Error;
        }
    }
}

/// [`read_entity`]'s reading.
fn read_entity_members(
    r: &mut Reader<'_, '_>,
    payload: JsonSlice<'_>,
    at: &JsonPath,
) -> Option<Entity> {
    let before = r.d.len();
    let shell: Shell<'_> = r.typed(payload, at, Severity::Error)?;
    if !r.spans {
        // A payload the steps wrote: its own unknown members were told by the parse of the
        // older struct, whose `extra` the steps write back; they are not told twice.
        let mut index = 0;
        r.d.retain(|finding| {
            let told_before = index >= before
                && matches!(finding.code, DiagnosticCode::UnrecognizedField { .. })
                && finding.location.json_path.len() == at.len() + 1
                && finding.location.json_path.starts_with(at);
            index += 1;
            !told_before
        });
    }
    let written = Written {
        components: shell.components.is_some(),
        component_groups: shell.component_groups.is_some(),
        events: shell.events.is_some(),
        ..Written::default()
    };
    let Some(description) = shell.description else {
        // The file is skipped; the game's own base-game packs ship such files, so there it is
        // only an information.
        let severity = if r.load.is_base_game_pack {
            // An information in a base-game pack: the game skips the entity without a message,
            // and its own packs ship such a file.
            Severity::Info
        } else {
            Severity::Error
        };
        r.report(
            DiagnosticCode::MissingRequiredField { text: None },
            severity,
            child(at, "description"),
            Some(span_of(payload)),
        );
        return None;
    };
    let description_at = child(at, "description");
    let read = read_description(r, nested(payload, &description), &description_at)?;
    let mut description = read.description;
    let components = match shell.components {
        Some(entries) => read_components(r, &entries, payload, &child(at, "components")),
        None => ComponentMap::new(),
    };
    let component_groups = read_groups(r, shell.component_groups, payload, at);
    let events = shell
        .events
        .unwrap_or_default()
        .into_iter()
        .map(|(name, node)| {
            let event = Event::new(name.clone(), node);
            (name, event)
        })
        .collect();
    // An unknown member is told and not kept. From 1.21.89 it refuses the
    // document (`unknown_members_are_errors`); below, the legacy slice ignores it and the
    // conversion to the strict newest struct does not carry it, so the write-back has none.
    let _ = shell.extra;
    let mut extra = Extra::default();
    let brain = shell.brain.and_then(|raw| {
        let slice = nested(payload, &raw);
        if r.original_version < BRAIN_SINCE {
            // The member arrives with the 1.21.130 format; in an older document it is
            // an unknown member.
            r.report(
                DiagnosticCode::UnrecognizedField { text: None },
                // The game logs this at the warning level for a document below 1.21.89 and lifts it
                // to an error from that version on.
                Severity::Warning,
                child(at, "brain"),
                Some(span_of(slice)),
            );
            return None;
        }
        r.typed::<Option<Brain>>(slice, &child(at, "brain"), Severity::Error)
            .flatten()
    });
    let mut deprecated = shell.deprecated.map(|shell| DeprecatedData {
        aliases: shell.aliases.unwrap_or_default(),
    });
    if let Some(aliases) = read.aliases {
        settle_aliases(r, aliases, &mut deprecated, &description_at);
    }
    let upgrades = settle_entity_version(
        r,
        &mut description,
        read.entity_version,
        shell.upgrades,
        &mut extra,
        at,
    );
    // `format_version` (the pre-1.8 in-payload version) was consumed by the version selection
    // (`payload_version_unread` tells of one that was not) and `do_not_upgrade` by the
    // chain; neither is kept. The skip key did what it asks for and has no
    // place in the newest struct: noted where it is.
    let _ = shell.format_version;
    if shell.do_not_upgrade.is_some() {
        r.report(
            DiagnosticCode::DroppedField { text: None },
            Severity::Info,
            child(at, "do_not_upgrade"),
            None,
        );
    }
    Some(Entity {
        description,
        components,
        component_groups,
        events,
        brain,
        upgrades,
        deprecated,
        extra,
        written,
    })
}

/// `component_groups`: each group's components; a group that is no object is dropped.
fn read_groups(
    r: &mut Reader<'_, '_>,
    groups: Option<IndexMap<String, RawJson<'_>>>,
    payload: JsonSlice<'_>,
    at: &JsonPath,
) -> IndexMap<String, ComponentGroup> {
    let mut out = IndexMap::new();
    let all_groups_at = child(at, "component_groups");
    for (name, raw) in groups.unwrap_or_default() {
        let slice = nested(payload, &raw);
        let group_at = child(&all_groups_at, &name);
        match bedrock_addon_core::json::from_str_borrowed_jsonc::<
            Option<IndexMap<String, RawJson<'_>>>,
        >(slice.text)
        {
            Ok(Some(entries)) => {
                let components = read_components(r, &entries, slice, &group_at);
                out.insert(name, ComponentGroup { components });
            }
            // A `null` group from 1.21.90 (an older document's is dropped by the 1.21.89 step).
            // The entity's input is JsonCpp (rule N1): a member written `null` is absent, so
            // the group is not there; the game says nothing, so the crate reports an
            // information.
            Ok(None) => r.report(
                DiagnosticCode::InvalidValue { text: None },
                Severity::Info,
                group_at,
                Some(span_of(slice)),
            ),
            Err(_) => r.report(
                DiagnosticCode::InvalidType { text: None },
                // A group that is no object refuses the entity: cereal's "expected an object" is
                // an error in both masks, followed by the error-level "failed to load" line.
                Severity::Error,
                group_at,
                Some(span_of(slice)),
            ),
        }
    }
    out
}

/// One component map (`components` or a group), each entry parsed from its own slice. A goal,
/// Brain memory goal or memory sensor is read by `goal::versioned` at the document's version
/// (lenient below the goal's typed version, strict with the later typed → typed conversions at
/// or above it); every other component by its table row, strictly when it is typed at or below
/// the document's version and leniently below (vanilla's legacy upgrade callbacks). A generic
/// memory goal `memory_behavior.move_to_position.<name>` is kept under its own key, one entry
/// per name; from 1.26.20 one without a valid name is `GenericGoalSuffixMissing` and dropped
/// A Brain key without the Brain Prototype experiment is `MissingToggles` and
/// dropped. A key nothing reads is kept and reported as `UnrecognizedComponent`.
#[allow(
    clippy::too_many_lines,
    reason = "one pass over the map: the key's routing, its read and what is told of it"
)]
pub(crate) fn read_components(
    r: &mut Reader<'_, '_>,
    entries: &IndexMap<String, RawJson<'_>>,
    within: JsonSlice<'_>,
    at: &JsonPath,
) -> ComponentMap<EntityComponent> {
    let table = <Entity as Family>::components();
    let mut map = ComponentMap::new();
    for (key, raw) in entries {
        let slice = nested(within, raw);
        let path = child(at, key);
        let generic = goal::generic_goal(key);
        if r.original_version >= GENERIC_GOALS_SINCE
            && let Some(mut finding) = generic.finding(path.clone())
        {
            // A warning: the component is dropped and the entity loads.
            finding.location.span = r.spans.then(|| span_of(slice));
            r.d.push(finding);
            continue;
        }
        if is_brain_key(key) && !r.load.experiments.contains(Experiment::BrainPrototype) {
            // The Brain group needs the experiment.
            r.report(
                DiagnosticCode::MissingToggles {
                    text: Some(VanillaText::TurnOnToggle {
                        what: "component",
                        name: key.clone(),
                        toggle: Experiment::BrainPrototype.save_name().unwrap_or_default(),
                    }),
                },
                // The game has no entity form of the toggle text (only the item's and the
                // block's), so the crate reports an information.
                Severity::Info,
                path,
                Some(span_of(slice)),
            );
            continue;
        }
        let family = match generic {
            goal::GenericGoal::Named { base, .. } => Some(base),
            _ => None,
        };
        let named = family.is_some();
        if goal::versioned(key).is_some() && slice.text.trim() == "null" {
            // Rule N1 at the component map, as for any component: a goal written as `null` is
            // absent; the write-back leaves it out, so its place carries an information.
            let mut told = Diagnostics::default();
            bedrock_addon_core::json::tell_null_members(&[JsonPath::new()], &path, &mut told);
            for finding in told {
                r.d.push(finding);
            }
            continue;
        }
        let mut found = Diagnostics::default();
        // A key read below the version that types it is converted by the legacy reader, out
        // of the typed slice: what it tells is the conversion's.
        let typed_key = family.unwrap_or(key.as_str());
        if BRAIN_1_26_50.contains(&typed_key) && r.original_version < BRAIN_1_26_50_SINCE {
            // No slice before 1.26.50 binds them and no legacy reader reads them: an unknown
            // component, refused from 1.21.89.
            r.report(
                DiagnosticCode::UnrecognizedComponent { text: None },
                // The game logs this at the warning level for a document below 1.21.89 and lifts it
                // to an error from that version on.
                Severity::Warning,
                path,
                Some(span_of(slice)),
            );
            continue;
        }
        let converted = first_typed(typed_key).is_some_and(|since| r.original_version < since);
        // The game's schema reader of the key validates the member tree before it parses; its lines are told and the read is of what the validation leaves (`{}` when it
        // failed: the component keeps its defaults).
        let original = slice;
        let mut schema = schema_read(typed_key, slice, r.original_version);
        let replaced = schema.as_mut().and_then(|read| read.text.take());
        let respanned = replaced.is_some();
        let slice = replaced.as_deref().map_or(slice, JsonSlice::new);
        // A goal the game types first after the JSON structs is read by its legacy reader in
        // this read: the `min` / `max` of its ranges are not read.
        let ranged = goal_ranges_read(key, typed_key, slice, r.original_version, &mut found);
        let respanned = respanned || ranged.is_some();
        let slice = ranged.as_deref().map_or(slice, JsonSlice::new);
        if let Some(read) = schema {
            for told in read.found.into_told() {
                match told {
                    Told::Finding(finding) => found.push(*finding),
                    Told::Moved { from, to } => found.moved(from, to),
                }
            }
        }
        let value = if let Some(parse) = goal::versioned(key) {
            parse(slice, r.original_version, r.load, &mut found)
        } else if let Some(row) = table.get(key) {
            let mode = if converted {
                ConvertMode::Lenient
            } else if first_typed(key).is_some() {
                ConvertMode::Strict
            } else {
                ConvertMode::Lenient
            };
            (row.parse)(slice, mode, r.load, &mut found)
        } else {
            // An unknown component is told and not kept (from 1.21.89 it refuses the document).
            r.report(
                DiagnosticCode::UnrecognizedComponent { text: None },
                // The game logs this at the warning level for a document below 1.21.89 and lifts it
                // to an error from that version on.
                Severity::Warning,
                path,
                Some(span_of(slice)),
            );
            continue;
        };
        if value.is_some() && found.is_empty() && slice.text.trim() == "null" {
            // A component that is `null` is read as the component of defaults (facet reads
            // `null` as a struct of defaults): kept so, and told.
            found.push(Diagnostic::new(
                DiagnosticCode::InvalidType { text: None },
                // The game reads a `null` component as absent without a line, so the crate
                // reports an information.
                Severity::Info,
                Phase::Parse,
                Location::new(None, None, JsonPath::new(), None),
            ));
        }
        let unknown: Vec<JsonPath> = found
            .iter()
            .filter(|f| matches!(f.code, DiagnosticCode::UnrecognizedField { .. }))
            .map(|f| f.location.json_path.clone())
            .collect();
        let value = match value {
            Some(_) if !unknown.is_empty() => {
                // What the read told as unknown is not kept: the component is read
                // again without it (its findings are the first read's).
                let text = without_members(&json_of(slice), &unknown);
                let again = JsonSlice::new(&text);
                let mut quiet = Diagnostics::default();
                let reread = if let Some(parse) = goal::versioned(key) {
                    parse(again, r.original_version, r.load, &mut quiet)
                } else if let Some(row) = table.get(key) {
                    let mode = if converted || first_typed(key).is_none() {
                        ConvertMode::Lenient
                    } else {
                        ConvertMode::Strict
                    };
                    (row.parse)(again, mode, r.load, &mut quiet)
                } else {
                    None
                };
                reread.or(value)
            }
            value => value,
        };
        let reported = !found.is_empty();
        let inserted = step_inserted_member(key, slice, r.original_version);
        for told in found.into_told() {
            let mut finding = match told {
                Told::Finding(finding) => *finding,
                // A member the lenient read moved before reading it: what is told of it
                // from here on is told at the member the file has.
                Told::Moved { from, to } => {
                    r.d.moved(append(&path, &from), append(&path, &to));
                    continue;
                }
            };
            if inserted.is_some_and(|member| is_gate_finding(&finding, member)) {
                continue;
            }
            if converted {
                finding.phase = Phase::Upgrade;
                // The legacy schema reader validated the component in its step and tells a required child of its own; the typed struct's requirement is not its.
                if schema_converted(typed_key)
                    && matches!(
                        finding.code,
                        DiagnosticCode::MissingRequiredField { text: None }
                    )
                {
                    continue;
                }
            }
            if (converted || first_typed(typed_key).is_none())
                && finding.severity == Severity::Warning
                && legacy_reader_is_silent(&finding.code)
            {
                // The game's legacy reader says nothing of a member it does not read or of a
                // value it cannot use, so the crate reports an information.
                finding.severity = Severity::Info;
            }
            finding.location.json_path = append(&path, &finding.location.json_path);
            if !r.spans || respanned {
                finding.location.span = None;
            }
            r.d.push(finding);
        }
        match value {
            Some(value) if value.name().is_some() && named => {
                map.insert_typed_as(key, value, false);
            }
            Some(value) if value.name().is_some() => {
                map.insert_typed(value, false);
            }
            Some(_) => map.insert_dynamic(key, json_of(slice)),
            None if reported => {}
            None => r.report(
                DiagnosticCode::DroppedComponent { text: None },
                // A read that failed without a finding of its own: the game writes no line, so
                // the crate reports an information.
                Severity::Info,
                path,
                Some(span_of(original)),
            ),
        }
    }
    map
}

/// `json` as text without the members at `paths` (relative to it; a path that names no member
/// is passed over).
fn without_members(json: &Value, paths: &[JsonPath]) -> String {
    let mut json = json.clone();
    for path in paths {
        let steps: Vec<_> = path.segments().collect();
        let Some((last, within)) = steps.split_last() else {
            continue;
        };
        let mut node = Some(&mut json);
        for step in within {
            node = node.and_then(|value| match step {
                bedrock_addon_core::diagnostic::JsonPathSegment::Key(key) => {
                    value.as_object_mut()?.get_mut(key)
                }
                bedrock_addon_core::diagnostic::JsonPathSegment::Index(index) => {
                    value.as_array_mut()?.get_mut(*index)
                }
            });
        }
        if let (Some(object), bedrock_addon_core::diagnostic::JsonPathSegment::Key(key)) =
            (node.and_then(Value::as_object_mut), last)
        {
            object.remove(key);
        }
    }
    bedrock_addon_core::json::to_json(&json).unwrap_or_else(|_| "null".to_owned())
}

/// What the game's schema reader of a component did with its JSON before the parse:
/// the findings (paths below the component) and the text the crate reads instead of the file's.
struct SchemaRead {
    /// The validation's findings, then the moves of the elements the lenient mode took out.
    found: Diagnostics,
    /// `{}` when the validation failed, the JSON without what the lenient mode took out.
    text: Option<String>,
}

/// The game's schema validation of the component `key` at `slice`, when the read itself is
/// where the game's schema reader reads it in a document at `version`: a reader that was never
/// migrated (every version), or a migrated one typed first after 1.26.40, whose conversion is the
/// lenient read of the 1.26.40 → 1.26.50 step. The others convert in their step
/// (`history::ops::schema_conversions`). Strict from 1.15.0 ([`schema::STRICT_SINCE`]). A
/// component written as `null` is left to rule N1.
fn schema_read(key: &str, slice: JsonSlice<'_>, version: FormatVersion) -> Option<SchemaRead> {
    let owner = schema::component(key).filter(|owner| {
        owner.reads(version)
            && owner
                .typed_since
                .is_none_or(|(a, b, c)| FormatVersion::new(a, b, c) > LAST_JSON_STRUCT)
    })?;
    let json = json_of(slice);
    if json.is_null() {
        return None;
    }
    let mut outcome = schema::validate(owner.members, &json, version >= schema::STRICT_SINCE);
    outcome.check_numbers(owner.name, &json);
    let text = outcome
        .replacement()
        .map(|json| bedrock_addon_core::json::to_json(&json).unwrap_or_else(|_| "{}".to_owned()));
    let mut found = outcome.findings(&JsonPath::new(), Phase::Parse);
    outcome.tell_left_out(&JsonPath::new(), &mut found);
    Some(SchemaRead { found, text })
}

/// The text the legacy reader of goal `key` reads in this read (a goal typed first after
/// [`LAST_JSON_STRUCT`], in a document below its typed version): without the `min` / `max` of its
/// ranges, each told as dropped (an information); `None` when nothing was taken out.
fn goal_ranges_read(
    key: &str,
    typed_key: &str,
    slice: JsonSlice<'_>,
    version: FormatVersion,
    found: &mut Diagnostics,
) -> Option<String> {
    goal::versioned(key)?;
    let since = first_typed(typed_key)?;
    if since <= LAST_JSON_STRUCT || version >= since {
        return None;
    }
    let mut json = json_of(slice);
    let dropped = goal::legacy_ranges(key, json.as_object_mut()?);
    if dropped.is_empty() {
        return None;
    }
    for (at, code) in dropped {
        found.push(Diagnostic::new(
            code,
            Severity::Info,
            Phase::Parse,
            Location::new(None, None, at, None),
        ));
    }
    bedrock_addon_core::json::to_json(&json).ok()
}

/// Whether the game converts `key` with a schema-driven legacy reader in a step (typed first at or
/// below [`LAST_JSON_STRUCT`]): `history::ops::schema_conversions` validated it there.
fn schema_converted(key: &str) -> bool {
    schema::component(key)
        .and_then(|owner| owner.typed_since)
        .is_some_and(|(a, b, c)| FormatVersion::new(a, b, c) <= LAST_JSON_STRUCT)
}

/// The newest struct that holds the payload as JSON: a component typed first after it converts
/// in the read of its step to 1.26.50.
const LAST_JSON_STRUCT: FormatVersion = FormatVersion::new(1, 26, 40);

/// Whether the game's legacy reader says nothing of what the lenient typed read reports with
/// `code`. A hand-written reader reads the keys it knows with `asFloat` / `asInt` / `asBool`
/// defaults and tells nothing of an unknown member, of a value of another type or of a value
/// outside the typed bounds; the few values it checks itself it tells with its own texts, which
/// the goals' checks carry. A schema-driven reader tells its own lines in its validation and
/// numeric checks (`schema`), before this read: what the read still tells is the crate's.
fn legacy_reader_is_silent(code: &DiagnosticCode) -> bool {
    matches!(
        code,
        DiagnosticCode::UnrecognizedField { .. } | DiagnosticCode::InvalidValue { text: None }
    )
}

/// The member a step inserted with the value the goal's member gate leaves: the
/// 1.26.20 → 1.26.30 step writes `pickup_items.stop_if_holding_item: false`, its default, into
/// every older document, so the gate's `UnrecognizedField` for it says nothing the author
/// wrote. A value other than `false` is the author's and keeps its finding.
fn step_inserted_member(
    key: &str,
    slice: JsonSlice<'_>,
    original_version: FormatVersion,
) -> Option<&'static str> {
    const MEMBER: &str = "stop_if_holding_item";
    let inserted = key == "minecraft:behavior.pickup_items"
        && original_version < STOP_IF_HOLDING_ITEM_SINCE
        && json_of(slice)
            .as_object()
            .and_then(|goal| goal.get(MEMBER))
            .is_some_and(|value| value.as_bool() == Some(false));
    inserted.then_some(MEMBER)
}

/// Whether `finding` is the member gate's `UnrecognizedField` for `member` of the component.
fn is_gate_finding(finding: &Diagnostic, member: &str) -> bool {
    matches!(finding.code, DiagnosticCode::UnrecognizedField { .. })
        && finding.location.json_path.is_key(member)
}

/// The Brain keys first bound in the 1.26.50 slice, with no reader before it.
const BRAIN_1_26_50: [&str; 4] = [
    "minecraft:memory_behavior.jump_to_block",
    "minecraft:memory_behavior.prepare_ram_attack",
    "minecraft:memory_behavior.ram_attack",
    "minecraft:memory_sensor.find_ram_charge_position",
];
/// The version that binds [`BRAIN_1_26_50`].
const BRAIN_1_26_50_SINCE: FormatVersion = FormatVersion::new(1, 26, 50);

/// The struct version that reads `pickup_items.stop_if_holding_item`.
const STOP_IF_HOLDING_ITEM_SINCE: FormatVersion = FormatVersion::new(1, 26, 30);

/// From 1.26.20 the generic memory goal needs its suffix; below, the bare key is a legacy key
/// the 1.26.20 step renames.
const GENERIC_GOALS_SINCE: FormatVersion = FormatVersion::new(1, 26, 20);

/// The Brain group: the memory goals and sensors other
/// than the generic `memory_behavior.move_to_position.<name>`, which is not gated.
fn is_brain_key(key: &str) -> bool {
    let Some(name) = key.strip_prefix("minecraft:") else {
        return false;
    };
    name.starts_with("memory_sensor.")
        || (name.starts_with("memory_behavior.")
            && !name.starts_with("memory_behavior.move_to_position"))
}

/// Below 1.21.10 the aliases move to the side-car with a `DeprecatedMember`
/// information (the game still registers them, without a line); from 1.21.10 vanilla reports "Deprecated field:
/// Aliases." and drops them.
fn settle_aliases(
    r: &mut Reader<'_, '_>,
    aliases: IndexMap<String, Value>,
    deprecated: &mut Option<DeprecatedData>,
    description_at: &JsonPath,
) {
    let path = child(description_at, "aliases");
    if r.original_version < ALIASES_DEPRECATED {
        r.report(
            DiagnosticCode::DeprecatedMember { text: None },
            // Below 1.21.10 the game registers the aliases without a line, so the crate
            // reports an information.
            Severity::Info,
            path,
            None,
        );
        deprecated
            .get_or_insert_with(DeprecatedData::default)
            .aliases = aliases;
    } else {
        // The game logs this at the error level yet goes on loading the entity, so the crate
        // reports a warning.
        r.report(
            DiagnosticCode::DeprecatedMember {
                text: Some(VanillaText::Fixed("Deprecated field: Aliases.")),
            },
            Severity::Warning,
            path,
            None,
        );
    }
}

/// What [`read_description`] read.
struct ReadDescription {
    description: Description,
    aliases: Option<IndexMap<String, Value>>,
    entity_version: Option<Value>,
}

/// Reads `description`. `aliases` and `entity_version` come back raw: the version rules settle
/// them.
fn read_description(
    r: &mut Reader<'_, '_>,
    slice: JsonSlice<'_>,
    at: &JsonPath,
) -> Option<ReadDescription> {
    let shell: DescriptionShell<'_> = r.typed(slice, at, Severity::Error)?;
    // An unknown member of `description` is told and not kept.
    let _ = shell.extra;
    let mut extra = Extra::default();
    if let Some(experimental) = shell.is_experimental {
        extra.insert("is_experimental".to_owned(), experimental);
    }
    let mut properties = IndexMap::new();
    let mut failed = false;
    for (name, raw) in shell.properties.unwrap_or_default() {
        let path = child(&child(at, "properties"), &name);
        match read_property(r, nested(slice, &raw), &path) {
            Some(property) => {
                properties.insert(name, property);
            }
            None => failed = true,
        }
    }
    if failed {
        // One bad property drops the whole property group; the entity loads. The game logs
        // the text at the error level and returns: a kept error.
        properties = IndexMap::new();
        r.report_kept(
            DiagnosticCode::DroppedField {
                text: Some(VanillaText::Fixed("Error loading Actor Properties")),
            },
            child(at, "properties"),
            None,
        );
    }
    let scripts = shell
        .scripts
        .and_then(|raw| read_scripts(r, nested(slice, &raw), &child(at, "scripts")));
    let spawn_category = shell.spawn_category.and_then(|value| {
        let category = value
            .as_string()
            .and_then(|s| SpawnCategory::from_name(s.as_str()));
        if category.is_none() {
            // an unknown string, a number or `null` is invalid; the field is dropped.
            r.report(
                DiagnosticCode::InvalidValue { text: None },
                // The game writes no content-log line for a bad `spawn_category`, so the crate
                // reports an information.
                Severity::Info,
                child(at, "spawn_category"),
                None,
            );
        }
        category
    });
    Some(ReadDescription {
        description: Description {
            identifier: shell.identifier,
            runtime_identifier: shell.runtime_identifier,
            is_spawnable: shell.is_spawnable,
            is_summonable: shell.is_summonable,
            properties,
            animations: shell.animations.unwrap_or_default(),
            scripts,
            spawn_category,
            property_memory_mapping: shell.property_memory_mapping,
            entity_version: None,
            extra,
        },
        aliases: shell.aliases,
        entity_version: shell.entity_version,
    })
}

/// One property. A `default` string of an `enum` property that is one of its `values` is the
/// value; any other string is a Molang expression. A bad `type`, `range` or `default` is a
/// kept error and `None`: the caller drops the whole property group, and the entity loads. The
/// game logs each line (a missing JSON node, "Error loading property %s", a bad type, default
/// or range) at the error level and then abandons the property group.
fn read_property(r: &mut Reader<'_, '_>, slice: JsonSlice<'_>, at: &JsonPath) -> Option<Property> {
    let shell: PropertyShell<'_> = r.typed_kept(slice, at)?;
    let kind = match shell.kind.as_str() {
        "bool" => PropertyType::Bool,
        "int" => PropertyType::Int,
        "float" => PropertyType::Float,
        "enum" => PropertyType::Enum,
        _ => {
            r.report_kept(
                DiagnosticCode::InvalidValue { text: None },
                child(at, "type"),
                None,
            );
            return None;
        }
    };
    let values = shell.values.unwrap_or_default();
    let default = match shell.default {
        None => None,
        Some(raw) => {
            let default_slice = nested(slice, &raw);
            let default_at = child(at, "default");
            let json = json_of(default_slice);
            if let Some(b) = json.as_bool() {
                Some(PropertyDefault::Bool(b))
            } else if let Some(n) = json.as_number() {
                if n.is_integer() {
                    // an `int` default keeps its `i32` value.
                    // jsoncpp's `asInt` asserts here (a developer line, not a content-log line)
                    // and the group drop is the crate's own, so it is an information.
                    let Some(i) = n.to_i64().and_then(|i| i32::try_from(i).ok()) else {
                        r.report(
                            DiagnosticCode::InvalidValue { text: None },
                            Severity::Info,
                            default_at,
                            Some(span_of(default_slice)),
                        );
                        return None;
                    };
                    Some(PropertyDefault::Int(i))
                } else {
                    Some(PropertyDefault::Float(n.to_f64_lossy()))
                }
            } else if let Some(s) = json.as_string() {
                if kind == PropertyType::Enum && values.iter().any(|v| v == s.as_str()) {
                    Some(PropertyDefault::Enum(s.as_str().to_owned()))
                } else {
                    Some(PropertyDefault::Expression(
                        r.typed_kept::<MolangSource>(default_slice, &default_at)?,
                    ))
                }
            } else {
                r.report_kept(
                    DiagnosticCode::InvalidType { text: None },
                    default_at,
                    Some(span_of(default_slice)),
                );
                return None;
            }
        }
    };
    let range = match shell.range {
        None => None,
        Some(range) => match <[Value; 2]>::try_from(range) {
            Ok(pair) if pair.iter().all(Value::is_number) => Some(pair),
            _ => {
                r.report_kept(
                    DiagnosticCode::InvalidValue { text: None },
                    child(at, "range"),
                    None,
                );
                return None;
            }
        },
    };
    Some(Property {
        kind,
        default,
        range,
        values,
        client_sync: shell.client_sync,
        // Told, not kept.
        extra: Extra::default(),
    })
}

/// `scripts`: string entries as written; an object entry's member values are Molang strings
/// (another type is `InvalidType` and the entry is dropped). Vanilla skips an object entry with
/// more or fewer than one member, with an error it logs and goes past.
fn read_scripts(r: &mut Reader<'_, '_>, slice: JsonSlice<'_>, at: &JsonPath) -> Option<Scripts> {
    let shell: ScriptsShell<'_> = r.typed(slice, at, Severity::Error)?;
    let animate_written = shell.animate.is_some();
    let mut animate = Vec::new();
    // The file's index of each entry held: a finding told later is at the file's element.
    let mut places = Vec::new();
    let animate_at = child(at, "animate");
    for (index, raw) in shell.animate.unwrap_or_default().iter().enumerate() {
        let entry_slice = nested(slice, raw);
        let mut entry_at = animate_at.clone();
        entry_at.push_index(index);
        if let Some(name) = json_of(entry_slice)
            .as_string()
            .map(|s| s.as_str().to_owned())
        {
            animate.push(AnimateEntry::Always(name));
            places.push(index);
            continue;
        }
        let Ok(members) = bedrock_addon_core::json::from_str_borrowed_jsonc::<
            IndexMap<String, RawJson<'_>>,
        >(entry_slice.text) else {
            r.report(
                DiagnosticCode::InvalidType { text: None },
                // The game takes a string or an object and skips another type without a line, so
                // the crate reports an information.
                Severity::Info,
                entry_at,
                Some(span_of(entry_slice)),
            );
            continue;
        };
        if members.len() != 1 {
            // The game logs this at the error level and keeps the script: a kept error.
            r.report_kept(
                DiagnosticCode::InvalidValue {
                    text: Some(VanillaText::AnimationScriptObjectSize {
                        size: members.len(),
                    }),
                },
                entry_at.clone(),
                Some(span_of(entry_slice)),
            );
        }
        let mut conditions = IndexMap::new();
        let mut valid = true;
        for (name, raw) in &members {
            let value_slice = nested(entry_slice, raw);
            let value_at = child(&entry_at, name);
            if !json_of(value_slice).is_string() {
                r.report(
                    DiagnosticCode::InvalidType { text: None },
                    // The condition is built from a string only and the game logs no line for
                    // another type, so the crate reports an information.
                    Severity::Info,
                    value_at,
                    Some(span_of(value_slice)),
                );
                valid = false;
                continue;
            }
            // A failed read of the condition: kept, as the expression node is.
            if let Some(source) = r.typed_kept::<MolangSource>(value_slice, &value_at) {
                conditions.insert(name.clone(), source);
            }
        }
        if valid {
            animate.push(AnimateEntry::Conditional(conditions));
            places.push(index);
        }
    }
    let whole = places.iter().copied().eq(0..places.len());
    Some(Scripts {
        animate,
        // Told, not kept.
        extra: Extra::default(),
        written: Written {
            animate: animate_written,
            animate_places: (!whole).then(|| places.into_boxed_slice()),
            ..Written::default()
        },
    })
}

/// `entity_version` and `upgrades`. Below 1.26.20 both are unknown members (kept
/// in `extra`); in [1.26.20, 1.26.30) both need Upcoming Creator Features; vanilla's structure
/// checks and the handler lookup drop both, and the entity loads versionless. Returns the
/// `upgrades` the entity keeps.
fn settle_entity_version(
    r: &mut Reader<'_, '_>,
    description: &mut Description,
    entity_version: Option<Value>,
    upgrades: Option<IndexMap<String, Value>>,
    extra: &mut Extra,
    at: &JsonPath,
) -> IndexMap<String, Value> {
    let description_at = child(at, "description");
    let version_at = child(&description_at, "entity_version");
    if r.original_version < ENTITY_VERSION_SINCE {
        if let Some(value) = entity_version {
            r.report(
                DiagnosticCode::UnrecognizedField { text: None },
                // The game logs this at the warning level for a document below 1.21.89 and lifts it
                // to an error from that version on.
                Severity::Warning,
                version_at,
                None,
            );
            // Told, not kept.
            let _ = value;
        }
        if let Some(upgrades) = upgrades {
            r.report(
                DiagnosticCode::UnrecognizedField { text: None },
                // The game logs this at the warning level for a document below 1.21.89 and lifts it
                // to an error from that version on.
                Severity::Warning,
                child(at, "upgrades"),
                None,
            );
            // Told, not kept.
            let _ = (upgrades, &extra);
        }
        return IndexMap::new();
    }
    if entity_version.is_none() && upgrades.is_none() {
        return IndexMap::new();
    }
    if r.original_version < ENTITY_VERSION_STABLE && !r.load.upcoming_creator_features {
        // The 1.26.20 changelog's toggle. The game has no such gate: it reads the entity
        // version unconditionally, and its toggle texts exist only for block paths. The note
        // is the crate's own, an information, and nothing is dropped for it.
        let (name, gated) = if entity_version.is_some() {
            ("entity_version", version_at.clone())
        } else {
            ("upgrades", child(at, "upgrades"))
        };
        r.report(
            DiagnosticCode::MissingToggles {
                text: Some(VanillaText::TurnOnToggleBefore {
                    what: "member",
                    name: name.to_owned(),
                    version: ENTITY_VERSION_STABLE,
                    toggle: Experiment::UpcomingCreatorFeatures
                        .save_name()
                        .unwrap_or_default(),
                }),
            },
            Severity::Info,
            gated,
            None,
        );
    }
    let had_upgrades = upgrades.is_some();
    let upgrades = upgrades.unwrap_or_default();
    let Some(value) = entity_version else {
        return upgrades;
    };
    match entity_version_of(&value, &upgrades, &description.identifier) {
        Ok(version) => {
            // The version is rebuilt from what vanilla's checks read: any other member is
            // not kept, and told.
            for unread in unread_version_members(&value) {
                r.report(
                    DiagnosticCode::UnrecognizedField { text: None },
                    // The game reads only the members it knows and logs no line for another, so
                    // the crate reports an information.
                    Severity::Info,
                    append(&version_at, &unread),
                    None,
                );
            }
            description.entity_version = Some(version);
            upgrades
        }
        Err((text, path)) => {
            let invalid_at = append(&version_at, &path);
            if matches!(text, VanillaText::UpgradeHandlerUndefined { .. }) {
                // The handler lookup's text is the crate's own (the game has no such line): an
                // information.
                r.report(
                    DiagnosticCode::InvalidValue { text: Some(text) },
                    Severity::Info,
                    invalid_at,
                    None,
                );
            } else {
                // The game logs every structure check of the entity version at the error level
                // and the entity loads versionless: a kept error.
                r.report_kept(
                    DiagnosticCode::InvalidValue { text: Some(text) },
                    invalid_at,
                    None,
                );
            }
            if !path.is_empty() {
                // The member goes whole for the value inside it: told at the member too.
                r.report(
                    DiagnosticCode::DroppedField { text: None },
                    Severity::Info,
                    version_at,
                    None,
                );
            }
            if had_upgrades {
                upgrades_dropped(r, at);
            }
            IndexMap::new()
        }
    }
}

/// The payload's `upgrades` go with an `entity_version` that is refused or gated (both are
/// dropped): told at the member, without a line of the game's, so as an information.
fn upgrades_dropped(r: &mut Reader<'_, '_>, at: &JsonPath) {
    r.report(
        DiagnosticCode::DroppedField { text: None },
        Severity::Info,
        child(at, "upgrades"),
        None,
    );
}

/// The members of an `entity_version` that [`entity_version_of`] does not read, below it:
/// every member but `upgrades`, and every member of an entry but `upgrade_to` and `name`.
fn unread_version_members(value: &Value) -> Vec<JsonPath> {
    let mut out = Vec::new();
    let Some(object) = value.as_object() else {
        return out;
    };
    for key in object.keys().map(facet_value::VString::as_str) {
        if key != "upgrades" {
            out.push(child(&JsonPath::new(), key));
        }
    }
    let list_at = child(&JsonPath::new(), "upgrades");
    let entries = object.get("upgrades").and_then(Value::as_array);
    for (index, entry) in entries.into_iter().flatten().enumerate() {
        let mut entry_at = list_at.clone();
        entry_at.push_index(index);
        let keys = entry.as_object().into_iter().flat_map(|entry| entry.keys());
        for key in keys.map(facet_value::VString::as_str) {
            if key != "upgrade_to" && key != "name" {
                out.push(child(&entry_at, key));
            }
        }
    }
    out
}

/// Vanilla's structure checks of `entity_version`, in its order, then the handler lookup.
fn entity_version_of(
    value: &Value,
    upgrades: &IndexMap<String, Value>,
    identifier: &EntityId,
) -> Result<EntityVersion, (VanillaText, JsonPath)> {
    let fixed = VanillaText::Fixed;
    let root = JsonPath::new();
    let list_at = child(&root, "upgrades");
    let object = value.as_object().ok_or_else(|| {
        (
            fixed("Error loading 'entity_version' as it is not a JSON object."),
            root.clone(),
        )
    })?;
    let entries = object
        .get("upgrades")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            (
                fixed("Error loading 'entity_version' as 'upgrades' is not an array."),
                list_at.clone(),
            )
        })?;
    let mut out: Vec<UpgradeEntry> = Vec::new();
    for (index, entry) in entries.iter().enumerate() {
        let mut entry_at = list_at.clone();
        entry_at.push_index(index);
        out.push(upgrade_entry(entry, &entry_at)?);
    }
    let mut versions: Vec<u32> = out.iter().map(|e| e.upgrade_to).collect();
    versions.sort_unstable();
    if versions.first().is_some_and(|&v| v != 1) {
        return Err((
            fixed("Error loading 'entity_version' as its version does not start at 1."),
            list_at,
        ));
    }
    for pair in versions.windows(2) {
        if pair[0] == pair[1] {
            return Err((
                VanillaText::EntityVersionDuplicate { version: pair[0] },
                list_at,
            ));
        }
        if pair[1] != pair[0] + 1 {
            return Err((
                VanillaText::EntityVersionGap {
                    identifier: identifier.as_str().to_owned(),
                    from: pair[0],
                    to: pair[1],
                },
                list_at,
            ));
        }
    }
    for (index, entry) in out.iter().enumerate() {
        if !upgrades.contains_key(&entry.name) {
            let mut entry_at = list_at.clone();
            entry_at.push_index(index);
            return Err((
                VanillaText::UpgradeHandlerUndefined {
                    name: entry.name.clone(),
                },
                child(&entry_at, "name"),
            ));
        }
    }
    Ok(EntityVersion { upgrades: out })
}

/// One entry of `entity_version.upgrades`, checked as vanilla checks it.
fn upgrade_entry(
    entry: &Value,
    entry_at: &JsonPath,
) -> Result<UpgradeEntry, (VanillaText, JsonPath)> {
    let fixed = VanillaText::Fixed;
    let entry = entry.as_object().ok_or_else(|| {
        (
            fixed(
                "Error loading 'entity_version' as 'upgrades' contains an entry that is not a JSON object.",
            ),
            entry_at.clone(),
        )
    })?;
    let upgrade_to = entry
        .get("upgrade_to")
        .and_then(Value::as_number)
        .filter(|n| n.is_integer())
        .and_then(facet_value::VNumber::to_u32)
        .ok_or_else(|| {
            (
                fixed(
                    "Error loading 'entity_version' as an upgrade entry does not contain an integer 'upgrade_to'.",
                ),
                child(entry_at, "upgrade_to"),
            )
        })?;
    let name = entry
        .get("name")
        .and_then(Value::as_string)
        .map(|s| s.as_str().to_owned())
        .ok_or_else(|| {
            (
                fixed(
                    "Error loading 'entity_version' as an upgrade entry does not contain a string 'name'.",
                ),
                child(entry_at, "name"),
            )
        })?;
    if name.is_empty() {
        return Err((
            fixed("Error loading 'entity_version' as an upgrade entry's 'name' is empty."),
            child(entry_at, "name"),
        ));
    }
    if upgrade_to == 0 {
        return Err((
            fixed(
                "Error loading 'entity_version' as an upgrade entry 'upgrade_to' version is 0, which is reserved for versionless entities.",
            ),
            child(entry_at, "upgrade_to"),
        ));
    }
    Ok(UpgradeEntry { upgrade_to, name })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What the lenient typed read of a converted component still tells as an unknown member
    /// or an unusable value is the crate's own for every legacy reader — a hand-written one is
    /// silent, a schema-driven one told its own lines in its validation (`schema`) — so it is
    /// an information, also for `timer`, a schema-driven reader, at 1.26.50.
    #[test]
    fn every_legacy_reader_is_silent_where_the_typed_read_tells_its_own_finding() {
        assert!(legacy_reader_is_silent(&DiagnosticCode::InvalidValue {
            text: None
        }));
        assert!(legacy_reader_is_silent(
            &DiagnosticCode::UnrecognizedField { text: None }
        ));
        // A text of the game's is no finding of the crate's.
        assert!(!legacy_reader_is_silent(&DiagnosticCode::InvalidValue {
            text: Some(VanillaText::Fixed("x"))
        }));
    }
}
