//! `sounds/sound_definitions.json` at its newest shape: the typed read of the upgraded JSON
//! and its rules.
#![allow(
    clippy::unnecessary_wraps,
    reason = "a `schema::read_with` function reads to an `Option`, which these never fail"
)]

use super::codes;
use facet::Facet;
use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::id::SoundEventName;
use bedrock_addon_core::json::jsoncpp::fork::{self, Stored};
use bedrock_addon_core::json::kit::write::{self, Object};
use bedrock_addon_core::json::kit::{Node, Present, Read, Written};
use bedrock_addon_core::json::{EncodeError, Value};
use bedrock_addon_core::load::sound::read_merged;
use bedrock_addon_core::schema::{self, FromJson, Schema};
use bedrock_addon_core::version::SemVersion;

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
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct SoundDefinitions {
    /// `format_version`, when it reads as a version (a string or three integers). Vanilla
    /// stamps `"1.26.50"` on every older copy. Written as a version string, `"1.26.50"` when
    /// `None`.
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
        // Always written: every read of an object stamps one (the version steps), so a value
        // without one (a stack without the file, a `null` root, a value built by hand) writes
        // the version the steps stamp, and its written text reads as what it was written from.
        root.member("format_version", false, || {
            let version = self
                .format_version
                .clone()
                .unwrap_or_else(|| SemVersion::from(super::upgrade::V1_26_50));
            Ok(Value::from(version.to_string().as_str()))
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
    #[allow(
        clippy::needless_pass_by_value,
        reason = "the signature of the other two documents' `from_json`"
    )]
    pub fn from_json(json: Value) -> Self {
        Self::read(&json, &mut Diagnostics::default())
    }

    /// Reads an upgraded document.
    pub(crate) fn read(json: &Value, diagnostics: &mut Diagnostics) -> Self {
        read_merged(json, diagnostics)
    }
}

/// The members of the root the client reads.
const ROOT_KEYS: [&str; 2] = ["format_version", "sound_definitions"];

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
///
/// The scalar members read as the game reads them; the three lists, which depend on the
/// definition (an external event replaces `sounds`, a category sets the default of an entry's
/// `is3D`), are read by [`after_read`] once the members are.
#[derive(Clone, Debug, Default, PartialEq, Facet, Schema)]
#[facet(schema::quiet = "volume pitch", schema::after_read = after_read)]
#[non_exhaustive]
pub struct SoundDefinition {
    /// `category`, as written (vanilla does not check it); `""` when absent or no string.
    #[facet(default, schema::read_with = text_or_empty)]
    pub category: String,
    /// `min_distance`; 0.0 (the engine's default) when absent or `null`.
    #[facet(default)]
    pub min_distance: f32,
    /// `max_distance`; 0.0 (the engine's default) when absent or `null`.
    #[facet(default)]
    pub max_distance: f32,
    /// `__use_legacy_max_distance`, which the 1.14.0 step sets on every definition of an
    /// older file; `false` when absent.
    #[facet(default, rename = "__use_legacy_max_distance", schema::read_with = legacy_flag)]
    pub use_legacy_max_distance: bool,
    /// `subtitle`; `""` when absent or no string.
    #[facet(default, schema::read_with = text_or_empty)]
    pub subtitle: String,
    /// `sounds`: the entries one of which is played. Empty when the definition names an
    /// external event.
    #[facet(default, schema::read_with = schema::read_later)]
    pub sounds: Vec<SoundEntry>,
    /// `external_event_name`, when it is a string; `sounds` is then not read.
    #[facet(schema::or_kept = "silent", schema::read_with = text_only)]
    pub external_event_name: Option<String>,
    /// `external_event_parameters`, read only with an external event.
    #[facet(default, schema::read_with = schema::read_later)]
    pub external_event_parameters: Vec<ExternalEventParameter>,
    /// `actions`.
    #[facet(default, schema::read_with = schema::read_later)]
    pub actions: Vec<SoundAction>,
    /// The members the client does not read, and those it reads that the definition did not:
    /// `sounds` and `external_event_parameters` beside an external event, an
    /// `external_event_name` that is no string; and a list written as an object of entries,
    /// as written (the typed list holds its reading).
    pub extra: UnknownMembers,
    /// The members the definition was read with.
    #[facet(skip, rename = "\u{0}read")]
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet)]
#[non_exhaustive]
#[repr(u8)]
pub enum SoundEntryForm {
    /// A bare path string.
    Path,
    /// An object with `name` and its options.
    #[default]
    Object,
}

/// One entry of a definition's `sounds`, with vanilla's defaults filled in.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::quiet = "load_on_low_memory pitch:")]
#[allow(
    clippy::struct_excessive_bools,
    reason = "four independent bool members of vanilla's entry"
)]
#[non_exhaustive]
pub struct SoundEntry {
    /// How the entry is written. The two forms differ in one default:
    /// [`interruptible`](Self::interruptible).
    #[facet(skip, rename = "\u{0}form")]
    pub form: SoundEntryForm,
    /// `name`: the path of the audio file without its extension, which the client appends
    /// (`.fsb`, `.ogg`, `.wav`) when the sound is first played.
    #[facet(default)]
    pub name: String,
    /// `volume`; 1.0 by default.
    #[facet(default = 1.0)]
    pub volume: f32,
    /// `pitch`; 1.0 by default. A file below format 1.14.0 loses it.
    #[facet(default = 1.0)]
    pub pitch: f32,
    /// `weight`; 1 by default.
    #[facet(default = 1)]
    pub weight: i32,
    /// `is3D`; by default whether the definition's category is neither `music` nor `ui`.
    #[facet(default = true, rename = "is3D")]
    pub is_3d: bool,
    /// `stream`; `false` by default.
    #[facet(default)]
    pub stream: bool,
    /// `interruptible`; `true` by default for an object entry and `false` for a path string.
    #[facet(default = true)]
    pub interruptible: bool,
    /// `allow_concurrent_streaming`: whether a streamed entry may play while another stream
    /// plays; `true` by default. The 1.26.50 step writes `false` into every streamed object
    /// entry of an older file that does not say. A path string does not say and reads `true`.
    #[facet(default = true)]
    pub allow_concurrent_streaming: bool,
    /// The members the client does not read.
    pub extra: UnknownMembers,
    /// The members an object entry was read with.
    #[facet(skip, rename = "\u{0}read")]
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
#[derive(Clone, Debug, Default, PartialEq, Facet, Schema)]
#[non_exhaustive]
pub struct ExternalEventParameter {
    /// `parameter`.
    #[facet(default)]
    pub parameter: String,
    /// `value`; 0.0 unless it is written with a fraction (an integer such as `1` is 0.0).
    #[facet(default, schema::read_with = real)]
    pub value: f32,
    /// The members the client does not read.
    pub extra: UnknownMembers,
    /// The members the parameter was read with.
    #[facet(skip, rename = "\u{0}read")]
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
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[non_exhaustive]
pub struct SoundAction {
    /// `action_type`; −1 unless it is an integer.
    #[facet(default = -1, schema::read_with = action_type)]
    pub action_type: i32,
    /// `name`.
    #[facet(default)]
    pub name: String,
    /// `target`.
    #[facet(default)]
    pub target: String,
    /// `enabled`; `false` by default.
    #[facet(default)]
    pub enabled: bool,
    /// `param`; 0.0 unless it is written with a fraction.
    #[facet(default, schema::read_with = real)]
    pub param: f32,
    /// `delay`; 0.0 unless it is written with a fraction.
    #[facet(default, schema::read_with = real)]
    pub delay: f32,
    /// The members the client does not read.
    pub extra: UnknownMembers,
    /// The members the action was read with.
    #[facet(skip, rename = "\u{0}read")]
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

/// The merged document: the definitions under `sound_definitions`, each a [`SoundDefinition`]
/// or kept as written with its finding, and the members of the root the client does not read. A
/// `null` root reads as nothing; one that is no object is a kept error, with nothing read.
impl FromJson for SoundDefinitions {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let root = node.value();
        let mut out = Self {
            format_version: super::upgrade::version_member(root),
            foreign_root: (!root.is_object() && !root.is_null()).then(|| root.clone()),
            json: Written(Some(root.clone())),
            ..Self::default()
        };
        if root.is_null() {
            return Some(out);
        }
        if !root.is_object() {
            // The game writes an error line to the content log; nothing of the file is registered
            // and the parser returns normally: every entry dropped, the file kept, a kept error.
            read.push(Diagnostic::of(&codes::CONTENTS_MALFORMED));
            return Some(out);
        }
        let members = read.object(node)?;
        out.read = Written(Some(
            ROOT_KEYS
                .into_iter()
                .filter(|key| members.contains(key))
                .collect(),
        ));
        for (key, member) in members.iter().filter(|(key, _)| !ROOT_KEYS.contains(key)) {
            read.push(Diagnostic::info(&core_codes::UNRECOGNIZED_FIELD).at(member.path().clone()));
            out.extra.insert(key.to_owned(), member.value().clone());
        }
        let section = Node::new(
            fork::member(root, "sound_definitions"),
            node.path().child("sound_definitions"),
        );
        if section.value().is_object() {
            for (name, member) in read.object(&section)?.iter() {
                if member.value().is_object() {
                    let definition = SoundDefinition::from_json(&member, read)?;
                    out.sound_definitions
                        .insert(SoundEventName::parse(name), definition);
                } else {
                    // The game registers an empty event.
                    let code = codes::MALFORMED_SOUND.with([name.to_owned()]);
                    read.push(Diagnostic::info(code).at(member.path().clone()));
                    out.not_definitions
                        .insert(name.to_owned(), member.value().clone());
                }
            }
        } else {
            // The game writes an error line to the content log; no definition is registered and
            // the parser returns normally: a kept error. The value holds the member as written.
            read.push(Diagnostic::of(&codes::SECTION_MALFORMED).at(section.path().clone()));
        }
        // The members the client reads that the typed ones cannot hold, kept as written.
        let unread = [
            ("format_version", out.format_version.is_none()),
            ("sound_definitions", !section.value().is_object()),
        ];
        for (key, unread) in unread {
            if unread && members.contains(key) {
                out.extra
                    .insert(key.to_owned(), fork::member(root, key).clone());
            }
        }
        Some(out)
    }
}

/// A text member as the game reads it without a word: the string, else `""`.
fn text_or_empty(node: &Node<'_>, _: &mut Read<'_>) -> Option<String> {
    Some(fork::text(node.value()).unwrap_or_default().to_owned())
}

/// A text member that is none when it is no string, and then kept as written
/// (`schema::or_kept`).
fn text_only(node: &Node<'_>, _: &mut Read<'_>) -> Option<String> {
    fork::text(node.value()).map(str::to_owned)
}

/// A float the game reads only when it is written with a fraction (`isDouble()`): else 0.0.
fn real(node: &Node<'_>, _: &mut Read<'_>) -> Option<f32> {
    Some(match fork::stored(node.value()) {
        Stored::Real => fork::as_float(node.value(), 0.0),
        _ => 0.0,
    })
}

/// An integer the game reads only when it is written as one (`isInt()`): else −1.
fn action_type(node: &Node<'_>, _: &mut Read<'_>) -> Option<i32> {
    Some(match fork::stored(node.value()) {
        Stored::Integer => fork::as_int(node.value(), 0),
        _ => -1,
    })
}

/// `__use_legacy_max_distance`: a bool member whose vanilla spelling, the string `"true"` of
/// the 1.14.0 step, is no finding.
fn legacy_flag(node: &Node<'_>, read: &mut Read<'_>) -> Option<bool> {
    if fork::text(node.value()) == Some("true") {
        return Some(true);
    }
    bool::from_json(node, read)
}

/// The elements of the list `key` of `owner` as jsoncpp iterates it: an array's elements at their
/// indices, an object's values in key order (at their keys when `by_key`, else at their
/// positions), nothing for another value.
fn iterated<'v>(owner: &Node<'v>, key: &str, by_key: bool) -> Vec<Node<'v>> {
    let (list, at) = (fork::member(owner.value(), key), owner.path().child(key));
    match fork::stored(list) {
        Stored::Array => fork::elements(list)
            .into_iter()
            .enumerate()
            .map(|(index, element)| Node::new(element, at.index(index)))
            .collect(),
        Stored::Object => fork::member_names(list)
            .into_iter()
            .enumerate()
            .map(|(index, name)| {
                let at = if by_key {
                    at.child(&name)
                } else {
                    at.index(index)
                };
                Node::new(fork::member(list, &name), at)
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// Which members of the branch not read are kept as written.
type Kept = fn(&Value) -> bool;

/// What the members of a definition cannot say alone, in the game's order: the `music.*` name
/// check and the category list, then either the external event's parameters or the sounds, then
/// the actions; each list written as an object of entries is kept as written, as is a member of
/// the branch not read.
fn after_read(definition: &mut SoundDefinition, node: &Node<'_>, read: &mut Read<'_>) {
    check_category(definition, node, read);
    let value = node.value();
    let external = fork::member(value, "external_event_name");
    if !external.is_null() && !fork::member(value, "sounds").is_null() {
        // The game never reads `external_event_name` when `sounds` is present and says nothing,
        // so the crate reports the combination as an information.
        read.push(Diagnostic::info(&core_codes::INVALID_VALUE).at(node.path().clone()));
    }
    // A member read on one branch alone is kept as written on the other; a list written as an
    // object is read in sorted key order and kept as written, the merge recurses into it.
    let mut keep = |key: &str, kept: Kept| {
        let member = fork::member(value, key);
        if kept(member) {
            definition.extra.insert(key.to_owned(), member.clone());
        }
    };
    let (written, object): (Kept, Kept) = (|member| !member.is_null(), Value::is_object);
    if external.is_string() {
        keep("sounds", written);
        keep("external_event_parameters", object);
    } else {
        keep("external_event_parameters", written);
        keep("sounds", object);
    }
    keep("actions", object);
    if external.is_string() {
        for parameter in iterated(node, "external_event_parameters", false) {
            let read_one = if parameter.value().is_object() {
                ExternalEventParameter::from_json(&parameter, read)
            } else {
                Some(ExternalEventParameter::default())
            };
            definition.external_event_parameters.extend(read_one);
        }
    } else {
        let is_3d = definition.is_3d_by_default();
        for entry in iterated(node, "sounds", true) {
            definition.sounds.extend(read_entry(&entry, is_3d, read));
        }
    }
    for action in iterated(node, "actions", false) {
        if action.value().is_object() {
            definition
                .actions
                .extend(SoundAction::from_json(&action, read));
        }
    }
}

/// the `music.*` name check and the category list.
fn check_category(definition: &SoundDefinition, node: &Node<'_>, read: &mut Read<'_>) {
    // Only a name with a `.` is split, and its first segment is compared with `music`
    // case-sensitively; the category is compared without case.
    let name = node.path().last_key().unwrap_or_default();
    let first = name.split_once('.').map(|(first, _)| first);
    if first == Some("music") && !definition.category.eq_ignore_ascii_case("music") {
        // The game logs this at the content log's warning level.
        let code = codes::NOT_MUSIC.with([name.to_owned()]);
        read.push(Diagnostic::of(code).at(node.path().clone()));
    }
    // The game stores the category as a free string and checks only the `music` prefix above;
    // a name the volume map lacks is ignored without a word, so the crate reports it as an
    // information.
    if fork::member(node.value(), "category").is_string()
        && !CATEGORIES.contains(&definition.category.as_str())
    {
        let at = node.path().child("category");
        read.push(Diagnostic::info(&core_codes::INVALID_VALUE).at(at));
    }
}

/// One element of `sounds`: a path string or an object; `null` is skipped silently, another
/// type with an information. The object's `is3D` is the category's when it has none.
fn read_entry(node: &Node<'_>, is_3d: bool, read: &mut Read<'_>) -> Option<SoundEntry> {
    let entry = match fork::stored(node.value()) {
        Stored::Null => return None,
        Stored::String => SoundEntry {
            form: SoundEntryForm::Path,
            name: fork::to_text(node.value()),
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
        Stored::Object => {
            let mut entry = SoundEntry::from_json(node, read)?;
            if fork::member(node.value(), "is3D").is_null() {
                entry.is_3d = is_3d;
            }
            entry
        }
        _ => {
            read.push(Diagnostic::info(&core_codes::INVALID_VALUE).at(node.path().clone()));
            return None;
        }
    };
    if entry.name.len() > MAX_NAME_BYTES {
        // The game asserts and keeps an empty path.
        read.push(Diagnostic::info(&core_codes::INVALID_VALUE).at(node.path().clone()));
        return None;
    }
    Some(entry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use facet::{Type, UserType};

    /// The keys of a written object, in order.
    fn keys(value: &Value) -> Vec<&str> {
        value
            .as_object()
            .expect("an object")
            .iter()
            .map(|(key, _)| key.as_str())
            .collect()
    }

    /// The members the derived read takes of a type (its fields but the three that are no
    /// member), sorted.
    fn read_keys<'a, T: Facet<'a>>() -> Vec<&'static str> {
        let Type::User(UserType::Struct(shape)) = T::SHAPE.ty else {
            unreachable!("a struct");
        };
        let mut keys: Vec<&str> = shape
            .fields
            .iter()
            .filter(|field| !matches!(field.name, "extra" | "read" | "form"))
            .map(|field| field.rename.unwrap_or(field.name))
            .collect();
        keys.sort_unstable();
        keys
    }

    /// The members the derived read takes are the members of the writers: an object whose every
    /// member is set writes exactly the keys of its type's fields.
    #[test]
    fn every_key_the_reader_knows_is_a_member_the_writer_writes() {
        let entry = SoundEntry {
            form: SoundEntryForm::Object,
            name: "a".to_owned(),
            volume: 0.5,
            pitch: 2.0,
            weight: 3,
            is_3d: false,
            stream: true,
            interruptible: false,
            allow_concurrent_streaming: false,
            extra: UnknownMembers::default(),
            read: Present::default(),
        };
        let parameter = ExternalEventParameter {
            parameter: "p".to_owned(),
            value: 1.5,
            extra: UnknownMembers::default(),
            read: Present::default(),
        };
        let action = SoundAction {
            action_type: 1,
            name: "n".to_owned(),
            target: "t".to_owned(),
            enabled: true,
            param: 0.5,
            delay: 0.25,
            extra: UnknownMembers::default(),
            read: Present::default(),
        };
        let definition = SoundDefinition {
            category: "neutral".to_owned(),
            min_distance: 1.0,
            max_distance: 2.0,
            use_legacy_max_distance: true,
            subtitle: "s".to_owned(),
            sounds: vec![entry.clone()],
            external_event_name: Some("e".to_owned()),
            external_event_parameters: vec![parameter.clone()],
            actions: vec![action.clone()],
            ..SoundDefinition::default()
        };
        let mut definitions = IndexMap::new();
        definitions.insert(SoundEventName::parse("x"), definition.clone());
        let root = SoundDefinitions {
            format_version: Some(SemVersion::new(1, 26, 50)),
            sound_definitions: definitions,
            not_definitions: UnknownMembers::default(),
            extra: UnknownMembers::default(),
            foreign_root: None,
            read: Present::default(),
            json: Written::default(),
        };
        let written = |value: Result<Value, EncodeError>| {
            let value = value.expect("written");
            let mut keys: Vec<String> = keys(&value).into_iter().map(str::to_owned).collect();
            keys.sort_unstable();
            keys
        };
        assert_eq!(written(root.to_value()), ROOT_KEYS);
        assert_eq!(
            written(definition.to_value()),
            read_keys::<SoundDefinition>()
        );
        assert_eq!(
            written(entry.to_value(&definition)),
            read_keys::<SoundEntry>()
        );
        assert_eq!(
            written(parameter.to_value()),
            read_keys::<ExternalEventParameter>()
        );
        assert_eq!(written(action.to_value()), read_keys::<SoundAction>());
    }

    #[test]
    fn a_value_without_a_format_version_writes_the_version_every_read_stamps() {
        use bedrock_addon_core::document::Content;
        use bedrock_addon_core::load::LoadContext;
        use bedrock_addon_core::pack_identity::PackPath;
        // A stack without the file merges no copy, and the value of a `null` root holds
        // nothing: the read of what such a value writes is upgraded, which stamps the
        // version, so the written text has to hold it to read as what it was written from.
        let empty = SoundDefinitions::from_json(Value::NULL);
        let written =
            bedrock_addon_core::json::to_json(&empty.to_value().expect("written")).expect("text");
        let json = |text: &str| bedrock_addon_core::json::from_str::<Value>(text).expect("JSON");
        assert_eq!(json(&written), json(r#"{"format_version": "1.26.50"}"#));
        let path = PackPath::new("sounds/sound_definitions.json").expect("a pack path");
        let again = SoundDefinitions::parse(&path, written.as_bytes(), &LoadContext::detached())
            .expect("loads");
        assert_eq!(
            again.to_value().expect("written again"),
            empty.to_value().expect("written"),
            "the written text reads as what it was written from"
        );
    }
}
