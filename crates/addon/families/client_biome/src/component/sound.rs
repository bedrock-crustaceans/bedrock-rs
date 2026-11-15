//! Sound components: `minecraft:ambient_sounds` and `minecraft:biome_music`.

use facet::Facet;

use bedrock_addon_core::diagnostic::Diagnostic;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::id::{MusicDefinitionId, SoundEventName};
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::schema::{self, FromJson, Schema};

/// The chance of an addition that names none: the game stores 1.0.
const DEFAULT_CHANCE: f32 = 1.0;

/// `minecraft:ambient_sounds`: the sounds that play in the biome, above and under water. At
/// least one sound is needed.
///
/// A member the document leaves out is `None`; the accessor of its name gives the game's
/// default (no sound). Every member is optional.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
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
pub struct SoundAddition {
    /// JSON `asset`: the sound event, not empty.
    #[facet(schema::read_with = asset)]
    pub asset: SoundEventName,
    /// JSON `chance`: how likely the sound is each tick, from 0 to 1.
    #[facet(schema::read_with = chance)]
    pub chance: f32,
}

/// The game's constraint of a [`SoundAddition`] member: told with a line of the crate's own and
/// the read goes on, so the document is refused with every one told.
fn constraint(node: &Node<'_>, read: &mut Read<'_>) {
    read.push(Diagnostic::of(&core_codes::INVALID_VALUE).at(node.path().clone()));
}

/// `asset` (`schema::read_with`): a sound event that is not empty.
fn asset(node: &Node<'_>, read: &mut Read<'_>) -> Option<SoundEventName> {
    let asset = SoundEventName::from_json(node, read)?;
    if asset.as_str().is_empty() {
        constraint(node, read);
    }
    Some(asset)
}

/// `chance` (`schema::read_with`): a number from 0 to 1, judged as the double it is written as
/// (a number beyond a float is out of range, not a wrong type).
fn chance(node: &Node<'_>, read: &mut Read<'_>) -> Option<f32> {
    let outside = node
        .value()
        .as_number()
        .is_some_and(|number| !(0.0..=1.0).contains(&number.to_f64_lossy()));
    if outside {
        constraint(node, read);
        return None;
    }
    f32::from_json(node, read)
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
