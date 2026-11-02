//! `sounds/sound_definitions.json` at its newest shape: the typed read of the upgraded JSON
//! and its rules.

use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::{DiagnosticCode, Diagnostics, Severity, VanillaText};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::id::SoundEventName;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::version::SemVersion;
use bedrock_addon_support::sound::fork::{self, Stored};
use bedrock_addon_support::sound::read::{boolean, float, integer, real, string};
use bedrock_addon_support::sound::report::{Reporter, check_unknown};

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
/// 1.26.50 shape. [`json`](Self::json) is that JSON, which is what the value writes back:
/// vanilla merges the copies of this file as JSON, where `null`, `16` and `16.0` differ, so
/// the text is kept exactly and the typed members are the read of it.
#[doc(alias = "sound_definitions.json")]
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct SoundDefinitions {
    /// `format_version`, when it reads as a version (a string or three integers). Vanilla
    /// stamps `"1.26.50"` on every older copy.
    pub format_version: Option<SemVersion>,
    /// `sound_definitions`: the definitions by sound event name, in file order.
    pub sound_definitions: IndexMap<SoundEventName, SoundDefinition>,
    /// The upgraded document.
    pub(crate) json: Value,
}

impl SoundDefinitions {
    /// The document as JSON, after the version steps: what the typed members were read from
    /// and what the value writes.
    #[must_use]
    pub fn json(&self) -> &Value {
        &self.json
    }

    /// The definition of the sound event `name`.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&SoundDefinition> {
        self.sound_definitions.get(&SoundEventName::parse(name))
    }

    /// The value of a document that is already the 1.26.50 shape, read as it is: no version
    /// step runs and the findings of the read are dropped. `Content::parse` reads a file with
    /// both.
    #[must_use]
    pub fn from_json(json: Value) -> Self {
        Self::read(json, &mut Diagnostics::default())
    }

    /// Reads an upgraded document.
    pub(crate) fn read(json: Value, diagnostics: &mut Diagnostics) -> Self {
        Self {
            format_version: super::upgrade::version_member(&json),
            sound_definitions: read_root(&json, diagnostics),
            json,
        }
    }
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
    /// The members the client does not read.
    pub extra: Extra,
}

impl SoundDefinition {
    /// Whether an entry is positional when it does not say: every category but `music` and
    /// `ui`, compared without case.
    #[must_use]
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
    pub extra: Extra,
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
    pub extra: Extra,
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
    pub extra: Extra,
}

/// Reads the root of an upgraded document: the definitions under `sound_definitions`, each a
/// `SoundDefinition` or skipped with its finding. A `null` root reads as nothing.
pub(crate) fn read_root(
    root: &Value,
    diagnostics: &mut Diagnostics,
) -> IndexMap<SoundEventName, SoundDefinition> {
    let mut out = IndexMap::new();
    let mut report = Reporter::new(diagnostics);
    if root.is_null() {
        return out;
    }
    if !root.is_object() {
        // The game writes an error line to the content log; nothing of the file is registered
        // and the parser returns normally: every entry dropped, the file kept, a kept error. The
        // crate's value holds the root as written.
        report.kept(DiagnosticCode::InvalidValue {
            text: Some(VanillaText::Fixed(CONTENTS_MALFORMED)),
        });
        return out;
    }
    check_unknown(
        root,
        &["format_version", "sound_definitions"],
        &[],
        &mut report,
    );
    report.enter("sound_definitions");
    let definitions = fork::member(root, "sound_definitions");
    if !definitions.is_object() {
        // The game writes an error line to the content log; no definition is registered and the
        // parser returns normally: a kept error. The crate's value holds the member as written.
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
            out.insert(
                SoundEventName::parse(name),
                read_definition(name, value, &mut report),
            );
        } else {
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
        extra: check_unknown(
            value,
            &[
                "category",
                "min_distance",
                "max_distance",
                "__use_legacy_max_distance",
                "subtitle",
                "sounds",
                "external_event_name",
                "external_event_parameters",
                "actions",
            ],
            &["volume", "pitch"],
            report,
        ),
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
    if let Some(event) = fork::text(external) {
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
                    extra: check_unknown(parameter, &["parameter", "value"], &[], report),
                });
            report.leave();
        }
        report.leave();
    } else {
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
            extra: check_unknown(
                action,
                &["action_type", "name", "target", "enabled", "param", "delay"],
                &[],
                report,
            ),
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
            extra: Extra::default(),
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
                &[
                    "name",
                    "volume",
                    "pitch",
                    "weight",
                    "is3D",
                    "stream",
                    "interruptible",
                    "allow_concurrent_streaming",
                ],
                &["load_on_low_memory", "pitch:"],
                report,
            ),
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
