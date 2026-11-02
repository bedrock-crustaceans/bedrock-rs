//! `sounds/music_definitions.json`: which sound event each music situation plays, and the
//! pause between tracks.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod load;

use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::{DiagnosticCode, Diagnostics, Severity, VanillaText};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::id::MusicDefinitionId;
use bedrock_addon_core::json::Value;
use bedrock_addon_support::sound::fork;
use bedrock_addon_support::sound::read::{boolean, float, string};
use bedrock_addon_support::sound::report::{Reporter, check_unknown};

/// Vanilla's text for a definition under the empty name.
const NO_NAME: &str = "Cannot define a music definition with no name.";

/// Game ticks per second, for the delays vanilla stores as ticks.
const TICKS_PER_SECOND: f32 = 20.0;

/// The music definitions of the resource packs, `sounds/music_definitions.json`
/// (`#[doc(alias)]` `music_definitions.json`): one definition per music name (`game`, `menu`,
/// a biome's music key).
///
/// [`json`](Self::json) is the document as read, which is what the value writes back.
#[doc(alias = "music_definitions.json")]
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct MusicDefinitions {
    /// The definitions by music name, in file order.
    pub definitions: IndexMap<MusicDefinitionId, MusicDefinition>,
    /// The document.
    pub(crate) json: Value,
}

impl MusicDefinitions {
    /// The document as JSON: what the typed members were read from and what the value writes.
    #[must_use]
    pub fn json(&self) -> &Value {
        &self.json
    }

    /// The definition of the music name `name`.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&MusicDefinition> {
        self.definitions.get(&MusicDefinitionId::parse(name))
    }

    /// The value of a document, read as it is; the findings of the read are dropped.
    /// `Content::parse` reads a file with them.
    #[must_use]
    pub fn from_json(json: Value) -> Self {
        Self::read(json, &mut Diagnostics::default())
    }

    /// Reads a document.
    pub(crate) fn read(json: Value, diagnostics: &mut Diagnostics) -> Self {
        Self {
            definitions: read_root(&json, diagnostics),
            json,
        }
    }
}

/// One music definition.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct MusicDefinition {
    /// `event_name`: the sound event that holds the tracks; `""` when absent.
    pub event_name: String,
    /// `min_delay`, in seconds; 0.0 when absent.
    pub min_delay: f32,
    /// `max_delay`, in seconds; 0.0 when absent.
    pub max_delay: f32,
    /// `play_in_order`: play the event's sounds as a playlist; `false` when absent.
    pub play_in_order: bool,
    /// `override_interruptible`: the `name` of each entry, the sound categories this music
    /// may interrupt. An entry that is no object, or has no `name`, is `""`.
    pub override_interruptible: Vec<String>,
    /// `replace`, as written: a truthy value makes this definition replace the lower packs'
    /// instead of merging into it. Only the merge reads it.
    pub replace: Option<Value>,
    /// The members the client does not read.
    pub extra: Extra,
}

impl MusicDefinition {
    /// The shortest pause in game ticks, as vanilla stores it: `min_delay` × 20, truncated.
    #[allow(clippy::cast_possible_truncation, reason = "vanilla truncates")]
    #[must_use]
    pub fn min_delay_ticks(&self) -> i32 {
        (self.min_delay * TICKS_PER_SECOND) as i32
    }

    /// The longest pause in game ticks, as vanilla stores it: `max_delay` × 20, truncated.
    #[allow(clippy::cast_possible_truncation, reason = "vanilla truncates")]
    #[must_use]
    pub fn max_delay_ticks(&self) -> i32 {
        (self.max_delay * TICKS_PER_SECOND) as i32
    }
}

/// The situation a music name stands for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
#[repr(u8)]
pub enum MusicCategory {
    /// `menu`.
    Menu = 0,
    /// `game`, `creative` and `water`.
    Overworld = 1,
    /// `nether`.
    Nether = 2,
    /// `end`.
    End = 3,
    /// `endboss`.
    Endboss = 4,
    /// `credits`.
    Credits = 5,
}

impl MusicCategory {
    /// The category vanilla gives a music definition of this name; `None` for every other
    /// name, which leaves the category as it was.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "menu" => Self::Menu,
            "game" | "creative" | "water" => Self::Overworld,
            "nether" => Self::Nether,
            "end" => Self::End,
            "endboss" => Self::Endboss,
            "credits" => Self::Credits,
            _ => return None,
        })
    }
}

/// The merge predicate of the music file: a copy's level with a truthy `replace` member
/// replaces the lower level whole.
///
/// The predicate tests that the member is there, then reads it as a boolean defaulting to
/// `false`.
pub(crate) fn replace_predicate(dst: &mut Value, src: &Value) -> bool {
    if fork::has_member(src, "replace") && fork::as_bool(fork::member(src, "replace"), false) {
        *dst = src.clone();
        return true;
    }
    false
}

/// Reads the merged document: one definition per member of the root object.
pub(crate) fn read_root(
    root: &Value,
    diagnostics: &mut Diagnostics,
) -> IndexMap<MusicDefinitionId, MusicDefinition> {
    let mut out = IndexMap::new();
    let mut report = Reporter::new(diagnostics);
    let Some(object) = root.as_object() else {
        return out;
    };
    for (name, value) in object {
        let name = name.as_str();
        report.enter(name);
        if name.is_empty() {
            // The game writes this line to the content log at the warning level.
            report.vanilla(
                DiagnosticCode::InvalidValue {
                    text: Some(VanillaText::Fixed(NO_NAME)),
                },
                Severity::Warning,
            );
        } else if value.is_object() {
            out.insert(
                MusicDefinitionId::parse(name),
                read_definition(value, &mut report),
            );
        } else {
            // The game registers an empty music definition.
            report.here(DiagnosticCode::InvalidValue { text: None }, Severity::Info);
        }
        report.leave();
    }
    out
}

/// One definition object.
fn read_definition(value: &Value, report: &mut Reporter<'_>) -> MusicDefinition {
    let mut override_interruptible = Vec::new();
    report.enter("override_interruptible");
    for (index, entry) in fork::elements(fork::member(value, "override_interruptible"))
        .into_iter()
        .enumerate()
    {
        report.enter_index(index);
        override_interruptible.push(string(entry, "name", report));
        report.leave();
    }
    report.leave();
    let replace = fork::has_member(value, "replace").then(|| {
        // Only the merge reads it, as a boolean:
        boolean(value, "replace", false, report);
        fork::member(value, "replace").clone()
    });
    let definition = MusicDefinition {
        event_name: string(value, "event_name", report),
        min_delay: float(value, "min_delay", 0.0, report),
        max_delay: float(value, "max_delay", 0.0, report),
        play_in_order: boolean(value, "play_in_order", false, report),
        override_interruptible,
        replace,
        extra: check_unknown(
            value,
            &[
                "event_name",
                "min_delay",
                "max_delay",
                "play_in_order",
                "override_interruptible",
                "replace",
            ],
            &[],
            report,
        ),
    };
    if definition.min_delay > definition.max_delay {
        // The game stores the values as given.
        report.at(
            "min_delay",
            DiagnosticCode::InvalidValue { text: None },
            Severity::Info,
        );
    }
    definition
}

#[doc(hidden)]
pub use load::load_music;
