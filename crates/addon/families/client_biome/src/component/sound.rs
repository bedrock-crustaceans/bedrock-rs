//! Sound components: `minecraft:ambient_sounds` and `minecraft:biome_music`.

use facet::Facet;

use super::{Checked, child};
use bedrock_addon_core::diagnostic::DiagnosticCode;
use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, JsonPath, Severity};
use bedrock_addon_core::id::{MusicDefinitionId, SoundEventName};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::schema::{self, Schema};

/// The chance of an addition that names none: the game stores 1.0.
const DEFAULT_CHANCE: f32 = 1.0;

/// `minecraft:ambient_sounds`: the sounds that play in the biome, above and under water. At
/// least one sound is needed.
///
/// A member the document leaves out is `None`; the accessor of its name gives the game's
/// default (no sound). Every member is optional.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct AmbientSounds {
    /// JSON `addition`: a sound played now and then.
    pub addition: Option<SoundAddition>,
    /// JSON `loop`: the sound that plays all the time.
    #[facet(rename = "loop")]
    pub loop_sound: Option<SoundEventName>,
    /// JSON `mood`: the sound played in dark, enclosed places.
    pub mood: Option<SoundEventName>,
    /// JSON `underwater_addition` (format 1.21.120).
    pub underwater_addition: Option<SoundAddition>,
    /// JSON `underwater_loop` (format 1.21.120).
    pub underwater_loop: Option<SoundEventName>,
    /// JSON `underwater_mood` (format 1.21.120).
    pub underwater_mood: Option<SoundEventName>,
}

impl AmbientSounds {
    /// The sound played now and then and its chance; no sound at chance 1 when absent.
    pub fn addition(&self) -> SoundAddition {
        self.addition.clone().unwrap_or_else(SoundAddition::none)
    }

    /// The sound that plays all the time; `""` (none) when absent.
    pub fn loop_sound(&self) -> &str {
        self.loop_sound.as_ref().map_or("", SoundEventName::as_str)
    }

    /// The sound of dark places; `""` (none) when absent.
    pub fn mood(&self) -> &str {
        self.mood.as_ref().map_or("", SoundEventName::as_str)
    }

    /// The underwater sound played now and then; no sound at chance 1 when absent.
    pub fn underwater_addition(&self) -> SoundAddition {
        self.underwater_addition
            .clone()
            .unwrap_or_else(SoundAddition::none)
    }

    /// The underwater sound that plays all the time; `""` (none) when absent.
    pub fn underwater_loop(&self) -> &str {
        self.underwater_loop
            .as_ref()
            .map_or("", SoundEventName::as_str)
    }

    /// The underwater sound of dark places; `""` (none) when absent.
    pub fn underwater_mood(&self) -> &str {
        self.underwater_mood
            .as_ref()
            .map_or("", SoundEventName::as_str)
    }

    /// Whether any of the six sounds is named (the game's own check).
    pub(crate) fn has_sound(&self) -> bool {
        let named = |addition: &Option<SoundAddition>| {
            addition
                .as_ref()
                .is_some_and(|a| !a.asset.as_str().is_empty())
        };
        named(&self.addition)
            || named(&self.underwater_addition)
            || !self.loop_sound().is_empty()
            || !self.mood().is_empty()
            || !self.underwater_loop().is_empty()
            || !self.underwater_mood().is_empty()
    }
}

/// A sound played now and then: `{"asset": "<sound>", "chance": 0.0111}`. Both members are
/// required; `asset` is not empty and `chance` lies in `[0, 1]`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct SoundAddition {
    /// JSON `asset`: the sound event, not empty.
    pub asset: SoundEventName,
    /// JSON `chance`: how likely the sound is each tick, from 0 to 1.
    pub chance: f32,
}

impl SoundAddition {
    /// No sound, at the game's default chance.
    fn none() -> Self {
        Self {
            asset: SoundEventName::parse(""),
            chance: DEFAULT_CHANCE,
        }
    }
}

/// `minecraft:biome_music`: the music of the biome. Every member is optional.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct BiomeMusic {
    /// JSON `volume_multiplier`, from 0 to 1.
    pub volume_multiplier: Option<f32>,
    /// JSON `music_definition` (format 1.21.70): a music definition of
    /// `sounds/music_definitions.json`.
    pub music_definition: Option<MusicDefinitionId>,
    /// JSON `underwater_music` (format 1.21.110); read it through
    /// [`underwater_music`](Self::underwater_music).
    /// Absent, the underwater music does not play (the game fills an absent member with zero).
    #[facet(schema::fallback = false)]
    pub underwater_music: Option<bool>,
}

/// A [`SoundAddition`]'s constraints on the member `key`: `asset` is not empty and `chance`
/// lies in `[0, 1]`. The findings carry no fixed text; the game's messages for these
/// constraints are its generic ones.
fn check_addition(json: &Value, key: &'static str, d: &mut Diagnostics) {
    let Some(addition) = json
        .as_object()
        .and_then(|object| object.get(key))
        .and_then(Value::as_object)
    else {
        return;
    };
    let asset = addition.get("asset").and_then(Value::as_string);
    if asset.is_some_and(|asset| asset.as_str().is_empty()) {
        d.push(
            Diagnostic::error(DiagnosticCode::InvalidValue { text: None })
                .at(child(&JsonPath::new(), &[key, "asset"])),
        );
    }
    let chance = addition
        .get("chance")
        .and_then(Value::as_number)
        .map(facet_value::VNumber::to_f64_lossy);
    if chance.is_some_and(|chance| !(0.0..=1.0).contains(&chance)) {
        d.push(
            Diagnostic::error(DiagnosticCode::InvalidValue { text: None })
                .at(child(&JsonPath::new(), &[key, "chance"])),
        );
    }
}

impl Checked for AmbientSounds {
    fn check(json: &Value, _: Severity, d: &mut Diagnostics) {
        check_addition(json, "addition", d);
        check_addition(json, "underwater_addition", d);
    }
}

impl Checked for BiomeMusic {}
