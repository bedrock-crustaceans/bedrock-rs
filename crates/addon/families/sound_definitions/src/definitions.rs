//! `sounds/sound_definitions.json` at its newest shape: the typed read of the upgraded JSON
//! and its rules.

use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::{DiagnosticCode, Diagnostics, Severity, VanillaText};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::id::SoundEventName;
use bedrock_addon_core::json::kit::Written;
use bedrock_addon_core::json::{EncodeError, Value};
use bedrock_addon_core::version::SemVersion;
use bedrock_addon_support::sound::fork::{self, Stored};
use bedrock_addon_support::sound::read::{boolean, float, integer, real, string};
use bedrock_addon_support::sound::report::{Reporter, check_unknown};
use bedrock_addon_support::sound::write::{self, Object, Present, present};

/// Vanilla's text for a merged root that is no object.
pub(crate) const CONTENTS_MALFORMED: &str = "sound_definitions.json contents is malformed";
/// Vanilla's text for a merged `sound_definitions` member that is no object.
pub(crate) const SECTION_MALFORMED: &str = "sound_definitions section is malformed";
/// Vanilla's text for a `music.*` definition of another category.
const NOT_MUSIC: &str = "The sound category of '{}' is not set as music in sound_definitions.json";

/// The longest sound path vanilla's 1024-byte stack string holds.
const MAX_NAME_BYTES: usize = 1023;

/// The categories a definition may name.
// The seven per-category volume options of the sound engine, plus `music`, `ui`, `bottle` and
// `bucket`.
pub const CATEGORIES: &[&str] = &[
    "ambient", "block", "bottle", "bucket", "hostile", "music", "neutral", "player", "record",
    "ui", "weather",
];

/// The sound definitions of the resource packs, `sounds/sound_definitions.json`
/// (`#[doc(alias)]` `sound_definitions.json`): every sound event a pack can play, by name.
///
/// The value is read from the JSON after vanilla's three version steps, so it is always the
/// 1.26.50 shape, and it writes that shape from its typed members: each object's members in
/// the order of its struct's fields, a number as the shortest decimal of its float (a real)
/// or as its integer, then the members the client does not read. Vanilla merges the copies of
/// this file as JSON, member by member, where an absent member keeps a lower pack's and a
/// member written with its default replaces it: each struct remembers the members it was read
/// with and writes those, and any other whose value is not what an absent member reads as. A
/// member the typed value did not read (`sounds` beside an external event, an
/// `external_event_name` that is no string) is kept as written in the struct's `extra`, and so
/// is a list written as an object of entries (`sounds`, `external_event_parameters`,
/// `actions`): the client reads its values in sorted key order, and its merge recurses into an
/// object where it replaces an array whole. A root that is no object is written as it is while
/// the value holds nothing ([`foreign_root`](Self::foreign_root)).
/// [`json`](Self::json) is the upgraded document the value was read from.
#[doc(alias = "sound_definitions.json")]
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct SoundDefinitions {
    /// `format_version`, when it reads as a version (a string or three integers). Vanilla
    /// stamps `"1.26.50"` on every older copy. Written as a version string.
    pub format_version: Option<SemVersion>,
    /// `sound_definitions`: the definitions by sound event name, in file order.
    pub sound_definitions: IndexMap<SoundEventName, SoundDefinition>,
    /// The members of `sound_definitions` whose value is no object, as written: the client
    /// registers an empty event for each. Written after the definitions.
    pub not_definitions: UnknownMembers,
    /// The members of the root the client does not read, and those it reads that the typed
    /// members cannot hold, as written: a `format_version` that is no version, a
    /// `sound_definitions` that is no object.
    pub extra: UnknownMembers,
    /// The root as written when it is no object and not `null` (an array, a number, a string,
    /// a bool): the value reads nothing of it. It is written as it is while the value holds no
    /// definition and no member, since the client's merge treats it otherwise than an empty
    /// object.
    pub foreign_root: Option<Value>,
    /// The members the root was read with.
    pub(crate) read: Present,
    /// The upgraded document, as read; left out of equality.
    pub(crate) json: Written,
}

impl SoundDefinitions {
    /// The upgraded document as JSON, as it was read (for a stack's value, the copies as the
    /// client merges them): what the typed members were read from. An edit of the value does
    /// not change it; the value writes its typed members.
    pub fn json(&self) -> &Value {
        static NULL: Value = Value::NULL;
        self.json.as_ref().unwrap_or(&NULL)
    }

    /// The document the value writes.
    ///
    /// # Errors
    ///
    /// `EncodeError::NotRepresentable` for a number that is no finite float.
    pub fn to_value(&self) -> Result<Value, EncodeError> {
        if let Some(root) = &self.foreign_root
            && self.format_version.is_none()
            && self.sound_definitions.is_empty()
            && self.not_definitions.is_empty()
            && self.extra.is_empty()
        {
            return Ok(root.clone());
        }
        let mut root = Object::new(&self.read, &self.extra);
        root.member("format_version", self.format_version.is_none(), || {
            Ok(self.format_version.as_ref().map_or(Value::NULL, |version| {
                Value::from(version.to_string().as_str())
            }))
        })?;
        let empty = self.sound_definitions.is_empty() && self.not_definitions.is_empty();
        root.member("sound_definitions", empty, || {
            let mut members = Vec::new();
            for (name, definition) in &self.sound_definitions {
                members.push((
                    name.as_str(),
                    write::in_member(name.as_str(), definition.to_value())?,
                ));
            }
            members.extend(
                self.not_definitions
                    .iter()
                    .map(|(key, value)| (key, value.clone())),
            );
            Ok(write::object(members))
        })?;
        Ok(root.finish())
    }

    /// The definition of the sound event `name`.
    pub fn get(&self, name: &str) -> Option<&SoundDefinition> {
        self.sound_definitions.get(&SoundEventName::parse(name))
    }

    /// The value of a document that is already the 1.26.50 shape, read as it is: no version
    /// step runs and the findings of the read are dropped. `Content::parse` reads a file with
    /// both.
    pub fn from_json(json: Value) -> Self {
        Self::read(json, &mut Diagnostics::default())
    }

    /// Reads an upgraded document.
    pub(crate) fn read(json: Value, diagnostics: &mut Diagnostics) -> Self {
        let root = read_root(&json, diagnostics);
        let format_version = super::upgrade::version_member(&json);
        let mut extra = root.extra;
        // The members the client reads that the typed ones cannot hold, kept as written.
        let unread = [
            ("format_version", format_version.is_none()),
            (
                "sound_definitions",
                !fork::member(&json, "sound_definitions").is_object(),
            ),
        ];
        for (key, unread) in unread {
            if unread && fork::has_member(&json, key) {
                extra.insert(key.to_owned(), fork::member(&json, key).clone());
            }
        }
        Self {
            format_version,
            sound_definitions: root.definitions,
            not_definitions: root.not_definitions,
            extra,
            foreign_root: (!json.is_object() && !json.is_null()).then(|| json.clone()),
            read: present(&json, &ROOT_KEYS),
            json: Written(Some(json)),
        }
    }
}

/// The members of the root the client reads.
const ROOT_KEYS: [&str; 2] = ["format_version", "sound_definitions"];

/// The members of a definition the client reads.
const DEFINITION_KEYS: [&str; 9] = [
    "category",
    "min_distance",
    "max_distance",
    "__use_legacy_max_distance",
    "subtitle",
    "sounds",
    "external_event_name",
    "external_event_parameters",
    "actions",
];

/// The members of an object entry of `sounds` the client reads.
const ENTRY_KEYS: [&str; 8] = [
    "name",
    "volume",
    "pitch",
    "weight",
    "is3D",
    "stream",
    "interruptible",
    "allow_concurrent_streaming",
];

/// The members of an external event's parameter the client reads.
const PARAMETER_KEYS: [&str; 2] = ["parameter", "value"];

/// The members of an action the client reads.
const ACTION_KEYS: [&str; 6] = ["action_type", "name", "target", "enabled", "param", "delay"];

/// A string member's JSON.
fn text(text: &str) -> Value {
    Value::from(text)
}

/// An integer member's JSON.
fn int(value: i32) -> Value {
    Value::from(i64::from(value))
}

/// A bool member's JSON.
fn flag(value: bool) -> Value {
    Value::from(value)
}

/// A list member's JSON.
fn list<T>(
    items: &[T],
    item: impl Fn(&T) -> Result<Value, EncodeError>,
) -> Result<Value, EncodeError> {
    Ok(write::array(
        items
            .iter()
            .enumerate()
            .map(|(index, value)| write::in_element(index, item(value)))
            .collect::<Result<Vec<_>, _>>()?,
    ))
}

/// One sound definition: the sounds an event plays and how.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct SoundDefinition {
    /// `category`, as written (vanilla does not check it); `""` when absent or no string.
    pub category: String,
    /// `min_distance`; 0.0 (the engine's default) when absent or `null`.
    pub min_distance: f32,
    /// `max_distance`; 0.0 (the engine's default) when absent or `null`.
    pub max_distance: f32,
    /// `__use_legacy_max_distance`, which the 1.14.0 step sets on every definition of an
    /// older file; `false` when absent.
    pub use_legacy_max_distance: bool,
    /// `subtitle`; `""` when absent or no string.
    pub subtitle: String,
    /// `sounds`: the entries one of which is played. Empty when the definition names an
    /// external event.
    pub sounds: Vec<SoundEntry>,
    /// `external_event_name`, when it is a string; `sounds` is then not read.
    pub external_event_name: Option<String>,
    /// `external_event_parameters`, read only with an external event.
    pub external_event_parameters: Vec<ExternalEventParameter>,
    /// `actions`.
    pub actions: Vec<SoundAction>,
    /// The members the client does not read, and those it reads that the definition did not:
    /// `sounds` and `external_event_parameters` beside an external event, an
    /// `external_event_name` that is no string; and a list written as an object of entries,
    /// as written (the typed list holds its reading).
    pub extra: UnknownMembers,
    /// The members the definition was read with.
    pub(crate) read: Present,
}

impl SoundDefinition {
    /// The definition as its JSON object.
    #[allow(
        clippy::float_cmp,
        reason = "exactly the value an absent member reads as"
    )]
    fn to_value(&self) -> Result<Value, EncodeError> {
        let mut out = Object::new(&self.read, &self.extra);
        out.member("category", self.category.is_empty(), || {
            Ok(text(&self.category))
        })?;
        out.member("min_distance", self.min_distance == 0.0, || {
            write::float(self.min_distance)
        })?;
        out.member("max_distance", self.max_distance == 0.0, || {
            write::float(self.max_distance)
        })?;
        out.member(
            "__use_legacy_max_distance",
            !self.use_legacy_max_distance,
            || Ok(flag(self.use_legacy_max_distance)),
        )?;
        out.member("subtitle", self.subtitle.is_empty(), || {
            Ok(text(&self.subtitle))
        })?;
        out.member("sounds", self.sounds.is_empty(), || {
            list(&self.sounds, |entry| entry.to_value(self))
        })?;
        out.member(
            "external_event_name",
            self.external_event_name.is_none(),
            || {
                Ok(self
                    .external_event_name
                    .as_deref()
                    .map_or(Value::NULL, Value::from))
            },
        )?;
        out.member(
            "external_event_parameters",
            self.external_event_parameters.is_empty(),
            || {
                list(
                    &self.external_event_parameters,
                    ExternalEventParameter::to_value,
                )
            },
        )?;
        out.member("actions", self.actions.is_empty(), || {
            list(&self.actions, SoundAction::to_value)
        })?;
        Ok(out.finish())
    }

    /// Whether an entry is positional when it does not say: every category but `music` and
    /// `ui`, compared without case.
    pub fn is_3d_by_default(&self) -> bool {
        !(self.category.eq_ignore_ascii_case("music") || self.category.eq_ignore_ascii_case("ui"))
    }
}

/// How an entry of `sounds` is written.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SoundEntryForm {
    /// A bare path string.
    Path,
    /// An object with `name` and its options.
    Object,
}

/// One entry of a definition's `sounds`, with vanilla's defaults filled in.
#[derive(Clone, Debug, PartialEq)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "four independent bool members of vanilla's entry"
)]
#[non_exhaustive]
pub struct SoundEntry {
    /// How the entry is written. The two forms differ in one default:
    /// [`interruptible`](Self::interruptible).
    pub form: SoundEntryForm,
    /// `name`: the path of the audio file without its extension, which the client appends
    /// (`.fsb`, `.ogg`, `.wav`) when the sound is first played.
    pub name: String,
    /// `volume`; 1.0 by default.
    pub volume: f32,
    /// `pitch`; 1.0 by default. A file below format 1.14.0 loses it.
    pub pitch: f32,
    /// `weight`; 1 by default.
    pub weight: i32,
    /// `is3D`; by default whether the definition's category is neither `music` nor `ui`.
    pub is_3d: bool,
    /// `stream`; `false` by default.
    pub stream: bool,
    /// `interruptible`; `true` by default for an object entry and `false` for a path string.
    pub interruptible: bool,
    /// `allow_concurrent_streaming`: whether a streamed entry may play while another stream
    /// plays; `true` by default. The 1.26.50 step writes `false` into every streamed object
    /// entry of an older file that does not say. A path string does not say and reads `true`.
    pub allow_concurrent_streaming: bool,
    /// The members the client does not read.
    pub extra: UnknownMembers,
    /// The members an object entry was read with.
    pub(crate) read: Present,
}

impl SoundEntry {
    /// The entry as its JSON: a path string while it is a path string with every member its
    /// default, else an object.
    #[allow(
        clippy::float_cmp,
        reason = "exactly the value an absent member reads as"
    )]
    fn to_value(&self, definition: &SoundDefinition) -> Result<Value, EncodeError> {
        let is_3d = definition.is_3d_by_default();
        let a_path = self.form == SoundEntryForm::Path
            && self.extra.is_empty()
            && self.volume == 1.0
            && self.pitch == 1.0
            && self.weight == 1
            && self.is_3d == is_3d
            && !self.stream
            && !self.interruptible
            && self.allow_concurrent_streaming;
        if a_path {
            return Ok(text(&self.name));
        }
        let mut out = Object::new(&self.read, &self.extra);
        out.member("name", self.name.is_empty(), || Ok(text(&self.name)))?;
        out.member("volume", self.volume == 1.0, || write::float(self.volume))?;
        out.member("pitch", self.pitch == 1.0, || write::float(self.pitch))?;
        out.member("weight", self.weight == 1, || Ok(int(self.weight)))?;
        out.member("is3D", self.is_3d == is_3d, || Ok(flag(self.is_3d)))?;
        out.member("stream", !self.stream, || Ok(flag(self.stream)))?;
        out.member("interruptible", self.interruptible, || {
            Ok(flag(self.interruptible))
        })?;
        out.member(
            "allow_concurrent_streaming",
            self.allow_concurrent_streaming,
            || Ok(flag(self.allow_concurrent_streaming)),
        )?;
        Ok(out.finish())
    }
}

/// One entry of `external_event_parameters`.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct ExternalEventParameter {
    /// `parameter`.
    pub parameter: String,
    /// `value`; 0.0 unless it is written with a fraction (an integer such as `1` is 0.0).
    pub value: f32,
    /// The members the client does not read.
    pub extra: UnknownMembers,
    /// The members the parameter was read with.
    pub(crate) read: Present,
}

impl ExternalEventParameter {
    /// The parameter as its JSON object.
    fn to_value(&self) -> Result<Value, EncodeError> {
        let mut out = Object::new(&self.read, &self.extra);
        out.member("parameter", self.parameter.is_empty(), || {
            Ok(text(&self.parameter))
        })?;
        out.member("value", self.value == 0.0, || write::float(self.value))?;
        Ok(out.finish())
    }
}

/// One entry of `actions` (used by vanilla's own packs only).
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct SoundAction {
    /// `action_type`; −1 unless it is an integer.
    pub action_type: i32,
    /// `name`.
    pub name: String,
    /// `target`.
    pub target: String,
    /// `enabled`; `false` by default.
    pub enabled: bool,
    /// `param`; 0.0 unless it is written with a fraction.
    pub param: f32,
    /// `delay`; 0.0 unless it is written with a fraction.
    pub delay: f32,
    /// The members the client does not read.
    pub extra: UnknownMembers,
    /// The members the action was read with.
    pub(crate) read: Present,
}

impl SoundAction {
    /// The action as its JSON object.
    fn to_value(&self) -> Result<Value, EncodeError> {
        let mut out = Object::new(&self.read, &self.extra);
        out.member("action_type", self.action_type == -1, || {
            Ok(int(self.action_type))
        })?;
        out.member("name", self.name.is_empty(), || Ok(text(&self.name)))?;
        out.member("target", self.target.is_empty(), || Ok(text(&self.target)))?;
        out.member("enabled", !self.enabled, || Ok(flag(self.enabled)))?;
        out.member("param", self.param == 0.0, || write::float(self.param))?;
        out.member("delay", self.delay == 0.0, || write::float(self.delay))?;
        Ok(out.finish())
    }
}

/// What [`read_root`] read of a root.
#[derive(Debug, Default)]
pub(crate) struct RootRead {
    /// The definitions.
    pub(crate) definitions: IndexMap<SoundEventName, SoundDefinition>,
    /// The members of `sound_definitions` that are no object.
    pub(crate) not_definitions: UnknownMembers,
    /// The root's members the client does not read.
    pub(crate) extra: UnknownMembers,
}

/// Reads the root of an upgraded document: the definitions under `sound_definitions`, each a
/// `SoundDefinition` or skipped with its finding. A `null` root reads as nothing.
pub(crate) fn read_root(root: &Value, diagnostics: &mut Diagnostics) -> RootRead {
    let mut out = RootRead::default();
    let mut report = Reporter::new(diagnostics);
    if root.is_null() {
        return out;
    }
    if !root.is_object() {
        // The game writes an error line to the content log; nothing of the file is registered
        // and the parser returns normally: every entry dropped, the file kept, a kept error. The
        // value holds no definition.
        report.kept(DiagnosticCode::InvalidValue {
            text: Some(VanillaText::Fixed(CONTENTS_MALFORMED)),
        });
        return out;
    }
    out.extra = check_unknown(root, &ROOT_KEYS, &[], &mut report);
    report.enter("sound_definitions");
    let definitions = fork::member(root, "sound_definitions");
    if !definitions.is_object() {
        // The game writes an error line to the content log; no definition is registered and the
        // parser returns normally: a kept error. The value holds the member as written.
        report.kept(DiagnosticCode::InvalidValue {
            text: Some(VanillaText::Fixed(SECTION_MALFORMED)),
        });
        return out;
    }
    let Some(object) = definitions.as_object() else {
        return out;
    };
    for (name, value) in object {
        let name = name.as_str();
        report.enter(name);
        if value.is_object() {
            out.definitions.insert(
                SoundEventName::parse(name),
                read_definition(name, value, &mut report),
            );
        } else {
            out.not_definitions.insert(name.to_owned(), value.clone());
            // The game registers an empty event.
            report.vanilla(
                DiagnosticCode::InvalidValue {
                    text: Some(VanillaText::Formatted {
                        template: super::upgrade::MALFORMED_SOUND,
                        args: vec![name.to_owned()],
                    }),
                },
                Severity::Info,
            );
        }
        report.leave();
    }
    out
}

/// Reads one definition object.
fn read_definition(name: &str, value: &Value, report: &mut Reporter<'_>) -> SoundDefinition {
    let mut definition = SoundDefinition {
        category: fork::text(fork::member(value, "category"))
            .unwrap_or_default()
            .to_owned(),
        min_distance: float(value, "min_distance", 0.0, report),
        max_distance: float(value, "max_distance", 0.0, report),
        use_legacy_max_distance: legacy_flag(value, report),
        subtitle: fork::text(fork::member(value, "subtitle"))
            .unwrap_or_default()
            .to_owned(),
        extra: check_unknown(value, &DEFINITION_KEYS, &["volume", "pitch"], report),
        read: present(value, &DEFINITION_KEYS),
        ..SoundDefinition::default()
    };
    check_category(name, value, &definition, report);
    let external = fork::member(value, "external_event_name");
    let sounds = fork::member(value, "sounds");
    if !external.is_null() && !sounds.is_null() {
        // The game never reads `external_event_name` when `sounds` is present and says nothing,
        // so the crate reports the combination as an information.
        report.here(DiagnosticCode::InvalidValue { text: None }, Severity::Info);
        let _ = name;
    }
    // The members read on one branch alone are kept as written on the other.
    let unread = |key: &str, definition: &mut SoundDefinition| {
        let member = fork::member(value, key);
        if !member.is_null() {
            definition.extra.insert(key.to_owned(), member.clone());
        }
    };
    // A list written as an object is read in sorted key order and kept as written: the merge
    // recurses into it.
    let as_object = |key: &str, definition: &mut SoundDefinition| {
        let member = fork::member(value, key);
        if member.is_object() {
            definition.extra.insert(key.to_owned(), member.clone());
        }
    };
    if let Some(event) = fork::text(external) {
        unread("sounds", &mut definition);
        as_object("external_event_parameters", &mut definition);
        definition.external_event_name = Some(event.to_owned());
        report.enter("external_event_parameters");
        let parameters = fork::member(value, "external_event_parameters");
        for (index, parameter) in fork::elements(parameters).into_iter().enumerate() {
            report.enter_index(index);
            definition
                .external_event_parameters
                .push(ExternalEventParameter {
                    parameter: string(parameter, "parameter", report),
                    value: real(parameter, "value"),
                    extra: check_unknown(parameter, &PARAMETER_KEYS, &[], report),
                    read: present(parameter, &PARAMETER_KEYS),
                });
            report.leave();
        }
        report.leave();
    } else {
        unread("external_event_name", &mut definition);
        unread("external_event_parameters", &mut definition);
        as_object("sounds", &mut definition);
        report.enter("sounds");
        // An entry is named as the file has it: by its key where `sounds` is an
        // object (walked in sorted key order), by its index in a list.
        let keys = sounds.is_object().then(|| fork::member_names(sounds));
        for (index, entry) in fork::elements(sounds).into_iter().enumerate() {
            match keys.as_ref().and_then(|keys| keys.get(index)) {
                Some(key) => report.enter(key),
                None => report.enter_index(index),
            }
            if let Some(entry) = read_entry(entry, &definition, report) {
                definition.sounds.push(entry);
            }
            report.leave();
        }
        report.leave();
    }
    definition.actions = read_actions(value, report);
    as_object("actions", &mut definition);
    definition
}

/// The `actions` of a definition; an element that is no object is skipped.
fn read_actions(value: &Value, report: &mut Reporter<'_>) -> Vec<SoundAction> {
    let mut actions = Vec::new();
    report.enter("actions");
    for (index, action) in fork::elements(fork::member(value, "actions"))
        .into_iter()
        .enumerate()
    {
        if !action.is_object() {
            continue;
        }
        report.enter_index(index);
        let action_type = fork::member(action, "action_type");
        actions.push(SoundAction {
            action_type: if fork::stored(action_type) == Stored::Integer {
                fork::as_int(action_type, 0)
            } else {
                -1
            },
            name: string(action, "name", report),
            target: string(action, "target", report),
            enabled: boolean(action, "enabled", false, report),
            param: real(action, "param"),
            delay: real(action, "delay"),
            extra: check_unknown(action, &ACTION_KEYS, &[], report),
            read: present(action, &ACTION_KEYS),
        });
        report.leave();
    }
    report.leave();
    actions
}

/// the `music.*` name check and the category list.
fn check_category(
    name: &str,
    value: &Value,
    definition: &SoundDefinition,
    report: &mut Reporter<'_>,
) {
    // Only a name with a `.` is split, and its first segment is compared with `music`
    // case-sensitively; the category is compared without case.
    let first = name.split_once('.').map(|(first, _)| first);
    if first == Some("music") && !definition.category.eq_ignore_ascii_case("music") {
        report.vanilla(
            DiagnosticCode::InvalidValue {
                text: Some(VanillaText::Formatted {
                    template: NOT_MUSIC,
                    args: vec![name.to_owned()],
                }),
            },
            // The game logs this at the content log's warning level.
            Severity::Warning,
        );
    }
    // The game stores the category as a free string and checks only the `music` prefix above;
    // a name the volume map lacks is ignored without a word, so the crate reports it as an
    // information.
    if fork::member(value, "category").is_string()
        && !CATEGORIES.contains(&definition.category.as_str())
    {
        report.at(
            "category",
            DiagnosticCode::InvalidValue { text: None },
            Severity::Info,
        );
    }
}

/// One element of `sounds`: a path string or an object; `null` is skipped silently, another
/// type with a warning.
fn read_entry(
    entry: &Value,
    definition: &SoundDefinition,
    report: &mut Reporter<'_>,
) -> Option<SoundEntry> {
    let is_3d = definition.is_3d_by_default();
    let read = match fork::stored(entry) {
        Stored::Null => return None,
        Stored::String => SoundEntry {
            form: SoundEntryForm::Path,
            name: fork::to_text(entry),
            volume: 1.0,
            pitch: 1.0,
            weight: 1,
            is_3d,
            stream: false,
            interruptible: false,
            allow_concurrent_streaming: true,
            extra: UnknownMembers::default(),
            read: Present::default(),
        },
        Stored::Object => SoundEntry {
            form: SoundEntryForm::Object,
            name: string(entry, "name", report),
            volume: float(entry, "volume", 1.0, report),
            pitch: float(entry, "pitch", 1.0, report),
            weight: integer(entry, "weight", 1, report),
            is_3d: boolean(entry, "is3D", is_3d, report),
            stream: boolean(entry, "stream", false, report),
            interruptible: boolean(entry, "interruptible", true, report),
            allow_concurrent_streaming: boolean(entry, "allow_concurrent_streaming", true, report),
            extra: check_unknown(
                entry,
                &ENTRY_KEYS,
                &["load_on_low_memory", "pitch:"],
                report,
            ),
            read: present(entry, &ENTRY_KEYS),
        },
        _ => {
            report.here(DiagnosticCode::InvalidValue { text: None }, Severity::Info);
            return None;
        }
    };
    if read.name.len() > MAX_NAME_BYTES {
        // The game asserts and keeps an empty path.
        report.here(DiagnosticCode::InvalidValue { text: None }, Severity::Info);
        return None;
    }
    Some(read)
}

/// `__use_legacy_max_distance`: a bool member whose vanilla spelling, the string `"true"` of
/// the 1.14.0 step, is no finding.
fn legacy_flag(value: &Value, report: &mut Reporter<'_>) -> bool {
    let member = fork::member(value, "__use_legacy_max_distance");
    if fork::text(member) == Some("true") {
        return true;
    }
    boolean(value, "__use_legacy_max_distance", false, report)
}
